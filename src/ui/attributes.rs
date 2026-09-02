// The Attributes tab: one row per text attribute, each with a rendered sample and a short
// description of the expected appearance.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::theme;
use crate::app::App;
use crate::tabs::Tab;

/// One attribute row: a label, the style applied to the sample text, and a short description of
/// what the reader should expect to see (including, honestly, when ratatui/the terminal cannot
/// distinguish a named style from a plainer fallback).
struct Row {
    label: &'static str,
    style: Style,
    description: &'static str,
}

const SAMPLE: &str = "The quick brown fox";

/// ratatui 0.30 exposes exactly one underline on/off bit (`Modifier::UNDERLINED`) plus a colour
/// (`Style::underline_color`) — there is no `UnderlineStyle` enum for double/curly/dotted/dashed
/// underlines in its public API (checked against `ratatui-core` 0.1.2's `style.rs`, the version
/// this crate resolves to). Some terminals implement the Kitty/VTE extended-underline escape
/// codes (`CSI 4:2m`, `4:3m`, etc.) and would render these distinctly if ratatui emitted them,
/// but ratatui itself does not — so every "underline style" row below renders identically, via
/// the same `UNDERLINED` modifier, and the description says so rather than pretending otherwise.
const UNDERLINE_STYLE_NOTE: &str =
    "ratatui has no distinct underline-style API; renders as a plain single underline regardless of the name of this row.";

fn rows() -> Vec<Row> {
    vec![
        Row {
            label: "Bold",
            style: Style::new().add_modifier(Modifier::BOLD),
            description: "Heavier / brighter weight text.",
        },
        Row {
            label: "Dim",
            style: Style::new().add_modifier(Modifier::DIM),
            description: "Lower-intensity / faded text.",
        },
        Row {
            label: "Italic",
            style: Style::new().add_modifier(Modifier::ITALIC),
            description: "Slanted text (terminal/font dependent; many terminals ignore this).",
        },
        Row {
            label: "Underline",
            style: Style::new().add_modifier(Modifier::UNDERLINED),
            description: "A single line under the text.",
        },
        Row {
            label: "Double underline",
            style: Style::new().add_modifier(Modifier::UNDERLINED),
            description: UNDERLINE_STYLE_NOTE,
        },
        Row {
            label: "Curly underline",
            style: Style::new().add_modifier(Modifier::UNDERLINED),
            description: UNDERLINE_STYLE_NOTE,
        },
        Row {
            label: "Dotted underline",
            style: Style::new().add_modifier(Modifier::UNDERLINED),
            description: UNDERLINE_STYLE_NOTE,
        },
        Row {
            label: "Dashed underline",
            style: Style::new().add_modifier(Modifier::UNDERLINED),
            description: UNDERLINE_STYLE_NOTE,
        },
        Row {
            label: "Coloured underline",
            style: Style::new()
                .add_modifier(Modifier::UNDERLINED)
                .underline_color(Color::Magenta),
            description: "A single underline drawn in a colour independent of the text colour (magenta here), via `Style::underline_color`.",
        },
        Row {
            label: "Strikethrough",
            style: Style::new().add_modifier(Modifier::CROSSED_OUT),
            description: "A line through the middle of the text.",
        },
        Row {
            label: "Reversed",
            style: Style::new().add_modifier(Modifier::REVERSED),
            description: "Foreground and background colours swapped.",
        },
        Row {
            label: "Slow blink",
            style: Style::new().add_modifier(Modifier::SLOW_BLINK),
            description: "Text blinks slowly; most modern terminal emulators ignore blink by default (expected, not a bug).",
        },
        Row {
            label: "Rapid blink",
            style: Style::new().add_modifier(Modifier::RAPID_BLINK),
            description: "Text blinks quickly; most modern terminal emulators ignore blink by default (expected, not a bug).",
        },
        Row {
            label: "Hidden",
            style: Style::new().add_modifier(Modifier::HIDDEN),
            description: "Text is present but invisible (same colour as background, conceptually) — this row should look blank.",
        },
        Row {
            label: "Bold + Italic",
            style: Style::new().add_modifier(Modifier::BOLD | Modifier::ITALIC),
            description: "Combination of bold and italic.",
        },
        Row {
            label: "Bold + Underline",
            style: Style::new().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            description: "Combination of bold and underline.",
        },
        Row {
            label: "Dim + Italic",
            style: Style::new().add_modifier(Modifier::DIM | Modifier::ITALIC),
            description: "Combination of dim and italic.",
        },
    ]
}

fn header(text: &str) -> Line<'static> {
    Line::styled(
        text.to_string(),
        theme::header_style(theme::accent(Tab::Attributes)),
    )
}

const LABEL_WIDTH: usize = 20;
const SAMPLE_WIDTH: usize = 22;

pub fn draw(frame: &mut Frame, area: Rect, _app: &App) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(header("Text attributes"));
    lines.push(Line::styled(
        "Attributes this terminal is known to ignore are still listed below — that is expected data about this terminal, not a bug in terminfo.",
        theme::muted_style(),
    ));
    lines.push(Line::from(""));

    for row in rows() {
        lines.push(Line::from(vec![
            Span::raw(format!("{:<width$} ", row.label, width = LABEL_WIDTH)),
            Span::styled(
                format!("{:<width$}", SAMPLE, width = SAMPLE_WIDTH),
                row.style,
            ),
            Span::styled(row.description, theme::muted_style()),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}
