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
pub(crate) mod theme;
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
    let accent = theme::accent(app.tab);
    let block = theme::frame_block(app.tab.title(), accent);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match app.tab {
        Tab::Overview => overview::draw(frame, inner, app),
        Tab::Colours => colours::draw(frame, inner, app),
        Tab::Attributes => attributes::draw(frame, inner, app),
        Tab::Unicode => unicode::draw(frame, inner, app),
        Tab::Input => input::draw(frame, inner, app),
        Tab::Mouse => mouse::draw(frame, inner, app),
        Tab::Graphics => graphics::draw(frame, inner, app),
    }
}

/// A key-hint span, styled to stand out, followed by a muted description span.
fn hint_pair(key: &'static str, desc: &'static str) -> [Span<'static>; 2] {
    [
        Span::styled(key, theme::key_style()),
        Span::styled(desc, theme::muted_style()),
    ]
}

fn draw_hint(frame: &mut Frame, area: Rect, app: &App) {
    let mut spans = Vec::new();
    spans.extend(hint_pair("q/Esc/Ctrl+C", " Quit   "));
    spans.extend(hint_pair("Tab/Shift+Tab/h/l/1-7", " Switch tabs   "));
    spans.push(Span::styled("click/scroll tab bar", theme::muted_style()));
    match app.tab {
        Tab::Input => {
            spans.push(Span::styled("   ", theme::muted_style()));
            spans.extend(hint_pair("c", " Clear log"));
        }
        Tab::Graphics => {
            spans.push(Span::styled("   ", theme::muted_style()));
            spans.extend(hint_pair("g", " Artwork "));
            spans.extend(hint_pair("p", " Protocol "));
            spans.extend(hint_pair("r", " Regenerate"));
        }
        _ => {}
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
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
