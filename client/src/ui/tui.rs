//! # Interactive NERV Terminal UI
//!
//! Interactive command console that understands natural language intent,
//! communicates with SpacetimeDB, and coordinates the MAGI Trinity without requiring
//! strict CLI flags.

use crate::config::MagiConfig;
use crate::core::MagiOrchestrator;
use crate::db::SpacetimeClient;
use crate::ui::intent::{process_user_intent, InferredIntent};
use crate::ui::nerv_theme::NervTheme;
use colored::*;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use std::fs;
use std::io;
use std::time::Duration;

/// Runs the event-driven NERV terminal interface.
pub async fn run_interactive_session(
    config: MagiConfig,
    is_mock: bool,
    custom_skill: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let db_client = SpacetimeClient::new(
        config.spacetimedb_uri.clone(),
        config.spacetimedb_database.clone(),
    );

    let orchestrator =
        MagiOrchestrator::new(config.clone(), is_mock)?.with_custom_skill(custom_skill);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut input = String::new();
    let mut last_event = "READY: awaiting MAGI input".to_string();
    let mut transcript = Vec::new();

    loop {
        terminal.draw(|frame| render_tui(frame, &config, &input, &last_event, &transcript))?;
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char(character) => input.push(character),
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Esc => break,
            KeyCode::Enter => {
                let trimmed = input.trim().to_string();
                input.clear();
                if trimmed.is_empty() {
                    continue;
                }
                let intent = process_user_intent(&trimmed);

                if let InferredIntent::SystemCommand(ref cmd) = intent {
                    match cmd.as_str() {
                        "exit" => {
                            break;
                        }
                        "clear" => {
                            transcript.clear();
                            last_event = "Console transcript cleared".to_string();
                        }
                        "history" => {
                            match db_client.list_history(10).await {
                                Ok(records) => {
                                    transcript.push("┌── RECENT SPACETIMEDB DELIBERATION RECORDS ──────────────".to_string());
                                    if records.is_empty() {
                                        transcript
                                            .push("│ (No deliberation records found)".to_string());
                                    } else {
                                        for r in &records {
                                            let short_title = if r.title.len() > 28 {
                                                format!("{}...", &r.title[..25])
                                            } else {
                                                r.title.clone()
                                            };
                                            transcript.push(format!(
                                                "│ MAGI-{:06} | {:<28} | {:<14} | [{}]",
                                                r.id, short_title, r.context_type, r.status
                                            ));
                                        }
                                    }
                                    transcript.push("└─────────────────────────────────────────────────────────".to_string());
                                }
                                Err(e) => {
                                    transcript
                                        .push(format!("[ERROR] Failed to fetch history: {}", e));
                                }
                            }
                            last_event = "History fetched".to_string();
                        }
                        "status" => {
                            match db_client.check_health().await {
                                Ok(true) => {
                                    transcript.push(format!(
                                        "[STATUS] SpacetimeDB engine: ONLINE / PERSISTENT (URI: {})",
                                        config.spacetimedb_uri
                                    ));
                                    transcript.push(format!(
                                        "         Database: {} | Active Nodes: Melchior-1, Balthasar-2, Casper-3",
                                        config.spacetimedb_database
                                    ));
                                }
                                Ok(false) => {
                                    transcript.push(
                                        "[STATUS] SpacetimeDB engine: UNINITIALIZED".to_string(),
                                    );
                                }
                                Err(e) => {
                                    transcript.push(format!(
                                        "[STATUS] SpacetimeDB connection error: {}",
                                        e
                                    ));
                                }
                            }
                            last_event = "Status check complete".to_string();
                        }
                        "help" => {
                            transcript.push(
                                "┌── MAGI NERV CONSOLE GUIDE ──────────────────────────────"
                                    .to_string(),
                            );
                            transcript.push(
                                "│ • status      : Ping SpacetimeDB engine and check connection"
                                    .to_string(),
                            );
                            transcript.push(
                                "│ • history     : Display recent deliberation records".to_string(),
                            );
                            transcript.push(
                                "│ • clear       : Clear console transcript buffer".to_string(),
                            );
                            transcript
                                .push("│ • exit / quit : Leave NERV Command Deck".to_string());
                            transcript.push("│ • <query>     : Deliberate on proposal, code, incident, or prompt:".to_string());
                            transcript.push("│     - 'revisa docs/IDEA.md'".to_string());
                            transcript.push(
                                "│     - 'panic: connection pool index out of bounds'".to_string(),
                            );
                            transcript.push(
                                "│     - '¿Es conveniente migrar a microservicios en Rust?'"
                                    .to_string(),
                            );
                            transcript.push(
                                "└─────────────────────────────────────────────────────────"
                                    .to_string(),
                            );
                            last_event = "Help displayed".to_string();
                        }
                        _ => {}
                    }
                    continue;
                }

                // Leave alternate screen to render full Evangelion terminal theme for deliberation
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                match intent {
                    InferredIntent::SystemCommand(_) => {}

                    InferredIntent::IdeaAssessment { path, question } => {
                        println!(
                            "{} Evaluating proposal [{}]",
                            "[INTENT: IDEA VIABILITY]".on_blue().bright_white().bold(),
                            path.display().to_string().bright_cyan()
                        );
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
                        let (verdict, summary) =
                            resolve_tui_consensus(&db_client, id, &evals).await;
                        NervTheme::render_verdict(&verdict, &summary);
                        record_result(&mut transcript, &verdict, &summary, &evals);

                        crate::ui::save_host_deliberation_report(
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

                    InferredIntent::CodeMaintenance {
                        code_path,
                        guidelines_path,
                        instructions,
                    } => {
                        println!(
                            "{} Auditing [{}]",
                            "[INTENT: CODE MAINTENANCE]".on_cyan().black().bold(),
                            code_path.display().to_string().bright_yellow()
                        );
                        let code_content = fs::read_to_string(&code_path)?;
                        let guidelines_content = if let Some(ref g) = guidelines_path {
                            fs::read_to_string(g).unwrap_or_default()
                        } else {
                            "Maintain code quality, prevent regressions, and avoid overengineering."
                                .to_string()
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
                            .deliberate_maintenance(
                                &code_content,
                                &guidelines_content,
                                &instructions,
                            )
                            .await?;

                        db_client.submit_evaluations(id, &evals).await?;

                        NervTheme::render_votes_table(&evals);
                        let (verdict, summary) =
                            resolve_tui_consensus(&db_client, id, &evals).await;
                        NervTheme::render_verdict(&verdict, &summary);
                        record_result(&mut transcript, &verdict, &summary, &evals);

                        let combined = format!(
                            "CODE:\n{}\n\nGUIDELINES:\n{}",
                            code_content, guidelines_content
                        );
                        crate::ui::save_host_deliberation_report(
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

                    InferredIntent::ErrorTriage {
                        error_text,
                        code_context,
                    } => {
                        let lead_node =
                            crate::core::MagiOrchestrator::select_lead_node_for_error(&error_text);
                        println!(
                            "{} Routing incident to Specialist Node [{}]",
                            "[INTENT: TARGETED TRIAGE]"
                                .on_magenta()
                                .bright_white()
                                .bold(),
                            lead_node.bright_yellow().bold()
                        );

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
                        NervTheme::render_triage_result(&lead, &eval);

                        if let Some(trinity_evals) = escalation {
                            db_client.submit_evaluations(id, &trinity_evals).await?;
                            println!(
                                "{}",
                                "Mandatory full Trinity incident deliberation:"
                                    .bright_yellow()
                                    .bold()
                            );
                            NervTheme::render_votes_table(&trinity_evals);
                            let (verdict, summary) =
                                resolve_tui_consensus(&db_client, id, &trinity_evals).await;
                            NervTheme::render_verdict(&verdict, &summary);
                            record_result(&mut transcript, &verdict, &summary, &trinity_evals);

                            crate::ui::save_host_deliberation_report(
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
                            crate::ui::save_host_deliberation_report(
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

                    InferredIntent::UniversalDeliberation {
                        prompt,
                        context_payload,
                        context_type,
                    } => {
                        println!(
                            "{} Initiating Trinity consensus on query",
                            "[INTENT: UNIVERSAL DELIBERATION]".on_green().black().bold()
                        );

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

                        NervTheme::print_deliberation_header(
                            id,
                            "Deliberation Session",
                            &config.author,
                        );
                        let evals = orchestrator
                            .deliberate_idea(&prompt, &context_payload)
                            .await?;

                        db_client.submit_evaluations(id, &evals).await?;

                        NervTheme::render_votes_table(&evals);
                        let (verdict, summary) =
                            resolve_tui_consensus(&db_client, id, &evals).await;
                        NervTheme::render_verdict(&verdict, &summary);
                        record_result(&mut transcript, &verdict, &summary, &evals);

                        crate::ui::save_host_deliberation_report(
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

                println!();
                println!(
                    "{}",
                    "  Press [ENTER] to return to NERV Command Deck..."
                        .truecolor(255, 174, 66)
                        .bold()
                );
                let mut pause_buf = String::new();
                let _ = std::io::stdin().read_line(&mut pause_buf);

                last_event = format!("COMPLETED: {}", trimmed);
                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen)?;
            }
            _ => {}
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn render_tui(
    frame: &mut Frame,
    config: &MagiConfig,
    input: &str,
    last_event: &str,
    transcript: &[String],
) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            "MAGI SYSTEM // NERV COMMAND DECK",
            Style::default().fg(Color::Red),
        )),
        Line::from(format!(
            "MELCHIOR-1: {}    BALTHASAR-2: {}    CASPER-3: {}",
            config.melchior.model, config.balthasar.model, config.casper.model
        )),
        Line::from("Two-round deliberation: opening positions -> peer debate -> Trinity verdict"),
    ])
    .block(Block::default().title("STATUS").borders(Borders::ALL))
    .wrap(Wrap { trim: true });
    frame.render_widget(header, areas[0]);

    let mut body_lines = vec![
        Line::from("Enter a natural-language request, file path, incident, or command."),
        Line::from("Commands: help | history | status | clear | exit"),
        Line::from("The designated triage specialist opens the case; the Trinity always decides."),
        Line::from(format!("Last event: {}", last_event)),
    ];
    body_lines.extend(transcript.iter().map(|entry| Line::from(entry.as_str())));
    let visible_body_height = areas[1].height.saturating_sub(2) as usize;
    let body_scroll = body_lines.len().saturating_sub(visible_body_height) as u16;
    let body = Paragraph::new(body_lines)
        .block(Block::default().title("NERV CONSOLE").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((body_scroll, 0));
    frame.render_widget(body, areas[1]);

    let prompt = Paragraph::new(Line::from(vec![
        Span::styled("MAGI::NERV> ", Style::default().fg(Color::Red)),
        Span::raw(input),
    ]))
    .block(Block::default().title("INPUT").borders(Borders::ALL));
    frame.render_widget(prompt, areas[2]);
    frame.set_cursor_position((areas[2].x + 12 + input.len() as u16, areas[2].y + 1));
}

fn record_result(
    transcript: &mut Vec<String>,
    verdict: &str,
    summary: &str,
    evaluations: &[crate::llm::NodeEvaluation],
) {
    transcript.push("──────────────────────────────────────────────────────────".to_string());
    transcript.push(format!("▶ VEREDICTO DE LA TRINIDAD: {}", verdict));
    transcript.push(format!("▶ SÍNTESIS: {}", summary));
    for evaluation in evaluations {
        transcript.push(format!(
            "  • {} : {} (Riesgo: {}/10)",
            evaluation.node_id, evaluation.vote, evaluation.risk_score
        ));
    }
    transcript.push("──────────────────────────────────────────────────────────".to_string());
    if transcript.len() > 100 {
        transcript.drain(..transcript.len() - 100);
    }
}

async fn resolve_tui_consensus(
    db_client: &SpacetimeClient,
    deliberation_id: u64,
    _evaluations: &[crate::llm::NodeEvaluation],
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
