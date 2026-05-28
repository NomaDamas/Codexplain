# Upstream Codex TUI Style Hook Proposal

Status: proposed for upstream or project-local fork work.
Tracking issue: Issue #4.
Scope: documentation and contract only. This document does not require cloning
or modifying `openai/codex`.

## Purpose

Define the upstream or fork-facing contract for Codex TUI assistant-message
styling hooks so Codexplain can request semantic color spans without relying on
terminal-frame scraping. The proposal records where the hook should live, what
semantic information it must receive, and which behavior is invariant even if
upstream Rust type names change.

This document is intentionally project-local. It gives Issue #4 a stable
proposal artifact for upstream discussion or future fork work while keeping the
implementation out of this repository until a compatible Codex renderer hook
exists.

## Proposal scope

This proposal is limited to the assistant-message styling boundary inside a
future upstream Codex TUI change or a project-local fork. It defines the
minimum contract Codexplain needs to request semantic color spans while Codex
continues to own message content, rendering order, and default styling.

In scope:

- A renderer-level hook that receives assistant-message spans with semantic
  roles before Ratatui `Line` or `Span` conversion.
- A disabled mode that fully restores Codex's default assistant-message
  rendering path.
- A strict-artifact bypass for JSON, code, diffs, patches, logs, test output,
  and commit messages.
- A semantic role vocabulary for headings, warnings, risks, next actions,
  commands, paths, artifacts, and architecture labels.
- Project-local configuration and routing expectations for Codexplain when a
  compatible upstream or forked Codex binary is available.

Out of scope:

- Cloning, vendoring, or modifying `openai/codex` from this repository.
- Implementing the hook in this repository before a compatible Codex renderer
  boundary exists.
- Replacing Codex's formatter, markdown parser, conversation state, or
  Ratatui layout system.
- Changing assistant message text, tool output, command strings, file paths, or
  structured artifacts.

## Non-goals

- Do not make PTY scraping the primary integration path. A PTY proxy can remain
  a diagnostic or best-effort fallback, but it is not the contract this proposal
  validates.
- Do not require global Codex configuration changes. Project-local enable,
  disable, and patched-binary routing must be reversible.
- Do not standardize exact Rust type names. Names such as
  `AssistantStyleAdapter`, `AssistantMessageSpan`, and `AdapterMode` are
  proposal vocabulary only; the behavioral invariants are binding.
- Do not require color to carry meaning by itself. Plain, no-color, and
  accessibility-constrained terminals must still receive readable labels and
  text roles.
- Do not style strict artifacts or rewrite exact output to make it fit a theme.

## Compatibility boundaries

The hook should be compatible with Codex's default behavior first. If no
adapter is registered, if the adapter cannot load, or if the adapter reports an
off mode, Codex must render through its existing style path with no semantic
styling side effects.

The adapter boundary is semantic text, not ANSI text and not terminal cells.
Adapter input must describe roles and exact-artifact status before Ratatui
conversion. Adapter output may map those roles to Ratatui styling, but it must
not depend on cursor movement, alternate-screen frames, or post-paint terminal
inspection.

Compatibility expectations:

- Upstream Codex can rename proposed Rust items, move modules, or choose a
  different trait/function shape if the binding invariants remain true.
- Codexplain can map semantic roles to ANSI or Ratatui colors, but must preserve
  exact artifacts and keep a full adapter-disable fallback.
- Existing Codex TUI users who do not enable the adapter should see default
  Codex styling.
- Future Codex renderer changes remain acceptable if the hook still runs before
  Ratatui `Line` or `Span` conversion and receives enough semantic information
  to style assistant-message spans without PTY scraping.

## Problem

External wrappers can color completed stdout, but they cannot reliably recolor
interactive Codex TUI assistant messages after Ratatui has painted them into
terminal cells.

The useful semantic boundary is earlier than terminal output. At that point the
renderer still knows whether a span is a heading, warning, command, path, code
artifact, risk, next action, or plain prose. Once the fullscreen TUI has emitted
cursor movement and cell repaint operations, a PTY proxy only sees terminal
frames and loses that message structure.

