//! # MAGI Server Module
//!
//! SpacetimeDB WebAssembly module implementing the transactional database schema,
//! provider-agnostic deliberation state machine, and consensus/triage reducers for:
//! - Case 1: Idea / Markdown Viability Review
//! - Case 2: Code Maintenance & Guidelines Adherence
//! - Case 3: Error / Incident Triage with mandatory Trinity resolution

use spacetimedb::{reducer, table, ReducerContext, Table, Timestamp};

/// Deliberation type constants.
pub const TYPE_IDEA_ASSESSMENT: &str = "IDEA_ASSESSMENT";
pub const TYPE_CODE_MAINTENANCE: &str = "CODE_MAINTENANCE";
pub const TYPE_ERROR_TRIAGE: &str = "ERROR_TRIAGE";

/// Status constants for a deliberation lifecycle.
pub const STATUS_PENDING: &str = "PENDING";
pub const STATUS_DEBATING: &str = "DEBATING";
pub const STATUS_RESOLVED: &str = "RESOLVED";
pub const STATUS_FAILED: &str = "FAILED";

/// Vote posture constants for MAGI nodes.
pub const VOTE_APPROVE: &str = "APPROVE";
pub const VOTE_REJECT: &str = "REJECT";
pub const VOTE_NEUTRAL: &str = "NEUTRAL";
pub const VOTE_RESOLVED: &str = "RESOLVED";
pub const VOTE_ESCALATE: &str = "ESCALATE";

/// Node identifier constants.
pub const NODE_MELCHIOR: &str = "Melchior-1";
pub const NODE_BALTHASAR: &str = "Balthasar-2";
pub const NODE_CASPER: &str = "Casper-3";
pub const NODE_ALL: &str = "ALL";

/// Verdict outcome constants.
pub const VERDICT_VETO_BALTHASAR: &str = "VETO_BALTHASAR_SECURITY";
pub const VERDICT_APPROVED_UNANIMOUS: &str = "APPROVED_UNANIMOUS";
pub const VERDICT_APPROVED_MAJORITY: &str = "APPROVED_MAJORITY";
pub const VERDICT_REJECTED_MAJORITY: &str = "REJECTED_MAJORITY";
pub const VERDICT_REJECTED_UNANIMOUS: &str = "REJECTED_UNANIMOUS";
pub const VERDICT_SPLIT_DECISION: &str = "SPLIT_DECISION_REQUIRES_REVIEW";
pub const VERDICT_TRIAGE_RESOLVED: &str = "TRIAGE_RESOLVED";
pub const VERDICT_ESCALATED_TO_TRINITY: &str = "ESCALATED_TO_TRINITY";

/// Minimum risk score threshold (inclusive) for Balthasar-2 to trigger a veto.
pub const BALTHASAR_VETO_RISK_THRESHOLD: u8 = 8;

/// Total required node votes for full Trinity consensus.
pub const REQUIRED_NODE_COUNT: usize = 3;

/// Represents an audit or deliberation record in SpacetimeDB.
#[derive(Clone)]
#[table(name = deliberation, public)]
pub struct Deliberation {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub request_id: String,
    #[index(btree)]
    pub author: String,
    #[index(btree)]
    pub deliberation_type: String,
    pub title: String,
    pub prompt: String,
    pub context_type: String,
    pub context_payload: String,
    pub assigned_node: String,
    #[index(btree)]
    pub status: String,
    pub created_at: Timestamp,
}

/// Represents an individual vote or assessment cast by a MAGI persona.
#[table(name = node_vote, public)]
pub struct NodeVote {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub deliberation_id: u64,
    #[index(btree)]
    pub node_id: String,
    pub argument: String,
    pub cwe_flags: String,
    pub vote: String,
    pub risk_score: u8,
    pub execution_time_ms: u32,
    pub created_at: Timestamp,
}

/// Represents the final immutable consensus or triage outcome.
#[table(name = consensus_result, public)]
pub struct ConsensusResult {
    #[primary_key]
    pub deliberation_id: u64,
    pub deliberation_type: String,
    pub verdict: String,
    pub lead_node: String,
    pub tally_approves: u8,
    pub tally_rejects: u8,
    pub tally_neutrals: u8,
    pub summary: String,
    pub created_at: Timestamp,
}

/// Initializes the MAGI System database module.
#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    log::info!(
        "MAGI System consensus module initialized at {}",
        ctx.timestamp
    );
}

