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
mod i18n;
mod llm;
mod mcp;
mod skills;
mod ui;

use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use config::MagiConfig;
use core::MagiOrchestrator;
use db::SpacetimeClient;
use error::MagiError;
use skills::PromptLoader;
use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::time::Duration;
use ui::NervTheme;

// ── Semantic exit codes ───────────────────────────────────────────────────────
/// Process exits with this code when the Trinity reaches an APPROVED verdict.
const EXIT_APPROVED: i32 = 0;
/// Process exits with this code when the Trinity reaches a REJECTED verdict.
const EXIT_REJECTED: i32 = 1;
/// Process exits with this code when the Trinity is split (NEUTRAL / no majority).
const EXIT_SPLIT: i32 = 2;
/// Process exits with this code on any runtime or configuration error.
const EXIT_ERROR: i32 = 3;

/// Output format selection for deliberation results.
#[derive(ValueEnum, Debug, Clone, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable NERV terminal output (default).
    #[default]
    Terminal,
    /// Machine-readable JSON printed to stdout (suitable for piping / CI).
    Json,
}

/// CLI argument parser for MAGI System.
#[derive(Parser, Debug)]
#[command(name = "magi")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable mock mode for testing without requiring external LLM API keys
    #[arg(long, global = true)]
    mock: bool,

    /// Optional path to custom skill or instructions file injected into the Trinity
    #[arg(long, global = true)]
    skill: Option<PathBuf>,

    /// Override system language ('en' or 'es')
    #[arg(long, global = true, value_name = "LANG")]
    lang: Option<String>,

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

        /// Number of debate rounds (minimum 2: one initial + one final)
        #[arg(long, default_value_t = 2, value_name = "N")]
        rounds: u8,

        /// Output format: terminal (default) or json
        #[arg(long, default_value = "terminal")]
        output: OutputFormat,

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

        /// Path to the guidelines document or rules file (e.g. GUIDELINES.md, RULES.md)
        #[arg(short, long, value_name = "GUIDELINES_FILE")]
        guidelines: PathBuf,

        /// Specific maintenance instructions or questions
        #[arg(
            short,
            long,
            default_value = "Evaluate how to maintain and evolve this code under these guidelines."
        )]
        prompt: String,

        /// Number of debate rounds (minimum 2: one initial + one final)
        #[arg(long, default_value_t = 2, value_name = "N")]
        rounds: u8,

        /// Output format: terminal (default) or json
        #[arg(long, default_value = "terminal")]
        output: OutputFormat,

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

        /// Number of debate rounds for the Trinity verdict phase (minimum 2)
        #[arg(long, default_value_t = 2, value_name = "N")]
        rounds: u8,

        /// Output format: terminal (default) or json
        #[arg(long, default_value = "terminal")]
        output: OutputFormat,

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

        /// Number of debate rounds (minimum 2: one initial + one final)
        #[arg(long, default_value_t = 2, value_name = "N")]
        rounds: u8,

        /// Output format: terminal (default) or json
        #[arg(long, default_value = "terminal")]
        output: OutputFormat,

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

    /// View deliberation history stored in SpacetimeDB and local archive
    History {
        /// Maximum number of records to retrieve
        #[arg(short, long, default_value_t = 50)]
        limit: usize,

        /// Render as static ANSI table without opening the interactive TUI browser
        #[arg(long)]
        table: bool,

        /// Search query to pre-filter deliberations by keyword
        #[arg(short, long)]
        query: Option<String>,
    },

    /// View detailed deliberation record and node debate by ID
    Show {
        /// ID of the deliberation to inspect
        #[arg(value_name = "DELIBERATION_ID")]
        id: u64,
    },

    /// Purge deliberation records from local archive
    Purge {
        /// Keep markdown reports in deliberations/ directory
        #[arg(long)]
        keep_files: bool,
    },

    /// Case 4: Audit repository git diff (uncommitted, staged, or against a target branch)
    Diff {
        /// Audit only staged changes (`git diff --staged`)
        #[arg(long)]
        staged: bool,

        /// Target git reference or branch to compare against (e.g. `origin/main`)
        #[arg(long, value_name = "REF")]
        branch: Option<String>,

        /// Optional path to guidelines or rules file (e.g. GUIDELINES.md)
        #[arg(short, long, value_name = "GUIDELINES_FILE")]
        guidelines: Option<PathBuf>,

        /// Specific audit instruction or question for the nodes
        #[arg(
            short,
            long,
            default_value = "Audit these git changes for architectural quality, regressions, security risks, and overengineering."
        )]
        prompt: String,

        /// Number of debate rounds (minimum 2: one initial + one final)
        #[arg(long, default_value_t = 2, value_name = "N")]
        rounds: u8,

        /// Output format: terminal (default) or json
        #[arg(long, default_value = "terminal")]
        output: OutputFormat,

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

    /// Case 5: Deliberate a technical dilemma, architectural decision, or technology choice
    Debate {
        /// Technical dilemma or question to debate (e.g. "WebSockets vs SSE for real-time notifications")
        #[arg(value_name = "DILEMMA")]
        query: String,

        /// Optional path to reference context or RFC document
        #[arg(short, long, value_name = "CONTEXT_FILE")]
        context: Option<PathBuf>,

        /// Number of debate rounds (minimum 2: one initial + one final)
        #[arg(long, default_value_t = 2, value_name = "N")]
        rounds: u8,

        /// Output format: terminal (default) or json
        #[arg(long, default_value = "terminal")]
        output: OutputFormat,

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

    /// Run as Model Context Protocol (MCP) server over stdio
    Mcp,

    /// Verify connectivity to the SpacetimeDB engine
    Status,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let exit_code = run().await;
    std::process::exit(exit_code);
}

