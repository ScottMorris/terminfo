// Graphics-protocol detection orchestration and the Graphics tab's live render/cache state.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use image::DynamicImage;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::FontSize;

pub mod artwork;
pub mod detect;

use artwork::Artwork;

/// The result of running graphics-protocol detection at startup: the built `Picker`, the
/// human-readable reasoning behind its choice, and whether `TERMINFO_FORCE_PROTOCOL` overrode it.
pub struct GraphicsDetection {
    pub picker: Picker,
    pub reasons: Vec<String>,
    pub forced_by_env: bool,
}

/// Runs graphics-protocol detection: `Picker::from_query_stdio()` performs live terminal queries
/// (DA1, Kitty graphics query, font-size query), falling back to `Picker::halfblocks()` on any
/// failure, then applies `TERMINFO_FORCE_PROTOCOL` if it names a recognised protocol. Must run
/// after entering the alternate screen but strictly before any other terminal event is read (see
/// SPEC.md's startup sequence) — `main.rs` calls this at exactly that point.
pub fn detect() -> GraphicsDetection {
    let forced = detect::env_force_protocol();

    let (mut picker, query_succeeded) = match Picker::from_query_stdio() {
        Ok(picker) => (picker, true),
        Err(_) => (Picker::halfblocks(), false),
    };

    if let Some((_, protocol_type)) = forced {
        picker.set_protocol_type(protocol_type);
    }

    let reasons = detect::build_reasons(
        &picker,
        query_succeeded,
        forced.as_ref().map(|(raw, _)| raw.as_str()),
    );

    GraphicsDetection {
        picker,
        reasons,
        forced_by_env: forced.is_some(),
    }
}

/// A short human-readable label for a `ProtocolType`, used on both the Overview summary line and
/// the Graphics tab's info panel.
pub fn protocol_label(protocol: ProtocolType) -> &'static str {
    match protocol {
        ProtocolType::Halfblocks => "Unicode half-blocks",
        ProtocolType::Sixel => "Sixel",
        ProtocolType::Kitty => "Kitty graphics protocol",
        ProtocolType::Iterm2 => "iTerm2 inline images",
    }
}

/// The last generated raw image, cached by the exact inputs that produced it, so redraws that
/// don't change size, artwork, or phase are free (SPEC.md's render-cache requirement).
struct RenderCache {
    width: u32,
    height: u32,
    artwork: Artwork,
    phase_bits: u32,
    image: image::RgbImage,
}

impl RenderCache {
    fn matches(&self, width: u32, height: u32, artwork: Artwork, phase: f32) -> bool {
        self.width == width
            && self.height == height
            && self.artwork == artwork
            && self.phase_bits == phase.to_bits()
    }
}

/// Live state for the Graphics tab: the picker/protocol, the current artwork and colour phase,
/// the render cache, and bookkeeping surfaced on the info panel (dimensions, render time,
/// encoding error).
pub struct GraphicsState {
    picker: Picker,
    phase: f32,
    built_protocol_type: ProtocolType,
    render_cache: RenderCache,

    pub reasons: Vec<String>,
    pub forced_by_env: bool,
    pub forced_by_key: bool,
    pub artwork: Artwork,
    pub protocol: StatefulProtocol,
    pub image_dims: (u32, u32),
    pub last_render_ms: Option<u128>,
    pub last_error: Option<String>,
}

