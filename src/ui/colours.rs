// The Colours tab: ANSI swatches, the 6x6x6 colour cube, the greyscale ramp, a truecolour hue
// sweep and gradient, and a truecolour-vs-256-colour banding test.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::theme;
use crate::app::App;
use crate::tabs::Tab;
use crate::terminfo::ColourDepth;

/// The 16 fixed ANSI colours (8 normal + 8 bright), name and RGB value, in the order most
/// terminals and colour charts (e.g. the standard xterm 256-colour chart) list them.
const ANSI16: [(&str, (u8, u8, u8)); 16] = [
    ("Black", (0, 0, 0)),
    ("Maroon", (128, 0, 0)),
    ("Green", (0, 128, 0)),
    ("Olive", (128, 128, 0)),
    ("Navy", (0, 0, 128)),
    ("Purple", (128, 0, 128)),
    ("Teal", (0, 128, 128)),
    ("Silver", (192, 192, 192)),
    ("Grey", (128, 128, 128)),
    ("Red", (255, 0, 0)),
    ("Lime", (0, 255, 0)),
    ("Yellow", (255, 255, 0)),
    ("Blue", (0, 0, 255)),
    ("Fuchsia", (255, 0, 255)),
    ("Aqua", (0, 255, 255)),
    ("White", (255, 255, 255)),
];

/// Converts a standard xterm 256-colour palette index to its RGB value: 0-15 are the fixed ANSI
/// colours, 16-231 are the 6x6x6 colour cube, and 232-255 are a 24-step greyscale ramp.
pub fn xterm256_to_rgb(index: u8) -> (u8, u8, u8) {
    if index < 16 {
        return ANSI16[index as usize].1;
    }
    if index <= 231 {
        let i = index - 16;
        let r = i / 36;
        let g = (i % 36) / 6;
        let b = i % 6;
        return (cube_level(r), cube_level(g), cube_level(b));
    }
    let v = 8 + 10 * (index - 232);
    (v, v, v)
}

/// Maps a 0-5 colour-cube level to its 0-255 channel value: level 0 is 0, otherwise `55 + 40 *
/// level`, matching the standard xterm 256-colour cube formula.
fn cube_level(level: u8) -> u8 {
    if level == 0 {
        0
    } else {
        55 + 40 * level
    }
}

/// Finds the nearest xterm 256-colour palette index to an arbitrary truecolour RGB value, by
/// squared Euclidean distance in RGB space. Used by the banding test to show what a truecolour
/// gradient looks like quantised down to 256 colours. Ties (e.g. a colour-cube entry that
/// exactly duplicates one of the 16 fixed ANSI colours) resolve to the lowest index checked.
pub fn nearest_256_index(rgb: (u8, u8, u8)) -> u8 {
    let mut best_index = 0u8;
    let mut best_distance = u32::MAX;
    for index in 0..=255u16 {
        let candidate = xterm256_to_rgb(index as u8);
        let distance = squared_distance(rgb, candidate);
        if distance < best_distance {
            best_distance = distance;
            best_index = index as u8;
        }
    }
    best_index
}

fn squared_distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    let dr = a.0 as i32 - b.0 as i32;
    let dg = a.1 as i32 - b.1 as i32;
    let db = a.2 as i32 - b.2 as i32;
    (dr * dr + dg * dg + db * db) as u32
}

/// A readable foreground colour (black or white) for text drawn on top of `bg`, chosen by
/// perceptual luminance so swatch labels stay legible against both light and dark backgrounds.
fn contrasting_fg((r, g, b): (u8, u8, u8)) -> Color {
    let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    if luminance > 140.0 {
        Color::Black
    } else {
        Color::White
    }
}

fn rgb_to_hsv_wheel_rgb(hue_deg: f32) -> (u8, u8, u8) {
    hsv_to_rgb(hue_deg, 1.0, 1.0)
}

