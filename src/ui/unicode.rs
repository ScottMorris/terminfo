// The Unicode tab: box drawing, block elements, braille, powerline glyphs, CJK width, emoji, and
// combining marks — each with a column ruler, a right-hand alignment marker, and both the
// `unicode-width`-computed and real terminal-measured width for that exact sample string.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

use super::theme;
use crate::app::App;
use crate::tabs::Tab;
use crate::widths;

/// The column width reserved for a sample's glyphs before the alignment marker. Padding is
/// computed from `unicode-width`'s notion of width, not from `char` count — a real terminal that
/// renders a sample wider or narrower than that will visibly push its trailing marker out of
/// line with the ruler above it, which is the whole diagnostic.
const GLYPH_FIELD_WIDTH: usize = 22;
const LABEL_WIDTH: usize = 18;

/// One row: a label, the id of the sample string it displays (looked up in `widths::SAMPLES`,
/// the single source of truth shared with the startup width probe), and a short description.
struct Row {
    label: &'static str,
    sample_id: &'static str,
    description: &'static str,
}

struct Section {
    title: &'static str,
    rows: &'static [Row],
}

const SECTIONS: &[Section] = &[
    Section {
        title: "Box drawing",
        rows: &[
            Row {
                label: "Light",
                sample_id: "box_light",
                description: "Single-weight lines and corners.",
            },
            Row {
                label: "Heavy",
                sample_id: "box_heavy",
                description: "Bold-weight lines and corners.",
            },
            Row {
                label: "Double",
                sample_id: "box_double",
                description: "Double-line lines and corners.",
            },
            Row {
                label: "Rounded",
                sample_id: "box_rounded",
                description: "Light lines with rounded (arc) corners.",
            },
            Row {
                label: "Dashed",
                sample_id: "box_dashed",
                description: "Dashed variants of light lines.",
            },
        ],
    },
    Section {
        title: "Block elements and shades",
        rows: &[Row {
            label: "Blocks/shades",
            sample_id: "blocks_shades",
            description: "Full block, three shade densities, and half-block/quadrant glyphs.",
        }],
    },
    Section {
        title: "Braille pattern strip",
        rows: &[Row {
            label: "Braille",
            sample_id: "braille",
            description: "Ascending dot-count braille patterns (used for sub-cell graphics tricks).",
        }],
    },
    Section {
        title: "Powerline / private-use glyphs",
        rows: &[Row {
            label: "Powerline",
            sample_id: "powerline",
            description: "Private-use-area separators (U+E0B0 range); needs a patched/Nerd Font to render as glyphs rather than boxes.",
        }],
    },
    Section {
        title: "CJK wide characters",
        rows: &[Row {
            label: "CJK",
            sample_id: "cjk",
            description: "Each character is double-width (computed width 2 per glyph).",
        }],
    },
    Section {
        title: "Emoji",
        rows: &[
            Row {
                label: "Single codepoint",
                sample_id: "emoji_single",
                description: "One emoji codepoint, typically double-width.",
            },
            Row {
                label: "ZWJ sequence",
                sample_id: "emoji_zwj_family",
                description: "Family emoji built from four people joined by ZWJ; renders as one glyph on terminals with sequence support, or as separate glyphs otherwise.",
            },
            Row {
                label: "Flag",
                sample_id: "emoji_flag",
                description: "A pair of regional-indicator symbols (Canada); renders as one flag glyph, or as two letter-in-box glyphs.",
            },
            Row {
                label: "Skin-tone modifier",
                sample_id: "emoji_skin_tone",
                description: "A base emoji plus a Fitzpatrick skin-tone modifier codepoint.",
            },
            Row {
                label: "VS16 text presentation",
                sample_id: "emoji_vs_text",
                description: "The bare codepoint, without a variation selector — typically renders narrow/monochrome.",
            },
            Row {
                label: "VS16 emoji presentation",
                sample_id: "emoji_vs_emoji",
                description: "The same codepoint plus U+FE0F (VS16) — requests the wide, coloured emoji rendering.",
            },
        ],
    },
    Section {
        title: "Combining marks, ZWJ, ZWSP, tab",
        rows: &[
            Row {
                label: "Combining mark",
                sample_id: "combining_mark",
                description: "\"e\" plus a combining acute accent (U+0301) — should render as a single accented glyph.",
            },
            Row {
                label: "Zero-width joiner",
                sample_id: "zwj_plain",
                description: "\"a\", ZWJ (U+200D), \"b\" — the joiner itself should contribute no visible width.",
            },
            Row {
                label: "Zero-width space",
                sample_id: "zwsp",
                description: "\"a\", ZWSP (U+200B), \"b\" — should contribute no visible width (unlike a regular space).",
            },
            Row {
                label: "Tab",
                sample_id: "tab",
                description: "\"a\", a tab character, \"b\" — tab stops are terminal-defined, not fixed-width.",
            },
        ],
    },
];