async fn run() -> i32 {
    let cli = Cli::parse();

    if let Some(ref l) = cli.lang {
        std::env::set_var("MAGI_LANG", l);
    }

    let mut config = match MagiConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} {}", "MAGI CONFIG ERROR:".bright_red().bold(), e);
            return EXIT_ERROR;
        }
    };

    let is_mock = cli.mock
        || (config.melchior.api_key.is_none()
            && config.balthasar.api_key.is_none()
            && config.casper.api_key.is_none());

    let custom_skill_content = if let Some(ref skill_path) = cli.skill {
        match PromptLoader::load_custom_skill(skill_path) {
            Ok(content) => {
                println!(
                    "{} Injected custom skill from [{}]",
                    "MAGI SKILL:".bright_magenta().bold(),
                    skill_path.display()
                );
                Some(content)
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to load custom skill from '{}': {}",
                    "MAGI ERROR:".bright_red().bold(),
                    skill_path.display(),
                    e
                );
                return EXIT_ERROR;
            }
        }
    } else {
        None
    };

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            if cli.query.is_empty() {
                return match ui::run_interactive_session(config, is_mock, custom_skill_content)
                    .await
                {
                    Ok(()) => EXIT_APPROVED,
                    Err(e) => {
                        eprintln!("{} {}", "MAGI ERROR:".bright_red().bold(), e);
                        EXIT_ERROR
                    }
                };
            } else {
                let query_str = cli.query.join(" ");
                let intent = ui::intent::process_user_intent(&query_str);
                return execute_inferred_intent(intent, config, is_mock, custom_skill_content)
                    .await;
            }
        }
    };

    match dispatch_command(command, &mut config, is_mock, custom_skill_content).await {
        Ok(verdict) => verdict_to_exit_code(&verdict),
        Err(e) => {
            eprintln!("{} {}", "MAGI ERROR:".bright_red().bold(), e);
            EXIT_ERROR
        }
    }
}

// ── Command dispatcher ────────────────────────────────────────────────────────

