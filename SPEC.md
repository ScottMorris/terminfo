# SPEC.md

This is the behavioural specification for `terminfo`. Implementation agents build against this document; keep it in sync with shipped behaviour.

## Goals

- Inspect and display real, live information about the terminal `terminfo` is currently running in: identity, geometry, colour support, text attribute support, Unicode rendering, live keyboard/mouse input, and the terminal graphics protocol in use.
- Render a procedurally generated, deterministic, colourful image through whichever graphics protocol is detected (Kitty, iTerm2, Sixel, or Unicode half-blocks), and explain which protocol was chosen and why.
- Present all of this as a tabbed TUI, tabs switchable by keyboard or mouse click.

## Non-goals

- No configuration file. No persisted state between runs.
- No network access.
- No parsing of the terminfo database itself (no `terminfo`/`termini` crate dependency) — terminal facts come from environment variables and live crossterm/protocol queries. The Overview tab may show the *path* of the resolved terminfo entry (probed via the standard terminfo directories) as a convenience, nothing more.
- No external image assets — the graphic is generated in memory at runtime.

## Startup sequence

Order matters because graphics-protocol detection performs live stdio queries that must happen before any other terminal input is read:

1. `enable_raw_mode()`.
2. `execute!(stdout, EnterAlternateScreen)`.
3. `graphics::detect()` — runs `Picker::from_query_stdio()` (or equivalent), reading terminal responses. Must run before any `crossterm::event::read`/`poll` call.
4. Unicode width probe (see the Unicode tab section below) — for each sample string, move the cursor to a scratch line, print it, and query `crossterm::cursor::position()` to get the terminal's own measured width; clear the scratch line afterwards. Must also run before any other terminal event is read, for the same reason as step 3, and `cursor::position()` has a built-in timeout so a non-responding terminal degrades to "unmeasured" rather than hanging.
5. `crossterm::terminal::supports_keyboard_enhancement()`; if `true`, push keyboard enhancement flags (`PushKeyboardEnhancementFlags`).
6. `EnableMouseCapture`, `EnableFocusChange`, `EnableBracketedPaste`.
7. Construct the `Terminal<CrosstermBackend<Stdout>>` and enter the app loop.

Teardown mirrors this in reverse (pop keyboard enhancement flags, disable mouse/focus/paste capture, leave alternate screen, disable raw mode). A panic hook must restore the terminal (raw mode off, leave alternate screen) before propagating the panic, so a crash never leaves the user's shell in a broken state.

## Layout

- Body area on top, a one-line hint bar, then the bottom tab bar.
- Minimum usable size is 40x12 cells; below that, replace the body with a centred "terminal too small" message (tab bar still renders if there is room for it).
- The bottom tab bar is drawn manually as a row of `Span`s (not the built-in `Tabs` widget) so each tab title's exact on-screen `Rect` is known and recorded every frame; mouse clicks are hit-tested (`col`/`row` against those exact rects), not inferred from a fixed layout assumption. Scrolling the mouse wheel while the pointer is over the tab bar also cycles to the next/previous tab (scroll-down = next, scroll-up = previous), in addition to a left-click jumping straight to the clicked tab.

## Tabs

Order below is both the visual order and the number-key order (`1`-`6`).

### 1. Overview

Displays:
- Terminal identity: a best-guess **detected terminal name** derived from a small heuristic table over `$TERM_PROGRAM`, `$KITTY_WINDOW_ID`, `$WEZTERM_EXECUTABLE`, `$ALACRITTY_SOCKET`, `$VTE_VERSION`, `$KONSOLE_VERSION`, `$WT_SESSION`, etc. (falls back to `$TERM` when nothing more specific matches), plus the raw `$TERM`, `$COLORTERM`, `$TERM_PROGRAM` (+ version), multiplexer detection (`$TMUX`, `$STY`), SSH detection (`$SSH_TTY`/`$SSH_CONNECTION`), `$LANG`/`$LC_ALL`, `$SHELL`, resolved terminfo entry path if found under the standard terminfo directories.
- Geometry: cell size (`cols x rows`), pixel size if reported (`terminal::window_size()`), derived per-cell pixel size when both are available.
- Capability summary: inferred colour depth with a one-line reason, keyboard enhancement (Kitty keyboard protocol) yes/no, graphics protocol detected (summary line, "see Graphics tab" for detail), mouse capture enabled yes/no.

