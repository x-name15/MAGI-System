//! # Interactive NERV Archive History Browser
//!
//! Dual-pane interactive terminal browser powered by Ratatui and Crossterm.
//! Allows operators to search, inspect, and read past Trinity deliberation records
//! and node positions with zero latency.

use crate::ui::helpers::history_loader::DeliberationHistoryEntry;
use crate::ui::helpers::layout_helper::safe_truncate_str;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color as TColor, ContentArrangement, Table as ComfyTable};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use std::io;
use std::time::Duration;

/// Launches the interactive dual-pane NERV history browser.
pub fn run_history_browser(
    entries: &[DeliberationHistoryEntry],
    initial_query: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if entries.is_empty() {
        println!("No past deliberation records found in SpacetimeDB or local archive.");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut selected_idx: usize = 0;
    let mut search_query = initial_query.unwrap_or("").to_string();
    let mut search_mode = false;
    let mut viewing_report = false;
    let mut report_scroll: u16 = 0;

    let res = run_browser_loop(
        &mut terminal,
        entries,
        &mut selected_idx,
        &mut search_query,
        &mut search_mode,
        &mut viewing_report,
        &mut report_scroll,
    );

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn filter_entries<'a>(
    entries: &'a [DeliberationHistoryEntry],
    query: &str,
) -> Vec<&'a DeliberationHistoryEntry> {
    if query.trim().is_empty() {
        return entries.iter().collect();
    }
    let q = query.to_lowercase();
    entries
        .iter()
        .filter(|e| {
            e.title.to_lowercase().contains(&q)
                || e.verdict.to_lowercase().contains(&q)
                || e.category.to_lowercase().contains(&q)
                || e.context_type.to_lowercase().contains(&q)
                || e.summary.to_lowercase().contains(&q)
                || format!("{:04}", e.id).contains(&q)
        })
        .collect()
}

