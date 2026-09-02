// A bounded ring buffer of formatted raw input events, backing the Input tab.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::collections::VecDeque;

use crossterm::event::{Event, KeyEventKind, MouseEventKind};

/// Maximum number of formatted entries retained in the log.
pub const CAPACITY: usize = 200;

/// A bounded ring buffer of formatted raw input events, most recent last.
#[derive(Debug, Default)]
pub struct InputLog {
    entries: VecDeque<String>,
}

impl InputLog {
    /// Creates an empty log.
    pub fn new() -> InputLog {
        InputLog {
            entries: VecDeque::with_capacity(CAPACITY),
        }
    }

    /// Formats and appends `event`, evicting the oldest entry if the log is full.
    pub fn push(&mut self, event: &Event) {
        if self.entries.len() == CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(format_event(event));
    }

    /// Removes all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Iterates the entries oldest-first.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &String> {
        self.entries.iter()
    }

    /// The number of entries currently held.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn format_event(event: &Event) -> String {
    match event {
        Event::Key(key) => {
            let kind = match key.kind {
                KeyEventKind::Press => "Press",
                KeyEventKind::Repeat => "Repeat",
                KeyEventKind::Release => "Release",
            };
            format!("key   {:?} {:?} {}", key.code, key.modifiers, kind)
        }
        Event::Mouse(mouse) => {
            let kind = match mouse.kind {
                MouseEventKind::Down(button) => format!("Down({button:?})"),
                MouseEventKind::Up(button) => format!("Up({button:?})"),
                MouseEventKind::Drag(button) => format!("Drag({button:?})"),
                MouseEventKind::Moved => "Moved".to_string(),
                MouseEventKind::ScrollDown => "ScrollDown".to_string(),
                MouseEventKind::ScrollUp => "ScrollUp".to_string(),
                MouseEventKind::ScrollLeft => "ScrollLeft".to_string(),
                MouseEventKind::ScrollRight => "ScrollRight".to_string(),
            };
            format!(
                "mouse {} at ({}, {}) {:?}",
                kind, mouse.column, mouse.row, mouse.modifiers
            )
        }
        Event::FocusGained => "focus gained".to_string(),
        Event::FocusLost => "focus  lost".to_string(),
        Event::Paste(text) => {
            let preview: String = text.chars().take(40).collect();
            let truncated = text.chars().count() > 40;
            format!(
                "paste {} char(s): \"{}{}\"",
                text.chars().count(),
                preview,
                if truncated { "..." } else { "" }
            )
        }
        Event::Resize(cols, rows) => format!("resize {cols}x{rows}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn push_formats_and_stores() {
        let mut log = InputLog::new();
        log.push(&Event::Key(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
        )));
        assert_eq!(log.len(), 1);
        assert!(log.iter().next().unwrap().contains('q'));
    }

    #[test]
    fn respects_capacity() {
        let mut log = InputLog::new();
        for _ in 0..(CAPACITY + 10) {
            log.push(&Event::Resize(80, 24));
        }
        assert_eq!(log.len(), CAPACITY);
    }

    #[test]
    fn clear_empties_log() {
        let mut log = InputLog::new();
        log.push(&Event::FocusGained);
        log.clear();
        assert!(log.is_empty());
    }
}