fn header(text: &str) -> Line<'static> {
    Line::styled(
        text.to_string(),
        theme::header_style(theme::accent(Tab::Unicode)),
    )
}

/// A tick-marked ruler the width of the glyph field, so misalignment of the trailing marker on
/// subsequent rows is visible against a fixed reference.
fn ruler_line() -> Line<'static> {
    let mut ruler = String::with_capacity(GLYPH_FIELD_WIDTH);
    for i in 0..GLYPH_FIELD_WIDTH {
        if i % 5 == 0 {
            ruler.push(char::from_digit(((i / 5) % 10) as u32, 10).unwrap_or('?'));
        } else {
            ruler.push('.');
        }
    }
    Line::from(vec![
        Span::raw(format!("{:width$} ", "", width = LABEL_WIDTH)),
        Span::styled(ruler, theme::muted_style()),
        Span::styled("│", theme::muted_style()),
    ])
}

fn row_line(row: &Row, probe: &widths::WidthProbe) -> Line<'static> {
    let text = widths::sample_text(row.sample_id);
    let computed = text.width();
    let measured = probe.measured_width(row.sample_id);

    let pad = GLYPH_FIELD_WIDTH.saturating_sub(computed);
    let measured_text = match measured {
        Some(w) => format!("measured={w}"),
        None => "unmeasured".to_string(),
    };
    let mismatch = matches!(measured, Some(w) if w as usize != computed);
    let width_style = if mismatch {
        theme::warning_style()
    } else {
        theme::muted_style()
    };

    Line::from(vec![
        Span::raw(format!("{:<width$} ", row.label, width = LABEL_WIDTH)),
        Span::styled(text.to_string(), Style::default()),
        Span::raw(" ".repeat(pad)),
        Span::styled("│", theme::muted_style()),
        Span::raw(" "),
        Span::styled(format!("computed={computed} {measured_text}"), width_style),
        Span::raw("  "),
        Span::styled(row.description, theme::muted_style()),
    ])
}

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::styled(
        "Two widths follow each sample: computed (via `unicode-width`) and measured (this terminal's own answer, from the startup probe). A mismatch is real, terminal-specific data.",
        theme::muted_style(),
    ));
    lines.push(Line::from(""));

    for section in SECTIONS {
        lines.push(header(section.title));
        lines.push(ruler_line());
        for row in section.rows {
            lines.push(row_line(row, &app.width_probe));
        }
        lines.push(Line::from(""));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn width_of(id: &str) -> usize {
        widths::sample_text(id).width()
    }

    #[test]
    fn box_drawing_and_cjk_widths_match_expected() {
        assert_eq!(width_of("box_light"), 11);
        assert_eq!(width_of("box_rounded"), 6);
        assert_eq!(width_of("cjk"), 8);
    }

    #[test]
    fn emoji_widths_match_expected() {
        assert_eq!(width_of("emoji_single"), 2);
        assert_eq!(width_of("emoji_zwj_family"), 2);
        assert_eq!(width_of("emoji_flag"), 2);
        assert_eq!(width_of("emoji_vs_text"), 1);
        assert_eq!(width_of("emoji_vs_emoji"), 2);
    }

    #[test]
    fn zero_width_and_combining_samples_match_expected() {
        // Combining mark: "e" + combining acute collapses to a single-width glyph.
        assert_eq!(width_of("combining_mark"), 1);
        // ZWJ/ZWSP contribute no width of their own: "a" + joiner/space + "b" == 2.
        assert_eq!(width_of("zwj_plain"), 2);
        assert_eq!(width_of("zwsp"), 2);
    }

    #[test]
    fn every_row_sample_id_resolves() {
        for section in SECTIONS {
            for row in section.rows {
                // Panics (via `widths::sample_text`) if a row references an unknown id.
                let _ = widths::sample_text(row.sample_id);
            }
        }
    }
}
