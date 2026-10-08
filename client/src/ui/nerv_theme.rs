//! # NERV / MAGI terminal presentation
//!
//! A compact, diegetic-inspired terminal theme for the MAGI code-audit system.
//! Rendering only: deliberation logic and NodeEvaluation schema are unchanged.

use crate::llm::NodeEvaluation;
use colored::*;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use std::io::Write;

/// Terminal presentation helper inspired by the NERV command center.
pub struct NervTheme;

impl NervTheme {
    const WIDTH: usize = 78;

    fn rule() {
        println!("{}", "━".repeat(Self::WIDTH).truecolor(214, 92, 20));
    }

    fn section(label: &str) {
        println!(
            "{} {}",
            "▌".truecolor(255, 112, 24).bold(),
            label.truecolor(255, 174, 66).bold()
        );
    }

    fn node_role(node_id: &str) -> (&'static str, Color) {
        match node_id {
            "Melchior-1" => ("MELCHIOR-1 / SCIENTIST · ARCHITECTURE", Color::Yellow),
            "Balthasar-2" => ("BALTHASAR-2 / MOTHER · SECURITY / VETO", Color::Green),
            "Casper-3" => ("CASPER-3 / WOMAN · PRACTICALITY", Color::Cyan),
            _ => ("MAGI AUXILIARY EVALUATOR", Color::White),
        }
    }

    fn risk_color(score: u8) -> Color {
        if score >= 8 {
            Color::Red
        } else if score >= 5 {
            Color::Yellow
        } else {
            Color::Green
        }
    }

    fn to_colored(c: Color) -> colored::Color {
        match c {
            Color::Red => colored::Color::Red,
            Color::Green => colored::Color::Green,
            Color::Yellow => colored::Color::Yellow,
            Color::Cyan => colored::Color::Cyan,
            Color::Magenta => colored::Color::Magenta,
            Color::White => colored::Color::White,
            Color::DarkGrey => colored::Color::BrightBlack,
            _ => colored::Color::White,
        }
    }

    /// Prints a NERV-style system identification header.
    pub fn print_banner() {
        Self::rule();
        println!("{}", "NERV  //  MAGI SYSTEM".truecolor(255, 112, 24).bold());
        println!(
            "{}",
            "TACTICAL CODE AUDIT  ·  TRINITY CONSENSUS PROTOCOL"
                .truecolor(255, 174, 66)
                .bold()
        );
        println!(
            "{}",
            "CENTRAL DOGMA / SECURE COMPUTATION DIVISION".truecolor(160, 160, 160)
        );
        println!(
            "{}",
            "SYSTEM: ONLINE     MAGI NODES: 03     CONSENSUS CHANNEL: ENCRYPTED"
                .truecolor(130, 180, 130)
        );
        Self::rule();
        println!();
    }

    /// Prints the active deliberation target.
    pub fn print_deliberation_header(deliberation_id: u64, title: &str, author: &str) {
        Self::section("ACTIVE DELIBERATION / TARGET IDENTIFICATION");
        println!(
            "{} {}",
            "CASE ID".truecolor(160, 160, 160),
            format!("MAGI-{deliberation_id:06}")
                .truecolor(255, 210, 120)
                .bold()
        );
        println!(
            "{} {}",
            "TARGET".truecolor(160, 160, 160),
            title.white().bold()
        );
        println!(
            "{} {}",
            "OPERATOR".truecolor(160, 160, 160),
            author.truecolor(190, 190, 190)
        );
        println!(
            "{}",
            "──────────────────────────────────────────────────────────────────────────────"
                .truecolor(110, 70, 40)
        );
    }

    /// Calculates the terminal display width of a string considering full-width CJK characters.
    fn str_display_width(s: &str) -> usize {
        crate::ui::helpers::str_display_width(s)
    }

    /// Formats a single MAGI node into lines for a 25-column diegetic CRT monitor box with bloom and Round 1 override support.
    fn format_node_monitor(
        eval: &NodeEvaluation,
        is_veto: bool,
        bloom: bool,
        is_r1: bool,
    ) -> Vec<String> {
        crate::ui::helpers::format_node_monitor(eval, is_veto, bloom, is_r1)
    }

