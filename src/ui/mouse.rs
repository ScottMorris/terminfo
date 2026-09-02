// The Mouse tab: live mouse capability, observed event kinds, and most-recent-event detail.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::mouse_state::MouseKindObserved;

/// Display order for observed mouse event kinds, independent of `HashSet` iteration order.
const KIND_ORDER: [MouseKindObserved; 5] = [
    MouseKindObserved::Press,
    MouseKindObserved::Release,
    MouseKindObserved::Drag,
    MouseKindObserved::Scroll,
    MouseKindObserved::Move,
];

fn header(text: &str) -> Line<'static> {
    Line::styled(
        text.to_string(),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
}

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let state = &app.mouse_state;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(header("Capability"));
    lines.push(Line::from(format!(
        "Mouse capture enabled: {}",
        if state.capture_enabled { "yes" } else { "no" }
    )));

    lines.push(Line::from(""));
    lines.push(header("Observed event kinds this session"));
    let observed: Vec<&str> = KIND_ORDER
        .iter()
        .filter(|kind| state.observed_kinds.contains(kind))
        .map(|kind| kind.label())
        .collect();
    if observed.is_empty() {
        lines.push(Line::from(
            "(none yet — move, click, drag, or scroll the mouse)",
        ));
    } else {
        lines.push(Line::from(observed.join(", ")));
    }

    lines.push(Line::from(""));
    lines.push(header("Most recent event"));
    match state.last_event {
        Some(event) => {
            lines.push(Line::from(format!("Kind: {:?}", event.kind)));
            lines.push(Line::from(format!(
                "Position: ({}, {})",
                event.column, event.row
            )));
            lines.push(Line::from(format!("Modifiers: {:?}", event.modifiers)));
        }
        None => lines.push(Line::from("(none yet)")),
    }

    lines.push(Line::from(""));
    lines.push(Line::from(
        "Tip: clicking a tab title in the bottom bar (from any tab) is itself a mouse event exercising this same capability.",
    ));

    frame.render_widget(Paragraph::new(lines), area);
}