Geometry re-queries and updates live on `Event::Resize`.

### 2. Colours

Displays, top to bottom:
- The 16 ANSI swatches (8 normal + 8 bright), each labelled with index and name.
- The 6x6x6 colour cube (indices 16-231) as six blocks of 36 cells.
- The 24-step greyscale ramp (indices 232-255).
- A full-width smooth truecolour hue sweep and a red-to-blue gradient.
- A banding test: the same gradient rendered once in truecolour and once quantised to the nearest 256-colour index, side by side, so visible stepping on the 256-colour side (and smoothness on the truecolour side) is the diagnostic.

A header line states the inferred colour depth and warns if truecolour is being drawn on a terminal that did not advertise support for it.

### 3. Attributes

One row per text attribute, each with a label, a sample string rendered with that attribute, and a short expected-appearance description. Attributes: bold, dim, italic, underline, double underline, curly underline, dotted underline, dashed underline, coloured underline, strikethrough, reversed, slow blink, rapid blink, hidden, plus a few combinations (bold+italic, bold+underline, dim+italic). Attributes the current terminal is known/observed to ignore are still listed — that is expected data, not a bug.

### 4. Unicode

Sections, each with a column ruler and a right-hand vertical alignment marker (`│`) so misalignment is visible:
- Box drawing: light, heavy, double, rounded, dashed.
- Block elements and shade characters.
- Braille pattern strip.
- Powerline/private-use glyphs.
- CJK wide characters.
- Emoji: single codepoint, a ZWJ sequence (e.g. family), a flag, a skin-tone modifier, and a variation-selector-16 text/emoji presentation pair.
- Combining marks, zero-width joiner, zero-width space, tab.

Each line prints two width figures next to the glyph: the width computed via `unicode-width`, and the width the running terminal actually measured for that exact string during the startup probe (step 4 of the startup sequence) — a mismatch between the two is the diagnostic, and is real, terminal-specific data rather than a static reference value. A row is marked "unmeasured" if the startup probe's `cursor::position()` query timed out.

### 5. Input

A live, scrolling log (most recent 200 entries) of raw crossterm events as they arrive: key code + modifiers + `KeyEventKind` (press/repeat/release — release only visible when keyboard enhancement is active), mouse events (kind, button, column/row), focus gained/lost, paste, resize. A header line states whether keyboard enhancement flags were successfully pushed. `c` clears the log. Global tab-switch keys are both logged here and acted on (they are not swallowed by this tab).

### 6. Mouse

Displays live mouse capability and state:
- Whether mouse capture is currently enabled (it is enabled for the whole app lifetime, per the startup sequence).
- What mouse event kinds have actually been observed so far this session (press/release/drag/scroll/move), since not all terminals report all kinds.
- The most recent mouse event in detail: kind, button, column/row, modifiers — updating live as the user moves/clicks/scrolls within the terminal.
- A short explanation that clicking a tab title in the bottom bar (visible from any tab) is itself a mouse event exercising this same capability.

### 7. Graphics

Displays:
- The procedurally generated image, filling the tab's image region.
- An info panel: protocol in use, bulleted detection reasoning (env vars observed, whether the stdio query succeeded, reported capabilities, font-cell size and its source, a tmux-passthrough caveat when running inside tmux), image pixel dimensions, current artwork name, last render time in milliseconds, and the last encoding error if one occurred.

