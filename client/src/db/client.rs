//! # SpacetimeDB Client Adapter
//!
//! Handles communication with the SpacetimeDB instance via HTTP API,
//! reducer invocation, and real-time query polling for consensus results.

use crate::error::MagiError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

/// Model for a deliberation fetched from database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliberationRecord {
    pub id: u64,
    pub author: String,
    pub title: String,
    pub prompt: String,
    pub context_type: String,
    pub context_payload: String,
    pub status: String,
}

/// Model for a consensus result fetched from database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResultRecord {
    pub deliberation_id: u64,
    pub verdict: String,
    pub tally_approves: u8,
    pub tally_rejects: u8,
    pub tally_neutrals: u8,
    pub summary: String,
}

fn sql_rows(value: &serde_json::Value) -> Vec<&serde_json::Value> {
    if let Some(rows) = value.get("rows").and_then(serde_json::Value::as_array) {
        rows.iter().collect()
    } else if let Some(rows) = value.as_array() {
        if let Some(envelope_rows) = rows
            .first()
            .and_then(|envelope| envelope.get("rows"))
            .and_then(serde_json::Value::as_array)
        {
            envelope_rows.iter().collect()
        } else {
            rows.iter().collect()
        }
    } else {
        Vec::new()
    }
}

fn sql_field<'a>(row: &'a serde_json::Value, index: usize, name: &str) -> Option<&'a serde_json::Value> {
    row.get(name).or_else(|| row.as_array().and_then(|values| values.get(index)))
}

/// SpacetimeDB client communicating via HTTP/REST protocol.
pub struct SpacetimeClient {
    client: Client,
    base_uri: String,
    database_name: String,
}

impl SpacetimeClient {
    /// Constructs a new SpacetimeDB client.
    pub fn new(base_uri: String, database_name: String) -> Self {
        Self {
            client: Client::new(),
            base_uri: base_uri.trim_end_matches('/').to_string(),
            database_name,
        }
    }