    /// Formats a single MAGI node in a CRT strobe / blackout state (inter-frame scanline blackout).
    fn format_blackout_monitor(eval: &NodeEvaluation) -> Vec<String> {
        crate::ui::helpers::format_blackout_monitor(eval)
    }

    /// Formats a single MAGI node into lines while in the active deliberating / Electric Cyan scanning state.
    fn format_deliberating_monitor(
        eval: &NodeEvaluation,
        sim_risk: usize,
        blink: bool,
    ) -> Vec<String> {
        crate::ui::helpers::format_deliberating_monitor(eval, sim_risk, blink)
    }

    /// Renders a 22-line Evangelion triangular MAGI layout with Balthasar-2 on top,
    /// pure geometric wireframe connecting lines, and Casper-3 + Melchior-1 at the base
    /// connected directly with a horizontal debate bridge line.
    fn render_triangular_screen(
        header: &str,
        lines_b: &[String],
        lines_c: &[String],
        lines_m: &[String],
        connector_color: (u8, u8, u8),
    ) {
        crate::ui::helpers::render_triangular_screen(
            header,
            lines_b,
            lines_c,
            lines_m,
            connector_color,
        )
    }

    /// Displays the three node evaluations as iconic Evangelion triangular CRT monitors
    /// animating through Round 1, inter-node cross debate, Round 2 resolution, and consensus.
    pub fn render_votes_table(evaluations: &[NodeEvaluation]) {
        let combined_text = evaluations
            .iter()
            .map(|e| e.argument.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let lang = crate::i18n::Language::detect(&combined_text);
        let bundle = crate::i18n::get_bundle(lang);

        Self::section(&bundle.ui.monitors_section);

        let m = evaluations.iter().find(|e| e.node_id == "Melchior-1");
        let b = evaluations.iter().find(|e| e.node_id == "Balthasar-2");
        let c = evaluations.iter().find(|e| e.node_id == "Casper-3");

        if let (Some(ev_m), Some(ev_b), Some(ev_c)) = (m, b, c) {
            use std::io::Write;

            println!();

            #[derive(Clone, Copy)]
            enum AnimState {
                Scan(usize, bool),
                Strobe,
                LockR1(bool),
                LockR2(bool),
            }

            let is_veto_m = false;
            let is_veto_b =
                ev_b.vote == "REJECT" && (ev_b.risk_score >= 8 || !ev_b.cwe_flags.is_empty());
            let is_veto_c = false;

            let render_node =
                |state: AnimState, eval: &NodeEvaluation, is_veto: bool| -> Vec<String> {
                    match state {
                        AnimState::Scan(sim_risk, blink) => {
                            Self::format_deliberating_monitor(eval, sim_risk, blink)
                        }
                        AnimState::Strobe => Self::format_blackout_monitor(eval),
                        AnimState::LockR1(bloom) => {
                            Self::format_node_monitor(eval, is_veto, bloom, true)
                        }
                        AnimState::LockR2(bloom) => {
                            Self::format_node_monitor(eval, is_veto, bloom, false)
                        }
                    }
                };

            let cyan_bus = (0, 200, 240);
            let amber_bus = (255, 174, 66);
            let strobe_bus = (255, 230, 100);
            let is_rejected_or_veto = is_veto_b || (ev_m.vote == "REJECT" && ev_b.vote == "REJECT");
            let consensus_bus = if is_rejected_or_veto {
                (255, 60, 60)
            } else {
                (80, 255, 80)
            };

            // 16-frame sequence capturing the exact triangular Evangelion deliberation lifecycle:
            // (State Balthasar, State Casper, State Melchior, Phase, BusColor, Delay ms)
            type DeliberationFrame = (AnimState, AnimState, AnimState, usize, (u8, u8, u8), u64);
            let frames: [DeliberationFrame; 16] = [
                // 0. Initial cyan scan
                (
                    AnimState::Scan(4, true),
                    AnimState::Scan(7, true),
                    AnimState::Scan(3, true),
                    0,
                    cyan_bus,
                    160,
                ),
                // 1. CRT flicker across bus
                (
                    AnimState::Scan(6, false),
                    AnimState::Strobe,
                    AnimState::Scan(5, false),
                    0,
                    cyan_bus,
                    80,
                ),
                // 2. Cyan scan resumes
                (
                    AnimState::Scan(3, true),
                    AnimState::Scan(8, true),
                    AnimState::Strobe,
                    0,
                    cyan_bus,
                    90,
                ),
                // 3. Fluctuating risk across nodes
                (
                    AnimState::Scan(8, false),
                    AnimState::Scan(5, false),
                    AnimState::Scan(9, false),
                    0,
                    cyan_bus,
                    140,
                ),
                // --- ROUND 1: INDEPENDENT EVALUATIONS LOCK ---
                // 4. Melchior locks R1
                (
                    AnimState::Scan(5, true),
                    AnimState::Scan(7, true),
                    AnimState::LockR1(true),
                    1,
                    amber_bus,
                    320,
                ),
                // 5. Balthasar locks R1
                (
                    AnimState::LockR1(true),
                    AnimState::Scan(8, false),
                    AnimState::LockR1(false),
                    1,
                    amber_bus,
                    320,
                ),
                // 6. Casper locks R1
                (
                    AnimState::LockR1(false),
                    AnimState::LockR1(true),
                    AnimState::LockR1(false),
                    1,
                    amber_bus,
                    380,
                ),
                // 7. Stabilized R1 display
                (
                    AnimState::LockR1(false),
                    AnimState::LockR1(false),
                    AnimState::LockR1(false),
                    1,
                    amber_bus,
                    700,
                ),
                // --- ROUND 2: PEER CROSS-DEBATE ACROSS MAGI BUS ---
                // 8. Inter-node debate initiates: bus strobe
                (
                    AnimState::Strobe,
                    AnimState::LockR1(false),
                    AnimState::Strobe,
                    2,
                    strobe_bus,
                    200,
                ),
                // 9. Arguments exchanged: Casper strobes, bus pulses
                (
                    AnimState::LockR1(false),
                    AnimState::Strobe,
                    AnimState::LockR1(false),
                    2,
                    strobe_bus,
                    200,
                ),
                // --- ROUND 2: FINAL POSITIONS LOCK ---
                // 10. Melchior locks final R2 stance
                (
                    AnimState::LockR1(false),
                    AnimState::LockR1(false),
                    AnimState::LockR2(true),
                    3,
                    amber_bus,
                    320,
                ),
                // 11. Balthasar locks final R2 stance
                (
                    AnimState::LockR2(true),
                    AnimState::LockR1(false),
                    AnimState::LockR2(false),
                    3,
                    amber_bus,
                    320,
                ),
                // 12. Casper locks final R2 stance
                (
                    AnimState::LockR2(false),
                    AnimState::LockR2(true),
                    AnimState::LockR2(false),
                    3,
                    amber_bus,
                    380,
                ),
                // 13. All 3 locked in R2
                (
                    AnimState::LockR2(false),
                    AnimState::LockR2(false),
                    AnimState::LockR2(false),
                    3,
                    amber_bus,
                    500,
                ),
                // --- TRINITY CONSENSUS RESOLUTION PULSE ---
                // 14. Synchronized bloom pulse across all 3 monitors & bus!
                (
                    AnimState::LockR2(true),
                    AnimState::LockR2(true),
                    AnimState::LockR2(true),
                    4,
                    consensus_bus,
                    350,
                ),
                // 15. Final stabilized consensus state
                (
                    AnimState::LockR2(false),
                    AnimState::LockR2(false),
                    AnimState::LockR2(false),
                    4,
                    consensus_bus,
                    0,
                ),
            ];

            for (idx, (st_b, st_c, st_m, phase, bus_col, delay)) in frames.iter().enumerate() {
                let lines_m = render_node(*st_m, ev_m, is_veto_m);
                let lines_b = render_node(*st_b, ev_b, is_veto_b);
                let lines_c = render_node(*st_c, ev_c, is_veto_c);

                let header_line = match *phase {
                    0 => format!(
                        "  {}  {}",
                        bundle.ui.phase_0_inquiry.truecolor(255, 174, 66).bold(),
                        bundle.ui.phase_0_resolution.truecolor(160, 130, 80)
                    ),
                    1 => format!(
                        "  {}  {}",
                        bundle.ui.phase_1_inquiry.truecolor(255, 195, 45).bold(),
                        bundle.ui.phase_1_resolution.truecolor(180, 150, 90)
                    ),
                    2 => format!(
                        "  {}  {}",
                        bundle.ui.phase_2_inquiry.truecolor(255, 230, 80).bold(),
                        bundle.ui.phase_2_resolution.truecolor(220, 180, 90)
                    ),
                    3 => format!(
                        "  {}  {}",
                        bundle.ui.phase_3_inquiry.truecolor(255, 195, 45).bold(),
                        bundle.ui.phase_3_resolution.truecolor(220, 190, 100)
                    ),
                    _ => {
                        let (res_r, res_g, res_b) = consensus_bus;
                        format!(
                            "  {}  {}",
                            bundle
                                .ui
                                .phase_complete_inquiry
                                .truecolor(255, 174, 66)
                                .bold(),
                            bundle
                                .ui
                                .phase_complete_resolution
                                .truecolor(res_r, res_g, res_b)
                                .bold()
                        )
                    }
                };

                if idx > 0 {
                    // Overwrite 22 lines in place (header + 9 Balthasar + 3 connector + 9 bottom monitors)
                    print!("\x1b[22A\r");
                }

                Self::render_triangular_screen(
                    &header_line,
                    &lines_b,
                    &lines_c,
                    &lines_m,
                    *bus_col,
                );

                let _ = std::io::stdout().flush();
                if *delay > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(*delay));
                }
            }
            println!();

            // Deliberate pause so the operator can view and appreciate the sealed MAGI consensus screen
            let _ = std::io::stdout().flush();
            std::thread::sleep(std::time::Duration::from_millis(2500));
        }