/// Converts an HSV colour (hue in degrees, saturation and value in 0.0-1.0) to RGB.
fn hsv_to_rgb(hue_deg: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = hue_deg.rem_euclid(360.0) / 60.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    )
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

/// The red-to-blue gradient used by both the plain gradient row and the banding test, at
/// position `t` (0.0-1.0).
fn red_to_blue(t: f32) -> (u8, u8, u8) {
    (lerp_u8(255, 0, t), 0, lerp_u8(0, 255, t))
}

fn header(text: impl Into<String>) -> Line<'static> {
    Line::styled(
        text.into(),
        theme::header_style(theme::accent(Tab::Colours)),
    )
}

/// Builds one row of coloured swatch cells, each labelled, from an iterator of (label, rgb).
fn swatch_row(entries: impl Iterator<Item = (String, (u8, u8, u8))>) -> Line<'static> {
    let mut spans = Vec::new();
    for (label, rgb) in entries {
        let (r, g, b) = rgb;
        spans.push(Span::styled(
            format!(" {label} "),
            Style::default()
                .bg(Color::Rgb(r, g, b))
                .fg(contrasting_fg(rgb)),
        ));
    }
    Line::from(spans)
}

/// A single unlabelled swatch cell (two spaces wide), used for the colour cube and greyscale
/// ramp where individual cells are too small to hold a label.
fn cell(rgb: (u8, u8, u8)) -> Span<'static> {
    let (r, g, b) = rgb;
    Span::styled("  ", Style::default().bg(Color::Rgb(r, g, b)))
}

/// Renders the 6x6x6 colour cube as six blocks of 36 cells: one block per blue level, each block
/// a 6 (green rows) x 6 (red columns) grid.
fn cube_lines() -> Vec<Line<'static>> {
    (0..6)
        .map(|green| {
            let mut spans = Vec::new();
            for blue in 0..6u8 {
                if blue > 0 {
                    spans.push(Span::raw(" "));
                }
                for red in 0..6u8 {
                    let rgb = (cube_level(red), cube_level(green), cube_level(blue));
                    spans.push(cell(rgb));
                }
            }
            Line::from(spans)
        })
        .collect()
}

fn greyscale_line() -> Line<'static> {
    let spans = (232..=255u16)
        .map(|index| cell(xterm256_to_rgb(index as u8)))
        .collect::<Vec<_>>();
    Line::from(spans)
}

/// A full-width line of single-column cells, each coloured by calling `colour_at(t)` for its
/// fractional position `t` (0.0-1.0) across `width` columns.
fn gradient_line(width: u16, colour_at: impl Fn(f32) -> (u8, u8, u8)) -> Line<'static> {
    let width = width.max(1);
    let spans = (0..width)
        .map(|x| {
            let t = x as f32 / (width.saturating_sub(1).max(1)) as f32;
            let (r, g, b) = colour_at(t);
            Span::styled(" ", Style::default().bg(Color::Rgb(r, g, b)))
        })
        .collect::<Vec<_>>();
    Line::from(spans)
}