Keys (active on this tab): `g` cycles the artwork (Julia -> Plasma -> ColourWheel -> Julia), `p` forces the next protocol in rotation (rebuilds the image, panel marks it "forced"), `r` bumps the colour phase and regenerates. The image regenerates automatically on resize.

Numbering note: this document lists Mouse as tab 6 and Graphics as tab 7 for readability; the shipped `Tab` enum order (and therefore the `1`-`7` key bindings) must match whatever order `src/tabs.rs` defines — keep this section's order and the enum in lockstep.

## Colour depth inference (`terminfo::infer_colour_depth`)

Evaluated in this order, first match wins:
1. `$COLORTERM` is `truecolor` or `24bit` -> `TrueColour`.
2. `$TERM` ends in `-direct` (e.g. `xterm-direct`) -> `TrueColour`.
3. `$TERM_PROGRAM` is a program known to always support truecolour (e.g. `iTerm.app`, `WezTerm`, `vscode`) -> `TrueColour`.
4. `$KITTY_WINDOW_ID` is set -> `TrueColour`.
5. `$TERM` contains `256color` -> `Xterm256`.
6. `$TERM` is `dumb` -> `Monochrome`.
7. `$TERM` is set to anything else recognised as colour-capable (e.g. `xterm`, `screen`, `linux`) -> `Ansi16`.
8. Otherwise -> `Ansi16` with a reason noting the fallback default.

Each branch records a human-readable reason string alongside the resulting `ColourDepth`, surfaced on the Overview and Colours tabs.

## Graphics protocol detection and selection

- Primary detection is `ratatui-image`'s `Picker::from_query_stdio()`, which combines environment-variable checks with live terminal queries (DA1, Kitty graphics query, font-size query) to pick the best supported protocol, falling back to Unicode half-blocks when nothing else is detected or the terminal is not a tty.
- `TERMINFO_FORCE_PROTOCOL` (values: `halfblocks`, `sixel`, `kitty`, `iterm2`) overrides detection for testing; when set, the detection panel notes the override explicitly rather than presenting it as organic detection.
- The `p` key on the Graphics tab cycles through protocols at runtime for comparison; the terminal is cleared before the next draw when switching to avoid artefacts.
- If no protocol can be detected and no override is set, fall back to `Picker::halfblocks()`, which always works.

## Procedural artwork

All artworks are pure functions of `(width: u32, height: u32, phase: f32) -> image::RgbImage` — deterministic, no RNG, cheap to recompute on resize, and cached by `(width, height, artwork, phase-bucket)` so redraws without a resize or `r`/`g` press are free.

- **Julia** (default): `c = -0.8 + 0.156i`, viewport `re in [-1.6, 1.6]` with `im` scaled to match the image aspect ratio, `MAX_ITER = 96`, bailout `|z|^2 > 16`. Escaped pixels use the smooth iteration count `mu = n + 1 - log2(ln|z|)` mapped through a cyclic cosine palette (`col(t) = a + b * cos(2*pi*(c*t + d))`, `a = b = (0.5, 0.5, 0.5)`, `c = (1.0, 1.0, 1.0)`, `d = (0.00, 0.33, 0.67)`) at `t = mu / MAX_ITER + phase`. Interior (non-escaping) pixels use a dark navy-to-violet gradient keyed on final `|z|^2` so the interior is not flat black.
- **Plasma**: sum of four sine terms over pixel coordinates, mapped through the same cosine palette.
- **ColourWheel**: polar HSV wheel (hue = angle, saturation = radius, value = 1) with a checkerboard border, useful for judging a protocol's colour accuracy.
- The rendered image is capped at 1024x768 regardless of the requested tab area.
- The `r` key increments `phase` by `0.05` (wrapping at 1.0) to demonstrate live regeneration; `g` cycles the artwork.

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

## Testing requirements

