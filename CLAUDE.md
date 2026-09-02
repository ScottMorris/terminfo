# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Purpose

`terminfo` is a small Ratatui TUI that inspects and displays information about the terminal it is currently running in — colour depth, text attribute support, Unicode rendering, live keyboard/mouse input, and the terminal graphics protocol (Kitty/iTerm2/Sixel/half-blocks), with a procedurally generated colour image rendered through whichever protocol is detected. See `AGENTS.md` for the full set of contribution conventions and `SPEC.md` for the behavioural spec each tab implements against.

## Commands

- `cargo build` — compile
- `cargo run` — run the TUI (needs a real terminal; won't render meaningfully piped/redirected)
- `cargo test` — run unit tests (colour-depth inference, tab navigation, artwork determinism, etc.)
- `cargo clippy -- -D warnings` — lint, warnings are errors
- `cargo fmt --check` — formatting check
- `TERMINFO_FORCE_PROTOCOL=halfblocks|sixel|kitty|iterm2 cargo run` — override graphics protocol detection for testing

## Architecture

- `src/main.rs` — terminal setup/teardown (raw mode, alternate screen, graphics-protocol detection, Unicode width probe, keyboard enhancement, mouse/focus/paste capture, panic hook), calls into `app::run`.
- `src/app.rs` — `App` state and the main event loop; dispatches key/mouse/resize events, including tab-bar click/scroll hit-testing.
- `src/tabs.rs` — the `Tab` enum (Overview, Colours, Attributes, Unicode, Input, Mouse, Graphics), navigation helpers.
- `src/terminfo.rs` — gathers terminal identity/geometry/capability facts from env vars and crossterm queries; infers colour depth and a detected terminal name.
- `src/input_log.rs` — ring buffer of recent raw input events for the Input tab.
- `src/mouse_state.rs` — live mouse capability/observed-event-kind tracking for the Mouse tab.
- `src/widths.rs` — the startup probe that measures each Unicode sample's real on-screen width via a scratch-line cursor-position query, for the Unicode tab.
- `src/graphics/` — `mod.rs` (protocol detection + picker state), `detect.rs` (human-readable detection reasoning), `artwork.rs` (procedural image generators: Julia set, plasma, colour wheel).
- `src/ui/` — one rendering module per tab, plus `tabs_bar.rs` for the clickable bottom tab bar and `theme.rs` for the shared per-tab accent palette and styling helpers (bordered/titled frame, key-hint style, muted style, yes/no colouring). Every tab renders inside the accent-bordered frame `ui::mod::draw_body` sets up — use `crate::ui::theme` rather than inventing ad hoc styles. All widgets are pure functions of `&App`.

## Conventions (see `AGENTS.md` for full detail)

- **Spelling:** Canadian English in comments, docs, commits, and UI strings.
- **Commit messages:** Conventional Commits. Write multi-line/markdown-heavy bodies to a file and commit with `git commit -F <file>` rather than piping backticks/`$()` through `-m`; verify with `git log -1 --pretty=fuller` afterward.
- **Licence headers:** every `.rs` file in `src/` starts with a one-line summary, a bare `//`, then `// (c) Copyright 2026 Liminal HQ, Scott Morris` / `// SPDX-License-Identifier: MIT`, a blank line, then `use` statements.
- **Markdown formatting:** do not manually hard-wrap prose — write each paragraph as one line and let the renderer/editor soft-wrap.
- **Git workflow:** do not push unless explicitly asked; this repo currently has no remote.
