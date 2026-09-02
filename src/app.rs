// `App` state and the main event loop: dispatches key, mouse, resize, focus, and paste events.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::Terminal;

use crate::input_log::InputLog;
use crate::mouse_state::MouseState;
use crate::tabs::Tab;
use crate::terminfo::TermInfo;
use crate::ui;
use crate::widths::WidthProbe;

/// How long to block waiting for the next terminal event before redrawing anyway.
const POLL_TICK: Duration = Duration::from_millis(250);

/// Top-level application state.
pub struct App {
    /// The currently active tab.
    pub tab: Tab,
    /// Set to request the event loop exit.
    pub should_quit: bool,
    /// Gathered terminal identity, geometry, and capability facts.
    pub term_info: TermInfo,
    /// Ring buffer of formatted raw input events, for the Input tab.
    pub input_log: InputLog,
    /// Live mouse capability and observation state, for the Mouse tab.
    pub mouse_state: MouseState,
    /// The on-screen column span of each tab title, as last rendered by the tab bar. Populated
    /// by the UI draw call each frame and read by the mouse-click handler to map clicks to tabs.
    pub tab_bar_regions: Vec<(Tab, Rect)>,
    /// Real terminal-measured widths for the Unicode tab's sample strings, gathered by the
    /// startup width probe (see `crate::widths`).
    pub width_probe: WidthProbe,
}

impl App {
    /// Constructs a fresh `App`, gathering terminal facts. `keyboard_enhancement` reflects
    /// whether the startup sequence successfully pushed keyboard enhancement flags. `width_probe`
    /// is the result of the startup Unicode width probe (see `crate::widths::measure`).
    pub fn new(keyboard_enhancement: Option<bool>, width_probe: WidthProbe) -> App {
        App {
            tab: Tab::Overview,
            should_quit: false,
            term_info: TermInfo::gather(keyboard_enhancement),
            input_log: InputLog::new(),
            mouse_state: MouseState::new(true),
            tab_bar_regions: Vec::new(),
            width_probe,
        }
    }

    /// Records the tab bar's last-rendered hit regions, for mapping subsequent mouse clicks.
    pub fn set_tab_bar_regions(&mut self, regions: Vec<(Tab, Rect)>) {
        self.tab_bar_regions = regions;
    }

    /// Dispatches one raw terminal event: logs it, then handles it (global bindings first, then
    /// per-tab bindings). Global tab-switch keys are logged and acted on, never swallowed.
    fn handle_event(&mut self, event: Event) {
        self.input_log.push(&event);
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::Resize(_, _) => self.term_info.refresh_geometry(),
            Event::FocusGained | Event::FocusLost | Event::Paste(_) => {}
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Release events (only visible with keyboard enhancement) are logged above but do not
        // trigger actions, to avoid double-handling press-then-release.
        if key.kind == KeyEventKind::Release {
            return;
        }

        if self.handle_global_key(key) {
            return;
        }

        self.handle_tab_key(key);
    }

    /// Handles the always-active global key bindings. Returns `true` if the key was consumed.
    fn handle_global_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                true
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                true
            }
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                self.tab = self.tab.next();
                true
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                self.tab = self.tab.prev();
                true
            }
            KeyCode::Char(c) if Tab::from_digit(c).is_some() => {
                self.tab = Tab::from_digit(c).expect("checked above");
                true
            }
            _ => false,
        }
    }

    /// Handles keys specific to the currently active tab.
    fn handle_tab_key(&mut self, key: KeyEvent) {
        // Graphics tab keys (`g`, `p`, `r`) arrive in chunk 3, once the graphics module exists
        // to act on them.
        if self.tab == Tab::Input && key.code == KeyCode::Char('c') {
            self.input_log.clear();
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        self.mouse_state.update(&mouse);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some((tab, _)) = self
                    .tab_bar_regions
                    .iter()
                    .find(|(_, rect)| rect_contains(*rect, mouse.column, mouse.row))
                {
                    self.tab = *tab;
                }
            }
            MouseEventKind::ScrollDown if self.row_is_in_tab_bar(mouse.row) => {
                self.tab = self.tab.next();
            }
            MouseEventKind::ScrollUp if self.row_is_in_tab_bar(mouse.row) => {
                self.tab = self.tab.prev();
            }
            _ => {}
        }
    }

    /// Whether `row` falls within the last-rendered tab bar's row band, i.e. the mouse pointer
    /// is hovering the tab bar (used for scroll-to-cycle, independent of horizontal position).
    fn row_is_in_tab_bar(&self, row: u16) -> bool {
        self.tab_bar_regions
            .iter()
            .any(|(_, rect)| row >= rect.y && row < rect.y + rect.height)
    }
}

fn rect_contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

/// Runs the main event loop: draw, poll for an event (250ms tick), dispatch, repeat until quit.
pub fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    B::Error: std::error::Error + Send + Sync + 'static,
{
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(POLL_TICK)? {
            let event = event::read()?;
            app.handle_event(event);
        }
    }
    Ok(())
}
