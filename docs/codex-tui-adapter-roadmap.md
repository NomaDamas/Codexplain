# Codex TUI Adapter Roadmap

## Goal

Build a reversible Codexplain adapter for Codex interactive TUI assistant-message
highlighting without making hidden global changes.

## Current boundary

Codexplain can fully control completed stdout paths such as `codex exec`,
`codexplain shape`, and `codexplain post-response`. Interactive Codex TUI is
different: it is a fullscreen Ratatui application that repaints frames from
internal state, so an external stdout filter cannot reliably identify and
recolor only assistant-message spans.

## Adapter options

1. Project-local shim

   Use `.codexplain/bin/codex` and `source .codexplain/activate` so only the
   current project routes Codex calls through Codexplain.

2. PTY proxy

   Run Codex inside a pseudo-terminal and inspect terminal frames. This can
   add best-effort color to completed text, but it is fragile because fullscreen
   TUIs use cursor movement, alternate screen buffers, and repaint operations.

3. Patched or forked Codex TUI

   Add a style adapter at the assistant-message renderer boundary. This is the
   reliable path for semantic multi-color assistant text because the renderer
   still has message structure before converting it into terminal cells.

## Tracking split

Issue #4 is the tracking/specification issue. It records the upstream/fork hook
contract and validation checklist without cloning or patching upstream Codex.

Issue #3 is the project-local adapter issue. It owns reversible shim behavior,
patched-binary detection, and on/off routing in this repository.

Future upstream or fork implementation should use
[`upstream-codex-tui-style-hook.md`](upstream-codex-tui-style-hook.md) as the
contract. The implementation can rename proposed Rust types, but it must keep
the binding invariants.

## Patched-binary detection

Codexplain treats interactive TUI assistant color as available only when
project-local routing can find an executable patched Codex binary in one of
these locations:

- `.codexplain/patched-codex/bin/codex`
- `.codexplain/state/codex-upstream/codex-rs/target/release/codex`
- `.codexplain/state/codex-upstream/codex-rs/target/debug/codex`

`tui-adapter build` should persist the final executable into
`.codexplain/patched-codex/bin/codex`. The multi-GB Cargo `target/` cache is a
temporary build cache and can be removed after the binary is copied.

If no patched binary exists, `codexplain tui-color status` must report that
ordinary `exec`/`review` shaping can still work but interactive assistant-message
recoloring requires the hook/fork path.

## Reversibility contract

- `codexplain on --local` writes only project-managed files and blocks.
- `codexplain off --local` removes only Codexplain-managed project files and
  blocks, leaving user-authored Codex configuration intact.
- `codexplain tui-color on` updates only `.codexplain/config.json`.
- `codexplain tui-color off` disables the TUI color adapter path without
  disabling normal Codexplain exec/review color.
- `codexplain off --global` removes only the Codexplain-managed block under
  `CODEX_HOME/AGENTS.md`.

## Acceptance criteria

- Turning Codexplain on makes new project-local Codex sessions use Codexplain
  explanation guidance and local shim routing.
- Turning Codexplain off restores the default Codex path for the project.
- Adapter work never modifies unrelated repository files or unmanaged Codex
  configuration.
- Any patched Codex build cache stays under `.codexplain/state` and can be
  removed with `codexplain build-clean --patched-codex` without deleting the
  persisted `.codexplain/patched-codex/bin/codex` executable.
- If semantic assistant color cannot be implemented externally, the issue must
  explicitly move the remaining work to a Codex TUI renderer hook/fork.

## Upstream/fork validation checklist

- Hook placement is before Ratatui `Line` or `Span` conversion.
- Adapter input contains semantic roles, not ANSI strings scraped from terminal
  frames.
- `AdapterMode::Off` or equivalent restores default Codex styling.
- Strict artifacts bypass coloring and remain exact.
- No global Codex configuration is modified by project-local enable/disable.
- Patched build cache remains removable with `codexplain build-clean
  --patched-codex`.
- Local regression remains green: `cargo fmt --check`, `cargo test`,
  `cargo build --release`, `codexplain quality-check --width 88`, and
  `codexplain storage-check --min-free-gb 5`.