impl GraphicsState {
    /// Builds the initial Graphics tab state from a completed detection. A minimal placeholder
    /// image is rendered up front; the real render happens on the first draw, once the tab's
    /// actual image-region pixel size is known (see `ensure_render`).
    pub fn new(detection: GraphicsDetection) -> GraphicsState {
        let artwork = Artwork::Julia;
        let phase = 0.0f32;
        let (image, render_ms) = timed_generate(artwork, 1, 1, phase);
        let picker = detection.picker;
        let built_protocol_type = picker.protocol_type();
        let protocol = picker.new_resize_protocol(DynamicImage::ImageRgb8(image.clone()));

        GraphicsState {
            picker,
            phase,
            built_protocol_type,
            render_cache: RenderCache {
                width: 1,
                height: 1,
                artwork,
                phase_bits: phase.to_bits(),
                image,
            },
            reasons: detection.reasons,
            forced_by_env: detection.forced_by_env,
            forced_by_key: false,
            artwork,
            protocol,
            image_dims: (1, 1),
            last_render_ms: Some(render_ms),
            last_error: None,
        }
    }

    pub fn protocol_type(&self) -> ProtocolType {
        self.picker.protocol_type()
    }

    pub fn font_size(&self) -> FontSize {
        self.picker.font_size()
    }

    /// Cycles to the next artwork (Julia -> Plasma -> ColourWheel -> Julia). Takes effect on the
    /// next `ensure_render` call, i.e. the next draw.
    pub fn cycle_artwork(&mut self) {
        self.artwork = self.artwork.next();
    }

    /// Bumps the colour phase by 0.05, wrapping at 1.0.
    pub fn bump_phase(&mut self) {
        self.phase = (self.phase + 0.05) % 1.0;
    }

    /// Force-cycles to the next graphics protocol, for runtime comparison. The caller is
    /// responsible for clearing the terminal before the next draw (see `App::needs_clear`), since
    /// switching protocols live can leave visual artefacts from the previous one.
    pub fn cycle_protocol(&mut self) {
        let next = self.picker.protocol_type().next();
        self.picker.set_protocol_type(next);
        self.forced_by_key = true;
    }

    /// Forces the next `ensure_render` call to regenerate, regardless of whether the target pixel
    /// size actually changed. Called on `Event::Resize`.
    pub fn invalidate(&mut self) {
        self.render_cache.width = 0;
    }

    /// Ensures the cached render matches the given target pixel size, the current artwork, and
    /// the current colour phase, regenerating (and rebuilding the terminal-protocol encoding)
    /// only when something actually changed. Called every draw with the image region's pixel
    /// size, which is also how the image regenerates automatically on resize.
    pub fn ensure_render(&mut self, width: u32, height: u32) {
        let width = width.clamp(1, artwork::MAX_WIDTH);
        let height = height.clamp(1, artwork::MAX_HEIGHT);

        if !self
            .render_cache
            .matches(width, height, self.artwork, self.phase)
        {
            let (image, render_ms) = timed_generate(self.artwork, width, height, self.phase);
            self.image_dims = (image.width(), image.height());
            self.last_render_ms = Some(render_ms);
            self.render_cache = RenderCache {
                width,
                height,
                artwork: self.artwork,
                phase_bits: self.phase.to_bits(),
                image,
            };
            self.rebuild_protocol();
        } else if self.built_protocol_type != self.picker.protocol_type() {
            self.rebuild_protocol();
        }
    }

    fn rebuild_protocol(&mut self) {
        let image = self.render_cache.image.clone();
        self.protocol = self
            .picker
            .new_resize_protocol(DynamicImage::ImageRgb8(image));
        self.built_protocol_type = self.picker.protocol_type();
    }

    /// Picks up the result of the most recent resize/encode (if any happened since the last
    /// check) and records it as the last encoding error, for the info panel. Call this once per
    /// draw, after rendering the `StatefulImage` widget.
    pub fn poll_encoding_result(&mut self) {
        if let Some(result) = self.protocol.last_encoding_result() {
            self.last_error = result.err().map(|e| e.to_string());
        }
    }
}

fn timed_generate(
    artwork: Artwork,
    width: u32,
    height: u32,
    phase: f32,
) -> (image::RgbImage, u128) {
    let start = std::time::Instant::now();
    let image = artwork::generate(artwork, width, height, phase);
    (image, start.elapsed().as_millis())
}