        // Detailed Node Diagnostic & Rationale Breakdown
        println!(
            "{}",
            bundle.ui.rationales_title.truecolor(255, 174, 66).bold()
        );
        println!(
            "{}",
            "──────────────────────────────────────────────────────────────────────────────"
                .truecolor(110, 70, 40)
        );
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(150));

        for eval in evaluations {
            let (role_title, role_color) = Self::node_role(&eval.node_id);
            let c = Self::to_colored(role_color);
            println!(
                "▶ {} ({})",
                role_title.color(c).bold(),
                eval.node_id.white()
            );
            let _ = std::io::stdout().flush();
            std::thread::sleep(std::time::Duration::from_millis(100));

            if let (Some(ref init_vote), Some(init_risk)) =
                (&eval.initial_vote, eval.initial_risk_score)
            {
                let traj_label = bundle.ui.format_trajectory(
                    &init_vote.bold().to_string(),
                    init_risk,
                    &eval.vote.bold().to_string(),
                    eval.risk_score,
                );
                println!("  {}", traj_label.truecolor(255, 174, 66));
                let _ = std::io::stdout().flush();
                std::thread::sleep(std::time::Duration::from_millis(80));
            }

            if !eval.cwe_flags.is_empty() {
                println!(
                    "  {} {}",
                    bundle.ui.cwe_detected.truecolor(255, 174, 66).bold(),
                    eval.cwe_flags.join(" · ").magenta()
                );
                let _ = std::io::stdout().flush();
                std::thread::sleep(std::time::Duration::from_millis(80));
            }

            if !eval.findings.is_empty() {
                for f in &eval.findings {
                    let sev_colored = match f.severity.to_lowercase().as_str() {
                        "critical" => f.severity.to_uppercase().red().bold(),
                        "high" => f.severity.to_uppercase().red(),
                        "medium" => f.severity.to_uppercase().yellow(),
                        _ => f.severity.to_uppercase().green(),
                    };
                    println!(
                        "  • [{}] {} — {}",
                        sev_colored,
                        f.title.bold(),
                        f.recommendation.truecolor(180, 180, 180)
                    );
                    let _ = std::io::stdout().flush();
                    std::thread::sleep(std::time::Duration::from_millis(120));
                }
            }

            if let Some(ref init_arg) = eval.initial_argument {
                println!(
                    "  {}",
                    bundle
                        .ui
                        .round1_initial_position
                        .truecolor(240, 200, 80)
                        .bold()
                );
                println!("    {}", init_arg.white());
                let _ = std::io::stdout().flush();
                std::thread::sleep(std::time::Duration::from_millis(180));

                let final_stance = if !eval.rationale.is_empty() {
                    &eval.rationale
                } else {
                    &eval.argument
                };
                println!(
                    "  {}",
                    bundle
                        .ui
                        .round2_post_debate_resolution
                        .truecolor(80, 230, 240)
                        .bold()
                );
                println!("    {}", final_stance.white());
                let _ = std::io::stdout().flush();
                std::thread::sleep(std::time::Duration::from_millis(220));
            } else {
                let stance = if !eval.rationale.is_empty() {
                    &eval.rationale
                } else {
                    &eval.argument
                };
                println!("  {}", stance.white());
                let _ = std::io::stdout().flush();
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
            println!();
        }
    }

    /// Displays the persisted consensus verdict and its summary without redundant text duplication.
    pub fn render_verdict(verdict: &str, summary: &str) {
        let lang = crate::i18n::Language::detect(summary);
        let bundle = crate::i18n::get_bundle(lang);

        println!();
        Self::rule();
        Self::section(&bundle.ui.central_dogma_section);
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(200));

        let (banner_title, detail, r, g, b) = match verdict {
            "CONSENSUS_UNAVAILABLE" => (
                bundle.ui.verdict_unavailable_title.as_str(),
                bundle.ui.verdict_unavailable_detail.as_str(),
                255,
                60,
                60,
            ),
            "VETO_BALTHASAR_SECURITY" => (
                bundle.ui.verdict_veto_title.as_str(),
                bundle.ui.verdict_veto_detail.as_str(),
                255,
                30,
                30,
            ),
            "APPROVED_UNANIMOUS" => (
                bundle.ui.verdict_unanimous_approve_title.as_str(),
                bundle.ui.verdict_unanimous_approve_detail.as_str(),
                80,
                255,
                80,
            ),
            "APPROVED_MAJORITY" => (
                bundle.ui.verdict_majority_approve_title.as_str(),
                bundle.ui.verdict_majority_approve_detail.as_str(),
                255,
                174,
                66,
            ),
            "REJECTED_MAJORITY" => (
                bundle.ui.verdict_majority_reject_title.as_str(),
                bundle.ui.verdict_majority_reject_detail.as_str(),
                255,
                80,
                80,
            ),
            "REJECTED_UNANIMOUS" => (
                bundle.ui.verdict_unanimous_reject_title.as_str(),
                bundle.ui.verdict_unanimous_reject_detail.as_str(),
                255,
                40,
                40,
            ),
            "SPLIT_DECISION_REQUIRES_REVIEW" => (
                bundle.ui.verdict_split_title.as_str(),
                bundle.ui.verdict_split_detail.as_str(),
                255,
                190,
                40,
            ),
            other => (
                other,
                bundle.ui.verdict_unrecognized_detail.as_str(),
                200,
                100,
                200,
            ),
        };

        // Giant NERV Consensus Banner Box with exact Unicode display width alignment
        let inner_width = 72;
        let border_line = "═".repeat(inner_width);
        let top_box = format!("╔{}╗", border_line).truecolor(r, g, b).bold();
        let bot_box = format!("╚{}╝", border_line).truecolor(r, g, b).bold();

        let text_width = Self::str_display_width(banner_title);
        let total_pad = inner_width.saturating_sub(text_width);
        let left_pad = total_pad / 2;
        let right_pad = total_pad - left_pad;
        let mid_box = format!(
            "║{}{}{}║",
            " ".repeat(left_pad),
            banner_title,
            " ".repeat(right_pad)
        )
        .truecolor(r, g, b)
        .bold();

        println!("  {}", top_box);
        println!("  {}", mid_box);
        println!("  {}", bot_box);
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(200));
        println!();

        // Strip legacy or verbose "Final positions:" dump from summary if present
        let clean_summary = if let Some(idx) = summary.find("\n\nFinal positions:\n") {
            summary[..idx].trim()
        } else {
            summary.trim()
        };

        println!(
            "  {} {}",
            bundle.ui.status_label.truecolor(255, 174, 66).bold(),
            detail.white()
        );
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(150));

        println!(
            "  {} {}",
            bundle.ui.synth_label.truecolor(255, 174, 66).bold(),
            clean_summary.truecolor(220, 220, 220)
        );
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(200));
        println!();
        Self::rule();
        println!();
    }

    /// Displays the specialist opening analysis before full Trinity deliberation.
    pub fn render_triage_result(lead_node: &str, eval: &NodeEvaluation) {
        println!();
        Self::rule();
        Self::section("TARGETED TRIAGE / SPECIALIST OPENING");
        let (role, role_color) = Self::node_role(lead_node);
        println!(
            "{} {}",
            "LEAD NODE".truecolor(160, 160, 160),
            role.color(Self::to_colored(role_color)).bold()
        );
        println!(
            "{} {}",
            "VOTE / RISK".truecolor(160, 160, 160),
            format!("{} / {}/10", eval.vote, eval.risk_score)
                .color(Self::to_colored(Self::risk_color(eval.risk_score)))
                .bold()
        );
        println!(
            "{} {} ms",
            "RESPONSE TIME".truecolor(160, 160, 160),
            eval.execution_time_ms
        );
        if !eval.cwe_flags.is_empty() {
            println!(
                "{} {}",
                "CWE FLAGS".truecolor(255, 174, 66).bold(),
                eval.cwe_flags.join(" · ").magenta()
            );
        }
        println!();
        println!("{}", "DIAGNOSIS".truecolor(255, 174, 66).bold());
        println!("{}", eval.argument.white());
        println!();
        println!(
            "{}",
            "NOTICE: Specialist triage is preliminary; full Trinity deliberation is mandatory."
                .truecolor(255, 174, 66)
                .bold()
        );
        Self::rule();
        println!();
    }

    /// Renders the deliberation history.
    #[allow(dead_code)]
    pub fn render_history(deliberations: &[crate::db::client::DeliberationRecord]) {
        Self::section("MAGI ARCHIVE / DELIBERATION HISTORY");
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("CASE")
                    .fg(Color::Yellow)
                    .add_attribute(Attribute::Bold),
                Cell::new("OPERATOR")
                    .fg(Color::Yellow)
                    .add_attribute(Attribute::Bold),
                Cell::new("TARGET")
                    .fg(Color::Yellow)
                    .add_attribute(Attribute::Bold),
                Cell::new("TYPE")
                    .fg(Color::Yellow)
                    .add_attribute(Attribute::Bold),
                Cell::new("STATE")
                    .fg(Color::Yellow)
                    .add_attribute(Attribute::Bold),
            ]);

        for d in deliberations {
            let status_cell = match d.status.as_str() {
                "RESOLVED" => Cell::new("● RESOLVED").fg(Color::Green),
                "DEBATING" => Cell::new("◉ DEBATING").fg(Color::Cyan),
                "PENDING" => Cell::new("○ PENDING").fg(Color::Yellow),
                _ => Cell::new(format!("! {}", d.status)).fg(Color::Red),
            };
            table.add_row(vec![
                Cell::new(format!("MAGI-{:06}", d.id)).add_attribute(Attribute::Bold),
                Cell::new(&d.author).fg(Color::Cyan),
                Cell::new(&d.title),
                Cell::new(&d.context_type).fg(Color::Magenta),
                status_cell,
            ]);
        }
        println!("{table}");
        println!();
    }
}
