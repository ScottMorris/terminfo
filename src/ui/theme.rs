// Shared colour palette and small styling helpers used across every tab for a consistent look.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders};

use crate::tabs::Tab;

/// Each tab gets its own accent colour, reused for its border, section headers, and its entry
/// in the tab bar, so the whole app reads as one coherent — but not monochrome — surface.
pub fn accent(tab: Tab) -> Color {
    match tab {
        Tab::Overview => Color::Cyan,
        Tab::Colours => Color::Magenta,
        Tab::Attributes => Color::Yellow,
        Tab::Unicode => Color::Green,
        Tab::Input => Color::Blue,
        Tab::Mouse => Color::LightBlue,
        Tab::Graphics => Color::LightMagenta,
    }
}

/// A bold section-header style tinted with the given accent colour.
pub fn header_style(accent: Color) -> Style {
    Style::default().fg(accent).add_modifier(Modifier::BOLD)
}

/// The rounded, accent-bordered frame every tab's content is drawn inside, titled with the
/// tab's name in its accent colour.
pub fn frame_block(title: &str, accent: Color) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ))
}

/// A short, bold key-hint style (for key names in the bottom hint bar).
pub fn key_style() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

/// A muted style for hint-bar descriptions and other secondary text.
pub fn muted_style() -> Style {
    Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::DIM)
}

/// Green "yes" / red "no", so capability booleans read at a glance.
pub fn yes_no_span(value: bool) -> Span<'static> {
    if value {
        Span::styled("yes", Style::default().fg(Color::Green))
    } else {
        Span::styled("no", Style::default().fg(Color::Red))
    }
}

pub fn positive_style() -> Style {
    Style::default().fg(Color::Green)
}

pub fn warning_style() -> Style {
    Style::default().fg(Color::Yellow)
}
