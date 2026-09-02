// Procedural artwork generators (Julia set, plasma, colour wheel): deterministic, pure functions
// of (width, height, phase) producing an in-memory RGB image for the Graphics tab.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use image::{Rgb, RgbImage};

/// The three artworks the Graphics tab can display, in cycle order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Artwork {
    Julia,
    Plasma,
    ColourWheel,
}

impl Artwork {
    /// The name shown on the Graphics tab's info panel.
    pub fn name(self) -> &'static str {
        match self {
            Artwork::Julia => "Julia set",
            Artwork::Plasma => "Plasma",
            Artwork::ColourWheel => "Colour wheel",
        }
    }

    /// The next artwork in the cycle: Julia -> Plasma -> ColourWheel -> Julia.
    pub fn next(self) -> Artwork {
        match self {
            Artwork::Julia => Artwork::Plasma,
            Artwork::Plasma => Artwork::ColourWheel,
            Artwork::ColourWheel => Artwork::Julia,
        }
    }
}

/// The rendered image is capped at this size regardless of the requested tab area, per SPEC.md's
/// "Procedural artwork" section.
pub const MAX_WIDTH: u32 = 1024;
pub const MAX_HEIGHT: u32 = 768;

const MAX_ITER: u32 = 96;
const JULIA_C: (f32, f32) = (-0.8, 0.156);

/// Generates the given artwork at (at most) `width` x `height`, at colour `phase` (0.0-1.0,
/// wrapping). Pure and deterministic: identical inputs always produce a byte-identical image. A
/// zero width or height (and a 1x1 request) are handled without panicking.
pub fn generate(artwork: Artwork, width: u32, height: u32, phase: f32) -> RgbImage {
    let width = width.min(MAX_WIDTH);
    let height = height.min(MAX_HEIGHT);
    if width == 0 || height == 0 {
        return RgbImage::new(width, height);
    }
    match artwork {
        Artwork::Julia => julia(width, height, phase),
        Artwork::Plasma => plasma(width, height, phase),
        Artwork::ColourWheel => colour_wheel(width, height, phase),
    }
}

/// The cyclic cosine palette shared by Julia and Plasma: `col(t) = a + b * cos(2*pi*(c*t + d))`,
/// with `a = b = (0.5, 0.5, 0.5)`, `c = (1, 1, 1)`, `d = (0.00, 0.33, 0.67)`.
fn cosine_palette(t: f32) -> Rgb<u8> {
    const A: f32 = 0.5;
    const B: f32 = 0.5;
    const D: (f32, f32, f32) = (0.00, 0.33, 0.67);
    let r = A + B * (std::f32::consts::TAU * (t + D.0)).cos();
    let g = A + B * (std::f32::consts::TAU * (t + D.1)).cos();
    let b = A + B * (std::f32::consts::TAU * (t + D.2)).cos();
    Rgb([to_channel(r), to_channel(g), to_channel(b)])
}

fn to_channel(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn lerp_channel(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)).round() as u8
}

/// The Julia set for `c = -0.8 + 0.156i`, viewport `re in [-1.6, 1.6]` with `im` scaled to match
/// the image's aspect ratio, `MAX_ITER = 96`, bailout `|z|^2 > 16`. Escaped pixels use the smooth
/// iteration count `mu = n + 1 - log2(ln|z|)` through the cyclic cosine palette; interior
/// (non-escaping) pixels get a dark navy-to-violet gradient keyed on their final `|z|^2`, so the
/// interior is not flat black.
fn julia(width: u32, height: u32, phase: f32) -> RgbImage {
    let (cr, ci) = JULIA_C;
    let half_re = 1.6f32;
    let half_im = half_re * height as f32 / width as f32;
    const INTERIOR_NAVY: (u8, u8, u8) = (10, 8, 40);
    const INTERIOR_VIOLET: (u8, u8, u8) = (90, 30, 130);

    RgbImage::from_fn(width, height, |x, y| {
        let mut zr = -half_re + (x as f32 / width as f32) * (2.0 * half_re);
        let mut zi = -half_im + (y as f32 / height as f32) * (2.0 * half_im);
        let mut n = 0u32;
        let mut abs2 = zr * zr + zi * zi;
        while n < MAX_ITER && abs2 <= 16.0 {
            let new_zr = zr * zr - zi * zi + cr;
            let new_zi = 2.0 * zr * zi + ci;
            zr = new_zr;
            zi = new_zi;
            abs2 = zr * zr + zi * zi;
            n += 1;
        }
        if n >= MAX_ITER {
            let g = (abs2 / 16.0).clamp(0.0, 1.0);
            Rgb([
                lerp_channel(INTERIOR_NAVY.0, INTERIOR_VIOLET.0, g),
                lerp_channel(INTERIOR_NAVY.1, INTERIOR_VIOLET.1, g),
                lerp_channel(INTERIOR_NAVY.2, INTERIOR_VIOLET.2, g),
            ])
        } else {
            let z_abs = abs2.sqrt();
            let mu = n as f32 + 1.0 - z_abs.ln().log2();
            let t = mu / MAX_ITER as f32 + phase;
            cosine_palette(t)
        }
    })
}

