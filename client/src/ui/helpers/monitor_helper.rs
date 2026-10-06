//! # Diegetic CRT Monitor Formatter Helper
//!
//! Formats individual MAGI node evaluations into lines for 25-column diegetic
//! CRT monitor boxes in active, blackout, and consensus states.

use crate::llm::NodeEvaluation;
use colored::*;

/// Formats a single MAGI node into lines for a 25-column diegetic CRT monitor box with bloom and Round 1 override support.
pub fn format_node_monitor(
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
pub fn format_blackout_monitor(eval: &NodeEvaluation) -> Vec<String> {
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
pub fn format_deliberating_monitor(
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

    let filled = "█".repeat(sim_risk.min(10));
    let empty = "░".repeat(10 - sim_risk.min(10));
    let gauge = format!("{}{}", filled, empty)
        .truecolor(r, g, b)
        .to_string();
    let risk_label = format!("RISK [{}]  --/10", gauge);
    let row3 = format!("{}{}{}", l_border, risk_label, r_border);

    let tele_raw = format!("SCANNING... {:>5}ms", eval.execution_time_ms.min(99999));
    let tele_str = format!("{:^23}", tele_raw)
        .truecolor(0, 150, 190)
        .to_string();
    let row4 = format!("{}{}{}", l_border, tele_str, r_border);

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
