// Gathers terminal identity, geometry, and capability facts, and infers colour depth.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::path::PathBuf;

/// The inferred colour depth of the running terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColourDepth {
    Monochrome,
    Ansi16,
    Xterm256,
    TrueColour,
}

impl ColourDepth {
    /// A short human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            ColourDepth::Monochrome => "Monochrome",
            ColourDepth::Ansi16 => "ANSI 16-colour",
            ColourDepth::Xterm256 => "Xterm 256-colour",
            ColourDepth::TrueColour => "Truecolour (24-bit)",
        }
    }
}

/// `$TERM_PROGRAM` values known to always support truecolour.
const TRUECOLOUR_PROGRAMS: [&str; 3] = ["iTerm.app", "WezTerm", "vscode"];

/// `$TERM` prefixes recognised as colour-capable at the ANSI-16 level.
const ANSI16_TERM_PREFIXES: [&str; 6] = ["xterm", "screen", "linux", "rxvt", "vt100", "tmux"];

/// Infers the terminal's colour depth from environment variables, following the priority order
/// documented in `SPEC.md`'s "Colour depth inference" section. Returns the inferred depth
/// alongside a human-readable reason.
pub fn infer_colour_depth(env: &HashMap<String, String>) -> (ColourDepth, String) {
    let colorterm = env.get("COLORTERM").map(String::as_str).unwrap_or("");
    let term = env.get("TERM").map(String::as_str).unwrap_or("");
    let term_program = env.get("TERM_PROGRAM").map(String::as_str).unwrap_or("");

    if colorterm == "truecolor" || colorterm == "24bit" {
        return (
            ColourDepth::TrueColour,
            format!("$COLORTERM is \"{colorterm}\""),
        );
    }

    if term.ends_with("-direct") {
        return (
            ColourDepth::TrueColour,
            format!("$TERM (\"{term}\") ends in \"-direct\""),
        );
    }

    if TRUECOLOUR_PROGRAMS.contains(&term_program) {
        return (
            ColourDepth::TrueColour,
            format!("$TERM_PROGRAM (\"{term_program}\") always supports truecolour"),
        );
    }

    if env.contains_key("KITTY_WINDOW_ID") {
        return (
            ColourDepth::TrueColour,
            "$KITTY_WINDOW_ID is set (kitty terminal)".to_string(),
        );
    }

    if term.contains("256color") {
        return (
            ColourDepth::Xterm256,
            format!("$TERM (\"{term}\") contains \"256color\""),
        );
    }

    if term == "dumb" {
        return (ColourDepth::Monochrome, "$TERM is \"dumb\"".to_string());
    }

    if !term.is_empty()
        && (ANSI16_TERM_PREFIXES
            .iter()
            .any(|prefix| term.starts_with(prefix))
            || term.contains("color")
            || term.contains("colour"))
    {
        return (
            ColourDepth::Ansi16,
            format!("$TERM (\"{term}\") is recognised as colour-capable"),
        );
    }

    (
        ColourDepth::Ansi16,
        "no colour-capable signal found; defaulting to ANSI 16-colour".to_string(),
    )
}

/// Standard directories the terminfo database is conventionally installed under.
const STANDARD_TERMINFO_DIRS: [&str; 4] = [
    "/etc/terminfo",
    "/lib/terminfo",
    "/usr/share/terminfo",
    "/usr/local/share/terminfo",
];

