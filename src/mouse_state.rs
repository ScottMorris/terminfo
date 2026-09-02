// Tracks mouse capability state and observed events for the Mouse tab.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::collections::HashSet;

use crossterm::event::{MouseEvent, MouseEventKind};

/// The distinct kinds of mouse event a terminal can report, collapsed away from button/scroll
/// specifics so the Mouse tab can show which broad categories have been observed this session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseKindObserved {
    Press,
    Release,
    Drag,
    Scroll,
    Move,
}

impl MouseKindObserved {
    /// A short label for display.
    pub fn label(self) -> &'static str {
        match self {
            MouseKindObserved::Press => "Press",
            MouseKindObserved::Release => "Release",
            MouseKindObserved::Drag => "Drag",
            MouseKindObserved::Scroll => "Scroll",
            MouseKindObserved::Move => "Move",
        }
    }

    fn from_event_kind(kind: MouseEventKind) -> MouseKindObserved {
        match kind {
            MouseEventKind::Down(_) => MouseKindObserved::Press,
            MouseEventKind::Up(_) => MouseKindObserved::Release,
            MouseEventKind::Drag(_) => MouseKindObserved::Drag,
            MouseEventKind::ScrollDown
            | MouseEventKind::ScrollUp
            | MouseEventKind::ScrollLeft
            | MouseEventKind::ScrollRight => MouseKindObserved::Scroll,
            MouseEventKind::Moved => MouseKindObserved::Move,
        }
    }
}

/// Live mouse capability and observation state, rendered by the Mouse tab.
#[derive(Debug, Clone)]
pub struct MouseState {
    /// Whether mouse capture is enabled for the app's lifetime.
    pub capture_enabled: bool,
    /// The set of mouse event kinds observed so far this session.
    pub observed_kinds: HashSet<MouseKindObserved>,
    /// The most recently observed mouse event, if any.
    pub last_event: Option<MouseEvent>,
}

impl MouseState {
    /// Creates a new `MouseState`; `capture_enabled` reflects whether mouse capture was
    /// successfully enabled at startup.
    pub fn new(capture_enabled: bool) -> MouseState {
        MouseState {
            capture_enabled,
            observed_kinds: HashSet::new(),
            last_event: None,
        }
    }

    /// Records a mouse event: updates the observed-kind set and the most-recent-event detail.
    pub fn update(&mut self, event: &MouseEvent) {
        self.observed_kinds
            .insert(MouseKindObserved::from_event_kind(event.kind));
        self.last_event = Some(*event);
    }
}