fn run_browser_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    all_entries: &[DeliberationHistoryEntry],
    selected_idx: &mut usize,
    search_query: &mut String,
    search_mode: &mut bool,
    viewing_report: &mut bool,
    report_scroll: &mut u16,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let filtered = filter_entries(all_entries, search_query);
        if filtered.is_empty() {
            *selected_idx = 0;
        } else if *selected_idx >= filtered.len() {
            *selected_idx = filtered.len().saturating_sub(1);
        }

        let current_entry = filtered.get(*selected_idx).copied();

        terminal.draw(|f| {
            let state = BrowserUiState {
                filtered: &filtered,
                selected_idx: *selected_idx,
                selected_entry: current_entry,
                search_query,
                search_mode: *search_mode,
                viewing_report: *viewing_report,
                report_scroll: *report_scroll,
                total_count: all_entries.len(),
            };
            render_browser_ui(f, &state);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if *viewing_report {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                            *viewing_report = false;
                            *report_scroll = 0;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            *report_scroll = report_scroll.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            *report_scroll = report_scroll.saturating_add(1);
                        }
                        KeyCode::PageUp => {
                            *report_scroll = report_scroll.saturating_sub(15);
                        }
                        KeyCode::PageDown => {
                            *report_scroll = report_scroll.saturating_add(15);
                        }
                        _ => {}
                    }
                    continue;
                }

                if *search_mode {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter => {
                            *search_mode = false;
                        }
                        KeyCode::Backspace => {
                            search_query.pop();
                            *selected_idx = 0;
                        }
                        KeyCode::Char(c) => {
                            search_query.push(c);
                            *selected_idx = 0;
                        }
                        _ => {}
                    }
                    continue;
                }

                // Normal navigation mode
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('/') => {
                        *search_mode = true;
                    }
                    KeyCode::Char('c') => {
                        search_query.clear();
                        *selected_idx = 0;
                    }
                    KeyCode::Enter => {
                        if current_entry.is_some() {
                            *viewing_report = true;
                            *report_scroll = 0;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        *selected_idx = selected_idx.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !filtered.is_empty() && *selected_idx + 1 < filtered.len() {
                            *selected_idx += 1;
                        }
                    }
                    KeyCode::PageUp => {
                        *selected_idx = selected_idx.saturating_sub(8);
                    }
                    KeyCode::PageDown => {
                        if !filtered.is_empty() {
                            *selected_idx = (*selected_idx + 8).min(filtered.len() - 1);
                        }
                    }
                    KeyCode::Home => {
                        *selected_idx = 0;
                    }
                    KeyCode::End => {
                        if !filtered.is_empty() {
                            *selected_idx = filtered.len() - 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn verdict_style(verdict: &str) -> Style {
    let v = verdict.to_uppercase();
    if v.contains("VETO") {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if v.contains("UNANIMOUS") && v.contains("APPROVE") {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else if v.contains("APPROVE") {
        Style::default().fg(Color::Green)
    } else if v.contains("REJECT") {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if v.contains("SPLIT") {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Cyan)
    }
}

struct BrowserUiState<'a> {
    filtered: &'a [&'a DeliberationHistoryEntry],
    selected_idx: usize,
    selected_entry: Option<&'a DeliberationHistoryEntry>,
    search_query: &'a str,
    search_mode: bool,
    viewing_report: bool,
    report_scroll: u16,
    total_count: usize,
}

fn render_browser_ui(f: &mut Frame, state: &BrowserUiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Dual-pane content
            Constraint::Length(3), // Footer / Hotkeys
        ])
        .split(f.area());

    // 1. Header
    let search_indicator = if state.search_mode {
        format!(" [SEARCH: {}_]", state.search_query)
    } else if !state.search_query.is_empty() {
        format!(" [FILTER: {} (press 'c' to clear)]", state.search_query)
    } else {
        String::new()
    };

    let header_text = vec![Line::from(vec![
        Span::styled(
            "NERV CENTRAL DOGMA // ARCHIVE EXPLORER  ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "RECORDS: {}/{} MATCHED{}",
                state.filtered.len(),
                state.total_count,
                search_indicator
            ),
            Style::default().fg(Color::Cyan),
        ),
    ])];
    let header = Paragraph::new(header_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Left);
    f.render_widget(header, chunks[0]);

    // 2. Dual Pane Content (Left: List, Right: Details)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    // Left Pane: Deliberation List
    let mut list_lines = Vec::new();
    if state.filtered.is_empty() {
        list_lines.push(Line::from(Span::styled(
            "  No deliberations match search filter.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let visible_items = body_chunks[0].height.saturating_sub(2) as usize;
        let start_idx = if state.selected_idx >= visible_items {
            state.selected_idx.saturating_sub(visible_items / 2)
        } else {
            0
        };
        let end_idx = (start_idx + visible_items).min(state.filtered.len());

        for (i, &entry) in state
            .filtered
            .iter()
            .enumerate()
            .take(end_idx)
            .skip(start_idx)
        {
            let is_selected = i == state.selected_idx;
            let (badge, _) = entry.verdict_badge();

            let cursor = if is_selected { "▶ " } else { "  " };
            let case_id = format!("#{:04} ", entry.id);

            let row_style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let badge_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                verdict_style(&entry.verdict)
            };

            let title_max = body_chunks[0].width.saturating_sub(26).max(10) as usize;
            let short_title = if entry.title.chars().count() > title_max {
                format!(
                    "{}…",
                    safe_truncate_str(&entry.title, title_max.saturating_sub(1))
                )
            } else {
                entry.title.clone()
            };

            list_lines.push(Line::from(vec![
                Span::styled(cursor, row_style),
                Span::styled(case_id, Style::default().fg(Color::DarkGray)),
                Span::styled(format!("[{:<14}] ", badge), badge_style),
                Span::styled(short_title, row_style),
            ]));
        }
    }

    let list_panel = Paragraph::new(list_lines).block(
        Block::default()
            .title("DELIBERATION ARCHIVES")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list_panel, body_chunks[0]);

    // Right Pane: Active Deliberation Inspector
    let detail_panel = if let Some(entry) = state.selected_entry {
        let (badge, icon) = entry.verdict_badge();
        let mut detail_lines = vec![
            Line::from(vec![
                Span::styled(
                    format!("CASE #{:04}: ", entry.id),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &entry.title,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("CATEGORY: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.category, Style::default().fg(Color::Cyan)),
                Span::styled("  |  TYPE: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.context_type, Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("CONSENSUS: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} {}", icon, badge), verdict_style(&entry.verdict)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("SUMMARY: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if entry.summary.is_empty() {
                        "No summary recorded."
                    } else {
                        &entry.summary
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "── THE TRINITY VOTES & SPECIALIST POSITIONS ──",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        if entry.node_votes.is_empty() {
            detail_lines.push(Line::from(Span::styled(
                "  (No individual node votes recorded)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for node in &entry.node_votes {
                let node_color = match node.node_id.as_str() {
                    "Melchior-1" => Color::Cyan,
                    "Balthasar-2" => Color::Red,
                    "Casper-3" => Color::Yellow,
                    _ => Color::White,
                };

                let risk_color = if node.risk_score >= 8 {
                    Color::Red
                } else if node.risk_score >= 5 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                detail_lines.push(Line::from(vec![
                    Span::styled(
                        format!("• {:<12} : ", node.node_id),
                        Style::default().fg(node_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("[{}] ", node.vote), verdict_style(&node.vote)),
                    Span::styled(
                        format!("(Risk: {}/10)", node.risk_score),
                        Style::default().fg(risk_color),
                    ),
                ]));

                if !node.argument.is_empty() {
                    let preview = if node.argument.chars().count() > 140 {
                        format!("   {}…", safe_truncate_str(&node.argument, 137))
                    } else {
                        format!("   {}", node.argument)
                    };
                    detail_lines.push(Line::from(Span::styled(
                        preview,
                        Style::default().fg(Color::Gray),
                    )));
                }
            }
        }

        if let Some(ref path) = entry.file_path {
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![
                Span::styled("ARCHIVE PATH: ", Style::default().fg(Color::DarkGray)),
                Span::styled(path.display().to_string(), Style::default().fg(Color::Cyan)),
            ]));
        }

        Paragraph::new(detail_lines)
            .block(
                Block::default()
                    .title("DELIBERATION INSPECTOR")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: true })
    } else {
        Paragraph::new("Select a deliberation to view audit details.").block(
            Block::default()
                .title("DELIBERATION INSPECTOR")
                .borders(Borders::ALL),
        )
    };
    f.render_widget(detail_panel, body_chunks[1]);

    // 3. Footer
    let footer_text = if state.search_mode {
        Line::from(vec![
            Span::styled(
                "SEARCH MODE: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Type query to filter archives. Press [Enter] or [Esc] to lock."),
        ])
    } else {
        Line::from(vec![
            Span::styled("[↑/↓, j/k]", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  |  "),
            Span::styled("[/]", Style::default().fg(Color::Yellow)),
            Span::raw(" Filter  |  "),
            Span::styled("[c]", Style::default().fg(Color::Yellow)),
            Span::raw(" Clear  |  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Full Report  |  "),
            Span::styled("[q/Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit"),
        ])
    };

    let footer = Paragraph::new(footer_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, chunks[2]);

    // 4. Modal Full Report Viewer Overlay
    if state.viewing_report {
        if let Some(entry) = state.selected_entry {
            let area = centered_rect(88, 88, f.area());
            f.render_widget(Clear, area);

            let synthesized;
            let content = match entry.raw_content.as_deref() {
                Some(raw) if !raw.trim().is_empty() => raw,
                _ => {
                    synthesized = entry.get_or_synthesize_markdown();
                    &synthesized
                }
            };

            let report_lines: Vec<Line> = content
                .lines()
                .map(|l| {
                    if l.starts_with("# ") {
                        Line::from(Span::styled(
                            l,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else if l.starts_with("## ") {
                        Line::from(Span::styled(
                            l,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else if l.starts_with("### ") {
                        Line::from(Span::styled(
                            l,
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else if l.starts_with("- **") {
                        Line::from(Span::styled(l, Style::default().fg(Color::White)))
                    } else {
                        Line::from(Span::raw(l))
                    }
                })
                .collect();

            let report_widget = Paragraph::new(report_lines)
                .block(
                    Block::default()
                        .title(format!(
                            "FULL REPORT [CASE #{:04}] — [Esc/Enter/q] CLOSE | [↑/↓] SCROLL",
                            entry.id
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .wrap(Wrap { trim: false })
                .scroll((state.report_scroll, 0));

            f.render_widget(report_widget, area);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Renders a non-interactive, rich ANSI table for terminal piping or `--table` flag.
pub fn render_history_table(entries: &[DeliberationHistoryEntry]) {
    let mut table = ComfyTable::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("CASE")
                .fg(TColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new("VERDICT")
                .fg(TColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new("TARGET")
                .fg(TColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new("TYPE")
                .fg(TColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new("TRINITY (M / B / C)")
                .fg(TColor::Yellow)
                .add_attribute(Attribute::Bold),
        ]);

    for e in entries {
        let (badge, _) = e.verdict_badge();
        let verdict_cell = match badge {
            "VETO" => Cell::new("🛑 VETO").fg(TColor::Red),
            "APPROVED (3-0)" => Cell::new("● APPROVED (3-0)").fg(TColor::Green),
            "APPROVED (2-1)" => Cell::new("✓ APPROVED (2-1)").fg(TColor::Green),
            "REJECTED (3-0)" => Cell::new("❌ REJECTED (3-0)").fg(TColor::Red),
            "REJECTED (2-1)" => Cell::new("✗ REJECTED (2-1)").fg(TColor::Red),
            "SPLIT (1-1-1)" => Cell::new("⚖ SPLIT").fg(TColor::Yellow),
            _ => Cell::new(badge).fg(TColor::Cyan),
        };

        let trinity_summary = if e.node_votes.is_empty() {
            "-".to_string()
        } else {
            e.node_votes
                .iter()
                .map(|n| {
                    let v = match n.vote.as_str() {
                        "APPROVE" => "APP",
                        "REJECT" => "REJ",
                        _ => "NEU",
                    };
                    format!("{}({})", v, n.risk_score)
                })
                .collect::<Vec<_>>()
                .join(" / ")
        };

        table.add_row(vec![
            Cell::new(format!("MAGI-{:04}", e.id)).add_attribute(Attribute::Bold),
            verdict_cell,
            Cell::new(&e.title),
            Cell::new(&e.context_type).fg(TColor::Magenta),
            Cell::new(trinity_summary).fg(TColor::Cyan),
        ]);
    }

    println!("{table}");
    println!();
}