Unit tests (run via `cargo test`):
- `tabs.rs`: navigation wraps correctly in both directions; `from_digit` maps every valid digit and rejects invalid ones; tab count matches the key-binding table above.
- `terminfo.rs`: `infer_colour_depth` is table-driven over representative env maps covering every branch in the inference order above (`COLORTERM=truecolor`, `TERM=xterm-256color`, `TERM=linux`, `TERM=dumb`, `TERM_PROGRAM=iTerm.app`, `KITTY_WINDOW_ID` set, `TERM=xterm-direct`).
- `ui/colours.rs`: `xterm256_to_rgb` spot-checked at indices 16, 21, 196, 231, 232, 255; the truecolour-to-256 quantiser used by the banding test is checked against known nearest-index mappings.
- `ui/unicode.rs`: width-table test comparing `unicode-width` output against the expected width for a sample of the listed glyphs.
- `graphics/artwork.rs`: output image dimensions match the request; two renders with identical inputs are byte-identical (determinism); a 64x64 Julia render contains at least 64 distinct colours (diversity); all three artworks handle a 1x1 request and a zero-size guard without panicking.

Manual verification (run in at least two terminals — the developer's daily driver and a plain `TERM=xterm-256color` terminal; after the Graphics tab exists, also run once with `TERMINFO_FORCE_PROTOCOL=halfblocks`):
1. Overview shows correct `$TERM`/program and geometry; resizing the window updates the numbers live.
2. All tab-switch bindings work (`Tab`, `Shift+Tab`, `Left`, `Right`, `h`, `l`, `1`-`7`), wrap around at both ends, and the highlighted tab title updates; clicking a tab title with the mouse switches to it.
3. Colours tab: swatches, cube, greyscale ramp, and hue sweep all render; the banding test shows visible steps only on the 256-colour side.
4. Attributes tab: each row visibly differs where the terminal supports it; attributes the terminal ignores are noted, not treated as failures.
5. Unicode tab: the right-hand alignment column lines up for box-drawing/block sections; note where emoji/CJK width pushes it out of alignment (expected, informative).
6. Input tab: key presses log with modifiers; release events appear when keyboard enhancement is active; mouse movement/clicks and focus in/out are logged.
7. Mouse tab: capability line reflects mouse capture is enabled; moving/clicking/scrolling updates the "most recent event" panel live; the set of observed event kinds grows as different interactions occur.
8. Graphics tab: an image appears; the reported protocol matches expectation for that terminal; the reason list is coherent; resizing regenerates the image without artefacts; `g` cycles all three artworks; `p` cycles protocols (half-blocks must always work); `r` visibly shifts the palette.
9. `q`, `Esc`, and `Ctrl+C` each quit cleanly and restore the shell (raw mode off, cursor visible, alternate screen exited, no leftover escape sequences).

"Done" for any change means: `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` all exit `0` with no warnings, the manual checklist above passes in at least two terminals, every `.rs` file under `src/` carries the licence header, and `README.md`/`SPEC.md` match shipped behaviour.

## Known risks / gotchas for implementers

- `Picker::from_query_stdio()` reads from stdin as part of detection — it must run after entering the alternate screen but strictly before any other terminal event is read, or detection breaks and stray bytes can leak into the Input tab's log.
- Kitty graphics inside tmux requires `allow-passthrough on` in the user's tmux config; Sixel over SSH depends on the client terminal. The Graphics tab's reasoning panel should say so rather than fail silently.
- Switching `ProtocolType` live (the `p` key) can leave visual artefacts from the previous protocol; clear the terminal before the next draw when this happens.
- A debug-build Julia render at the 1024x768 cap with 96 iterations can approach 100ms; keep the resolution cap and the render cache keyed by `(width, height, artwork, phase)`. Set `[profile.dev] opt-level = 1` in `Cargo.toml` (cheap to add from chunk 1 onward, independent of when the artwork code itself lands) so debug builds stay responsive once the fractal renderer is in place.
