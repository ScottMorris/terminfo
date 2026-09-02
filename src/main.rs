// Terminal setup/teardown (the startup sequence from SPEC.md) and the application entry point.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use std::io::{self, Write};

use anyhow::Result;
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

mod app;
mod graphics;
mod input_log;
mod mouse_state;
mod tabs;
mod terminfo;
mod ui;
mod widths;

fn main() -> Result<()> {
    install_panic_hook();

    let mut stdout = io::stdout();

    // 1. Raw mode.
    enable_raw_mode()?;
    // 2. Alternate screen.
    execute!(stdout, EnterAlternateScreen)?;

    // 3. Graphics-protocol detection — after alt-screen entry, strictly before any other
    // terminal event is read. Runs `Picker::from_query_stdio()`, which performs live stdio
    // queries (DA1, Kitty graphics query, font-size query); reading any other event first would
    // corrupt that exchange.
    let graphics_detection = graphics::detect();

    // 4. Unicode width probe — after alt-screen entry, before keyboard-enhancement and mouse-
    // capture setup, before any other event is read. It moves the cursor to a scratch line,
    // prints each Unicode sample, queries `crossterm::cursor::position()` to get the terminal's
    // own measured width, then clears the scratch line. `cursor::position()` has a built-in
    // timeout, so a non-responding terminal degrades to "unmeasured" rather than hanging.
    let width_probe = widths::measure(&mut stdout);

    // 5. Keyboard enhancement (Kitty keyboard protocol), if supported.
    let keyboard_enhancement = crossterm::terminal::supports_keyboard_enhancement()?;
    if keyboard_enhancement {
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
    }

    // 6. Mouse, focus-change, and bracketed-paste capture, for the whole app lifetime.
    execute!(
        stdout,
        EnableMouseCapture,
        EnableFocusChange,
        EnableBracketedPaste
    )?;

    // 7. Construct the terminal and enter the app loop.
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new(Some(keyboard_enhancement), width_probe, graphics_detection);
    let run_result = app::run(&mut terminal, &mut app);

    let teardown_result = teardown(terminal.backend_mut(), keyboard_enhancement);

    run_result?;
    teardown_result?;
    Ok(())
}

/// Mirrors the startup sequence in reverse: pop keyboard enhancement flags, disable mouse/focus/
/// paste capture, leave the alternate screen, disable raw mode.
fn teardown<W: Write>(writer: &mut W, keyboard_enhancement: bool) -> Result<()> {
    if keyboard_enhancement {
        execute!(writer, PopKeyboardEnhancementFlags)?;
    }
    execute!(
        writer,
        DisableMouseCapture,
        DisableFocusChange,
        DisableBracketedPaste,
        LeaveAlternateScreen
    )?;
    disable_raw_mode()?;
    Ok(())
}

/// Installs a panic hook that restores the terminal (raw mode off, alternate screen left)
/// before the default panic behaviour runs, so a crash never leaves the shell in a broken state.
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_hook(info);
    }));
}
