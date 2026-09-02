// The Colours tab. Placeholder for this chunk — full implementation lands in chunk 2.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, _app: &App) {
    let text = "Coming in chunk 2: ANSI swatches, the 6x6x6 colour cube, the greyscale ramp, a truecolour hue sweep, and the 256-colour banding test.";
    let rect = super::centred_rect(area, area.width.min(56), 4.min(area.height));
    let paragraph = Paragraph::new(Line::from(text))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, rect);
}
