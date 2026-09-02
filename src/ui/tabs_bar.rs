// Renders the bottom tab bar as a hand-laid-out row of spans, exposing each title's exact rect for hit-testing.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

use crate::app::App;
use crate::tabs::Tab;

/// Draws the tab bar into `area` and returns the on-screen rect of each tab title, in tab
/// order. Deliberately hand-rolled (not the built-in `Tabs` widget) so the exact rect of each
/// title is known for hit-testing, rather than assumed from a fixed layout.
pub fn draw(frame: &mut Frame, area: Rect, app: &App) -> Vec<(Tab, Rect)> {
    let mut regions = Vec::with_capacity(Tab::ALL.len());
    if area.width == 0 || area.height == 0 {
        return regions;
    }

    let mut spans: Vec<Span<'static>> = Vec::with_capacity(Tab::ALL.len() * 2);
    let mut x = area.x;
    let right_edge = area.x + area.width;

    for (i, &tab) in Tab::ALL.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
            x = x.saturating_add(1);
        }
        if x >= right_edge {
            break;
        }

        let label = format!(" {} ", tab.title());
        let label_width = label.width() as u16;
        let style = if tab == app.tab {
            Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
        } else {
            Style::default()
        };
        spans.push(Span::styled(label, style));

        let clamped_width = label_width.min(right_edge.saturating_sub(x));
        regions.push((
            tab,
            Rect {
                x,
                y: area.y,
                width: clamped_width,
                height: 1,
            },
        ));
        x = x.saturating_add(label_width);
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
    regions
}
