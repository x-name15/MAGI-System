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
        s.chars()
            .map(|c| {
                if ('\u{1100}'..='\u{115F}').contains(&c)
                    || ('\u{2E80}'..='\u{A4CF}').contains(&c)
                    || ('\u{AC00}'..='\u{D7A3}').contains(&c)
                    || ('\u{F900}'..='\u{FAFF}').contains(&c)
                    || ('\u{FE10}'..='\u{FE19}').contains(&c)
                    || ('\u{FE30}'..='\u{FE6F}').contains(&c)
                    || ('\u{FF00}'..='\u{FF60}').contains(&c)
                    || ('\u{FFE0}'..='\u{FFE6}').contains(&c)
                {
                    2
                } else {
                    1
                }
            })
            .sum()
    }

    /// Formats a single MAGI node into lines for a 25-column diegetic CRT monitor box with bloom and Round 1 override support.
    fn format_node_monitor(
        eval: &NodeEvaluation,
        is_veto: bool,
        bloom: bool,
        is_r1: bool,
    ) -> Vec<String> {
        let (name, role) = match eval.node_id.as_str() {
            "Melchior-1" => ("MELCHIOR-1", "SCIENTIST · ARCH"),
            "Balthasar-2" => ("BALTHASAR-2", "MOTHER · SECURITY"),
            "Casper-3" => ("CASPER-3", "WOMAN · PRAGMATICS"),
            _ => ("MAGI-AUX", "AUXILIARY EVALUATOR"),
        };

        let (active_vote, active_risk) = if is_r1 && eval.initial_vote.is_some() {
            (
                eval.initial_vote.as_deref().unwrap_or("APPROVE"),
                eval.initial_risk_score.unwrap_or(eval.risk_score),
            )
        } else {
            (eval.vote.as_str(), eval.risk_score)
        };

        let node_veto = is_veto
            || (eval.node_id == "Balthasar-2"
                && active_vote == "REJECT"
                && (active_risk >= 8 || !eval.cwe_flags.is_empty()));

        // Luminous CRT phosphor palette
        let (r, g, b) = match (node_veto, active_vote, bloom) {
            (true, _, true) => (255, 90, 90),
            (true, _, false) => (255, 30, 30),
            (false, "APPROVE", true) => (140, 255, 140),
            (false, "APPROVE", false) => (80, 255, 80),
            (false, "REJECT", true) => (255, 110, 110),
            (false, "REJECT", false) => (255, 60, 60),
            (_, _, true) => (255, 230, 140),
            (_, _, false) => (200, 200, 200),
        };

        let border_h = "─".repeat(23);
        // Interconnected wireframe borders: Balthasar connects down; Casper/Melchior connect up
        let line_top = if eval.node_id == "Balthasar-2" {
            format!("┌{}┐", border_h).truecolor(r, g, b).to_string()
        } else {
            "┌───────────┴───────────┐".truecolor(r, g, b).to_string()
        };

        let line_mid = match eval.node_id.as_str() {
            "Casper-3" => format!("├{}┼", border_h).truecolor(r, g, b).to_string(),
            "Melchior-1" => format!("┼{}┤", border_h).truecolor(r, g, b).to_string(),
            _ => format!("├{}┤", border_h).truecolor(r, g, b).to_string(),
        };

        let line_bot = if eval.node_id == "Balthasar-2" {
            "└───────────┬───────────┘".truecolor(r, g, b).to_string()
        } else {
            format!("└{}┘", border_h).truecolor(r, g, b).to_string()
        };

        let l_border = "│".truecolor(r, g, b).to_string();
        let r_border = "│".truecolor(r, g, b).to_string();

        // Row 1: Node Header (23 cols)
        let header_str = format!("{:^23}", name)
            .truecolor(r, g, b)
            .bold()
            .to_string();
        let row1 = format!("{}{}{}", l_border, header_str, r_border);

        // Row 2: Archetype (23 cols)
        let role_str = format!("{:^23}", role).truecolor(180, 180, 180).to_string();
        let row2 = format!("{}{}{}", l_border, role_str, r_border);

        // Row 3: Risk gauge (exact 23 chars)
        let clamped_risk = active_risk.min(10) as usize;
        let filled = "█".repeat(clamped_risk);
        let empty = "░".repeat(10 - clamped_risk);
        let gauge = format!("{}{}", filled, empty);
        let (gr, gg, gb) = if active_risk >= 8 {
            (255, 60, 60)
        } else if active_risk >= 5 {
            (255, 200, 40)
        } else {
            (80, 220, 100)
        };
        let gauge_colored = gauge.truecolor(gr, gg, gb).to_string();
        let risk_label = format!("RISK [{}] {:>2}/10", gauge_colored, active_risk);
        let row3 = format!("{}{}{}", l_border, risk_label, r_border);

        // Row 4: Telemetry (exact 23 chars)
        let conf_pct = (eval.confidence * 100.0).round() as u32;
        let round_tag = if is_r1 { "R1" } else { "R2" };
        let tele_raw = format!(
            "{}: {:>3}%  LAT:{:>5}ms",
            round_tag,
            conf_pct,
            eval.execution_time_ms.min(99999)
        );
        let tele_str = format!("{:^23}", tele_raw)
            .truecolor(150, 150, 150)
            .to_string();
        let row4 = format!("{}{}{}", l_border, tele_str, r_border);

        // Row 5 & 6: Vote Badge & Subtitle (exact 23 chars)
        let (vote_badge, subtitle) = if node_veto {
            (
                "  ██ SECURITY VETO ██  "
                    .truecolor(
                        255,
                        if bloom { 90 } else { 30 },
                        if bloom { 90 } else { 30 },
                    )
                    .bold()
                    .to_string(),
                "    [ SECURITY VETO ]  ".truecolor(255, 60, 60).to_string(),
            )
        } else {
            match active_vote {
                "APPROVE" => (
                    "    ██ AGREEMENT ██    "
                        .truecolor(
                            if bloom { 180 } else { 80 },
                            255,
                            if bloom { 180 } else { 80 },
                        )
                        .bold()
                        .to_string(),
                    "     [  AGREEMENT  ]   "
                        .truecolor(100, 220, 100)
                        .to_string(),
                ),
                "REJECT" => (
                    "     ██ DENIAL ██      "
                        .truecolor(
                            255,
                            if bloom { 110 } else { 60 },
                            if bloom { 110 } else { 60 },
                        )
                        .bold()
                        .to_string(),
                    "     [    DENIAL   ]   ".truecolor(220, 80, 80).to_string(),
                ),
                _ => (
                    "    ◇◇  NEUTRAL  ◇◇    "
                        .truecolor(255, 200, 60)
                        .bold()
                        .to_string(),
                    "     [   RESERVED  ]   "
                        .truecolor(200, 180, 80)
                        .to_string(),
                ),
            }
        };
        let row5 = format!("{}{}{}", l_border, vote_badge, r_border);
        let row6 = format!("{}{}{}", l_border, subtitle, r_border);

        vec![
            line_top, row1, row2, row3, row4, line_mid, row5, row6, line_bot,
        ]
    }

    /// Formats a single MAGI node in a CRT strobe / blackout state (inter-frame scanline blackout).
    fn format_blackout_monitor(eval: &NodeEvaluation) -> Vec<String> {
        let name = match eval.node_id.as_str() {
            "Melchior-1" => "MELCHIOR-1",
            "Balthasar-2" => "BALTHASAR-2",
            "Casper-3" => "CASPER-3",
            _ => "MAGI-AUX",
        };

        let (dim_r, dim_g, dim_b) = (50, 42, 30);
        let border_h = "─".repeat(23);
        let line_top = if eval.node_id == "Balthasar-2" {
            format!("┌{}┐", border_h)
                .truecolor(dim_r, dim_g, dim_b)
                .to_string()
        } else {
            "┌───────────┴───────────┐"
                .truecolor(dim_r, dim_g, dim_b)
                .to_string()
        };

        let line_mid = match eval.node_id.as_str() {
            "Casper-3" => format!("├{}┼", border_h)
                .truecolor(dim_r, dim_g, dim_b)
                .to_string(),
            "Melchior-1" => format!("┼{}┤", border_h)
                .truecolor(dim_r, dim_g, dim_b)
                .to_string(),
            _ => format!("├{}┤", border_h)
                .truecolor(dim_r, dim_g, dim_b)
                .to_string(),
        };

        let line_bot = if eval.node_id == "Balthasar-2" {
            "└───────────┬───────────┘"
                .truecolor(dim_r, dim_g, dim_b)
                .to_string()
        } else {
            format!("└{}┘", border_h)
                .truecolor(dim_r, dim_g, dim_b)
                .to_string()
        };

        let l_border = "│".truecolor(dim_r, dim_g, dim_b).to_string();
        let r_border = "│".truecolor(dim_r, dim_g, dim_b).to_string();

        let row1 = format!("{}{:^23}{}", l_border, name.truecolor(75, 60, 42), r_border);
        let row2 = format!(
            "{}{:^23}{}",
            l_border,
            "· · · · · · ·".truecolor(40, 35, 28),
            r_border
        );
        let row3 = format!("{}{:^23}{}", l_border, "                     ", r_border);
        let row4 = format!(
            "{}{:^23}{}",
            l_border,
            " · CATHODE STROBE ·  ".truecolor(65, 55, 40),
            r_border
        );
        let row5 = format!("{}{:^23}{}", l_border, "                     ", r_border);
        let row6 = format!(
            "{}{:^23}{}",
            l_border,
            "· · · · · · ·".truecolor(40, 35, 28),
            r_border
        );

        vec![
            line_top, row1, row2, row3, row4, line_mid, row5, row6, line_bot,
        ]
    }

    /// Formats a single MAGI node into lines while in the active deliberating / Electric Cyan scanning state.
    fn format_deliberating_monitor(
        eval: &NodeEvaluation,
        sim_risk: usize,
        blink: bool,
    ) -> Vec<String> {
        let (name, role) = match eval.node_id.as_str() {
            "Melchior-1" => ("MELCHIOR-1", "SCIENTIST · ARCH"),
            "Balthasar-2" => ("BALTHASAR-2", "MOTHER · SECURITY"),
            "Casper-3" => ("CASPER-3", "WOMAN · PRAGMATICS"),
            _ => ("MAGI-AUX", "AUXILIARY EVALUATOR"),
        };

        // Electric Cyan CRT Phosphor directly inspired by the Evangelion sprite sheets
        let (r, g, b) = if blink { (0, 225, 255) } else { (0, 130, 180) };

        let border_h = "─".repeat(23);
        let line_top = if eval.node_id == "Balthasar-2" {
            format!("┌{}┐", border_h).truecolor(r, g, b).to_string()
        } else {
            "┌───────────┴───────────┐".truecolor(r, g, b).to_string()
        };

        let line_mid = match eval.node_id.as_str() {
            "Casper-3" => format!("├{}┼", border_h).truecolor(r, g, b).to_string(),
            "Melchior-1" => format!("┼{}┤", border_h).truecolor(r, g, b).to_string(),
            _ => format!("├{}┤", border_h).truecolor(r, g, b).to_string(),
        };

        let line_bot = if eval.node_id == "Balthasar-2" {
            "└───────────┬───────────┘".truecolor(r, g, b).to_string()
        } else {
            format!("└{}┘", border_h).truecolor(r, g, b).to_string()
        };

        let l_border = "│".truecolor(r, g, b).to_string();
        let r_border = "│".truecolor(r, g, b).to_string();

        let row1 = format!(
            "{}{:^23}{}",
            l_border,
            name.truecolor(r, g, b).bold(),
            r_border
        );
        let row2 = format!(
            "{}{:^23}{}",
            l_border,
            role.truecolor(0, 170, 210),
            r_border
        );

        // Fluctuating risk gauge (exact 23 chars)
        let filled = "█".repeat(sim_risk.min(10));
        let empty = "░".repeat(10 - sim_risk.min(10));
        let gauge = format!("{}{}", filled, empty)
            .truecolor(r, g, b)
            .to_string();
        let risk_label = format!("RISK [{}]  --/10", gauge);
        let row3 = format!("{}{}{}", l_border, risk_label, r_border);

        // Scanning telemetry (exact 23 chars)
        let tele_raw = format!("SCANNING... {:>5}ms", eval.execution_time_ms.min(99999));
        let tele_str = format!("{:^23}", tele_raw)
            .truecolor(0, 150, 190)
            .to_string();
        let row4 = format!("{}{}{}", l_border, tele_str, r_border);

        // Deliberation blinking badge (exact 23 chars)
        let badge_text = if blink {
            "  ██ DELIBERATING ██   "
                .truecolor(0, 240, 255)
                .bold()
                .to_string()
        } else {
            "  ▒▒ DELIBERATING ▒▒   ".truecolor(0, 160, 200).to_string()
        };
        let sub_text = "   [ SYNCHRONIZING ]   "
            .truecolor(100, 200, 230)
            .to_string();
        let row5 = format!("{}{}{}", l_border, badge_text, r_border);
        let row6 = format!("{}{}{}", l_border, sub_text, r_border);

        vec![
            line_top, row1, row2, row3, row4, line_mid, row5, row6, line_bot,
        ]
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
        // Line 0: Header
        println!("\x1b[2K{}", header);

        // Lines 1..9: Balthasar-2 (top center, padded with 21 spaces -> center is col 33)
        let pad_b = " ".repeat(21);
        for line in lines_b {
            println!("\x1b[2K{}{}", pad_b, line);
        }

        // Lines 10..12: Connector lines without central box or arrows (3 lines)
        let (cr, cg, cb) = connector_color;
        let c_wire = |s: &str| s.truecolor(cr, cg, cb).to_string();

        let conn_0 = format!("{}│", " ".repeat(33));
        let conn_1 = format!("{}┌────────────────┴────────────────┐", " ".repeat(16));
        let conn_2 = format!("{}│{}│", " ".repeat(16), " ".repeat(33));

        println!("\x1b[2K{}", c_wire(&conn_0));
        println!("\x1b[2K{}", c_wire(&conn_1));
        println!("\x1b[2K{}", c_wire(&conn_2));

        // Lines 13..21: Casper-3 (left) and Melchior-1 (right) with horizontal connection bridge
        let pad_left = " ".repeat(4);
        let bridge_wire = "─".repeat(9).truecolor(cr, cg, cb).to_string();
        let pad_gap = " ".repeat(9);

        for i in 0..lines_c.len() {
            // Line mid (index 5) connects Casper and Melchior directly with a horizontal line
            let gap_str = if i == 5 { &bridge_wire } else { &pad_gap };

            println!("\x1b[2K{}{}{}{}", pad_left, lines_c[i], gap_str, lines_m[i]);
        }
    }

    /// Displays the three node evaluations as iconic Evangelion triangular CRT monitors
    /// animating through Round 1, inter-node cross debate, Round 2 resolution, and consensus.
    pub fn render_votes_table(evaluations: &[NodeEvaluation]) {
        Self::section("MAGI TRINITY MONITORS / INDIVIDUAL EVALUATIONS");

        let is_es = evaluations
            .iter()
            .any(|e| crate::llm::is_spanish_text(&e.argument));

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

                let header_line = match (*phase, is_es) {
                    (0, true) => format!(
                        "  {}  {}",
                        "⟳ 質問 · CONSULTA: SINCRONIZANDO NODOS"
                            .truecolor(255, 174, 66)
                            .bold(),
                        "解決 · RESOLUCIÓN: INICIANDO EVALUACIÓN...".truecolor(160, 130, 80)
                    ),
                    (0, false) => format!(
                        "  {}  {}",
                        "⟳ 質問 · INQUIRY: SYNCHRONIZING NODES"
                            .truecolor(255, 174, 66)
                            .bold(),
                        "解決 · RESOLUTION: INITIALIZING EVALUATION...".truecolor(160, 130, 80)
                    ),
                    (1, true) => format!(
                        "  {}  {}",
                        "↳ 質問 · RONDA 1: EVALUACIÓN INDEPENDIENTE"
                            .truecolor(255, 195, 45)
                            .bold(),
                        "解決 · RESOLUCIÓN: PENDIENTE DE DEBATE...".truecolor(180, 150, 90)
                    ),
                    (1, false) => format!(
                        "  {}  {}",
                        "↳ 質問 · ROUND 1: INDEPENDENT VOTES LOCKED"
                            .truecolor(255, 195, 45)
                            .bold(),
                        "解決 · RESOLUTION: CROSS-DEBATE PENDING...".truecolor(180, 150, 90)
                    ),
                    (2, true) => format!(
                        "  {}  {}",
                        "↳ 質問 · RONDA 2: DEBATE CRUZADO INTER-NODOS"
                            .truecolor(255, 230, 80)
                            .bold(),
                        "解決 · RESOLUCIÓN: EVALUANDO CONTRAPOSICIONES...".truecolor(220, 180, 90)
                    ),
                    (2, false) => format!(
                        "  {}  {}",
                        "↳ 質問 · ROUND 2: PEER CROSS-DEBATE ACTIVE"
                            .truecolor(255, 230, 80)
                            .bold(),
                        "解決 · RESOLUTION: EVALUATING COUNTERARGUMENTS...".truecolor(220, 180, 90)
                    ),
                    (3, true) => format!(
                        "  {}  {}",
                        "↳ 質問 · RONDA 2: POSICIONES FINALES SELLADAS"
                            .truecolor(255, 195, 45)
                            .bold(),
                        "解決 · RESOLUCIÓN: FORMALIZANDO CONSENSO...".truecolor(220, 190, 100)
                    ),
                    (3, false) => format!(
                        "  {}  {}",
                        "↳ 質問 · ROUND 2: FINAL POSITIONS SEALED"
                            .truecolor(255, 195, 45)
                            .bold(),
                        "解決 · RESOLUTION: FORMALIZING CONSENSUS...".truecolor(220, 190, 100)
                    ),
                    (_, true) => {
                        let (res_r, res_g, res_b) = consensus_bus;
                        format!(
                            "  {}  {}",
                            "✓ 質問 · TRINIDAD: EVALUACIÓN COMPLETA"
                                .truecolor(255, 174, 66)
                                .bold(),
                            "解決 · RESOLUCIÓN: CONSENSO ALCANZADO"
                                .truecolor(res_r, res_g, res_b)
                                .bold()
                        )
                    }
                    (_, false) => {
                        let (res_r, res_g, res_b) = consensus_bus;
                        format!(
                            "  {}  {}",
                            "✓ 質問 · TRINITY: EVALUATION COMPLETE"
                                .truecolor(255, 174, 66)
                                .bold(),
                            "解決 · RESOLUTION: CONSENSUS ACHIEVED"
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
        let rationales_title = if is_es {
            "▌ POSTURAS DE DEBATE Y HALLAZGOS ESPECIALIZADOS"
        } else {
            "▌ POST-DEBATE RATIONALES & SPECIALIST FINDINGS"
        };
        println!("{}", rationales_title.truecolor(255, 174, 66).bold());
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
                let traj_label = if is_es {
                    format!("• TRAYECTORIA DE VOTO: R1: {} (Riesgo: {}/10)  ➔  R2 Final: {} (Riesgo: {}/10)", init_vote.bold(), init_risk, eval.vote.bold(), eval.risk_score)
                } else {
                    format!(
                        "• VOTING TRAJECTORY: R1: {} (Risk: {}/10)  ➔  R2 Final: {} (Risk: {}/10)",
                        init_vote.bold(),
                        init_risk,
                        eval.vote.bold(),
                        eval.risk_score
                    )
                };
                println!("  {}", traj_label.truecolor(255, 174, 66));
                let _ = std::io::stdout().flush();
                std::thread::sleep(std::time::Duration::from_millis(80));
            }

            if !eval.cwe_flags.is_empty() {
                let cwe_label = if is_es {
                    "CWE DETECTADOS:"
                } else {
                    "CWE DETECTED:"
                };
                println!(
                    "  {} {}",
                    cwe_label.truecolor(255, 174, 66).bold(),
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
                let r1_label = if is_es {
                    "• [RONDA 1: EVALUACIÓN INICIAL]:"
                } else {
                    "• [ROUND 1: INITIAL POSITION]:"
                };
                println!("  {}", r1_label.truecolor(240, 200, 80).bold());
                println!("    {}", init_arg.white());
                let _ = std::io::stdout().flush();
                std::thread::sleep(std::time::Duration::from_millis(180));

                let final_stance = if !eval.rationale.is_empty() {
                    &eval.rationale
                } else {
                    &eval.argument
                };
                let r2_label = if is_es {
                    "• [RONDA 2: CONCLUSIÓN TRAS DEBATE]:"
                } else {
                    "• [ROUND 2: POST-DEBATE RESOLUTION]:"
                };
                println!("  {}", r2_label.truecolor(80, 230, 240).bold());
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
        let is_es = crate::llm::is_spanish_text(summary);

        println!();
        Self::rule();
        Self::section("NERV CENTRAL DOGMA // TRINITY CONSENSUS");
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(200));

        let (banner_title, detail, r, g, b) = match verdict {
            "CONSENSUS_UNAVAILABLE" => {
                if is_es {
                    (
                        "CONSENSUS UNAVAILABLE // 通信途絶",
                        "Sin veredicto persistido en SpacetimeDB. No se usó fallback local.",
                        255,
                        60,
                        60,
                    )
                } else {
                    (
                        "CONSENSUS UNAVAILABLE // 通信途絶",
                        "No persisted verdict returned by SpacetimeDB. No local fallback was used.",
                        255,
                        60,
                        60,
                    )
                }
            }
            "VETO_BALTHASAR_SECURITY" => {
                if is_es {
                    ("⚠ EMERGENCY SECURITY VETO // 第2使徒絶対防衛発令 ⚠", "Veto de seguridad activado. Umbral de riesgo crítico alcanzado en nodo defensivo.", 255, 30, 30)
                } else {
                    ("⚠ EMERGENCY SECURITY VETO // 第2使徒絶対防衛発令 ⚠", "Security veto activated. Critical risk threshold reached on defensive node.", 255, 30, 30)
                }
            }
            "APPROVED_UNANIMOUS" => {
                if is_es {
                    (
                        "UNANIMOUS AGREEMENT // 全会一致合意 (3–0)",
                        "Los tres nodos de MAGI aprobaron la propuesta sin objeciones.",
                        80,
                        255,
                        80,
                    )
                } else {
                    (
                        "UNANIMOUS AGREEMENT // 全会一致合意 (3–0)",
                        "All three MAGI nodes approved the proposal unconditionally.",
                        80,
                        255,
                        80,
                    )
                }
            }
            "APPROVED_MAJORITY" => {
                if is_es {
                    (
                        "MAJORITY AGREEMENT // 多数決合意 (2–1)",
                        "La propuesta fue aprobada por consenso mayoritario de la Trinidad.",
                        255,
                        174,
                        66,
                    )
                } else {
                    (
                        "MAJORITY AGREEMENT // 多数決合意 (2–1)",
                        "The proposal was approved by majority Trinity consensus.",
                        255,
                        174,
                        66,
                    )
                }
            }
            "REJECTED_MAJORITY" => {
                if is_es {
                    (
                        "MAJORITY DENIAL // 多数決拒絶 (1–2)",
                        "La propuesta fue rechazada por mayoría de votos en la Trinidad.",
                        255,
                        80,
                        80,
                    )
                } else {
                    (
                        "MAJORITY DENIAL // 多数決拒絶 (1–2)",
                        "The proposal was rejected by majority vote across the Trinity.",
                        255,
                        80,
                        80,
                    )
                }
            }
            "REJECTED_UNANIMOUS" => {
                if is_es {
                    (
                        "UNANIMOUS DENIAL // 全会一致拒絶 (0–3)",
                        "Los tres nodos de MAGI rechazaron la propuesta categóricamente.",
                        255,
                        40,
                        40,
                    )
                } else {
                    (
                        "UNANIMOUS DENIAL // 全会一致拒絶 (0–3)",
                        "All three MAGI nodes rejected the proposal outright.",
                        255,
                        40,
                        40,
                    )
                }
            }
            "SPLIT_DECISION_REQUIRES_REVIEW" => {
                if is_es {
                    ("SPLIT DECISION // 分裂評決 · 再審議要求 (1–1–1)", "La Trinidad no alcanzó una mayoría decisiva. Requiere revisión de operador humano.", 255, 190, 40)
                } else {
                    ("SPLIT DECISION // 分裂評決 · 再審議要求 (1–1–1)", "The Trinity did not reach a decisive majority. Human operator review required.", 255, 190, 40)
                }
            }
            other => (other, "Unrecognized consensus state.", 200, 100, 200),
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

        let status_label = if is_es { "▶ ESTADO:" } else { "▶ STATUS:" };
        let synth_label = if is_es {
            "▶ SÍNTESIS:"
        } else {
            "▶ SYNTHESIS:"
        };

        println!(
            "  {} {}",
            status_label.truecolor(255, 174, 66).bold(),
            detail.white()
        );
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(150));

        println!(
            "  {} {}",
            synth_label.truecolor(255, 174, 66).bold(),
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