    /// Verifies connectivity to the SpacetimeDB instance.
    pub async fn check_health(&self) -> Result<bool, MagiError> {
        let url = format!("{}/v1/database/{}", self.base_uri, self.database_name);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(e) => Err(MagiError::Database(format!(
                "Cannot connect to SpacetimeDB at {}: {}",
                self.base_uri, e
            ))),
        }
    }

    /// Invokes a reducer on the SpacetimeDB database module.
    pub async fn call_reducer(
        &self,
        reducer_name: &str,
        args: Vec<serde_json::Value>,
    ) -> Result<(), MagiError> {
        let url = format!(
            "{}/v1/database/{}/call/{}",
            self.base_uri, self.database_name, reducer_name
        );

        let response = self
            .client
            .post(&url)
            .json(&args)
            .send()
            .await
            .map_err(|e| {
                MagiError::Database(format!(
                    "Reducer call [{}] network error: {}",
                    reducer_name, e
                ))
            })?;

        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(MagiError::Database(format!(
                "Reducer call [{}] failed: {}",
                reducer_name, error_body
            )));
        }

        Ok(())
    }

    /// Executes an SQL query against SpacetimeDB.
    pub async fn query_sql(&self, sql_query: &str) -> Result<serde_json::Value, MagiError> {
        let url = format!("{}/v1/database/{}/sql", self.base_uri, self.database_name);

        let response = self
            .client
            .post(&url)
            .header("content-type", "text/plain")
            .body(sql_query.to_string())
            .send()
            .await
            .map_err(|e| MagiError::Database(format!("SQL query error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(MagiError::Database(format!(
                "SQL query failed: {}",
                error_text
            )));
        }

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| MagiError::Database(format!("Failed to parse SQL response JSON: {}", e)))
    }

    /// Creates a new deliberation and returns its generated ID.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_deliberation(
        &self,
        author: &str,
        deliberation_type: &str,
        title: &str,
        prompt: &str,
        context_type: &str,
        context_payload: &str,
        assigned_node: &str,
    ) -> Result<u64, MagiError> {
        let request_id = format!(
            "{}-{}-{}",
            author,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| MagiError::Database(format!("System clock error: {}", e)))?
                .as_nanos(),
            process::id()
        );
        let args = vec![
            json!(&request_id),
            json!(author),
            json!(deliberation_type),
            json!(title),
            json!(prompt),
            json!(context_type),
            json!(context_payload),
            json!(assigned_node),
        ];

        self.call_reducer("create_deliberation", args).await?;

        // Query by the unique request reference, never by the latest author row.
        let escaped_request_id = request_id.replace('\'', "''");
        let sql = format!(
            "SELECT id FROM deliberation WHERE request_id = '{}' LIMIT 1;",
            escaped_request_id
        );

        // Allow a small delay for transactional propagation
        sleep(Duration::from_millis(50)).await;

        let res = self.query_sql(&sql).await?;
        let deliberation_id = sql_rows(&res)
            .first()
            .and_then(|row| sql_field(row, 0, "id"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                MagiError::Database(format!(
                    "Deliberation created with request ID {} but no ID was returned",
                    request_id
                ))
            })?;

        Ok(deliberation_id)
    }

    /// Submits a vote for a deliberation.
    #[allow(clippy::too_many_arguments)]
    pub async fn submit_vote(
        &self,
        deliberation_id: u64,
        node_id: &str,
        argument: &str,
        cwe_flags: &[String],
        vote: &str,
        risk_score: u8,
        execution_time_ms: u32,
    ) -> Result<(), MagiError> {
        let cwe_json = serde_json::to_string(cwe_flags).unwrap_or_else(|_| "[]".to_string());
        let args = vec![
            json!(deliberation_id),
            json!(node_id),
            json!(argument),
            json!(cwe_json),
            json!(vote),
            json!(risk_score),
            json!(execution_time_ms),
        ];

        self.call_reducer("submit_node_vote", args).await
    }

    pub async fn submit_evaluations(
        &self,
        deliberation_id: u64,
        evaluations: &[crate::llm::NodeEvaluation],
    ) -> Result<(), MagiError> {
        for evaluation in evaluations {
            self.submit_vote(
                deliberation_id,
                &evaluation.node_id,
                &evaluation.argument,
                &evaluation.cwe_flags,
                &evaluation.vote,
                evaluation.risk_score,
                evaluation.execution_time_ms,
            )
            .await?;
        }
        Ok(())
    }

    /// Awaits consensus result for a deliberation by polling SpacetimeDB with timeout.
    pub async fn wait_for_consensus(
        &self,
        deliberation_id: u64,
        max_duration: Duration,
    ) -> Result<ConsensusResultRecord, MagiError> {
        let sql = format!(
            "SELECT deliberation_id, verdict, tally_approves, tally_rejects, tally_neutrals, summary \
             FROM consensus_result WHERE deliberation_id = {};",
            deliberation_id
        );

        let start = std::time::Instant::now();
        while start.elapsed() < max_duration {
            if let Ok(res) = self.query_sql(&sql).await {
                if let Some(row) = sql_rows(&res).first() {
                        let verdict = sql_field(row, 1, "verdict")
                            .and_then(|v| v.as_str())
                            .unwrap_or("UNKNOWN")
                            .to_string();

                        let tally_approves = sql_field(row, 2, "tally_approves")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u8;

                        let tally_rejects = sql_field(row, 3, "tally_rejects")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u8;

                        let tally_neutrals = sql_field(row, 4, "tally_neutrals")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u8;

                        let summary = sql_field(row, 5, "summary")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        return Ok(ConsensusResultRecord {
                            deliberation_id,
                            verdict,
                            tally_approves,
                            tally_rejects,
                            tally_neutrals,
                            summary,
                        });
                }
            }

            sleep(Duration::from_millis(200)).await;
        }

        Err(MagiError::Timeout(max_duration.as_secs()))
    }

    /// Fetches past deliberations from SpacetimeDB.
    pub async fn list_history(&self, limit: usize) -> Result<Vec<DeliberationRecord>, MagiError> {
        let sql = "SELECT id, author, title, prompt, context_type, context_payload, status FROM deliberation;";

        let res = self.query_sql(sql).await?;
        let mut list = Vec::new();

        for row in sql_rows(&res) {
            let id = sql_field(row, 0, "id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let author = sql_field(row, 1, "author")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = sql_field(row, 2, "title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let prompt = sql_field(row, 3, "prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let context_type = sql_field(row, 4, "context_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let context_payload = sql_field(row, 5, "context_payload")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let status = sql_field(row, 6, "status")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            list.push(DeliberationRecord {
                id,
                author,
                title,
                prompt,
                context_type,
                context_payload,
                status,
            });
        }

        list.sort_by(|a, b| b.id.cmp(&a.id));
        list.truncate(limit);

        Ok(list)
    }

    /// Fetches detailed records for a specific deliberation ID.
    pub async fn get_deliberation_details(
        &self,
        deliberation_id: u64,
    ) -> Result<
        Option<(
            DeliberationRecord,
            Vec<crate::llm::NodeEvaluation>,
            Option<ConsensusResultRecord>,
        )>,
        MagiError,
    > {
        let delib_sql = format!(
            "SELECT id, author, title, prompt, context_type, context_payload, status \
             FROM deliberation WHERE id = {};",
            deliberation_id
        );

        let delib_res = self.query_sql(&delib_sql).await?;
        let deliberation = if let Some(row) = sql_rows(&delib_res).first() {
            let id = sql_field(row, 0, "id")
                .and_then(|v| v.as_u64())
                .unwrap_or(deliberation_id);
            let author = sql_field(row, 1, "author")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = sql_field(row, 2, "title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let prompt = sql_field(row, 3, "prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let context_type = sql_field(row, 4, "context_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let context_payload = sql_field(row, 5, "context_payload")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let status = sql_field(row, 6, "status")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            DeliberationRecord {
                id,
                author,
                title,
                prompt,
                context_type,
                context_payload,
                status,
            }
        } else {
            return Ok(None);
        };

        // Query votes
        let votes_sql = format!(
            "SELECT node_id, argument, cwe_flags, vote, risk_score, execution_time_ms \
             FROM node_vote WHERE deliberation_id = {};",
            deliberation_id
        );

        let votes_res = self.query_sql(&votes_sql).await?;
        let mut evaluations = Vec::new();

        for row in sql_rows(&votes_res) {
                let node_id = sql_field(row, 0, "node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let argument = sql_field(row, 1, "argument")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let cwe_str = sql_field(row, 2, "cwe_flags")
                    .and_then(|v| v.as_str())
                    .unwrap_or("[]");
                let cwe_flags: Vec<String> = serde_json::from_str(cwe_str).unwrap_or_default();
                let vote = sql_field(row, 3, "vote")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let risk_score = sql_field(row, 4, "risk_score")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8;
                let execution_time_ms = sql_field(row, 5, "execution_time_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                evaluations.push(crate::llm::NodeEvaluation {
                    node_id,
                    vote,
                    risk_score,
                    findings: Vec::new(),
                    rationale: argument.clone(),
                    argument,
                    confidence: 1.0,
                    cwe_flags,
                    execution_time_ms,
                    prompt_version: String::new(),
                    model: String::new(),
                    initial_argument: None,
                    initial_vote: None,
                    initial_risk_score: None,
                });
        }

        // Query consensus result
        let cons_sql = format!(
            "SELECT deliberation_id, verdict, tally_approves, tally_rejects, tally_neutrals, summary \
             FROM consensus_result WHERE deliberation_id = {};",
            deliberation_id
        );

        let consensus = if let Ok(cons_res) = self.query_sql(&cons_sql).await {
            if let Some(row) = sql_rows(&cons_res).first() {
                let verdict = sql_field(row, 1, "verdict")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tally_approves = sql_field(row, 2, "tally_approves")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8;
                let tally_rejects = sql_field(row, 3, "tally_rejects")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8;
                let tally_neutrals = sql_field(row, 4, "tally_neutrals")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8;
                let summary = sql_field(row, 5, "summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                Some(ConsensusResultRecord {
                    deliberation_id,
                    verdict,
                    tally_approves,
                    tally_rejects,
                    tally_neutrals,
                    summary,
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(Some((deliberation, evaluations, consensus)))
    }
}
