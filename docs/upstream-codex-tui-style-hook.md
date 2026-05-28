# Upstream Codex TUI Style Hook Proposal

## Problem

External wrappers can color completed stdout, but they cannot reliably recolor
interactive Codex TUI assistant messages after Ratatui has painted them into
terminal cells.

## Proposed hook

Expose a style adapter at the assistant-message rendering boundary in Codex TUI.
The adapter should receive semantic text spans before they are converted into
Ratatui `Line` or `Span` values.

## Desired behavior

- Preserve exact code, JSON, diffs, logs, and test output.
- Allow semantic coloring for headings, warnings, risks, next actions, commands,
  paths, and architecture labels.
- Allow the adapter to be disabled completely.
- Keep Codex's default style guide as the fallback when no adapter is active.
- Avoid PTY frame scraping as the primary integration path.

## Why an internal hook is preferable

Codex TUI is a fullscreen immediate-mode terminal UI. External PTY tools see
cursor moves and repainted cells, not stable assistant-message structure. A
renderer hook keeps the transformation at the only layer where semantic message
intent is still available.