/// Creates a new deliberation (Idea assessment, Code maintenance, or Error triage).
///
/// # Arguments
/// * `request_id` - Unique client correlation token used to retrieve this exact deliberation.
/// * `author` - Requesting user or system identifier.
/// * `deliberation_type` - One of `IDEA_ASSESSMENT`, `CODE_MAINTENANCE`, or `ERROR_TRIAGE`.
/// * `title` - Short subject line.
/// * `prompt` - Question or instructions.
/// * `context_type` - Format classification (e.g. `MARKDOWN`, `CODE_SNIPPET`, `STACKTRACE`).
/// * `context_payload` - The document, code, or error message to inspect.
/// * `assigned_node` - `ALL` for Trinity consensus, or a specific node for targeted triage.
#[reducer]
#[allow(clippy::too_many_arguments)]
pub fn create_deliberation(
    ctx: &ReducerContext,
    request_id: String,
    author: String,
    deliberation_type: String,
    title: String,
    prompt: String,
    context_type: String,
    context_payload: String,
    assigned_node: String,
) -> Result<(), String> {
    if request_id.trim().is_empty() {
        return Err("Deliberation request ID cannot be empty".to_string());
    }
    if author.trim().is_empty() {
        return Err("Deliberation author cannot be empty".to_string());
    }
    if title.trim().is_empty() {
        return Err("Deliberation title cannot be empty".to_string());
    }

    let node_assignment = if assigned_node.trim().is_empty() {
        NODE_ALL.to_string()
    } else {
        assigned_node
    };
    if !matches!(
        node_assignment.as_str(),
        NODE_ALL | NODE_MELCHIOR | NODE_BALTHASAR | NODE_CASPER
    ) {
        return Err(format!(
            "Unknown deliberation assignment: {}",
            node_assignment
        ));
    }

    ctx.db.deliberation().insert(Deliberation {
        id: 0,
        request_id,
        author,
        deliberation_type,
        title,
        prompt,
        context_type,
        context_payload,
        assigned_node: node_assignment,
        status: STATUS_PENDING.to_string(),
        created_at: ctx.timestamp,
    });

    log::info!("New deliberation registered in state {}", STATUS_PENDING);
    Ok(())
}

/// Submits a node evaluation or fix and evaluates consensus/resolution.
#[reducer]
#[allow(clippy::too_many_arguments)]
pub fn submit_node_vote(
    ctx: &ReducerContext,
    deliberation_id: u64,
    node_id: String,
    argument: String,
    cwe_flags: String,
    vote: String,
    risk_score: u8,
    execution_time_ms: u32,
) -> Result<(), String> {
    let mut deliberation = ctx
        .db
        .deliberation()
        .id()
        .find(deliberation_id)
        .ok_or_else(|| format!("Deliberation {} does not exist", deliberation_id))?;

    if deliberation.status == STATUS_RESOLVED {
        return Err(format!(
            "Deliberation {} is already resolved",
            deliberation_id
        ));
    }

    let normalized_vote = vote.trim().to_uppercase();

    if !matches!(
        node_id.as_str(),
        NODE_MELCHIOR | NODE_BALTHASAR | NODE_CASPER
    ) {
        return Err(format!("Unknown MAGI node: {}", node_id));
    }
    if !matches!(
        normalized_vote.as_str(),
        VOTE_APPROVE | VOTE_REJECT | VOTE_NEUTRAL | VOTE_ESCALATE
    ) {
        return Err(format!("Unsupported vote posture: {}", normalized_vote));
    }
    if risk_score == 0 || risk_score > 10 {
        return Err(format!(
            "Risk score must be between 1 and 10, got {}",
            risk_score
        ));
    }

    // Prevent duplicate votes from the same node
    for existing_vote in ctx.db.node_vote().deliberation_id().filter(deliberation_id) {
        if existing_vote.node_id == node_id {
            return Err(format!(
                "Node {} has already cast an evaluation for deliberation {}",
                node_id, deliberation_id
            ));
        }
    }

    // The specialist opening analysis only routes the incident. Every triage
    // must continue to a full Trinity decision.
    if deliberation.deliberation_type == TYPE_ERROR_TRIAGE && deliberation.assigned_node != NODE_ALL
    {
        if node_id != deliberation.assigned_node {
            return Err(format!(
                "Node {} cannot vote on triage assigned to {}",
                node_id, deliberation.assigned_node
            ));
        }
        deliberation.assigned_node = NODE_ALL.to_string();
        deliberation.status = STATUS_DEBATING.to_string();
        ctx.db.deliberation().id().update(deliberation);
        log::info!(
            "Deliberation {} opened by specialist and convened the full Trinity",
            deliberation_id
        );
        return Ok(());
    }

    if deliberation.assigned_node == NODE_ALL && normalized_vote == VOTE_ESCALATE {
        return Err("ESCALATE is only valid for a targeted triage lead".to_string());
    }

    // ESCALATE changes the routing mode; it is not a final Trinity vote.
    ctx.db.node_vote().insert(NodeVote {
        id: 0,
        deliberation_id,
        node_id: node_id.clone(),
        argument,
        cwe_flags,
        vote: normalized_vote.clone(),
        risk_score,
        execution_time_ms,
        created_at: ctx.timestamp,
    });

    // Update status to DEBATING if it was PENDING
    if deliberation.status == STATUS_PENDING {
        deliberation.status = STATUS_DEBATING.to_string();
        ctx.db.deliberation().id().update(deliberation.clone());
    }

    // Collect all votes for full Trinity debate
    let votes: Vec<NodeVote> = ctx
        .db
        .node_vote()
        .deliberation_id()
        .filter(deliberation_id)
        .collect();

    let has_complete_trinity = votes.len() == REQUIRED_NODE_COUNT
        && [NODE_MELCHIOR, NODE_BALTHASAR, NODE_CASPER]
            .iter()
            .all(|node| votes.iter().any(|vote| vote.node_id == *node));

    if has_complete_trinity {
        log::info!(
            "All {} votes received for deliberation {}. Triggering consensus evaluation.",
            REQUIRED_NODE_COUNT,
            deliberation_id
        );
        evaluate_and_persist_consensus(ctx, &mut deliberation, &votes)?;
    }

    Ok(())
}

