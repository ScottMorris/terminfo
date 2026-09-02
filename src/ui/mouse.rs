// The Mouse tab: live mouse capability, observed event kinds, and most-recent-event detail.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::theme;
use crate::app::App;
use crate::mouse_state::MouseKindObserved;
use crate::tabs::Tab;

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
        theme::header_style(theme::accent(Tab::Mouse)),
    )
}

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let state = &app.mouse_state;
    let mut lines: Vec<Line> = vec![
        header("Capability"),
        Line::from(vec![
            Span::raw("Mouse capture enabled: "),
            theme::yes_no_span(state.capture_enabled),
        ]),
        Line::from(""),
        header("Observed event kinds this session"),
    ];
    let observed: Vec<&str> = KIND_ORDER
        .iter()
        .filter(|kind| state.observed_kinds.contains(kind))
        .map(|kind| kind.label())
        .collect();
    if observed.is_empty() {
        lines.push(Line::styled(
            "(none yet — move, click, drag, or scroll the mouse)",
            theme::muted_style(),
        ));
    } else {
        lines.push(Line::styled(observed.join(", "), theme::positive_style()));
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
        None => lines.push(Line::styled("(none yet)", theme::muted_style())),
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        "Tip: clicking a tab title in the bottom bar (from any tab) is itself a mouse event exercising this same capability.",
        theme::muted_style(),
    ));

    frame.render_widget(Paragraph::new(lines), area);
}
