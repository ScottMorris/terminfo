// Startup probe that measures the terminal's own rendered width for each Unicode sample string
// shown on the Unicode tab, by printing it to a scratch line and reading the cursor back.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::io::Write;

use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};

/// One Unicode sample string, identified by a stable id, that both the startup probe and the
/// Unicode tab reference — a single source of truth so the tab's displayed glyphs are always
/// exactly the strings that were measured.
pub struct Sample {
    pub id: &'static str,
    pub text: &'static str,
}

/// The full set of Unicode samples probed at startup and shown on the Unicode tab, grouped in
/// the order SPEC.md's Unicode tab section lists them.
pub const SAMPLES: &[Sample] = &[
    // Box drawing.
    Sample {
        id: "box_light",
        text: "─│┌┐└┘├┤┬┴┼",
    },
    Sample {
        id: "box_heavy",
        text: "━┃┏┓┗┛┣┫┳┻╋",
    },
    Sample {
        id: "box_double",
        text: "═║╔╗╚╝╠╣╦╩╬",
    },
    Sample {
        id: "box_rounded",
        text: "─│╭╮╰╯",
    },
    Sample {
        id: "box_dashed",
        text: "╌╍┄┅┈┉╎╏",
    },
    // Block elements and shades.
    Sample {
        id: "blocks_shades",
        text: "█▓▒░▀▄▌▐▖▗▘▝",
    },
    // Braille pattern strip.
    Sample {
        id: "braille",
        text: "⠁⠃⠇⠏⠟⠿⡿⣿",
    },
    // Powerline / private-use glyphs.
    Sample {
        id: "powerline",
        text: "\u{e0b0}\u{e0b1}\u{e0b2}\u{e0b3}\u{e0b4}\u{e0b6}",
    },
    // CJK wide characters.
    Sample {
        id: "cjk",
        text: "你好世界",
    },
    // Emoji.
    Sample {
        id: "emoji_single",
        text: "😀",
    },
    Sample {
        id: "emoji_zwj_family",
        text: "👨\u{200d}👩\u{200d}👧\u{200d}👦",
    },
    Sample {
        id: "emoji_flag",
        text: "🇨🇦",
    },
    Sample {
        id: "emoji_skin_tone",
        text: "👍🏽",
    },
    Sample {
        id: "emoji_vs_text",
        text: "☺",
    },
    Sample {
        id: "emoji_vs_emoji",
        text: "☺\u{fe0f}",
    },
    // Combining marks, zero-width joiner, zero-width space, tab.
    Sample {
        id: "combining_mark",
        text: "e\u{0301}",
    },
    Sample {
        id: "zwj_plain",
        text: "a\u{200d}b",
    },
    Sample {
        id: "zwsp",
        text: "a\u{200b}b",
    },
    Sample {
        id: "tab",
        text: "a\tb",
    },
];

/// Looks up a sample's text by id. Panics on an unknown id — the id/text pairing is a fixed,
/// compile-time-known set, so a lookup miss is a programming error, not runtime data.
pub fn sample_text(id: &str) -> &'static str {
    SAMPLES
        .iter()
        .find(|sample| sample.id == id)
        .map(|sample| sample.text)
        .unwrap_or_else(|| panic!("no such width sample: {id}"))
}

/// One sample's terminal-measured width, or `None` if the probe timed out or otherwise failed
/// for that sample.
struct Measurement {
    id: &'static str,
    measured: Option<u16>,
}

/// The results of the startup width probe: for each sample in `SAMPLES`, the terminal's own
/// measured column width, or `None` if that sample could not be measured.
#[derive(Default)]
pub struct WidthProbe {
    measurements: Vec<Measurement>,
}

impl WidthProbe {
    /// The terminal-measured width for the sample with the given id, or `None` if it was not
    /// measured (probe timeout, probe failure, or unknown id).
    pub fn measured_width(&self, id: &str) -> Option<u16> {
        self.measurements
            .iter()
            .find(|m| m.id == id)
            .and_then(|m| m.measured)
    }
}

/// Runs the startup width probe: for each sample string, moves the cursor to a scratch line at
/// the top-left of the screen, clears the line, prints the sample, and reads back the terminal's
/// own cursor column via `crossterm::cursor::position()` to get its real measured width, then
/// clears the scratch line again. Must run after entering the alternate screen but strictly
/// before any other terminal event is read (see SPEC.md's startup sequence). Each measurement
/// that fails or times out — `cursor::position()` has a built-in timeout — is recorded as
/// unmeasured rather than blocking the whole probe.
pub fn measure<W: Write>(writer: &mut W) -> WidthProbe {
    let measurements = SAMPLES
        .iter()
        .map(|sample| Measurement {
            id: sample.id,
            measured: measure_one(writer, sample.text),
        })
        .collect();
    WidthProbe { measurements }
}

/// Measures a single sample string's real rendered width by printing it at the scratch line's
/// start column and reading the cursor position back. Always attempts to leave the scratch line
/// clear afterwards, even on failure.
fn measure_one<W: Write>(writer: &mut W, text: &str) -> Option<u16> {
    if execute!(writer, MoveTo(0, 0), Clear(ClearType::CurrentLine)).is_err() {
        return None;
    }
    if write!(writer, "{text}").is_err() || writer.flush().is_err() {
        let _ = execute!(writer, MoveTo(0, 0), Clear(ClearType::CurrentLine));
        return None;
    }
    let result = crossterm::cursor::position();
    let _ = execute!(writer, MoveTo(0, 0), Clear(ClearType::CurrentLine));
    match result {
        Ok((col, _row)) => Some(col),
        Err(_) => None,
    }
}