/// The banding test: the red-to-blue gradient rendered twice within one line, left half in full
/// truecolour, right half quantised to the nearest 256-colour index — visible stepping on the
/// right and smoothness on the left is the diagnostic.
fn banding_line(width: u16) -> Line<'static> {
    let half = (width / 2).max(1);
    let mut spans = Vec::new();
    for x in 0..half {
        let t = x as f32 / half.saturating_sub(1).max(1) as f32;
        let (r, g, b) = red_to_blue(t);
        spans.push(Span::styled(" ", Style::default().bg(Color::Rgb(r, g, b))));
    }
    for x in 0..half {
        let t = x as f32 / half.saturating_sub(1).max(1) as f32;
        let index = nearest_256_index(red_to_blue(t));
        spans.push(Span::styled(
            " ",
            Style::default().bg(Color::Indexed(index)),
        ));
    }
    Line::from(spans)
}

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let ti = &app.term_info;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::raw("Colour depth: "),
        Span::styled(
            ti.colour_depth.label(),
            theme::header_style(theme::accent(Tab::Colours)),
        ),
        Span::raw(format!(" — {}", ti.colour_depth_reason)),
    ]));
    if ti.colour_depth != ColourDepth::TrueColour {
        lines.push(Line::styled(
            format!(
                "Warning: this tab renders 24-bit truecolour swatches even though the terminal only advertised {} — colours below may clip or degrade unexpectedly.",
                ti.colour_depth.label()
            ),
            theme::warning_style(),
        ));
    }
    lines.push(Line::from(""));

    lines.push(header("16 ANSI colours (normal + bright)"));
    lines.push(swatch_row(
        ANSI16[0..8]
            .iter()
            .enumerate()
            .map(|(i, (name, rgb))| (format!("{i}:{name}"), *rgb)),
    ));
    lines.push(swatch_row(
        ANSI16[8..16]
            .iter()
            .enumerate()
            .map(|(i, (name, rgb))| (format!("{}:{name}", i + 8), *rgb)),
    ));
    lines.push(Line::from(""));

    lines.push(header(
        "6x6x6 colour cube (indices 16-231), six blocks of 36 cells",
    ));
    lines.extend(cube_lines());
    lines.push(Line::from(""));

    lines.push(header("24-step greyscale ramp (indices 232-255)"));
    lines.push(greyscale_line());
    lines.push(Line::from(""));

    lines.push(header("Truecolour hue sweep"));
    lines.push(gradient_line(area.width, |t| {
        rgb_to_hsv_wheel_rgb(t * 360.0)
    }));
    lines.push(Line::from(""));

    lines.push(header("Red-to-blue truecolour gradient"));
    lines.push(gradient_line(area.width, red_to_blue));
    lines.push(Line::from(""));

    lines.push(header(
        "Banding test: truecolour (left) vs. nearest 256-colour index (right)",
    ));
    lines.push(banding_line(area.width));

    frame.render_widget(Paragraph::new(lines), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_corner_indices_match_known_values() {
        // Index 16: the cube's (0,0,0) corner.
        assert_eq!(xterm256_to_rgb(16), (0, 0, 0));
        // Index 21: r=0 g=0 b=5 -> pure blue.
        assert_eq!(xterm256_to_rgb(21), (0, 0, 255));
        // Index 196: r=5 g=0 b=0 -> pure red.
        assert_eq!(xterm256_to_rgb(196), (255, 0, 0));
        // Index 231: the cube's (5,5,5) corner -> white.
        assert_eq!(xterm256_to_rgb(231), (255, 255, 255));
    }

    #[test]
    fn greyscale_endpoints_match_known_values() {
        assert_eq!(xterm256_to_rgb(232), (8, 8, 8));
        assert_eq!(xterm256_to_rgb(255), (238, 238, 238));
    }

    #[test]
    fn cube_interior_index_matches_known_hex() {
        // Index 110 is the well-known #87afd7 ("SkyBlue3"): r=2 g=3 b=4.
        assert_eq!(xterm256_to_rgb(110), (135, 175, 215));
    }

    #[test]
    fn quantiser_round_trips_exact_black() {
        assert_eq!(nearest_256_index((0, 0, 0)), 0);
    }

    #[test]
    fn quantiser_round_trips_unambiguous_cube_colour() {
        // (135, 175, 215) exactly matches only index 110 in the whole 256-colour palette.
        assert_eq!(nearest_256_index((135, 175, 215)), 110);
    }

    #[test]
    fn quantiser_round_trips_unambiguous_greyscale_colour() {
        // (38, 38, 38) exactly matches only index 235.
        assert_eq!(nearest_256_index((38, 38, 38)), 235);
    }

    #[test]
    fn quantiser_prefers_lowest_index_on_exact_tie() {
        // (255, 0, 0) exactly matches both index 9 (ANSI "Red") and index 196 (cube). The lowest
        // index wins, deterministically.
        assert_eq!(nearest_256_index((255, 0, 0)), 9);
    }
}