/// Pure data struct used for calculating consensus outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteRecord {
    pub node_id: String,
    pub vote: String,
    pub risk_score: u8,
}

/// Computes the consensus verdict, vote tallies, and whether Balthasar's veto triggered.
pub fn calculate_verdict(votes: &[VoteRecord]) -> (String, u8, u8, u8, bool) {
    let mut tally_approves: u8 = 0;
    let mut tally_rejects: u8 = 0;
    let mut tally_neutrals: u8 = 0;
    let mut balthasar_veto_triggered = false;
    let mut seen_nodes = [false; REQUIRED_NODE_COUNT];

    for v in votes {
        let node_index = match v.node_id.as_str() {
            NODE_MELCHIOR => 0,
            NODE_BALTHASAR => 1,
            NODE_CASPER => 2,
            _ => continue,
        };
        if seen_nodes[node_index] {
            continue;
        }
        seen_nodes[node_index] = true;

        match v.vote.as_str() {
            VOTE_APPROVE => tally_approves += 1,
            VOTE_REJECT => tally_rejects += 1,
            _ => tally_neutrals += 1,
        }

        // Balthasar-2 Security Veto Rule:
        // If Balthasar votes REJECT with risk_score >= 8, veto is triggered immediately.
        if v.node_id == NODE_BALTHASAR
            && v.vote == VOTE_REJECT
            && v.risk_score >= BALTHASAR_VETO_RISK_THRESHOLD
        {
            balthasar_veto_triggered = true;
        }
    }

    let verdict = if balthasar_veto_triggered {
        VERDICT_VETO_BALTHASAR.to_string()
    } else if tally_approves == 3 {
        VERDICT_APPROVED_UNANIMOUS.to_string()
    } else if tally_approves == 2 {
        VERDICT_APPROVED_MAJORITY.to_string()
    } else if tally_rejects == 3 {
        VERDICT_REJECTED_UNANIMOUS.to_string()
    } else if tally_rejects == 2 {
        VERDICT_REJECTED_MAJORITY.to_string()
    } else {
        VERDICT_SPLIT_DECISION.to_string()
    };

    (
        verdict,
        tally_approves,
        tally_rejects,
        tally_neutrals,
        balthasar_veto_triggered,
    )
}

/// Evaluates consensus tally and Balthasar veto, persisting the final outcome.
fn evaluate_and_persist_consensus(
    ctx: &ReducerContext,
    deliberation: &mut Deliberation,
    votes: &[NodeVote],
) -> Result<(), String> {
    let vote_records: Vec<VoteRecord> = votes
        .iter()
        .map(|v| VoteRecord {
            node_id: v.node_id.clone(),
            vote: v.vote.clone(),
            risk_score: v.risk_score,
        })
        .collect();

    let (verdict, tally_approves, tally_rejects, tally_neutrals, balthasar_veto_triggered) =
        calculate_verdict(&vote_records);

    let dissent_note = if balthasar_veto_triggered {
        " | SECURITY VETO ENFORCED: Balthasar-2 (Risk >= 8)".to_string()
    } else if tally_rejects > 0 && tally_approves > 0 {
        let dissenters: Vec<String> = votes
            .iter()
            .filter(|v| v.vote == VOTE_REJECT)
            .map(|v| format!("{} (risk {}/10)", v.node_id, v.risk_score))
            .collect();
        format!(" | Dissenting: {}", dissenters.join(", "))
    } else {
        String::new()
    };

    let summary = format!(
        "Trinity Consensus: {}. Tallies -> [APPROVE: {}, REJECT: {}, NEUTRAL: {}]{}",
        verdict, tally_approves, tally_rejects, tally_neutrals, dissent_note
    );

    ctx.db.consensus_result().insert(ConsensusResult {
        deliberation_id: deliberation.id,
        deliberation_type: deliberation.deliberation_type.clone(),
        verdict: verdict.clone(),
        lead_node: NODE_ALL.to_string(),
        tally_approves,
        tally_rejects,
        tally_neutrals,
        summary,
        created_at: ctx.timestamp,
    });

    deliberation.status = STATUS_RESOLVED.to_string();
    ctx.db.deliberation().id().update(deliberation.clone());

    log::info!(
        "Deliberation {} successfully resolved with verdict: {}",
        deliberation.id,
        verdict
    );

    Ok(())
}
