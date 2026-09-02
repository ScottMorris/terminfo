// The Graphics tab: the procedurally generated image rendered through the detected graphics
// protocol, alongside an info panel with detection reasoning and render bookkeeping.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;
use ratatui_image::StatefulImage;

use super::theme;
use crate::app::App;
use crate::graphics::protocol_label;
use crate::tabs::Tab;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);
    let (image_area, info_area) = (chunks[0], chunks[1]);

    // Target the raw generated image at the image region's actual pixel size (region cells x the
    // terminal's font-cell size), capped by `graphics::artwork::{MAX_WIDTH,MAX_HEIGHT}` inside
    // `ensure_render` itself. This is also how the image regenerates automatically on resize.
    let font = app.graphics.font_size();
    let px_width = image_area.width as u32 * font.width.max(1) as u32;
    let px_height = image_area.height as u32 * font.height.max(1) as u32;
    app.graphics.ensure_render(px_width, px_height);

    if image_area.width > 0 && image_area.height > 0 {
        frame.render_stateful_widget(
            StatefulImage::default(),
            image_area,
            &mut app.graphics.protocol,
        );
    }
    app.graphics.poll_encoding_result();

    draw_info_panel(frame, info_area, app);
}

fn draw_info_panel(frame: &mut Frame, area: Rect, app: &App) {
    let accent = theme::accent(Tab::Graphics);
    let g = &app.graphics;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::styled("Protocol", theme::header_style(accent)));
    let mut protocol_line = vec![Span::raw(protocol_label(g.protocol_type()))];
    if g.forced_by_key {
        protocol_line.push(Span::styled(" (forced via 'p')", theme::warning_style()));
    } else if g.forced_by_env {
        protocol_line.push(Span::styled(
            " (forced via TERMINFO_FORCE_PROTOCOL)",
            theme::warning_style(),
        ));
    }
    lines.push(Line::from(protocol_line));
    lines.push(Line::from(""));

    lines.push(Line::styled(
        "Detection reasoning",
        theme::header_style(accent),
    ));
    for reason in &g.reasons {
        lines.push(Line::from(format!("- {reason}")));
    }
    lines.push(Line::from(""));

    lines.push(Line::styled("Render", theme::header_style(accent)));
    lines.push(Line::from(format!(
        "Image size: {} x {} px",
        g.image_dims.0, g.image_dims.1
    )));
    lines.push(Line::from(format!("Artwork: {}", g.artwork.name())));
    lines.push(Line::from(format!(
        "Last render: {}",
        g.last_render_ms
            .map(|ms| format!("{ms} ms"))
            .unwrap_or_else(|| "n/a".to_string())
    )));
    match &g.last_error {
        Some(err) => lines.push(Line::styled(
            format!("Encoding error: {err}"),
            theme::warning_style(),
        )),
        None => lines.push(Line::styled("Encoding error: none", theme::muted_style())),
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}
