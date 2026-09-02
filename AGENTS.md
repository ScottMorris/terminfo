# AGENTS.md

## Coding Standards

- **Spelling:** Must use Canadian Spelling for things that don't require American spelling (e.g., UI strings, variables, comments). Examples: "colour", "center" -> "centre", "behavior" -> "behaviour".
- **Commit Messages:** Use Conventional Commits (e.g., `feat: add scanner`, `fix: typo in header`).

## Commit Messages

**Format:** Use Conventional Commits format (e.g., `feat: ...`, `fix: ...`, `docs: ...`, `test: ...`).

- Use `test:` for test-related changes, including fixes to tests themselves (do not use `fix:` unless it fixes application code).

**Body Requirements:**

- Explain what and why (not how)
- Use markdown: **bold**, _italics_, `code`, bullet lists
- **NO markdown headings** - use **bold labels** for sections (not always required)
- When a commit body includes backticked code in shell commands, avoid command substitution by using single-quoted `-m` strings (preferred) or escaping backticks.
  - Example (preferred): `git commit -m 'fix: ...' -m 'Use `scanStatus` in footer'`
  - Example (escape): `git commit -m "Use \`scanStatus\` in footer"`

**Specific Updates**: Each commit message should reflect the specific changes made in that commit. Do not just recap the entire project history or scope. Focus on the now.

**Shell Interpolation Safety:**

- Do not pass markdown-heavy commit bodies directly via `git commit -m "..."` when they include backticks, `$()`, or shell-sensitive characters.
- Prefer writing the message to a file with a single-quoted heredoc and commit with `git commit -F <file>` to prevent shell expansion.
- If using `-m`, escape shell-sensitive characters explicitly before running the command.
- After committing, verify the stored message with `git log -1 --pretty=fuller` and amend immediately if interpolation altered content.

## Pull Request Titles

**Requirement:** PR titles must be human-readable summaries of the PR change.

- Start with a capital letter, write in the imperative mood, and keep to roughly one 70-character line.
- Do not use Conventional Commit prefixes in PR titles (for example, no `feat:`, `fix:`, `chore:`).
- Describe the outcome or behaviour change, not internal process language.
- Ignore internal planning document notes in PR titles and descriptions unless they directly map to repository changes.
- Keep title style consistent across every open PR in the same stack.
- Do not rename merged PRs unless explicitly requested.

## Pull Request Content

**Requirement:** PR titles and descriptions must not mention internal workflow artefacts.

- Do not mention deferred-review documents, internal queue labels, or internal-only planning notes in outward PR content.
- Use user-facing, outcome-focused language in PR titles and descriptions.
- Only include internal process details in PR content when explicitly requested by the user.

**PR Description Format:**

- Prefer a compact markdown structure with `## Summary` and `## Test plan`.
- Under `## Summary`, use `###` sub-sections when they help group the change cleanly.
- Under each summary section, use flat bullets with bold lead-ins for scanability.
- Keep the summary focused on outcomes and behaviour changes, not commit history or implementation chronology.
- Under `## Test plan`, use checklist bullets (`- [x]` / `- [ ]`) and include the concrete commands, validations, or remaining gaps.
- If something could not be verified, state that plainly at the end of `## Test plan` or immediately below it.

## Pull Request Labels

**Requirement:** Every PR must include labels that describe the change and map to release-note categories.

- Add at least one primary category label to every PR: `enhancement`, `bug`, `documentation`, `testing`, `ci`, `build`, or `chore`.
- Add product and subsystem scope labels where helpful (for example, `tui`, `core`, `graphics`).
- Prefer the broader Liminal HQ label style over Conventional Commit terms for PR labelling.
- Use `skip-changelog` only when a change should be excluded from generated release notes.

## Git Workflow

**Requirement:** Do not push changes (especially force pushes) to the repository unless explicitly requested by the user.

- **Fix branch naming:** When creating a branch for a fix, use `fix/issue-<number>-<short-description>`.

## Testing

- **Mandatory Testing:** Make sure the unit tests are run after changes to the code.
- **Verification:** Always verify code changes by running relevant tests.
- **Build Check:** Run `cargo build` and `cargo clippy -- -D warnings` to surface any errors.
- **Format Check:** Run `cargo fmt --check` to ensure consistent formatting.
- **Test Command:** `cargo test` — run all tests.

## Documentation

- **Updates:** When user-facing behaviour, CLI options, or features change, update `README.md` and `SPEC.md`.
- **No hard wrapping:** Never hard-wrap paragraph prose in markdown files — write each paragraph or list item as a single line and let viewers soft-wrap. Deliberate short standalone lines, one-liners, and separate bullets are intentional structure — never collapse them.

## Project Structure

- This is a Rust project (2021 edition), a single binary crate: `terminfo`.
- TUI uses ratatui + crossterm.
- App loop and state in `src/app.rs`; tab enum in `src/tabs.rs`; terminal introspection in `src/terminfo.rs`; input event log in `src/input_log.rs`; graphics detection and procedural artwork in `src/graphics/`; all rendering in `src/ui/`, one file per tab.

## Licence and Copyright

- **Requirement:** New source files (and substantially rewritten source files) should include a short header as the first content in the file.
- **Applies to:** `.rs` source files in `src/` directories.
- **Do not add headers to:** generated files, lockfiles, config files (`.json`, `.yml`, `.toml`), markdown docs, or man pages.

Preferred header format for Rust:

```rust
// Brief one-line summary of what this file does
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT
```

- Keep the summary to one concise sentence.
- Place the header before `use` statements.
- Leave one blank line between the header and the first code line.
- Preserve existing valid licence headers when already present.