## Proposed hook

Expose a style adapter at the assistant-message rendering boundary in Codex TUI.
The adapter should receive semantic text spans before they are converted into
Ratatui `Line` or `Span` values.

## Codex TUI rendering flow integration points

The hook belongs inside Codex's assistant-message rendering flow after Codex has
identified assistant-message structure and before the TUI converts that
structure into Ratatui widgets. The intent is to style known message spans, not
to intercept terminal output after paint.

Expected flow:

1. Conversation state stores an assistant message.
2. Codex prepares that message for the transcript or active response view.
3. Markdown or message-block segmentation identifies prose, headings, inline
   code, fenced artifacts, command text, paths, warnings, risks, and next
   actions.
4. Codex marks strict artifacts such as JSON, code blocks, diffs, patches, logs,
   test output, and commit messages as non-stylable.
5. Codex dispatches stylable assistant-message spans to the adapter with their
   semantic role, strict-artifact status, default style, and terminal color
   capability.
6. Codex applies only the returned style decision to the original span text.
7. Codex converts the resulting text plus style decisions into Ratatui `Line`,
   `Span`, paragraph, list, or transcript widgets.
8. Ratatui performs wrapping, layout, buffering, diffing, and terminal painting.

Hook placement requirements:

- The adapter call is after message segmentation because the adapter needs
  semantic roles, not raw assistant text.
- The adapter call is before Ratatui `Line` or `Span` construction because text
  structure is still available and no terminal-cell information is required.
- Strict-artifact detection happens before adapter dispatch so exact artifacts
  can bypass the adapter entirely.
- The disabled path bypasses adapter dispatch and enters the same default style
  conversion Codex would have used without Codexplain.
- The adapter returns style data only. Codex remains responsible for markdown
  parsing, wrapping, widget selection, transcript ordering, scroll state, and
  terminal painting.

Non-integration points:

- Do not attach the primary hook to stdout, stderr, a PTY proxy, terminal-frame
  replay, alternate-screen capture, or cursor-cell inspection.
- Do not place the hook after Ratatui has already produced terminal buffer
  cells because semantic span roles are no longer reliable there.
- Do not let the adapter own or rewrite the assistant-message text, command
  strings, file paths, tool output, or strict artifact bodies.
- Do not require Codexplain-specific global Codex settings to activate the
  renderer path; project-local routing must be enough.

Upstream or fork implementations can choose different module names, but the
observable boundary must remain: assistant-message semantic spans enter the
adapter before Ratatui conversion, strict artifacts bypass it, and `Off` mode
uses Codex's default renderer path.

## Formal API shape

The preferred hook shape is style-only. Codex should continue to own the source
message text, markdown segmentation, rendering order, wrapping, and Ratatui
conversion. The adapter should receive immutable assistant-message span metadata
and return only a style decision for that exact span. This keeps the integration
compatible with strict artifacts and makes the off path identical to Codex's
default renderer.

Proposed Rust-level interface shape, with non-binding names:

```rust
use ratatui::style::{Color, Modifier, Style};

pub enum AssistantSpanRole {
    Plain,
    Heading,
    Emphasis,
    Muted,
    Success,
    Warning,
    Danger,
    Risk,
    Command,
    Path,
    Artifact,
    ArchitectureLabel,
    NextAction,
}

pub enum StrictArtifactKind {
    Json,
    CodeBlock,
    Diff,
    Patch,
    Log,
    TestOutput,
    CommitMessage,
}

pub enum TerminalColorCapability {
    None,
    Ansi16,
    Ansi256,
    TrueColor,
}

pub struct AssistantMessageSpan<'a> {
    pub role: AssistantSpanRole,
    pub text: &'a str,
    pub strict_artifact: Option<StrictArtifactKind>,
    pub source_range: Option<std::ops::Range<usize>>,
}

pub enum AdapterMode {
    Off,
    Semantic,
}

pub struct AssistantStyleContext {
    pub color_capability: TerminalColorCapability,
    pub default_style: Style,
    pub adapter_mode: AdapterMode,
}

pub struct SemanticStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifiers: Modifier,
}

pub enum StyleDecision {
    UseCodexDefault,
    Apply(SemanticStyle),
}

pub trait AssistantStyleAdapter {
    fn mode(&self) -> AdapterMode;
    fn style_span(
        &self,
        span: &AssistantMessageSpan<'_>,
        context: &AssistantStyleContext,
    ) -> StyleDecision;
}
```

