// The Overview tab: terminal identity, geometry, and a capability summary.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

fn header(text: &str) -> Line<'static> {
    Line::styled(
        text.to_string(),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn or_unset(value: &Option<String>) -> String {
    match value {
        Some(v) if !v.is_empty() => v.clone(),
        _ => "(unset)".to_string(),
    }
}

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let ti = &app.term_info;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(header("Terminal identity"));
    lines.push(Line::from(format!(
        "Detected terminal: {}",
        ti.detected_name
    )));
    lines.push(Line::from(format!("$TERM: {}", or_unset(&ti.term))));
    lines.push(Line::from(format!(
        "$COLORTERM: {}",
        or_unset(&ti.colorterm)
    )));
    let term_program = match (&ti.term_program, &ti.term_program_version) {
        (Some(program), Some(version)) => format!("{program} ({version})"),
        (Some(program), None) => program.clone(),
        (None, _) => "(unset)".to_string(),
    };
    lines.push(Line::from(format!("$TERM_PROGRAM: {term_program}")));
    lines.push(Line::from(format!(
        "Multiplexer: {}",
        ti.multiplexer
            .clone()
            .unwrap_or_else(|| "none detected".to_string())
    )));
    lines.push(Line::from(format!(
        "SSH: {}",
        ti.ssh.clone().unwrap_or_else(|| "not detected".to_string())
    )));
    lines.push(Line::from(format!("$LANG: {}", or_unset(&ti.lang))));
    lines.push(Line::from(format!("$LC_ALL: {}", or_unset(&ti.lc_all))));
    lines.push(Line::from(format!("$SHELL: {}", or_unset(&ti.shell))));
    lines.push(Line::from(format!(
        "Terminfo entry: {}",
        ti.terminfo_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "not found".to_string())
    )));

    lines.push(Line::from(""));
    lines.push(header("Geometry"));
    lines.push(Line::from(format!(
        "Cells: {} x {}",
        ti.geometry.cols, ti.geometry.rows
    )));
    match (ti.geometry.pixel_width, ti.geometry.pixel_height) {
        (Some(w), Some(h)) => {
            lines.push(Line::from(format!("Pixels: {w} x {h}")));
            if let Some((cw, ch)) = ti.geometry.cell_pixel_size() {
                lines.push(Line::from(format!("Per-cell: {cw:.1} x {ch:.1} px")));
            }
        }
        _ => lines.push(Line::from("Pixels: not reported")),
    }

    lines.push(Line::from(""));
    lines.push(header("Capabilities"));
    lines.push(Line::from(format!(
        "Colour depth: {} — {}",
        ti.colour_depth.label(),
        ti.colour_depth_reason
    )));
    lines.push(Line::from(format!(
        "Keyboard enhancement (Kitty protocol): {}",
        match ti.keyboard_enhancement {
            Some(v) => yes_no(v).to_string(),
            None => "unknown".to_string(),
        }
    )));
    lines.push(Line::from(
        "Graphics protocol: not yet detected (see Graphics tab)",
    ));
    lines.push(Line::from(format!(
        "Mouse capture: {}",
        yes_no(ti.mouse_capture_enabled)
    )));

    let block = Block::default().borders(Borders::NONE);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}