/// A sum of four sine terms over normalised pixel coordinates, mapped through the same cyclic
/// cosine palette as Julia.
fn plasma(width: u32, height: u32, phase: f32) -> RgbImage {
    RgbImage::from_fn(width, height, |x, y| {
        let nx = x as f32 / width as f32;
        let ny = y as f32 / height as f32;
        let v1 = (nx * 10.0).sin();
        let v2 = (ny * 10.0).sin();
        let v3 = ((nx + ny) * 10.0).sin();
        let v4 = (((nx - 0.5).powi(2) + (ny - 0.5).powi(2)).sqrt() * 20.0).sin();
        let v = (v1 + v2 + v3 + v4) / 4.0; // -1.0..=1.0
        let t = (v + 1.0) / 2.0 + phase;
        cosine_palette(t)
    })
}

/// A polar HSV wheel (hue = angle, saturation = radius, value = 1), with a checkerboard border
/// outside the wheel — useful for judging a protocol's colour accuracy.
fn colour_wheel(width: u32, height: u32, phase: f32) -> RgbImage {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let radius = cx.min(cy);

    RgbImage::from_fn(width, height, |x, y| {
        let dx = x as f32 + 0.5 - cx;
        let dy = y as f32 + 0.5 - cy;
        let r = (dx * dx + dy * dy).sqrt();
        if r > radius {
            let checker = ((x / 8) + (y / 8)) % 2 == 0;
            if checker {
                Rgb([40, 40, 40])
            } else {
                Rgb([210, 210, 210])
            }
        } else {
            let hue = (dy.atan2(dx).to_degrees() + 360.0 + phase * 360.0).rem_euclid(360.0);
            let sat = (r / radius).clamp(0.0, 1.0);
            hsv_to_rgb(hue, sat, 1.0)
        }
    })
}

/// Converts an HSV colour (hue in degrees, saturation and value in 0.0-1.0) to RGB.
fn hsv_to_rgb(hue_deg: f32, s: f32, v: f32) -> Rgb<u8> {
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
    Rgb([to_channel(r1 + m), to_channel(g1 + m), to_channel(b1 + m)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: [Artwork; 3] = [Artwork::Julia, Artwork::Plasma, Artwork::ColourWheel];

    #[test]
    fn output_dimensions_match_request() {
        for artwork in ALL {
            let image = generate(artwork, 200, 100, 0.0);
            assert_eq!(image.width(), 200);
            assert_eq!(image.height(), 100);
        }
    }

    #[test]
    fn requested_size_is_capped() {
        let image = generate(Artwork::Julia, MAX_WIDTH + 500, MAX_HEIGHT + 500, 0.0);
        assert_eq!(image.width(), MAX_WIDTH);
        assert_eq!(image.height(), MAX_HEIGHT);
    }

    #[test]
    fn identical_inputs_are_byte_identical() {
        for artwork in ALL {
            let a = generate(artwork, 96, 64, 0.37);
            let b = generate(artwork, 96, 64, 0.37);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn julia_64x64_has_colour_diversity() {
        let image = generate(Artwork::Julia, 64, 64, 0.0);
        let distinct: HashSet<[u8; 3]> = image.pixels().map(|p| p.0).collect();
        assert!(
            distinct.len() >= 64,
            "expected at least 64 distinct colours, got {}",
            distinct.len()
        );
    }

    #[test]
    fn handles_1x1_request_without_panicking() {
        for artwork in ALL {
            let image = generate(artwork, 1, 1, 0.5);
            assert_eq!((image.width(), image.height()), (1, 1));
        }
    }

    #[test]
    fn handles_zero_size_without_panicking() {
        for artwork in ALL {
            assert_eq!(generate(artwork, 0, 100, 0.0).as_raw().len(), 0);
            assert_eq!(generate(artwork, 100, 0, 0.0).as_raw().len(), 0);
            assert_eq!(generate(artwork, 0, 0, 0.0).as_raw().len(), 0);
        }
    }

    #[test]
    fn artwork_cycle_wraps() {
        assert_eq!(Artwork::Julia.next(), Artwork::Plasma);
        assert_eq!(Artwork::Plasma.next(), Artwork::ColourWheel);
        assert_eq!(Artwork::ColourWheel.next(), Artwork::Julia);
    }
}