/// Probes the standard terminfo directories (plus `$TERMINFO` and `$TERMINFO_DIRS`) for an
/// entry matching `term`, trying both the single-letter and hex-code subdirectory conventions.
/// Returns the first matching path found, if any.
fn resolve_terminfo_path(term: &str, env: &HashMap<String, String>) -> Option<PathBuf> {
    if term.is_empty() {
        return None;
    }
    let first = term.chars().next()?;
    let subdirs = [first.to_string(), format!("{:02x}", first as u32)];

    let mut candidate_dirs: Vec<PathBuf> = Vec::new();
    if let Some(terminfo) = env.get("TERMINFO") {
        candidate_dirs.push(PathBuf::from(terminfo));
    }
    if let Some(home) = env.get("HOME") {
        candidate_dirs.push(PathBuf::from(home).join(".terminfo"));
    }
    if let Some(dirs) = env.get("TERMINFO_DIRS") {
        for dir in dirs.split(':').filter(|d| !d.is_empty()) {
            candidate_dirs.push(PathBuf::from(dir));
        }
    }
    for dir in STANDARD_TERMINFO_DIRS {
        candidate_dirs.push(PathBuf::from(dir));
    }

    for dir in candidate_dirs {
        for subdir in &subdirs {
            let candidate = dir.join(subdir).join(term);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Best-guess human-readable terminal name, derived from a small heuristic table over
/// environment markers set by specific terminal emulators. Falls back to the raw `$TERM` value
/// when nothing more specific matches, and to "unknown" when even that is unset.
fn detect_terminal_name(env: &HashMap<String, String>) -> String {
    if let Some(program) = env.get("TERM_PROGRAM") {
        if !program.is_empty() {
            return program.clone();
        }
    }
    if env.contains_key("KITTY_WINDOW_ID") {
        return "kitty".to_string();
    }
    if env.contains_key("WEZTERM_EXECUTABLE") {
        return "WezTerm".to_string();
    }
    if env.contains_key("ALACRITTY_SOCKET") {
        return "Alacritty".to_string();
    }
    if env.contains_key("KONSOLE_VERSION") {
        return "Konsole".to_string();
    }
    if env.contains_key("WT_SESSION") {
        return "Windows Terminal".to_string();
    }
    if env.contains_key("VTE_VERSION") {
        return "VTE-based (e.g. GNOME Terminal)".to_string();
    }
    match env.get("TERM") {
        Some(term) if !term.is_empty() => term.clone(),
        _ => "unknown".to_string(),
    }
}

/// Detects a terminal multiplexer from the environment, if any.
fn detect_multiplexer(env: &HashMap<String, String>) -> Option<String> {
    if env.contains_key("TMUX") {
        Some("tmux".to_string())
    } else if env.contains_key("STY") {
        Some("screen".to_string())
    } else {
        None
    }
}

/// Detects an SSH session from the environment, if any.
fn detect_ssh(env: &HashMap<String, String>) -> Option<String> {
    if let Some(tty) = env.get("SSH_TTY") {
        Some(format!("SSH_TTY={tty}"))
    } else {
        env.get("SSH_CONNECTION")
            .map(|conn| format!("SSH_CONNECTION={conn}"))
    }
}

/// Live geometry: cell and, where reported, pixel size.
#[derive(Debug, Clone, Copy, Default)]
pub struct Geometry {
    pub cols: u16,
    pub rows: u16,
    pub pixel_width: Option<u16>,
    pub pixel_height: Option<u16>,
}

impl Geometry {
    /// The per-cell pixel size, when both cell and pixel geometry are available.
    pub fn cell_pixel_size(&self) -> Option<(f32, f32)> {
        let (pw, ph) = (self.pixel_width?, self.pixel_height?);
        if self.cols == 0 || self.rows == 0 {
            return None;
        }
        Some((pw as f32 / self.cols as f32, ph as f32 / self.rows as f32))
    }

    fn query() -> Geometry {
        let (cols, rows) = crossterm::terminal::size().unwrap_or((0, 0));
        let (pixel_width, pixel_height) = match crossterm::terminal::window_size() {
            Ok(size) if size.width > 0 && size.height > 0 => (Some(size.width), Some(size.height)),
            _ => (None, None),
        };
        Geometry {
            cols,
            rows,
            pixel_width,
            pixel_height,
        }
    }
}

/// Gathered facts about the running terminal: identity, geometry, and capabilities.
#[derive(Debug, Clone)]
pub struct TermInfo {
    pub detected_name: String,
    pub term: Option<String>,
    pub colorterm: Option<String>,
    pub term_program: Option<String>,
    pub term_program_version: Option<String>,
    pub multiplexer: Option<String>,
    pub ssh: Option<String>,
    pub lang: Option<String>,
    pub lc_all: Option<String>,
    pub shell: Option<String>,
    pub terminfo_path: Option<PathBuf>,
    pub geometry: Geometry,
    pub colour_depth: ColourDepth,
    pub colour_depth_reason: String,
    pub keyboard_enhancement: Option<bool>,
    pub mouse_capture_enabled: bool,
}

impl TermInfo {
    /// Gathers all terminal facts. `keyboard_enhancement` is `Some(true)`/`Some(false)` when the
    /// startup sequence has already checked `supports_keyboard_enhancement()`, or `None` if not
    /// yet known.
    pub fn gather(keyboard_enhancement: Option<bool>) -> TermInfo {
        let env: HashMap<String, String> = std::env::vars().collect();
        let (colour_depth, colour_depth_reason) = infer_colour_depth(&env);
        let term = env.get("TERM").cloned();
        let terminfo_path = term
            .as_deref()
            .and_then(|term| resolve_terminfo_path(term, &env));

        TermInfo {
            detected_name: detect_terminal_name(&env),
            term,
            colorterm: env.get("COLORTERM").cloned(),
            term_program: env.get("TERM_PROGRAM").cloned(),
            term_program_version: env.get("TERM_PROGRAM_VERSION").cloned(),
            multiplexer: detect_multiplexer(&env),
            ssh: detect_ssh(&env),
            lang: env.get("LANG").cloned(),
            lc_all: env.get("LC_ALL").cloned(),
            shell: env.get("SHELL").cloned(),
            terminfo_path,
            geometry: Geometry::query(),
            colour_depth,
            colour_depth_reason,
            keyboard_enhancement,
            mouse_capture_enabled: true,
        }
    }

    /// Re-queries terminal geometry; call this on `Event::Resize`.
    pub fn refresh_geometry(&mut self) {
        self.geometry = Geometry::query();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_with(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn colorterm_truecolor_wins() {
        let env = env_with(&[("COLORTERM", "truecolor"), ("TERM", "xterm")]);
        let (depth, _) = infer_colour_depth(&env);
        assert_eq!(depth, ColourDepth::TrueColour);
    }

    #[test]
    fn term_256color_is_xterm256() {
        let env = env_with(&[("TERM", "xterm-256color")]);
        let (depth, _) = infer_colour_depth(&env);
        assert_eq!(depth, ColourDepth::Xterm256);
    }

    #[test]
    fn term_linux_is_ansi16() {
        let env = env_with(&[("TERM", "linux")]);
        let (depth, _) = infer_colour_depth(&env);
        assert_eq!(depth, ColourDepth::Ansi16);
    }

    #[test]
    fn term_dumb_is_monochrome() {
        let env = env_with(&[("TERM", "dumb")]);
        let (depth, _) = infer_colour_depth(&env);
        assert_eq!(depth, ColourDepth::Monochrome);
    }

    #[test]
    fn term_program_iterm_is_truecolour() {
        let env = env_with(&[("TERM_PROGRAM", "iTerm.app"), ("TERM", "xterm-256color")]);
        let (depth, _) = infer_colour_depth(&env);
        assert_eq!(depth, ColourDepth::TrueColour);
    }

    #[test]
    fn kitty_window_id_is_truecolour() {
        let env = env_with(&[("KITTY_WINDOW_ID", "1"), ("TERM", "xterm-256color")]);
        let (depth, _) = infer_colour_depth(&env);
        assert_eq!(depth, ColourDepth::TrueColour);
    }

    #[test]
    fn term_xterm_direct_is_truecolour() {
        let env = env_with(&[("TERM", "xterm-direct")]);
        let (depth, _) = infer_colour_depth(&env);
        assert_eq!(depth, ColourDepth::TrueColour);
    }

    #[test]
    fn empty_env_falls_back_to_ansi16() {
        let env = HashMap::new();
        let (depth, reason) = infer_colour_depth(&env);
        assert_eq!(depth, ColourDepth::Ansi16);
        assert!(reason.contains("default"));
    }

    #[test]
    fn priority_order_colorterm_beats_term_direct() {
        let env = env_with(&[("COLORTERM", "24bit"), ("TERM", "xterm-256color")]);
        let (depth, _) = infer_colour_depth(&env);
        assert_eq!(depth, ColourDepth::TrueColour);
    }

    #[test]
    fn detects_terminal_name_from_term_program() {
        let env = env_with(&[("TERM_PROGRAM", "iTerm.app")]);
        assert_eq!(detect_terminal_name(&env), "iTerm.app");
    }

    #[test]
    fn detects_terminal_name_from_kitty_marker() {
        let env = env_with(&[("KITTY_WINDOW_ID", "1")]);
        assert_eq!(detect_terminal_name(&env), "kitty");
    }

    #[test]
    fn detects_terminal_name_falls_back_to_term() {
        let env = env_with(&[("TERM", "xterm-256color")]);
        assert_eq!(detect_terminal_name(&env), "xterm-256color");
    }

    #[test]
    fn detects_terminal_name_falls_back_to_unknown() {
        let env = HashMap::new();
        assert_eq!(detect_terminal_name(&env), "unknown");
    }
}
