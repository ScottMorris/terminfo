// Human-readable reasoning behind graphics-protocol detection: environment markers observed,
// whether the live stdio query succeeded, reported capabilities, font-cell size and its source,
// a tmux-passthrough caveat, and any `TERMINFO_FORCE_PROTOCOL` testing override.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui_image::picker::{Capability, Picker, ProtocolType};

/// Environment variables consulted (directly or indirectly, via ratatui-image's own detection)
/// when guessing a graphics protocol, surfaced on the "environment markers observed" line.
const DETECTION_ENV_VARS: [&str; 7] = [
    "TERM",
    "TERM_PROGRAM",
    "KITTY_WINDOW_ID",
    "WEZTERM_EXECUTABLE",
    "ITERM_SESSION_ID",
    "KONSOLE_VERSION",
    "LC_TERMINAL",
];

/// Parses `$TERMINFO_FORCE_PROTOCOL`, returning its raw value and the protocol it names, if it is
/// set to one of the recognised values (`halfblocks`, `sixel`, `kitty`, `iterm2`). An unset or
/// unrecognised value is treated as no override, per SPEC.md's "Graphics protocol detection and
/// selection" section.
pub fn env_force_protocol() -> Option<(String, ProtocolType)> {
    let raw = std::env::var("TERMINFO_FORCE_PROTOCOL").ok()?;
    let protocol = match raw.to_ascii_lowercase().as_str() {
        "halfblocks" => ProtocolType::Halfblocks,
        "sixel" => ProtocolType::Sixel,
        "kitty" => ProtocolType::Kitty,
        "iterm2" => ProtocolType::Iterm2,
        _ => return None,
    };
    Some((raw, protocol))
}

fn describe_capability(cap: &Capability) -> String {
    match cap {
        Capability::Kitty => "Terminal reported Kitty graphics protocol support".to_string(),
        Capability::Sixel => "Terminal reported Sixel graphics support".to_string(),
        Capability::RectangularOps => "Terminal reported rectangular Sixel operations".to_string(),
        Capability::CellSize(Some((w, h))) => {
            format!("Terminal reported a {w}x{h}px font cell size")
        }
        Capability::CellSize(None) => {
            "Terminal answered the font-cell-size query with no usable size".to_string()
        }
        Capability::TextSizingProtocol => {
            "Terminal reported the Kitty text-sizing protocol".to_string()
        }
        Capability::Background(r, g, b) => {
            format!("Terminal reported background colour #{r:02x}{g:02x}{b:02x}")
        }
    }
}

fn observed_env_vars() -> Vec<String> {
    DETECTION_ENV_VARS
        .iter()
        .filter_map(|key| {
            std::env::var(key)
                .ok()
                .filter(|v| !v.is_empty())
                .map(|v| format!("${key}={v}"))
        })
        .collect()
}

/// Builds the bulleted detection-reasoning list shown on the Graphics tab's info panel:
/// environment markers observed, whether the live stdio query succeeded, reported capabilities,
/// font-cell size and its source, a tmux-passthrough caveat when `$TMUX` is set, and — first, if
/// present — a note that `TERMINFO_FORCE_PROTOCOL` overrode detection.
pub fn build_reasons(
    picker: &Picker,
    query_succeeded: bool,
    forced_raw: Option<&str>,
) -> Vec<String> {
    let mut reasons = Vec::new();

    if let Some(raw) = forced_raw {
        reasons.push(format!(
            "TERMINFO_FORCE_PROTOCOL={raw} overrides detection (testing override, not organic detection)"
        ));
    }

    let vars = observed_env_vars();
    reasons.push(if vars.is_empty() {
        "Environment markers observed: none".to_string()
    } else {
        format!("Environment markers observed: {}", vars.join(", "))
    });

    reasons.push(if query_succeeded {
        "Live stdio query (DA1 / Kitty graphics query / font-size query) succeeded".to_string()
    } else {
        "Live stdio query failed, timed out, or stdin is not a tty; falling back to half-blocks"
            .to_string()
    });

    for cap in picker.capabilities() {
        reasons.push(describe_capability(cap));
    }

    let font_size = picker.font_size();
    reasons.push(format!(
        "Font cell size: {}x{}px ({})",
        font_size.width,
        font_size.height,
        if query_succeeded {
            "measured via stdio query"
        } else {
            "default fallback, not measured"
        }
    ));

    if std::env::var("TMUX").is_ok() {
        reasons.push(
            "Running inside tmux: Kitty graphics needs `allow-passthrough on` in tmux.conf, and Sixel support depends on the outer terminal.".to_string(),
        );
    }

    reasons
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_force_protocol_parses_known_and_rejects_unknown_values() {
        std::env::set_var("TERMINFO_FORCE_PROTOCOL", "kitty");
        assert_eq!(
            env_force_protocol(),
            Some(("kitty".to_string(), ProtocolType::Kitty))
        );

        std::env::set_var("TERMINFO_FORCE_PROTOCOL", "nonsense");
        assert_eq!(env_force_protocol(), None);

        std::env::remove_var("TERMINFO_FORCE_PROTOCOL");
        assert_eq!(env_force_protocol(), None);
    }

    #[test]
    fn build_reasons_notes_the_override_first_when_forced() {
        let picker = Picker::halfblocks();
        let reasons = build_reasons(&picker, false, Some("sixel"));
        assert!(reasons[0].contains("TERMINFO_FORCE_PROTOCOL=sixel"));
    }

    #[test]
    fn build_reasons_notes_a_failed_query() {
        let picker = Picker::halfblocks();
        let reasons = build_reasons(&picker, false, None);
        assert!(reasons.iter().any(|r| r.contains("failed")));
    }

    #[test]
    fn build_reasons_includes_font_cell_size() {
        let picker = Picker::halfblocks();
        let reasons = build_reasons(&picker, true, None);
        assert!(reasons.iter().any(|r| r.contains("Font cell size")));
    }
}