The names can change in upstream Codex. The invariants below are the binding
part of the contract.

## Hook binding invariants

The following invariants are normative for Issue #4. Proposed Rust names,
module names, enum shapes, and trait boundaries are non-binding; these
behaviors are binding for any upstream or fork implementation.

- `HI-1: pre-Ratatui semantic boundary`. The hook must run after Codex has
  segmented an assistant message into semantic spans and before those spans are
  converted into Ratatui `Line` or `Span` values. The primary hook must not
  depend on stdout filtering, PTY frame scraping, cursor-cell inspection, or
  terminal repaint replay.
- `HI-2: Codex owns text and layout`. Codex remains the owner of assistant
  message text, markdown parsing, block ordering, wrapping, widget selection,
  transcript state, scroll state, and terminal painting. The adapter can only
  influence style decisions for spans that Codex has already identified.
- `HI-3: strict artifacts bypass`. JSON, code blocks, diffs, patches, logs,
  test output, commit messages, and equivalent exact-output blocks must bypass
  adapter styling. They must render byte-for-byte or span-for-span equivalent
  to Codex default output.
- `HI-4: full default fallback`. If no adapter is registered, adapter loading
  fails, the project-local adapter mode is disabled, or the effective mode is
  equivalent to `Off`, Codex must render through the same default
  assistant-message style path it would use without Codexplain.
- `HI-5: style-only output`. Adapter output must be style metadata only. It
  must not replace, reorder, trim, normalize, redact, expand, or otherwise
  rewrite assistant text, tool output, command strings, paths, or structured
  artifacts.
- `HI-6: semantic color spans`. Stylable spans must preserve semantic role
  metadata such as heading, emphasis, warning, danger, risk, success, command,
  path, artifact label, architecture label, next action, muted text, and plain
  prose so Codexplain can map roles to terminal colors or Ratatui styles.
- `HI-7: color is supplemental`. No-color, plain, low-color, and accessibility
  constrained terminals must remain readable. Color can highlight meaning, but
  labels and text roles must still carry the meaning without color.
- `HI-8: project-local disable`. Project-local Codexplain configuration must be
  able to disable the adapter path completely without changing global Codex
  configuration or disabling ordinary non-TUI Codexplain `exec` or `review`
  shaping.

Required call sequence:

1. Codex parses and segments the assistant message into semantic spans.
2. Codex checks the adapter registration and effective `AdapterMode`.
3. If there is no adapter, the adapter cannot load, or the mode is `Off`, Codex
   renders every span through the existing default style path.
4. If a span has `strict_artifact: Some(_)`, Codex bypasses adapter styling and
   renders that span through the existing default style path.
5. Otherwise Codex calls `style_span` before Ratatui `Line` or `Span`
   conversion.
6. Codex applies the returned style to the original `span.text`; the adapter
   must not replace, reorder, trim, normalize, or otherwise rewrite text.

API requirements:

- `AssistantMessageSpan::text` is read-only input. The adapter must not receive
  ownership of assistant-message text or any API that allows replacement text.
- `StyleDecision::UseCodexDefault` must be available per span so the adapter can
  decline styling without disabling the whole renderer.
- `AdapterMode::Off` must be a full renderer bypass, not a low-color mode. It
  restores Codex's default assistant-message style path for all spans.
- `TerminalColorCapability::None` must remain valid. The adapter can decline
  color or use non-color modifiers, but meaning must remain present in the text
  labels Codex already renders.
- `StrictArtifactKind` can be represented differently upstream, but the hook
  must distinguish strict artifacts from stylable prose before adapter dispatch.
