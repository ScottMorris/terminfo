# terminfo

`terminfo` is a small Ratatui TUI that inspects and displays real, live information about the terminal it is currently running in — identity, geometry, colour support, text attribute support, Unicode rendering, live keyboard/mouse input, and the terminal graphics protocol in use, with a procedurally generated colour image rendered through whichever graphics protocol is detected (Kitty, iTerm2, Sixel, or Unicode half-blocks).

## Install / run

```sh
cargo run
```

This needs a real terminal (it will not render meaningfully piped or redirected). Once running, switch tabs with the keyboard or by clicking a tab title in the bottom bar.

## Tabs

1. Overview — terminal identity, geometry, and a capability summary (colour depth, keyboard enhancement, graphics protocol, mouse capture).
2. Colours — ANSI swatches, the 6x6x6 colour cube, the greyscale ramp, a truecolour hue sweep, and a 256-colour banding test.
3. Attributes — one row per text attribute (bold, dim, italic, underline styles, strikethrough, reversed, blink, hidden, and combinations) with a rendered sample.
4. Unicode — box drawing, block elements, braille, powerline glyphs, CJK width, emoji, and combining marks, each with a column ruler and computed vs. measured width.
5. Input — a live, scrolling log of the most recent 200 raw crossterm events (key, mouse, focus, paste, resize).
6. Mouse — live mouse capability, the set of event kinds observed this session, and the most recent event in detail.
7. Graphics — a procedurally generated image rendered through the detected graphics protocol, with detection reasoning and artwork controls.

## Key bindings

Global (handled first, always active regardless of tab):

| Key | Action |
|---|---|
| `q`, `Esc`, `Ctrl+C` | Quit |
| `Tab`, `Right`, `l` | Next tab (wraps) |
| `Shift+Tab` (`BackTab`), `Left`, `h` | Previous tab (wraps) |
| `1`-`7` | Jump to tab by position |
| Mouse click on a tab title in the bottom bar | Switch to that tab (works from any tab, at any time) |
| Mouse scroll over the tab bar | Next tab (scroll down) / previous tab (scroll up) |

Per-tab:

| Tab | Key | Action |
|---|---|---|
| Input | `c` | Clear the input log |
| Graphics | `g` | Cycle artwork |
| Graphics | `p` | Force-cycle protocol |
| Graphics | `r` | Regenerate (bump colour phase) |

## Development

```sh
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

`TERMINFO_FORCE_PROTOCOL=halfblocks|sixel|kitty|iterm2 cargo run` overrides graphics protocol detection for testing (Graphics tab, landing in a later chunk).

See `SPEC.md` for the full behavioural specification and `AGENTS.md`/`CLAUDE.md` for contribution conventions.

## Licence

MIT — see `LICENSE`.
