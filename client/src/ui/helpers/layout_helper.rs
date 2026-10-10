//! # Terminal Layout and Unicode Helper
//!
//! Provides utilities for display width calculation (supporting CJK full-width characters)
//! and geometric rendering of the Evangelion triangular monitor layout.

use colored::*;

/// Calculates the terminal display width of a string considering full-width CJK characters.
pub fn str_display_width(s: &str) -> usize {
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

/// Renders a 22-line Evangelion triangular MAGI layout with Balthasar-2 on top,
/// pure geometric wireframe connecting lines, and Casper-3 + Melchior-1 at the base
/// connected directly with a horizontal debate bridge line.
pub fn render_triangular_screen(
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

/// Truncates a string slice safely at character boundaries up to `max_chars`.
/// Never panics on multibyte UTF-8 characters (e.g. Spanish 'ñ', accents, CJK, emojis).
pub fn safe_truncate_str(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

/// Truncates a string to at most `max_chars` characters, appending `ellipsis` (e.g. "…" or "...") if truncated.
/// Safely respects Unicode character boundaries.
#[allow(dead_code)]
pub fn truncate_with_ellipsis(s: &str, max_chars: usize, ellipsis: &str) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        let budget = max_chars.saturating_sub(ellipsis.chars().count());
        format!("{}{}", safe_truncate_str(s, budget), ellipsis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_display_width() {
        assert_eq!(str_display_width("ASCII"), 5);
        assert_eq!(str_display_width("質問"), 4);
    }

    #[test]
    fn test_safe_truncate_str_utf8() {
        let spanish = "Mantengo el voto NEUTRAL: diseño de arquitectura con migración";
        // 'diseño' contains 'ñ' which is 2 bytes (0xc3, 0xb1)
        let truncated = safe_truncate_str(spanish, 30);
        assert!(truncated.chars().count() <= 30);
        assert_eq!(safe_truncate_str("hello", 10), "hello");
        assert_eq!(safe_truncate_str("hello", 3), "hel");
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        let text = "Mantengo el voto NEUTRAL tras el debate cruzado, con confianza moderada. La consulta es una solicitud de análisis de principios de diseño";
        let res = truncate_with_ellipsis(text, 50, "…");
        assert!(res.ends_with('…'));
        assert_eq!(res.chars().count(), 50);
    }
}
