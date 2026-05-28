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
- Any patched Codex build cache stays under `.codexplain/state` or
  `.codexplain/patched-codex` and can be removed with
  `codexplain build-clean --patched-codex`.
- If semantic assistant color cannot be implemented externally, the issue must
  explicitly move the remaining work to a Codex TUI renderer hook/fork.