- If upstream chooses a Ratatui `Span`-returning API instead, Codex must enforce
  text identity by constructing the final `Span` from the original `span.text`
  and adapter-provided style only.

## Adapter responsibilities

The adapter is a narrow style decision surface. It is responsible for mapping
Codex-provided semantic roles to safe terminal styles and for declining styling
when the current span or terminal cannot support it safely.

- `AR-1: accept immutable semantic spans`. The adapter consumes immutable span
  metadata from Codex, including role, original text reference, optional source
  range, strict-artifact status, default style, and terminal color capability.
- `AR-2: return style decisions only`. The adapter returns either
  `UseCodexDefault` or an additive semantic style. It never returns replacement
  text, alternate markdown, terminal escape strings, cursor operations, or
  Ratatui layout instructions.
- `AR-3: preserve strict artifacts`. If Codex dispatches a strict artifact by
  mistake, the adapter must still decline styling and return the default-style
  decision. Codex should also enforce strict-artifact bypass before dispatch.
- `AR-4: honor disable mode`. When the effective mode is disabled or off, the
  adapter must not style individual spans as a degraded mode. Off means full
  Codex default rendering for assistant messages.
- `AR-5: map semantic roles consistently`. The adapter may map roles to ANSI,
  Ratatui colors, or non-color modifiers, but it must keep a stable semantic
  vocabulary so headings, warnings, risks, next actions, commands, paths,
  artifacts, and architecture labels remain distinguishable when color is
  available.
- `AR-6: degrade safely by terminal capability`. For
  `TerminalColorCapability::None` or equivalent, the adapter should return
  Codex defaults or non-color emphasis only. It must not inject textual markers
  into the assistant message to compensate for missing color.
- `AR-7: fail closed to Codex default`. Adapter errors, unknown roles, unsupported
  color capabilities, and invalid project-local config must resolve to Codex's
  default style for the affected span or the whole assistant message.
- `AR-8: avoid global side effects`. Adapter activation, patched-binary routing,
  and disable state must remain project-local. The adapter must not require
  unmanaged edits to global Codex settings or unrelated repository files.

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

## Binding invariant summary

- Strict artifacts bypass styling. JSON, code blocks, diffs, patches, logs, test
  output, and commit messages must render byte-for-byte or span-for-span
  equivalent to Codex default output. See `HI-3`.
- Default fallback is Codex. If no adapter is registered, or `AdapterMode::Off`
  is active, the renderer must use Codex's existing style path. See `HI-4`.
- The adapter receives semantic spans before Ratatui `Line` or `Span`
  conversion. It must not depend on reading terminal frames after paint. See
  `HI-1`.
- The adapter is fully disableable through project-local Codexplain config. See
  `HI-8`.
- Styling is additive. It must not change assistant message content, tool
  output, commands, file paths, or structured artifacts. See `HI-5`.
- Color cannot be the only carrier of meaning. Labels and text roles must still
  be readable in plain terminals. See `HI-7`.

## Codexplain integration boundary

Codexplain already controls these surfaces:

- `codexplain shape`: local text shaping with strict-artifact bypass.
- `codexplain post-response`: completed response shaping.
- `codexplain codex --local-shape`: project-local wrapper around non-TUI Codex
  output.
- `.codexplain/bin/codex`: project-local shim that can route to a patched Codex
  binary when one exists.

The TUI hook belongs inside the patched or upstream Codex renderer, not in the
PTY layer. Codexplain should only provide configuration, semantic role mapping,
patched-binary detection, and safe on/off routing.

## Validation checklist for upstream or fork work

- Add a unit fixture where assistant prose receives multiple semantic roles.
- Add a strict-artifact fixture proving JSON, code, diff, log, and test output
  bypass adapter styling.
- Add an `AdapterMode::Off` fixture proving byte-equivalent fallback to Codex
  default rendering.
- Add a no-color or plain-terminal fixture proving labels remain meaningful.
- Confirm the hook runs before Ratatui line/span conversion.
- Confirm `.codexplain/config.json` can disable project-local routing without
  touching global Codex settings.
