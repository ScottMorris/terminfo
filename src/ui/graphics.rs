// The Graphics tab. Placeholder for this chunk — full implementation lands in chunk 3.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, _app: &App) {
    let text = "Coming in chunk 3: a procedurally generated image rendered through the detected graphics protocol (Kitty, iTerm2, Sixel, or Unicode half-blocks), with detection reasoning and artwork controls.";
    let rect = super::centred_rect(area, area.width.min(56), 4.min(area.height));
    let paragraph = Paragraph::new(Line::from(text))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, rect);
}
