//! # MAGI System CLI
//!
//! Event-driven distributed multi-agent consensus code auditor inspired by
//! the MAGI supercomputer from Neon Genesis Evangelion.
//!
//! Directly supports the 3 core operational workflows:
//! - Case 1 (`magi idea`): Markdown / Idea Viability Deliberation
//! - Case 2 (`magi maintain`): Code Maintenance & Guidelines Adherence Deliberation
//! - Case 3 (`magi triage`): Specialist opening analysis followed by Trinity resolution

mod config;
mod core;
mod db;
mod error;
mod llm;
mod skills;
mod ui;

use clap::{Parser, Subcommand};
use colored::*;
use config::MagiConfig;
use core::MagiOrchestrator;
use db::SpacetimeClient;
use error::MagiError;
use skills::PromptLoader;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use ui::NervTheme;

/// CLI argument parser for MAGI System.
#[derive(Parser, Debug)]
#[command(name = "magi")]
#[command(author = "Felix Manrique")]
#[command(version = "0.1.3")]
#[command(about = "Event-driven multi-agent LLM consensus engine powered by SpacetimeDB", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable mock mode for testing without requiring external LLM API keys
    #[arg(long, global = true)]
    mock: bool,

    /// Optional path to custom skill or instructions file injected into the Trinity
    #[arg(long, global = true)]
    skill: Option<PathBuf>,

    /// Natural language prompt or file path when no subcommand is specified
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch the interactive NERV terminal UI
    #[command(alias = "console", alias = "tui", alias = "ui")]
    Interactive,

    /// Case 1: Evaluate technical viability of a Markdown proposal / idea document
    Idea {
        /// Path to the Markdown proposal (e.g. IDEA.md, RFC.md)
        #[arg(value_name = "MARKDOWN_FILE")]
        path: PathBuf,

        /// Additional instructions or questions for the Trinity
        #[arg(
            short,
            long,
            default_value = "Is this idea viable? Deliberate across architecture, security, and pragmatism."
        )]
        prompt: String,

        /// Enable mock mode for testing without requiring external LLM API keys
        #[arg(long)]
        mock: bool,

        /// In mock mode, simulate a severe security vulnerability triggering Balthasar's veto
        #[arg(long)]
        simulate_veto: bool,

        /// Override request timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,
    },

    /// Case 2: Deliberate code maintenance and refactoring under specific guidelines
    Maintain {
        /// Path to the source code file to maintain
        #[arg(value_name = "CODE_FILE")]
        path: PathBuf,

        /// Path to the guidelines document or rules file (e.g. SKILL.md, GUIDELINES.md)
        #[arg(short, long, value_name = "GUIDELINES_FILE")]
        guidelines: PathBuf,

        /// Specific maintenance instructions or questions
        #[arg(
            short,
            long,
            default_value = "Evaluate how to maintain and evolve this code under these guidelines."
        )]
        prompt: String,

        /// Enable mock mode for testing without requiring external LLM API keys
        #[arg(long)]
        mock: bool,

        /// In mock mode, simulate a severe security vulnerability triggering Balthasar's veto
        #[arg(long)]
        simulate_veto: bool,

        /// Override request timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,
    },

    /// Case 3: Intelligently triage an error or stack trace to the single best node
    Triage {
        /// Error message string or path to an error log file
        #[arg(value_name = "ERROR_TEXT_OR_FILE")]
        error: String,

        /// Optional path to the relevant source code file
        #[arg(short, long, value_name = "CODE_FILE")]
        code: Option<PathBuf>,

        /// Enable mock mode for testing without requiring external LLM API keys
        #[arg(long)]
        mock: bool,

        /// Override request timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,
    },

    /// General multi-agent deliberation on any code, document, or technical dilemma
    #[command(alias = "deliberate")]
    Audit {
        /// Optional path to the target file; omit it to deliberate on the prompt itself
        #[arg(value_name = "FILE_PATH")]
        path: Option<PathBuf>,

        /// Specific audit instruction or question for the nodes
        #[arg(
            short,
            long,
            default_value = "Audit this code for architectural quality, security risks, and implementation pragmatism."
        )]
        prompt: String,

        /// Optional title for the deliberation
        #[arg(short, long)]
        title: Option<String>,

        /// Context type classification (e.g. CODE_SNIPPET, DOCKERFILE, SPEC)
        #[arg(short = 'c', long, default_value = "CODE_SNIPPET")]
        context_type: String,

        /// Enable mock mode for testing without requiring external LLM API keys
        #[arg(long)]
        mock: bool,

        /// In mock mode, simulate a severe security vulnerability triggering Balthasar's veto
        #[arg(long)]
        simulate_veto: bool,

        /// Override request timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,

        /// Override LLM provider for Melchior-1
        #[arg(long)]
        melchior_provider: Option<String>,

        /// Override model for Melchior-1
        #[arg(long)]
        melchior_model: Option<String>,

        /// Override LLM provider for Balthasar-2
        #[arg(long)]
        balthasar_provider: Option<String>,

        /// Override model for Balthasar-2
        #[arg(long)]
        balthasar_model: Option<String>,

        /// Override LLM provider for Casper-3
        #[arg(long)]
        casper_provider: Option<String>,

        /// Override model for Casper-3
        #[arg(long)]
        casper_model: Option<String>,
    },

    /// View deliberation history stored in SpacetimeDB
    History {
        /// Maximum number of records to retrieve
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },

    /// View detailed deliberation record and node debate by ID
    Show {
        /// ID of the deliberation to inspect
        #[arg(value_name = "DELIBERATION_ID")]
        id: u64,
    },

    /// Verify connectivity to the SpacetimeDB engine
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();
    let mut config = MagiConfig::from_env()?;
    let is_mock = cli.mock
        || (config.melchior.api_key.is_none()
            && config.balthasar.api_key.is_none()
            && config.casper.api_key.is_none());

    let custom_skill_content = if let Some(ref skill_path) = cli.skill {
        let content = PromptLoader::load_custom_skill(skill_path).map_err(|e| {
            MagiError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to load custom skill from '{}': {}", skill_path.display(), e),
            ))
        })?;
        println!(
            "{} Injected custom skill from [{}]",
            "MAGI SKILL:".bright_magenta().bold(),
            skill_path.display()
        );
        Some(content)
    } else {
        None
    };

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            if cli.query.is_empty() {
                return ui::run_interactive_session(config, is_mock, custom_skill_content).await;
            } else {
                let query_str = cli.query.join(" ");
                let intent = ui::intent::process_user_intent(&query_str);
                return execute_inferred_intent(intent, config, is_mock, custom_skill_content).await;
            }
        }
    };

    match command {
        Commands::Interactive => {
            ui::run_interactive_session(config, is_mock, custom_skill_content).await?;
        }

        Commands::Status => {
            NervTheme::print_banner();
            println!("Connecting to SpacetimeDB at {}...", config.spacetimedb_uri);
            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );
            match db_client.check_health().await {
                Ok(true) => println!("SpacetimeDB connection status: [ONLINE / HEALTHY]"),
                Ok(false) => println!("SpacetimeDB responded but database is not yet initialized."),
                Err(e) => eprintln!("SpacetimeDB connection error: {}", e),
            }
        }

        Commands::History { limit } => {
            NervTheme::print_banner();
            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );
            match db_client.list_history(limit).await {
                Ok(records) => {
                    println!(
                        "PAST DELIBERATIONS RECORDED IN SPACETIMEDB (Top {}):",
                        limit
                    );
                    NervTheme::render_history(&records);
                }
                Err(e) => eprintln!("Error fetching history: {}", e),
            }
        }

        Commands::Show { id } => {
            NervTheme::print_banner();
            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );
            match db_client.get_deliberation_details(id).await {
                Ok(Some((delib, evals, consensus))) => {
                    NervTheme::print_deliberation_header(delib.id, &delib.title, &delib.author);
                    println!("PROMPT: {}", delib.prompt);
                    println!("CONTEXT TYPE: {}", delib.context_type);
                    println!("STATUS: {}", delib.status);
                    if !evals.is_empty() {
                        NervTheme::render_votes_table(&evals);
                    }
                    if let Some(c) = consensus {
                        NervTheme::render_verdict(&c.verdict, &c.summary);
                    }
                }
                Ok(None) => println!("Deliberation #{} not found in SpacetimeDB.", id),
                Err(e) => eprintln!("Error fetching deliberation details: {}", e),
            }
        }

        // =====================================================================
        // CASE 1: IDEA / MARKDOWN VIABILITY REVIEW
        // =====================================================================
        Commands::Idea {
            path,
            prompt,
            mock,
            simulate_veto,
            timeout,
        } => {
            if let Some(t) = timeout {
                config.timeout_seconds = t;
            }

            NervTheme::print_banner();
            let markdown_content = fs::read_to_string(&path).map_err(|e| {
                MagiError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Proposal file '{}' not found: {}", path.display(), e),
                ))
            })?;

            let title = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("Idea Review")
                .to_string();

            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );

            let deliberation_id = db_client
                .create_deliberation(
                    &config.author,
                    "IDEA_ASSESSMENT",
                    &title,
                    &prompt,
                    "MARKDOWN",
                    &markdown_content,
                    "ALL",
                )
                .await?;

            NervTheme::print_deliberation_header(deliberation_id, &title, &config.author);

            let is_mock = mock
                || (config.melchior.api_key.is_none()
                    && config.balthasar.api_key.is_none()
                    && config.casper.api_key.is_none());
            let orchestrator = MagiOrchestrator::new(config.clone(), is_mock)?
                .with_custom_skill(custom_skill_content.clone());

            let eval_content = if simulate_veto {
                format!("{}\n## Security Warning\nHigh risk data leak and hardcoded credentials detected.", markdown_content)
            } else {
                markdown_content
            };

            let evaluations = orchestrator.deliberate_idea(&prompt, &eval_content).await?;

            db_client
                .submit_evaluations(deliberation_id, &evaluations)
                .await?;

            NervTheme::render_votes_table(&evaluations);
            let consensus_result =
                resolve_consensus(&db_client, deliberation_id, &evaluations).await;
            NervTheme::render_verdict(&consensus_result.0, &consensus_result.1);

            ui::save_host_deliberation_report(
                deliberation_id,
                &title,
                "IDEA_PROPOSAL",
                "MARKDOWN",
                &eval_content,
                &evaluations,
                &consensus_result.0,
                &consensus_result.1,
            );
        }

        // =====================================================================
        // CASE 2: CODE MAINTENANCE & GUIDELINES COMPLIANCE
        // =====================================================================
        Commands::Maintain {
            path,
            guidelines,
            prompt,
            mock,
            simulate_veto,
            timeout,
        } => {
            if let Some(t) = timeout {
                config.timeout_seconds = t;
            }

            NervTheme::print_banner();
            let code_content = fs::read_to_string(&path)?;
            let guidelines_content = fs::read_to_string(&guidelines)?;

            let title = format!(
                "Maintenance of {} under {}",
                path.file_name().and_then(|f| f.to_str()).unwrap_or("code"),
                guidelines
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("guidelines")
            );

            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );

            let deliberation_id = db_client
                .create_deliberation(
                    &config.author,
                    "CODE_MAINTENANCE",
                    &title,
                    &prompt,
                    "SOURCE_CODE",
                    &code_content,
                    "ALL",
                )
                .await?;

            NervTheme::print_deliberation_header(deliberation_id, &title, &config.author);

            let is_mock = mock
                || (config.melchior.api_key.is_none()
                    && config.balthasar.api_key.is_none()
                    && config.casper.api_key.is_none());
            let orchestrator = MagiOrchestrator::new(config.clone(), is_mock)?
                .with_custom_skill(custom_skill_content.clone());

            let eval_content = if simulate_veto {
                format!(
                    "{}\n// format!(\"SELECT * FROM users WHERE token = '\" + token + \"'\");",
                    code_content
                )
            } else {
                code_content.clone()
            };

            let evaluations = orchestrator
                .deliberate_maintenance(&eval_content, &guidelines_content, &prompt)
                .await?;

            db_client
                .submit_evaluations(deliberation_id, &evaluations)
                .await?;

            NervTheme::render_votes_table(&evaluations);
            let consensus_result =
                resolve_consensus(&db_client, deliberation_id, &evaluations).await;
            NervTheme::render_verdict(&consensus_result.0, &consensus_result.1);

            let combined_context = format!(
                "CODE:\n{}\n\nGUIDELINES:\n{}",
                code_content, guidelines_content
            );
            ui::save_host_deliberation_report(
                deliberation_id,
                &title,
                "CODE_MAINTENANCE",
                "SOURCE_CODE",
                &combined_context,
                &evaluations,
                &consensus_result.0,
                &consensus_result.1,
            );
        }

        // =====================================================================
        // CASE 3: TARGETED ERROR / INCIDENT TRIAGE
        // =====================================================================
        Commands::Triage {
            error,
            code,
            mock,
            timeout,
        } => {
            if let Some(t) = timeout {
                config.timeout_seconds = t;
            }

            NervTheme::print_banner();
            let error_text = if std::path::Path::new(&error).exists() {
                fs::read_to_string(&error)?
            } else {
                error
            };

            let code_content = if let Some(ref c) = code {
                Some(fs::read_to_string(c)?)
            } else {
                None
            };

            let lead_node = core::MagiOrchestrator::select_lead_node_for_error(&error_text);
            println!(
                "{} Error classified. Assigned Lead Node: [{}]",
                "MAGI ROUTER:".bright_yellow().bold(),
                lead_node.bright_cyan().bold()
            );

            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );

            let deliberation_id = db_client
                .create_deliberation(
                    &config.author,
                    "ERROR_TRIAGE",
                    "Error Incident Triage",
                    "Resolve root cause with targeted remediation",
                    "ERROR_LOG",
                    &error_text,
                    lead_node,
                )
                .await?;

            let is_mock = mock
                || (config.melchior.api_key.is_none()
                    && config.balthasar.api_key.is_none()
                    && config.casper.api_key.is_none());
            let orchestrator = MagiOrchestrator::new(config.clone(), is_mock)?
                .with_custom_skill(custom_skill_content.clone());

            let (lead, eval, escalation) = orchestrator
                .triage_error(&error_text, code_content.as_deref())
                .await?;

            db_client
                .submit_vote(
                    deliberation_id,
                    &eval.node_id,
                    &eval.argument,
                    &eval.cwe_flags,
                    &eval.vote,
                    eval.risk_score,
                    eval.execution_time_ms,
                )
                .await?;
            NervTheme::render_triage_result(&lead, &eval);

            if let Some(trinity_evals) = escalation {
                println!(
                    "{}",
                    "Opening mandatory full Trinity incident deliberation...".bright_yellow()
                );
                db_client
                    .submit_evaluations(deliberation_id, &trinity_evals)
                    .await?;
                NervTheme::render_votes_table(&trinity_evals);
                let consensus =
                    resolve_consensus(&db_client, deliberation_id, &trinity_evals).await;
                NervTheme::render_verdict(&consensus.0, &consensus.1);

                ui::save_host_deliberation_report(
                    deliberation_id,
                    "Incident Triage - Trinity Deliberation",
                    "ERROR_TRIAGE",
                    "ERROR_LOG",
                    &error_text,
                    &trinity_evals,
                    &consensus.0,
                    &consensus.1,
                );
            } else {
                db_client
                    .submit_vote(
                        deliberation_id,
                        &eval.node_id,
                        &eval.argument,
                        &eval.cwe_flags,
                        &eval.vote,
                        eval.risk_score,
                        eval.execution_time_ms,
                    )
                    .await?;
                let v = eval.vote.clone();
                let s = eval.argument.clone();
                ui::save_host_deliberation_report(
                    deliberation_id,
                    "Incident Triage",
                    "ERROR_TRIAGE",
                    "ERROR_LOG",
                    &error_text,
                    &[eval],
                    &v,
                    &s,
                );
            }
        }

        // =====================================================================
        // GENERAL AUDIT
        // =====================================================================
        Commands::Audit {
            path,
            prompt,
            title,
            context_type,
            mock,
            simulate_veto,
            timeout,
            melchior_provider,
            melchior_model,
            balthasar_provider,
            balthasar_model,
            casper_provider,
            casper_model,
        } => {
            if let Some(t) = timeout {
                config.timeout_seconds = t;
            }
            if let Some(p) = melchior_provider {
                config.melchior.provider = p;
            }
            if let Some(m) = melchior_model {
                config.melchior.model = m;
            }
            if let Some(p) = balthasar_provider {
                config.balthasar.provider = p;
            }
            if let Some(m) = balthasar_model {
                config.balthasar.model = m;
            }
            if let Some(p) = casper_provider {
                config.casper.provider = p;
            }
            if let Some(m) = casper_model {
                config.casper.model = m;
            }

            NervTheme::print_banner();
            let (file_content, default_title) = match path {
                Some(path) => (
                    fs::read_to_string(&path)?,
                    path.file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("audit")
                        .to_string(),
                ),
                None => (prompt.clone(), "Universal Deliberation".to_string()),
            };
            let file_title = title.unwrap_or(default_title);

            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );

            let deliberation_id = db_client
                .create_deliberation(
                    &config.author,
                    "GENERAL_AUDIT",
                    &file_title,
                    &prompt,
                    &context_type,
                    &file_content,
                    "ALL",
                )
                .await?;

            NervTheme::print_deliberation_header(deliberation_id, &file_title, &config.author);
            let is_mock = mock
                || (config.melchior.api_key.is_none()
                    && config.balthasar.api_key.is_none()
                    && config.casper.api_key.is_none());
            let orchestrator = MagiOrchestrator::new(config.clone(), is_mock)?
                .with_custom_skill(custom_skill_content.clone());

            let eval_content = if simulate_veto {
                format!(
                    "{}\n// format!(\"SELECT * FROM users WHERE id = '\" + id + \"'\");",
                    file_content
                )
            } else {
                file_content
            };

            let evaluations = orchestrator.deliberate_idea(&prompt, &eval_content).await?;

            db_client
                .submit_evaluations(deliberation_id, &evaluations)
                .await?;

            NervTheme::render_votes_table(&evaluations);
            let consensus_result =
                resolve_consensus(&db_client, deliberation_id, &evaluations).await;
            NervTheme::render_verdict(&consensus_result.0, &consensus_result.1);

            ui::save_host_deliberation_report(
                deliberation_id,
                &file_title,
                "UNIVERSAL_AUDIT",
                &context_type,
                &eval_content,
                &evaluations,
                &consensus_result.0,
                &consensus_result.1,
            );
        }
    }

    Ok(())
}

