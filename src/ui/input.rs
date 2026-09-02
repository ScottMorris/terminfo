// The Input tab: a live, scrolling log of raw crossterm events.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::theme;
use crate::app::App;
use crate::tabs::Tab;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let enhancement = match app.term_info.keyboard_enhancement {
        Some(true) => "yes",
        Some(false) => "no",
        None => "unknown",
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::styled(
        format!(
            "Keyboard enhancement flags pushed: {enhancement}  ({} event(s) logged, release events only visible when this is \"yes\")  —  'c' clears the log",
            app.input_log.len()
        ),
        theme::header_style(theme::accent(Tab::Input)),
    ));
    lines.push(Line::from(""));

    if app.input_log.is_empty() {
        lines.push(Line::styled(
            "(no events yet — press a key or move the mouse)",
            theme::muted_style(),
        ));
    } else {
        let available_rows = area.height.saturating_sub(lines.len() as u16) as usize;
        let entries: Vec<&String> = app.input_log.iter().collect();
        let start = entries.len().saturating_sub(available_rows);
        for entry in &entries[start..] {
            lines.push(Line::from(entry.as_str()));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
}