/// Dispatches a parsed command and returns the consensus verdict string.
async fn dispatch_command(
    command: Commands,
    config: &mut MagiConfig,
    is_mock: bool,
    custom_skill_content: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    match command {
        Commands::Interactive => {
            ui::run_interactive_session(config.clone(), is_mock, custom_skill_content).await?;
            Ok("APPROVED".to_string())
        }

        Commands::Mcp => {
            crate::mcp::run_stdio_server(config.clone(), custom_skill_content, is_mock).await?;
            Ok("APPROVED".to_string())
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
            Ok("APPROVED".to_string())
        }

        Commands::History {
            limit,
            table,
            query,
        } => {
            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );
            let deliberations_dir = crate::ui::report::get_deliberations_dir();
            let records = crate::ui::helpers::history_loader::load_hybrid(
                Some(&db_client),
                &deliberations_dir,
                limit,
            )
            .await;

            let is_interactive = !table && std::io::stdout().is_terminal();

            if is_interactive {
                crate::ui::run_history_browser(&records, query.as_deref())
                    .map_err(|e| MagiError::Internal(e.to_string()))?;
            } else {
                NervTheme::print_banner();
                println!(
                    "MAGI ARCHIVE // DELIBERATION HISTORY ({} records loaded):",
                    records.len()
                );
                let filtered = if let Some(ref q) = query {
                    let q_lower = q.to_lowercase();
                    records
                        .into_iter()
                        .filter(|r| {
                            r.title.to_lowercase().contains(&q_lower)
                                || r.verdict.to_lowercase().contains(&q_lower)
                                || r.category.to_lowercase().contains(&q_lower)
                        })
                        .collect()
                } else {
                    records
                };
                crate::ui::render_history_table(&filtered);
            }
            Ok("APPROVED".to_string())
        }

        Commands::Show { id } => {
            NervTheme::print_banner();
            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );

            // 1. Try SpacetimeDB first
            let mut db_found = false;
            match db_client.get_deliberation_details(id).await {
                Ok(Some((delib, evals, consensus))) => {
                    db_found = true;
                    NervTheme::print_deliberation_header(delib.id, &delib.title, &delib.author);
                    println!("PROMPT: {}", delib.prompt);
                    println!("CONTEXT TYPE: {}", delib.context_type);
                    println!("STATUS: {}", delib.status);
                    if !evals.is_empty() {
                        NervTheme::render_votes_table(&evals);
                    }
                    if let Some(c) = consensus {
                        NervTheme::render_verdict(&c.verdict, &c.summary);
                        return Ok(c.verdict);
                    }
                }
                Ok(None) => {}
                Err(_) => {
                    eprintln!("Notice: SpacetimeDB unreachable, searching local archive...");
                }
            }

            // 2. Fallback to local deliberations/ directory
            if !db_found {
                let deliberations_dir = crate::ui::report::get_deliberations_dir();
                let disk_records =
                    crate::ui::helpers::history_loader::load_from_disk(&deliberations_dir);
                if let Some(entry) = disk_records.into_iter().find(|e| e.id == id) {
                    println!(
                        "{}",
                        format!(
                            "  [*] [MAGI ARCHIVE RECORD: CASE #{:04}] {}",
                            entry.id, entry.title
                        )
                        .bright_yellow()
                        .bold()
                    );
                    println!("  CATEGORY: {}", entry.category);
                    println!("  CONTEXT TYPE: {}", entry.context_type);
                    println!("  STATUS: {}", entry.status);
                    if let Some(ref path) = entry.file_path {
                        println!("  REPORT FILE: {}", path.display());
                    }
                    println!();
                    if !entry.node_votes.is_empty() {
                        println!("  THE TRINITY VOTES & POSITIONS:");
                        for n in &entry.node_votes {
                            println!(
                                "    • {:<12} : [{}] (Risk: {}/10) - {}",
                                n.node_id, n.vote, n.risk_score, n.argument
                            );
                        }
                        println!();
                    }
                    NervTheme::render_verdict(&entry.verdict, &entry.summary);
                    return Ok(entry.verdict);
                } else {
                    println!(
                        "Deliberation #{} not found in SpacetimeDB or local archive.",
                        id
                    );
                }
            }

            Ok("NEUTRAL".to_string())
        }

        Commands::Purge { keep_files } => {
            NervTheme::print_banner();
            println!(
                "{}",
                "MAGI ARCHIVE PURGE // RECLAIMING STATE & MEMORY"
                    .bright_yellow()
                    .bold()
            );

            if !keep_files {
                let delib_dir = crate::ui::report::get_deliberations_dir();
                if delib_dir.exists() {
                    let mut count = 0;
                    if let Ok(entries) = fs::read_dir(&delib_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                                if let Ok(()) = fs::remove_file(path) {
                                    count += 1;
                                }
                            }
                        }
                    }
                    println!(
                        "  [*] Purged {} local markdown report(s) from {}",
                        count,
                        delib_dir.display()
                    );
                }
            }

            println!("  [*] Notice: to fully drop and re-publish the SpacetimeDB database module, run '.\\magi.ps1 purge'.");
            println!(
                "{}",
                "✔ Local deliberation archive successfully purged!".bright_green()
            );
            Ok("APPROVED".to_string())
        }

        // =====================================================================
        // CASE 1: IDEA / MARKDOWN VIABILITY REVIEW
        // =====================================================================
        Commands::Idea {
            path,
            prompt,
            rounds,
            output,
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

            let use_mock = mock
                || (config.melchior.api_key.is_none()
                    && config.balthasar.api_key.is_none()
                    && config.casper.api_key.is_none());
            let orchestrator = MagiOrchestrator::new(config.clone(), use_mock)?
                .with_custom_skill(custom_skill_content.clone());

            let eval_content = if simulate_veto {
                format!(
                    "{}\n## Security Warning\nHigh risk data leak and hardcoded credentials detected.",
                    markdown_content
                )
            } else {
                markdown_content
            };

            let evaluations = orchestrator
                .deliberate_idea(&prompt, &eval_content, rounds)
                .await?;

            db_client
                .submit_evaluations(deliberation_id, &evaluations)
                .await?;

            let consensus_result =
                resolve_consensus(&db_client, deliberation_id, &evaluations).await;

            match output {
                OutputFormat::Json => {
                    let json = ui::JsonOutput::build(
                        deliberation_id,
                        &title,
                        "IDEA_PROPOSAL",
                        "MARKDOWN",
                        &consensus_result.0,
                        &consensus_result.1,
                        rounds,
                        &evaluations,
                    );
                    json.print();
                }
                OutputFormat::Terminal => {
                    NervTheme::render_votes_table(&evaluations);
                    NervTheme::render_verdict(&consensus_result.0, &consensus_result.1);
                }
            }

            if !use_mock {
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

            Ok(consensus_result.0)
        }

        // =====================================================================
        // CASE 2: CODE MAINTENANCE & GUIDELINES COMPLIANCE
        // =====================================================================
        Commands::Maintain {
            path,
            guidelines,
            prompt,
            rounds,
            output,
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

            let use_mock = mock
                || (config.melchior.api_key.is_none()
                    && config.balthasar.api_key.is_none()
                    && config.casper.api_key.is_none());
            let orchestrator = MagiOrchestrator::new(config.clone(), use_mock)?
                .with_custom_skill(custom_skill_content.clone());

            if let Some(ctx) = orchestrator.project_context() {
                if output == OutputFormat::Terminal {
                    println!(
                        "{} {}",
                        "MAGI DISCOVERY:".bright_cyan().bold(),
                        ctx.summary().bright_yellow()
                    );
                }
            }

            let eval_content = if simulate_veto {
                format!(
                    "{}\n// format!(\"SELECT * FROM users WHERE token = '\" + token + \"'\");",
                    code_content
                )
            } else {
                code_content.clone()
            };

            let evaluations = orchestrator
                .deliberate_maintenance(&eval_content, &guidelines_content, &prompt, rounds)
                .await?;

            db_client
                .submit_evaluations(deliberation_id, &evaluations)
                .await?;

            let consensus_result =
                resolve_consensus(&db_client, deliberation_id, &evaluations).await;

            match output {
                OutputFormat::Json => {
                    let json = ui::JsonOutput::build(
                        deliberation_id,
                        &title,
                        "CODE_MAINTENANCE",
                        "SOURCE_CODE",
                        &consensus_result.0,
                        &consensus_result.1,
                        rounds,
                        &evaluations,
                    );
                    json.print();
                }
                OutputFormat::Terminal => {
                    NervTheme::render_votes_table(&evaluations);
                    NervTheme::render_verdict(&consensus_result.0, &consensus_result.1);
                }
            }

            let combined_context = format!(
                "CODE:\n{}\n\nGUIDELINES:\n{}",
                code_content, guidelines_content
            );
            if !use_mock {
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

            Ok(consensus_result.0)
        }

        // =====================================================================
        // CASE 3: TARGETED ERROR / INCIDENT TRIAGE
        // =====================================================================
        Commands::Triage {
            error,
            code,
            rounds,
            output,
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

            let use_mock = mock
                || (config.melchior.api_key.is_none()
                    && config.balthasar.api_key.is_none()
                    && config.casper.api_key.is_none());
            let orchestrator = MagiOrchestrator::new(config.clone(), use_mock)?
                .with_custom_skill(custom_skill_content.clone());

            let (lead, eval, escalation) = orchestrator
                .triage_error(&error_text, code_content.as_deref(), rounds)
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

            if let Some(trinity_evals) = escalation {
                println!(
                    "{}",
                    "Opening mandatory full Trinity incident deliberation...".bright_yellow()
                );
                db_client
                    .submit_evaluations(deliberation_id, &trinity_evals)
                    .await?;

                let consensus =
                    resolve_consensus(&db_client, deliberation_id, &trinity_evals).await;

                match output {
                    OutputFormat::Json => {
                        let json = ui::JsonOutput::build(
                            deliberation_id,
                            "Error Incident Triage",
                            "ERROR_TRIAGE",
                            "ERROR_LOG",
                            &consensus.0,
                            &consensus.1,
                            rounds,
                            &trinity_evals,
                        );
                        json.print();
                    }
                    OutputFormat::Terminal => {
                        NervTheme::render_triage_result(&lead, &eval);
                        NervTheme::render_votes_table(&trinity_evals);
                        NervTheme::render_verdict(&consensus.0, &consensus.1);
                    }
                }

                if !use_mock {
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
                }

                Ok(consensus.0)
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

                match output {
                    OutputFormat::Json => {
                        let json = ui::JsonOutput::build(
                            deliberation_id,
                            "Incident Triage",
                            "ERROR_TRIAGE",
                            "ERROR_LOG",
                            &v,
                            &s,
                            rounds,
                            std::slice::from_ref(&eval),
                        );
                        json.print();
                    }
                    OutputFormat::Terminal => {
                        NervTheme::render_triage_result(&lead, &eval);
                    }
                }

                if !use_mock {
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

                Ok(v)
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
            rounds,
            output,
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

            let use_mock = mock
                || (config.melchior.api_key.is_none()
                    && config.balthasar.api_key.is_none()
                    && config.casper.api_key.is_none());
            let orchestrator = MagiOrchestrator::new(config.clone(), use_mock)?
                .with_custom_skill(custom_skill_content.clone());

            if let Some(ctx) = orchestrator.project_context() {
                if output == OutputFormat::Terminal {
                    println!(
                        "{} {}",
                        "MAGI DISCOVERY:".bright_cyan().bold(),
                        ctx.summary().bright_yellow()
                    );
                }
            }

            let eval_content = if simulate_veto {
                format!(
                    "{}\n// format!(\"SELECT * FROM users WHERE id = '\" + id + \"'\");",
                    file_content
                )
            } else {
                file_content
            };

            let evaluations = orchestrator
                .deliberate_idea(&prompt, &eval_content, rounds)
                .await?;

            db_client
                .submit_evaluations(deliberation_id, &evaluations)
                .await?;

            let consensus_result =
                resolve_consensus(&db_client, deliberation_id, &evaluations).await;

            match output {
                OutputFormat::Json => {
                    let json = ui::JsonOutput::build(
                        deliberation_id,
                        &file_title,
                        "UNIVERSAL_AUDIT",
                        &context_type,
                        &consensus_result.0,
                        &consensus_result.1,
                        rounds,
                        &evaluations,
                    );
                    json.print();
                }
                OutputFormat::Terminal => {
                    NervTheme::render_votes_table(&evaluations);
                    NervTheme::render_verdict(&consensus_result.0, &consensus_result.1);
                }
            }

            if !use_mock {
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

            Ok(consensus_result.0)
        }

        // =====================================================================
        // CASE 4: GIT DIFF AUDITING
        // =====================================================================
        Commands::Diff {
            staged,
            branch,
            guidelines,
            prompt,
            rounds,
            output,
            mock,
            simulate_veto,
            timeout,
        } => {
            if let Some(t) = timeout {
                config.timeout_seconds = t;
            }
            NervTheme::print_banner();

            let filtered_diff =
                crate::core::helpers::get_git_diff_filtered(staged, branch.as_deref()).map_err(
                    |e| MagiError::Internal(format!("Failed to retrieve git diff: {}", e)),
                )?;

            if filtered_diff.is_empty() {
                let msg = if staged {
                    "No staged git changes detected to audit (use 'git add <files>' first)."
                } else if let Some(ref b) = branch {
                    &format!("No git diff detected against reference '{}'.", b)
                } else {
                    "No uncommitted git changes detected to audit (working tree is clean)."
                };
                match output {
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::json!({
                                "status": "clean",
                                "message": msg,
                                "verdict": "APPROVED"
                            })
                        );
                    }
                    OutputFormat::Terminal => {
                        println!("{} {}", "MAGI GIT:".bright_cyan().bold(), msg.green());
                    }
                }
                return Ok("APPROVED".to_string());
            }

            if !filtered_diff.ignored_files.is_empty() && output == OutputFormat::Terminal {
                println!(
                    "{} Filtered {} noise/lockfile(s) ({} lines excluded): {}",
                    "MAGI SMART INGESTION:".bright_magenta().bold(),
                    filtered_diff
                        .ignored_files
                        .len()
                        .to_string()
                        .bright_yellow()
                        .bold(),
                    filtered_diff
                        .original_lines
                        .saturating_sub(filtered_diff.filtered_lines)
                        .to_string()
                        .bright_green()
                        .bold(),
                    filtered_diff.ignored_files.join(", ").dimmed()
                );
            }

            let diff_text = filtered_diff.content;

            let title = if staged {
                "Git Diff (Staged Changes)".to_string()
            } else if let Some(ref b) = branch {
                format!("Git Diff (vs {})", b)
            } else {
                "Git Diff (Working Tree)".to_string()
            };

            let guidelines_content = if let Some(ref g) = guidelines {
                Some(fs::read_to_string(g)?)
            } else {
                None
            };

            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );

            let deliberation_id = db_client
                .create_deliberation(
                    &config.author,
                    "GIT_DIFF_AUDIT",
                    &title,
                    &prompt,
                    "CODE_DIFF",
                    &diff_text,
                    "ALL",
                )
                .await?;

            NervTheme::print_deliberation_header(deliberation_id, &title, &config.author);

            let use_mock = mock
                || (config.melchior.api_key.is_none()
                    && config.balthasar.api_key.is_none()
                    && config.casper.api_key.is_none());
            let orchestrator = MagiOrchestrator::new(config.clone(), use_mock)?
                .with_custom_skill(custom_skill_content.clone());

            if let Some(ctx) = orchestrator.project_context() {
                if output == OutputFormat::Terminal {
                    println!(
                        "{} {}",
                        "MAGI DISCOVERY:".bright_cyan().bold(),
                        ctx.summary().bright_yellow()
                    );
                }
            }

            let eval_diff = if simulate_veto {
                format!(
                    "{}\n+// EMERGENCY VETO TEST: let secret_api_key = \"sk-admin-secret-unencrypted\";",
                    diff_text
                )
            } else {
                diff_text.clone()
            };

            let evaluations = if let Some(ref guide) = guidelines_content {
                orchestrator
                    .deliberate_maintenance(&eval_diff, guide, &prompt, rounds)
                    .await?
            } else {
                orchestrator
                    .deliberate_idea(&prompt, &eval_diff, rounds)
                    .await?
            };

            db_client
                .submit_evaluations(deliberation_id, &evaluations)
                .await?;

            let consensus_result =
                resolve_consensus(&db_client, deliberation_id, &evaluations).await;

            match output {
                OutputFormat::Json => {
                    let json = ui::JsonOutput::build(
                        deliberation_id,
                        &title,
                        "GIT_DIFF_AUDIT",
                        "CODE_DIFF",
                        &consensus_result.0,
                        &consensus_result.1,
                        rounds,
                        &evaluations,
                    );
                    json.print();
                }
                OutputFormat::Terminal => {
                    NervTheme::render_votes_table(&evaluations);
                    NervTheme::render_verdict(&consensus_result.0, &consensus_result.1);
                }
            }

            if !use_mock {
                ui::save_host_deliberation_report(
                    deliberation_id,
                    &title,
                    "GIT_DIFF_AUDIT",
                    "CODE_DIFF",
                    &diff_text,
                    &evaluations,
                    &consensus_result.0,
                    &consensus_result.1,
                );
            }

            Ok(consensus_result.0)
        }

        // =====================================================================
        // CASE 5: ARCHITECTURAL DEBATE & DILEMMA
        // =====================================================================
        Commands::Debate {
            query,
            context,
            rounds,
            output,
            mock,
            simulate_veto,
            timeout,
        } => {
            if let Some(t) = timeout {
                config.timeout_seconds = t;
            }
            NervTheme::print_banner();

            let context_content = if let Some(ref c) = context {
                Some(fs::read_to_string(c)?)
            } else {
                None
            };

            let title = format!(
                "Debate: {}",
                if query.len() > 60 {
                    format!("{}...", &query[..57])
                } else {
                    query.clone()
                }
            );

            let db_client = SpacetimeClient::new(
                config.spacetimedb_uri.clone(),
                config.spacetimedb_database.clone(),
            );

            let context_payload = context_content.as_deref().unwrap_or(&query);

            let deliberation_id = db_client
                .create_deliberation(
                    &config.author,
                    "TECHNICAL_DEBATE",
                    &title,
                    &query,
                    "DILEMMA",
                    context_payload,
                    "ALL",
                )
                .await?;

            NervTheme::print_deliberation_header(deliberation_id, &title, &config.author);

            let use_mock = mock
                || (config.melchior.api_key.is_none()
                    && config.balthasar.api_key.is_none()
                    && config.casper.api_key.is_none());
            let orchestrator = MagiOrchestrator::new(config.clone(), use_mock)?
                .with_custom_skill(custom_skill_content.clone());

            if let Some(ctx) = orchestrator.project_context() {
                if output == OutputFormat::Terminal {
                    println!(
                        "{} {}",
                        "MAGI DISCOVERY:".bright_cyan().bold(),
                        ctx.summary().bright_yellow()
                    );
                }
            }

            let eval_context = if simulate_veto {
                format!(
                    "{}\nNote: Critical security flaw: unrestricted arbitrary remote code execution accepted.",
                    context_payload
                )
            } else {
                context_payload.to_string()
            };

            let evaluations = orchestrator
                .deliberate_debate(&query, Some(&eval_context), rounds)
                .await?;

            db_client
                .submit_evaluations(deliberation_id, &evaluations)
                .await?;

            let consensus_result =
                resolve_consensus(&db_client, deliberation_id, &evaluations).await;

            match output {
                OutputFormat::Json => {
                    let json = ui::JsonOutput::build(
                        deliberation_id,
                        &title,
                        "TECHNICAL_DEBATE",
                        "DILEMMA",
                        &consensus_result.0,
                        &consensus_result.1,
                        rounds,
                        &evaluations,
                    );
                    json.print();
                }
                OutputFormat::Terminal => {
                    NervTheme::render_votes_table(&evaluations);
                    NervTheme::render_verdict(&consensus_result.0, &consensus_result.1);
                }
            }

            if !use_mock {
                ui::save_host_deliberation_report(
                    deliberation_id,
                    &title,
                    "TECHNICAL_DEBATE",
                    "DILEMMA",
                    context_payload,
                    &evaluations,
                    &consensus_result.0,
                    &consensus_result.1,
                );
            }

            Ok(consensus_result.0)
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Maps a consensus verdict string to a semantic exit code.
fn verdict_to_exit_code(verdict: &str) -> i32 {
    match verdict.to_uppercase().as_str() {
        "APPROVED" | "APPROVE" => EXIT_APPROVED,
        "REJECTED" | "REJECT" => EXIT_REJECTED,
        _ => EXIT_SPLIT,
    }
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
) -> i32 {
    match execute_inferred_intent_inner(intent, config, is_mock, custom_skill).await {
        Ok(verdict) => verdict_to_exit_code(&verdict),
        Err(e) => {
            eprintln!("{} {}", "MAGI ERROR:".bright_red().bold(), e);
            EXIT_ERROR
        }
    }
}

async fn execute_inferred_intent_inner(
    intent: ui::intent::InferredIntent,
    config: MagiConfig,
    is_mock: bool,
    custom_skill: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let db_client = SpacetimeClient::new(
        config.spacetimedb_uri.clone(),
        config.spacetimedb_database.clone(),
    );
    let orchestrator =
        MagiOrchestrator::new(config.clone(), is_mock)?.with_custom_skill(custom_skill);

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
                Ok("APPROVED".to_string())
            }
            "history" => {
                NervTheme::print_banner();
                let deliberations_dir = crate::ui::report::get_deliberations_dir();
                let records = crate::ui::helpers::history_loader::load_hybrid(
                    Some(&db_client),
                    &deliberations_dir,
                    20,
                )
                .await;
                crate::ui::render_history_table(&records);
                Ok("APPROVED".to_string())
            }
            _ => {
                println!("MAGI Operational Console. Type 'magi' for interactive session.");
                Ok("APPROVED".to_string())
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
            let evals = orchestrator.deliberate_idea(&question, &content, 2).await?;

            db_client.submit_evaluations(id, &evals).await?;

            NervTheme::render_votes_table(&evals);
            let (verdict, summary) = resolve_consensus(&db_client, id, &evals).await;
            NervTheme::render_verdict(&verdict, &summary);

            if !is_mock {
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
            Ok(verdict)
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
                .deliberate_maintenance(&code_content, &guidelines_content, &instructions, 2)
                .await?;

            db_client.submit_evaluations(id, &evals).await?;

            NervTheme::render_votes_table(&evals);
            let (verdict, summary) = resolve_consensus(&db_client, id, &evals).await;
            NervTheme::render_verdict(&verdict, &summary);

            let combined = format!(
                "CODE:\n{}\n\nGUIDELINES:\n{}",
                code_content, guidelines_content
            );
            if !is_mock {
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
            Ok(verdict)
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
                .triage_error(&error_text, code_content.as_deref(), 2)
                .await?;

            NervTheme::render_triage_result(&lead, &eval);

            if let Some(trinity_evals) = escalation {
                db_client.submit_evaluations(id, &trinity_evals).await?;
                println!("Specialist opening complete. Full Trinity consensus:");
                NervTheme::render_votes_table(&trinity_evals);
                let (verdict, summary) = resolve_consensus(&db_client, id, &trinity_evals).await;
                NervTheme::render_verdict(&verdict, &summary);

                if !is_mock {
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
                }
                Ok(verdict)
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
                if !is_mock {
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
                Ok(v)
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
                .deliberate_idea(&prompt, &context_payload, 2)
                .await?;

            db_client.submit_evaluations(id, &evals).await?;

            NervTheme::render_votes_table(&evals);
            let (verdict, summary) = resolve_consensus(&db_client, id, &evals).await;
            NervTheme::render_verdict(&verdict, &summary);

            if !is_mock {
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
            Ok(verdict)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_to_exit_code() {
        assert_eq!(verdict_to_exit_code("APPROVED"), EXIT_APPROVED);
        assert_eq!(verdict_to_exit_code("APPROVE"), EXIT_APPROVED);
        assert_eq!(verdict_to_exit_code("approved"), EXIT_APPROVED);
        assert_eq!(verdict_to_exit_code("REJECTED"), EXIT_REJECTED);
        assert_eq!(verdict_to_exit_code("REJECT"), EXIT_REJECTED);
        assert_eq!(verdict_to_exit_code("rejected"), EXIT_REJECTED);
        assert_eq!(verdict_to_exit_code("SPLIT"), EXIT_SPLIT);
        assert_eq!(verdict_to_exit_code("NEUTRAL"), EXIT_SPLIT);
        assert_eq!(verdict_to_exit_code("CONSENSUS_UNAVAILABLE"), EXIT_SPLIT);
    }

    #[test]
    fn test_cli_diff_subcommand_parse() {
        let cli = Cli::try_parse_from(["magi", "diff", "--staged"]).expect("parse diff");
        match cli.command {
            Some(Commands::Diff { staged, .. }) => assert!(staged),
            _ => panic!("Expected Commands::Diff"),
        }
    }

    #[test]
    fn test_cli_debate_subcommand_parse() {
        let cli =
            Cli::try_parse_from(["magi", "debate", "WebSockets vs SSE"]).expect("parse debate");
        match cli.command {
            Some(Commands::Debate { query, .. }) => assert_eq!(query, "WebSockets vs SSE"),
            _ => panic!("Expected Commands::Debate"),
        }
    }
}