async fn resolve_consensus(
    db_client: &SpacetimeClient,
    deliberation_id: u64,
    _evaluations: &[llm::NodeEvaluation],
) -> (String, String) {
    match db_client
        .wait_for_consensus(deliberation_id, Duration::from_secs(5))
        .await
    {
        Ok(res) => (res.verdict, res.summary),
        Err(error) => (
            "CONSENSUS_UNAVAILABLE".to_string(),
            format!("SpacetimeDB consensus result unavailable: {}", error),
        ),
    }
}

async fn execute_inferred_intent(
    intent: ui::intent::InferredIntent,
    config: MagiConfig,
    is_mock: bool,
    custom_skill: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let db_client = SpacetimeClient::new(
        config.spacetimedb_uri.clone(),
        config.spacetimedb_database.clone(),
    );
    let orchestrator = MagiOrchestrator::new(config.clone(), is_mock)?
        .with_custom_skill(custom_skill);

    match intent {
        ui::intent::InferredIntent::SystemCommand(cmd) => match cmd.as_str() {
            "status" => {
                NervTheme::print_banner();
                match db_client.check_health().await {
                    Ok(true) => println!("SpacetimeDB connection status: [ONLINE / HEALTHY]"),
                    Ok(false) => {
                        println!("SpacetimeDB responded but database is not yet initialized.")
                    }
                    Err(e) => eprintln!("SpacetimeDB connection error: {}", e),
                }
            }
            "history" => {
                NervTheme::print_banner();
                let records = db_client.list_history(20).await?;
                NervTheme::render_history(&records);
            }
            _ => {
                println!("MAGI Operational Console. Type 'magi' for interactive session.");
            }
        },

        ui::intent::InferredIntent::IdeaAssessment { path, question } => {
            NervTheme::print_banner();
            let content = fs::read_to_string(&path)?;
            let title = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("Idea")
                .to_string();
            let id = db_client
                .create_deliberation(
                    &config.author,
                    "IDEA_ASSESSMENT",
                    &title,
                    &question,
                    "MARKDOWN",
                    &content,
                    "ALL",
                )
                .await?;

            NervTheme::print_deliberation_header(id, &title, &config.author);
            let evals = orchestrator.deliberate_idea(&question, &content).await?;

            db_client.submit_evaluations(id, &evals).await?;

            NervTheme::render_votes_table(&evals);
            let (verdict, summary) = resolve_consensus(&db_client, id, &evals).await;
            NervTheme::render_verdict(&verdict, &summary);

            ui::save_host_deliberation_report(
                id,
                &title,
                "IDEA_ASSESSMENT",
                "MARKDOWN",
                &content,
                &evals,
                &verdict,
                &summary,
            );
        }

        ui::intent::InferredIntent::CodeMaintenance {
            code_path,
            guidelines_path,
            instructions,
        } => {
            NervTheme::print_banner();
            let code_content = fs::read_to_string(&code_path)?;
            let guidelines_content = if let Some(ref g) = guidelines_path {
                fs::read_to_string(g).unwrap_or_default()
            } else {
                "Audit against code quality and regression prevention.".to_string()
            };

            let title = format!(
                "Maintain {}",
                code_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("Code")
            );
            let id = db_client
                .create_deliberation(
                    &config.author,
                    "CODE_MAINTENANCE",
                    &title,
                    &instructions,
                    "SOURCE_CODE",
                    &code_content,
                    "ALL",
                )
                .await?;

            NervTheme::print_deliberation_header(id, &title, &config.author);
            let evals = orchestrator
                .deliberate_maintenance(&code_content, &guidelines_content, &instructions)
                .await?;

            db_client.submit_evaluations(id, &evals).await?;

            NervTheme::render_votes_table(&evals);
            let (verdict, summary) = resolve_consensus(&db_client, id, &evals).await;
            NervTheme::render_verdict(&verdict, &summary);

            let combined = format!(
                "CODE:\n{}\n\nGUIDELINES:\n{}",
                code_content, guidelines_content
            );
            ui::save_host_deliberation_report(
                id,
                &title,
                "CODE_MAINTENANCE",
                "SOURCE_CODE",
                &combined,
                &evals,
                &verdict,
                &summary,
            );
        }

        ui::intent::InferredIntent::ErrorTriage {
            error_text,
            code_context,
        } => {
            NervTheme::print_banner();
            let lead_node = core::MagiOrchestrator::select_lead_node_for_error(&error_text);
            println!("MAGI ROUTER: Incident assigned directly to [{}]", lead_node);

            let code_content = if let Some(ref c) = code_context {
                fs::read_to_string(c).ok()
            } else {
                None
            };

            let id = db_client
                .create_deliberation(
                    &config.author,
                    "ERROR_TRIAGE",
                    "Triage Incident",
                    "Targeted resolution",
                    "ERROR_LOG",
                    &error_text,
                    lead_node,
                )
                .await?;

            let (lead, eval, escalation) = orchestrator
                .triage_error(&error_text, code_content.as_deref())
                .await?;

            NervTheme::render_triage_result(&lead, &eval);

            if let Some(trinity_evals) = escalation {
                db_client.submit_evaluations(id, &trinity_evals).await?;
                println!("Specialist opening complete. Full Trinity consensus:");
                NervTheme::render_votes_table(&trinity_evals);
                let (verdict, summary) = resolve_consensus(&db_client, id, &trinity_evals).await;
                NervTheme::render_verdict(&verdict, &summary);

                ui::save_host_deliberation_report(
                    id,
                    "Incident Triage - Trinity Deliberation",
                    "ERROR_TRIAGE",
                    "ERROR_LOG",
                    &error_text,
                    &trinity_evals,
                    &verdict,
                    &summary,
                );
            } else {
                db_client
                    .submit_vote(
                        id,
                        &eval.node_id,
                        &eval.argument,
                        &eval.cwe_flags,
                        &eval.vote,
                        eval.risk_score,
                        eval.execution_time_ms,
                    )
                    .await?;
                let v = eval.vote.clone();
                let s = eval.argument.clone();
                ui::save_host_deliberation_report(
                    id,
                    "Incident Triage",
                    "ERROR_TRIAGE",
                    "ERROR_LOG",
                    &error_text,
                    &[eval],
                    &v,
                    &s,
                );
            }
        }

        ui::intent::InferredIntent::UniversalDeliberation {
            prompt,
            context_payload,
            context_type,
        } => {
            NervTheme::print_banner();
            let id = db_client
                .create_deliberation(
                    &config.author,
                    "UNIVERSAL",
                    "Universal Deliberation",
                    &prompt,
                    &context_type,
                    &context_payload,
                    "ALL",
                )
                .await?;

            NervTheme::print_deliberation_header(id, "Deliberation Session", &config.author);
            let evals = orchestrator
                .deliberate_idea(&prompt, &context_payload)
                .await?;

            db_client.submit_evaluations(id, &evals).await?;

            NervTheme::render_votes_table(&evals);
            let (verdict, summary) = resolve_consensus(&db_client, id, &evals).await;
            NervTheme::render_verdict(&verdict, &summary);

            ui::save_host_deliberation_report(
                id,
                "Universal Deliberation",
                "UNIVERSAL",
                &context_type,
                &context_payload,
                &evals,
                &verdict,
                &summary,
            );
        }
    }

    Ok(())
}
