// Top-level draw dispatch: lays out body / hint line / tab bar and routes to the active tab.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::tabs::Tab;

mod attributes;
mod colours;
mod graphics;
mod input;
mod mouse;
mod overview;
mod tabs_bar;
mod unicode;

/// Minimum usable terminal size, per SPEC.md's Layout section.
const MIN_WIDTH: u16 = 40;
const MIN_HEIGHT: u16 = 12;

/// Draws one frame: body on top, a one-line hint bar, then the bottom tab bar.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        draw_too_small(frame, area, app);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);
    let (body_area, hint_area, tab_bar_area) = (chunks[0], chunks[1], chunks[2]);

    draw_body(frame, body_area, app);
    draw_hint(frame, hint_area, app);

    let regions = tabs_bar::draw(frame, tab_bar_area, app);
    app.set_tab_bar_regions(regions);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &mut App) {
    match app.tab {
        Tab::Overview => overview::draw(frame, area, app),
        Tab::Colours => colours::draw(frame, area, app),
        Tab::Attributes => attributes::draw(frame, area, app),
        Tab::Unicode => unicode::draw(frame, area, app),
        Tab::Input => input::draw(frame, area, app),
        Tab::Mouse => mouse::draw(frame, area, app),
        Tab::Graphics => graphics::draw(frame, area, app),
    }
}

fn draw_hint(frame: &mut Frame, area: Rect, app: &App) {
    let mut hint =
        String::from("q/Esc/Ctrl+C Quit  Tab/Shift+Tab/h/l/1-7 Switch tabs  click/scroll tab bar");
    match app.tab {
        Tab::Input => hint.push_str("  c Clear log"),
        Tab::Graphics => hint.push_str("  g Artwork  p Protocol  r Regenerate"),
        _ => {}
    }
    let paragraph = Paragraph::new(Line::from(Span::styled(
        hint,
        Style::default().add_modifier(Modifier::DIM),
    )));
    frame.render_widget(paragraph, area);
}

/// Computes a `width`x`height` rect centred within `area`, clamped to fit. Used by the tabs
/// that are still placeholders in this chunk to centre their "coming soon" message.
pub(crate) fn centred_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn draw_too_small(frame: &mut Frame, area: Rect, app: &mut App) {
    // Leave room for the tab bar if there is any height left at all.
    if area.height > 1 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);
        let message = Paragraph::new(Line::from(Span::styled(
            "terminal too small",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(message, chunks[0]);
        let regions = tabs_bar::draw(frame, chunks[1], app);
        app.set_tab_bar_regions(regions);
    } else if area.height == 1 && area.width > 0 {
        let regions = tabs_bar::draw(frame, area, app);
        app.set_tab_bar_regions(regions);
    }
}
