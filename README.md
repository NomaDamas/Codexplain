# Codexplain

Codexplain is a project-local explanation UX layer for Codex.

It does not replace Codex or depend on a specific GPT version. The goal is to
make Codex answers easier to understand across model versions by adding a
stable explanation contract, terminal-friendly rendering, and feedback-driven
preference tuning.

## 🧭 Index

- [📌 Project Introduction](#-project-introduction)
- [👀 Before / After](#-before--after)
- [⚡ One-Line Setup](#-one-line-setup)
- [🚀 One-Line Use](#-one-line-use)
- [✨ What It Improves](#-what-it-improves)
- [🧩 Adaptive UX Components](#-adaptive-ux-components)
- [📚 Deep Docs](#-deep-docs)
- [🧠 Model-Agnostic Goal](#-model-agnostic-goal)
- [👍 RLHF-Lite](#-rlhf-lite)
- [🎛️ CLI](#️-cli)
- [🦀 Rust Core](#-rust-core)
- [🧪 Renderer Quality Gate](#-renderer-quality-gate)
- [📚 Research Basis](#-research-basis)
- [💾 Storage Safety](#-storage-safety)
- [🌀 Ouroboros Readiness](#-ouroboros-readiness)
- [📁 Project Files](#-project-files)
- [✅ Verification](#-verification)

## 📌 Project Introduction

Codexplain is a local-first readability layer for Codex responses. It is built
for developers who want Codex answers to keep their technical accuracy while
becoming easier to scan in terminals, CI logs, and CLI chat transcripts.

Codexplain does three things:

- Preserves strict artifacts such as JSON, code blocks, diffs, patches, logs,
  test output, and commit messages exactly as Codex produced them.
- Uses English by default for global open-source use, while mirroring the
  user's language when the user asks in Korean or another language.
- Shapes explanatory prose into predictable UX patterns such as TLDRs, numbered
  steps, width-safe tables, architecture panels, progress reports, risk panels,
  decision matrices, and next-action footers.
- Stores preference and adapter configuration project-locally under
  `.codexplain/` so teams can enable, tune, and remove the integration without
  hidden global side effects.

Codexplain is intentionally not a model, prompt replacement, or cloud service.
The Rust binary runs locally, reads the prompt/response/profile context, applies
deterministic safety checks, and renders a terminal-friendly explanation. It can
wrap `codex exec` output, post-process saved responses, install project-local
Codex guidance, and optionally route interactive TUI sessions through a patched
project-local Codex binary when that adapter is available.

The practical result is a stable answer contract:

```text
Codex does the coding work.
Codexplain controls how explanatory answers are structured, colored, and scanned.
Strict artifacts stay exact.
Project-local setup stays reversible.
```

## 👀 Before / After

Codexplain is built around issues that made terminal explanations hard to read:
dense prose, broken hand-drawn tables, file-first architecture dumps, and noisy
color. These are the three representative fixes.

### 1. Architecture: file dump → capability map

Before:

```text
README.md explains the project. rust/codexplain.rs implements the CLI.
package.json exposes commands. .codexplain contains config and shims.
```

After:

```text
• TLDR
  Codexplain is a presentation boundary around Codex answers.
  It changes how explanations are shaped, not what strict artifacts contain.

┌───────────────────────┐
│ Activation Boundary   │
│ scope: session/project│
└───────────┬───────────┘
────────────▼────────────
┌───────────┴───────────┐
│ Strict Safety         │
│ JSON/code/diff pass   │
└───────────┬───────────┘
────────────▼────────────
┌───────────┴───────────┐
│ UX Planner            │
│ chooses useful blocks │
└───────────┬───────────┘
────────────▼────────────
┌───────────┴───────────┐
│ Semantic Renderer     │
│ width-safe + colored  │
└───────────┬───────────┘
────────────▼────────────
┌───────────┴───────────┐
│ Rollback Boundary     │
│ removes managed state │
└───────────────────────┘
```

### 2. Tables: overflow → row-divided wrapping

Before:

```text
┌────────┬────────────────────────┐
│ Area   │ Description            │
├────────┼────────────────────────┤
│ Policy │ JSON/code/diff/log/test output must remain exact but this long text spills outside the table
│ Render │ Flow/table/progress/risk/color should be selected dynamically
└────────┴────────────────────────┘
```

After:

```text
 Area     Description
━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Policy   JSON/code/diff/log/test output stays exact before rendering.
────────  ─────────────────────────────────────────────────────────────
 Render   Flow, table, progress, risk, and color are chosen by prompt,
          profile, and safety rules.
────────  ─────────────────────────────────────────────────────────────
 Scope    Local on/off removes only Codexplain-managed state.
```

Codexplain also repairs non-strict hand-drawn Unicode tables that forgot body
row separators. Exact JSON, code, diffs, logs, and patches still bypass this
repair path.

### 3. Color: rainbow noise → semantic highlights

Before:

```text
Every sentence can become colorful, but the reader still does not know what
needs attention.
```

After:

```text
 Meaning   Color role            Rule
━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Success   success               PASS, DONE, APPROVED, completed
────────  ────────────────────  ─────────────────────────────────────
 Warning   warning               risk, warning, drift, regression
────────  ────────────────────  ─────────────────────────────────────
 Danger    danger                FAIL, blocked, unsafe, OOM, error
────────  ────────────────────  ─────────────────────────────────────
 Reference command/path/artifact commands, paths, JSON/code/diff/log/test
```

Color is intentionally semantic-sparse: it is a task-relevance signal, not
decoration. Codexplain highlights outcomes, risks, strict artifacts, commands,
and paths; it avoids coloring ordinary nouns such as "renderer" or
"architecture" unless they are structural labels. Emoji cues are used sparingly
for scanning, never as the color system.

## ⚡ One-Line Setup

Install from this repository and enable it in the current project:

```bash
npm install -g github:NomaDamas/Codexplain && codexplain install-codex --local --force
```

Optional global Codex guidance for release users:

```bash
codexplain install-codex --global --force
```

If you are inside this repository while developing it:

```bash
npm run on
```

Cleanly uninstall the managed local integration:

```bash
npm run off
```

Cleanly uninstall only the managed global Codexplain block:

```bash
codexplain uninstall-codex --global
```

Keep feedback preferences by default. To remove `.codexplain/ux-profile.json` too:

```bash
codexplain uninstall-codex --local --remove-profile
```

After local setup, Codex receives project-local `AGENTS.md` guidance. Release users can opt into global Codex guidance with `codexplain install-codex --global --force`. `npm run on` and `npm run off` in this repo are intentionally project-local; use `npm run on:global` and `npm run off:global` only when you explicitly want to modify global Codex guidance.

### Reversible on/off contract

`codexplain on --local` is project-scoped. It writes only Codexplain-managed
project files such as `.codexplain/bin/codex`, `.codexplain/activate`,
`.codexplain/post-response`, `.codexplain/config.json`, and the managed
`CODEXPLAIN:START` block in `AGENTS.md`. If the project-local upstream Codex
clone exists and no patched Codex binary is present, `on --local` also builds
the patched TUI binary once. If the binary already exists, it skips the build.

`codexplain off --local` removes only those managed files and blocks. It does
not remove user-authored Codex settings, unrelated `AGENTS.md` content, or
global Codex configuration. Use `codexplain off --global` only when you want to
remove the managed global Codexplain block under `CODEX_HOME/AGENTS.md`.

Interactive TUI assistant-message color is tracked separately through
`codexplain tui-color on|off`. This switch changes the project-local
`.codexplain/config.json` adapter mode only; turning it off restores the normal
Codex TUI path while keeping ordinary Codexplain `exec`/`review` ANSI shaping
available unless `codexplain color off` is also used.

## 🚀 One-Line Use

Run Codex through Codexplain and locally shape the captured output:

```bash
codexplain-codex --local-shape --prompt "Explain this project architecture with a TLDR, table, and flow diagram" exec "Explain this project architecture"
```

Set your preferred style:

```bash
codexplain profile --detail deep --set-style tutorial --theme ocean --frame unicode
```

Open the dependency-free Rust settings UI:

```bash
codexplain settings-ui
```

The settings UI is organized by user-facing capability rather than internal
files:

```text
 Capability             What You Control
━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Explanation Depth      light, standard, or deep response detail
─────────────────────  ─────────────────────────────────────────────────────
 Architecture View      overview, system, or internals structure depth
─────────────────────  ─────────────────────────────────────────────────────
 Abstraction Level      concrete, architecture, or strategy explanation mode
─────────────────────  ─────────────────────────────────────────────────────
 Color Rules            semantic-sparse role colors, not decorative rainbow UI
─────────────────────  ─────────────────────────────────────────────────────
 Style Library          custom explanation styles with preview and rollback
─────────────────────  ─────────────────────────────────────────────────────
 Scope Control          session, project-local, global guidance, off/uninstall
```

Use the status-bar control surface directly:

```bash
codexplain statusbar status
codexplain statusbar on
codexplain statusbar set --expression-mode concept --architecture-depth internals --theme forest
codexplain statusbar off
```

`statusbar on/off` calls the same project-local install/uninstall path as the
CLI. It does not touch unrelated global Codex settings.

Install lightweight launchers for macOS, Linux, and Windows under
`.codexplain/app`:

```bash
codexplain install-app
```

Run the project-local compatibility gate before release or harness work:

```bash
codexplain compat-check
```

The compatibility gate is intentionally read-only. It verifies that OMX-style
harness state is ignored, local/session/global activation scopes are managed
and reversible, strict artifacts remain unchanged, and the width-safe renderer
contracts still pass.

Control explanation depth with 3-stage levels:

```bash
codexplain profile \
  --explanation-depth deep \
  --architecture-depth internals \
  --abstraction-level architecture \
  --layers tldr,summary,architecture,implementation,evidence,next-step
```

The 3-stage controls are intentionally non-numeric:

```text
explanation-depth  light | standard | deep
architecture-depth overview | system | internals
abstraction-level  concrete | architecture | strategy
```

Numeric controls still exist for selector tuning, not explanation depth:

```bash
codexplain profile --ux-density 70 --risk-sensitivity 80
```

This project defaults to ANSI terminal color output through `.codexplain/config.json`. Use `--theme ocean`, `forest`, or `warm` to make Codexplain terminal output
color-highlight important labels. Use `--theme none` when copy/paste-safe plain
text is more important than visual scanning.

Available color themes include:

```text
none, ocean, forest, warm, sunset, grape, slate, rose, mono
```

If your shell sets `NO_COLOR=1` but you want Codexplain color anyway, force it
for this project command:

```bash
CODEXPLAIN_COLOR=always codexplain shape --theme grape --prompt "Highlight only meaningful signals" --response "DONE: quality-check PASS"
```

Turn the Codexplain color layer on or off for this project:

```bash
codexplain color on
codexplain color off
codexplain color status
codexplain color rules
```

Color is governed by a semantic-sparse policy:

```text
 Role                  Meaning
━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 border                Structure lines for tables, boxes, and flows
────────────────────  ───────────────────────────────────────────────
 heading               Structural labels such as TLDR or section headings
────────────────────  ───────────────────────────────────────────────
 success               PASS, DONE, APPROVED, completed
────────────────────  ───────────────────────────────────────────────
 warning               risk, warning, drift, regression, required
────────────────────  ───────────────────────────────────────────────
 danger                FAIL, blocked, unsafe, OOM, error
────────────────────  ───────────────────────────────────────────────
 command/path/artifact  Commands, paths, JSON/code/diff/log/test refs
```

Add a custom explanation style when a team has its own preferred explanation
shape:

```bash
codexplain style add research-card \
  --trigger "research card" \
  --renderers "tldr,table,formula" \
  --tone "research" \
  --description "Separate background, evidence, limitations, and next action." \
  --example "Explain this design as a research card"

codexplain style list
codexplain style preview research-card
codexplain style remove research-card
```

Codexplain can be enabled at three scopes:

```bash
codexplain on --project  # writes only this repo's managed adapter files
codexplain on --global   # writes only managed guidance under CODEX_HOME
codexplain on --session  # prints the source command for the current shell
```

`--session` cannot mutate a parent shell by itself, so use the printed
`source .codexplain/activate` command when you want the shim only for the
currently open terminal session.

When the project-local shim is active, `codex` startup checks GitHub releases
best-effort. If a newer Codexplain release exists and this repository is on a
clean branch, the shim runs `git pull --ff-only` and rebuilds the release
binary before starting Codex. Network failures or dirty worktrees are skipped
so Codex still opens. Disable this for one command with:

```bash
CODEXPLAIN_AUTO_UPDATE=off codex
```

Turn the project-local interactive Codex TUI assistant-message color hook on or
off. `codexplain install-codex --local` and `npm run on` default this to `full`
so newly opened Codex TUI sessions show color immediately after
`source .codexplain/activate`. This does not modify global Codex settings.
It only routes through a project-local patched Codex binary. `on --local`
builds that binary only when it is missing:

```bash
codexplain tui-color on
codexplain tui-color full
codexplain tui-color off
codexplain tui-color status
```

Use the adapter-specific status surface when you need rollback and routing
details:

```bash
codexplain tui-adapter status
codexplain tui-adapter on
codexplain tui-adapter full
codexplain tui-adapter off
codexplain tui-adapter apply
codexplain tui-adapter build
```

`tui-adapter on` is an alias for the existing `full` enable behavior. It exits
successfully even when no patched Codex binary is available. In that case it
enables project-local config, keeps exec/review shaping available, and reports
that interactive TUI assistant-message recoloring requires a project-local
patched Codex binary. `tui-adapter build` applies the tracked
`patches/codex-tui-assistant-color.patch` and builds only the ignored
project-local patched Codex binary.

For Codex CLI chat output, explicitly use `--chat-color` when you want real
terminal text color. It keeps ANSI styling instead of substituting emoji chips,
while still preserving strict JSON, diffs, code, and logs unchanged:

```bash
codexplain shape --chat-color --theme sunset \
  --prompt "Explain with sparse semantic color" \
  --response "Codexplain is a Rust-only explanation UX renderer."
```

For HTML-capable surfaces only, use the explicit HTML form:

```bash
codexplain shape --color-output html --theme sunset --prompt "Summarize" --response "Body text"
```

Renderer-owned tables are the safe path. If a cell can become long, do not
hand-draw a Unicode table in the prompt; use Codexplain rendering, a Markdown
table, or short per-item boxes. The Rust table model wraps by visible width,
pads every cell, and inserts body row dividers.

Interactive Codex TUI note: the stock npm-installed Codex binary renders
assistant messages inside its native ratatui renderer, so stdout post-processing
cannot recolor those in-place widgets. Codexplain's `tui-color` hook solves this
only when the project-local shim can route to a patched Codex binary under
`.codexplain/state/codex-upstream/codex-rs/target/release/codex` or
`.codexplain/state/codex-upstream/codex-rs/target/debug/codex`.
The upstream/fork contract is tracked in
[`docs/upstream-codex-tui-style-hook.md`](docs/upstream-codex-tui-style-hook.md),
with adapter routing and validation notes in
[`docs/codex-tui-adapter-roadmap.md`](docs/codex-tui-adapter-roadmap.md).

Give feedback after an answer:

```bash
codexplain rlhf --rating 5 --comment "This level of depth and plain language works well"
```

## ✨ What It Improves

Codexplain improves the explanation layer, not the underlying coding model.

```text
Codex answer
    │
    ▼
Codexplain policy
    │ protects exact JSON, code, logs, diffs, patches
    ▼
UX profile
    │ detail, style, audience, color, frame, feedback reward
    ▼
Rust shaper / renderer
    │ TLDR, evidence, next action, table/flow layout
    ▼
Readable terminal answer
```

The result should be:

- TLDR first when the output is explanatory.
- English by default for global open-source use.
- Korean or another user language when the user writes in that language.
- Short paragraphs instead of scattered process narration.
- Unicode box tables and diagrams when they help scanning.
- Row dividers in dense tables so long architecture lists are easier to track.
- Width-safe Codexplain-rendered tables instead of hand-drawn long raw tables
  when cell text may wrap.
- Numbered `1.` `2.` sections for two paths, process, and step-by-step
  explanations instead of one dense paragraph.
- Semantic color highlights for task-critical labels such as TLDR, PASS, FAIL,
  risk, command, path, and strict artifacts.
- Sparse emoji cues only for short status, warning, inspection, ETA, and
  next-action labels; emojis supplement color and text labels, never replace
  them.
- Three-stage explanation depth: light, standard, deep.
- Three-stage architecture depth: overview, system, internals.
- Three-stage abstraction level: concrete, architecture, strategy.
- Adjustable detail layers: TLDR, summary, concept, mechanism, architecture,
  implementation, evidence, next-step.
- Dynamic renderer selection: TLDR prose, progress reports, table, flow,
  pros/cons panels, cause-effect reports, numbered index lists, formula boxes,
  and richer status UX components when they help scanning.
- Notion-style static blocks: toggle-style summaries, quote bars, dividers,
  callouts, and checklists can be composed as supplemental UX blocks without
  replacing the primary renderer.
- Numbered index lists stay compact; when one numbered item needs multiple
  details, use bullet-style sublines instead of blank lines inside the item.
- Compositional renderer selection: when a prompt asks for architecture,
  tradeoffs, and formulas together, Codexplain can combine table/flow context,
  pros/cons comparison, and formula boxes instead of choosing only the first
  matching format.
- Pros/cons and tradeoff questions as comparison panels instead of loose bullets.
- Cause-effect questions as cause/result/response reports instead of
  unstructured prose.
- Progress reports with a short status label above the bar, then a compact
  checkpoint table for current state, percentage, and next action.
- Workflow progress blocks for development, harness, and user-defined workflows
  with a shared schema: type, phase, percent, completed/current/next steps,
  evidence, and trigger source.
- Macro progress reports that collapse verbose `Explored` / `Ran` / `Read`
  transcripts into phase-level UX such as discovery, search, execution,
  configuration, and conclusion.
- Tool-calling-like UX composition: status badges, checklists, risk panels,
  confidence meters, diff summary cards, decision matrices, ETA strips,
  attention callouts, and next-action footers are selected from prompt and
  response signals instead of always being shown.
- Three selector modes: rules for explicit requests, score thresholds for
  implicit components, and optional planner hints through `CODEXPLAIN_UX_PLAN`
  or `CODEXPLAIN_UX_PLANNER_COMMAND`.
- Formula boxes for decision rules or simple math explanations.
- Side-by-side table/flow panels only when terminal width allows it.
- Exact commands, file paths, risks, test evidence, and dates preserved.

## 🧩 Adaptive UX Components

Codexplain treats terminal UX blocks like a small renderer toolbox. It selects
only the components that match the question, the answer state, 3-stage depth
profile, optional selector controls, and the terminal width. A simple
explanation can stay as TLDR prose; a work-status answer can add a badge,
progress bar, checklist, risk panel, and next action; a decision answer can add
pros/cons, formula, confidence, and a decision matrix.

Selector versions now available:

- V1 rules: explicit prompt signals such as `progress`, `risk`, `formula`, or
  `pros/cons` map to known renderers.
- V2 scores: implicit UX signals are scored against `uxDensity` and
  `riskSensitivity`, while explanation depth itself uses the 3-stage controls.
- V3 planner hints: set `CODEXPLAIN_UX_PLAN="risk-panel,next-action"` or
  `CODEXPLAIN_UX_PLANNER_COMMAND` to let an external planner output component
  names; Rust still performs the final safe rendering.

Available visual components:

- Status badge: shows running, blocked, done, or review-needed state.
- Progress report: status text above a bar, followed by checkpoint details.
- Workflow progress: shows development, harness, or custom workflow phase,
  percent, completed/current/next steps, evidence, and trigger source.
- Macro progress: turns micro tool-call transcripts into overall work phases.
- Checklist: separates completed, current, and remaining work.
- Risk panel: calls out hidden assumptions, failures, drift, or blockers.
- Confidence meter: shows certainty as a labeled bar without relying on color.
- Diff summary card: summarizes what changed, impact, and verification.
- Decision matrix: compares options with score and rationale.
- ETA strip: gives elapsed/remaining-state language for progress answers.
- Attention callout: highlights important warnings or copy/paste-safe notes.
- Next-action footer: ends with the single most useful next step.

Example:

```bash
codexplain shape \
  --ux-density 90 \
  --risk-sensitivity 80 \
  --prompt "Show progress with rich UX: risk, confidence, next action" \
  --response "Currently at step 4 of 5. Tests passed and release validation remains."
```

Planner hint example:

```bash
CODEXPLAIN_UX_PLAN="status-badge,risk-panel,next-action" \
  codexplain shape --prompt "Status report" --response "FAILED: provider timeout"
```

## 📚 Deep Docs

README keeps the promotional path short. Detailed design notes live in `docs/`:

```text
 Document                                 Covers
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 docs/architecture.md                     Capability boundaries, runtime flow, safety
───────────────────────────────────────  ─────────────────────────────────────────────
 docs/features.md                         Renderer catalog, settings UI, custom styles
───────────────────────────────────────  ─────────────────────────────────────────────
 docs/explanation-ux-methodology.md       Explanation UX rules and quality gates
───────────────────────────────────────  ─────────────────────────────────────────────
 docs/explanation-research.md             Research basis for structured explanation
───────────────────────────────────────  ─────────────────────────────────────────────
 docs/upstream-codex-tui-style-hook.md    Full TUI assistant-message color hook design
───────────────────────────────────────  ─────────────────────────────────────────────
 docs/codex-tui-adapter-roadmap.md        Adapter routing, rollback, and release path
```

## 🧠 Model-Agnostic Goal

Codexplain is designed to work whether Codex is backed by a newer or older GPT
model. The model may change; the UX contract stays stable.

```text
GPT backend changes
        │
        ▼
Codex coding behavior
        │
        ▼
Codexplain explanation contract
        │
        ▼
Consistent user-facing explanation style
```

## 👍 RLHF-Lite

This project does not train a full RLHF model. Full RLHF needs preference data,
reward modeling, offline evaluation, and model fine-tuning.

Codexplain implements a practical project-local version:

```text
User rates answer
        │
        ▼
Signal extraction
        │ too hard / too short / too long / needs examples
        ▼
Preference reward profile
        │ .codexplain/ux-profile.json
        ▼
Next answer uses adjusted detail and style
```

The profile stores compact preference signals only. It does not store raw answer
text.

## 🎛️ CLI

```bash
codexplain demo
codexplain guide --prompt "Explain the current state plainly"
codexplain shape --prompt "Explain as a flow diagram" --response "Implementation is complete."
codexplain shape --prompt "Explain the Rust-only transition with pros, cons, and a formula" --response "Rust helps with a single binary and low runtime dependency risk."
codexplain shape --prompt "Explain as a cause-effect report" --response "The table cell is too long and spills outside the box, reducing readability."
codexplain post-response --prompt "Explain plainly"
codexplain feedback --rating 2 --comment "Too difficult and missing context"
codexplain rlhf --rating 5 --comment "This style works well"
codexplain build-size
codexplain build-clean --target
```

Renderer composition is local and deterministic by default. If you need an
external planner to choose UX blocks, set `CODEXPLAIN_UX_PLAN` or
`CODEXPLAIN_UX_PLANNER_COMMAND`; Rust still performs the final strict-output
check and terminal rendering.

Legacy `claudex` and `claudex-codex` command names remain as compatibility
aliases, but `codexplain` is the official command.

## 🦀 Rust Core

The runtime implementation is Rust-only. Shell launchers in `bin/` locate the
release binary at `target/release/codexplain` and fall back to `cargo run` while
developing. There are no project JS/MJS runtime files.

The Rust core is dependency-free, fast to start, and keeps build output under
ignored `target/`. Build artifact controls are explicit:

```bash
cargo run --bin codexplain -- pros-cons
cargo run --bin codexplain -- formula --frame ascii
cargo run --bin codexplain -- build-size
cargo run --bin codexplain -- build-clean --target
cargo run --bin codexplain -- build-clean --patched-codex
cargo run --bin codexplain -- storage-check --min-free-gb 5
cargo run --bin codexplain -- storage-check --min-free-gb 5 --clean
cargo test
```

## 🧪 Renderer Quality Gate

Codexplain includes a self-check for the terminal formatting failures that make
answers hard to read:

```bash
./bin/codexplain quality-check --width 88
```

The contract fails if generated output exceeds the requested width, table body
row dividers disappear, architecture explanations do not contain enough boxes,
flow arrows are missing, flow boxes/connectors break, expansion diagrams
overflow, or two-path/process explanations are not numbered.

```text
contract=codexplain.quality-check.v1
overflow_lines=0
row_dividers>=3
architecture_boxes>=6
architecture_panel_overflows=0
flow_arrows>=4
flow_box_overflows=0
flow_connector_breaks=0
expansion_overflows=0
numbered_sections>=2
score>=90
```

## 📚 Research Basis

Codexplain's current explanation UX is mapped in
[`docs/explanation-research.md`](docs/explanation-research.md). The current
implementation borrows from visual attention research, WCAG redundant-coding
guidance, chain-of-thought style stepwise decomposition, RLHF-style preference
feedback, constitutional critique/revision loops, personalized RLHF summaries,
and ICLR 2026 CoT-rubric work.

The highlight policy is intentionally sparse: color is reserved for
task-critical outcomes, risks, strict artifacts, commands, and paths. Ordinary
technical nouns are left plain unless they are structural labels. Candidate
research additions are tracked in the research note before implementation.

## 💾 Storage Safety

The default threshold is configured as `5 GB` in the Rust configuration layer.
Projects may override it in `.codexplain/config.json`:

```json
{
  "storageCheck": {
    "minFree": { "value": 5, "unit": "gb" }
  }
}
```

Invalid values or unsupported units are ignored and fall back to the safe
default of `5 GB`; the CLI flag `--min-free-gb` still overrides configuration.

`storage-check` resolves the effective threshold from the CLI flag, then
project config, then the Rust default. It prints both the compatibility
`min_free_gb` field and `effective_min_free_gb`, and its pass/fail `message`
uses that effective value. If free storage drops below the configured
threshold, it reports cleanup candidates such as `target/`, `dist/`, and
`node_modules/`. With `--clean`, it only removes `target/`, which is regenerated
by Cargo. The command prints stable `key=value` lines beginning with
`contract=codexplain.storage-check.v1` for script-safe parsing.

This cleanup rule is intentionally narrow:

- `target/` can be deleted because Cargo can rebuild it.
- `.codexplain/state/codex-upstream/codex-rs/target/` can be deleted only with
  explicit `build-clean --patched-codex` because it is the project-local patched
  Codex build cache.
- `dist/` is reported but never removed automatically.
- `node_modules/` is reported but never removed automatically.
- Cleanup only runs when available storage is below the effective threshold.

## 🌀 Ouroboros Readiness

Codexplain can be evaluated with Ouroboros, but automation must remain scoped to
this repository. If an evolve or Ralph run emits acceptance criteria from a
different project, cancel that job and restart from an explicit project-local
Seed instead of reusing the drifted lineage.

Current Seed coverage should include:

- Rust is the actual terminal explanation UX core, not a prototype.
- JS runtime has been removed; only shell launchers and the Rust binary remain.
- Dynamic renderer selection can compose multiple requested formats.
- Project-local profile, storage config, and post-response adapter are written
  under `.codexplain/` only.
- Storage cleanup only removes `target/` below the effective threshold.
- Verification includes Rust tests, release build, diff whitespace, forbidden
  trace grep, JS-file trace check, `build-size`, and `storage-check`.

Gap checks added for the renderer migration:

- `compound_prompts_combine_architecture_tradeoff_and_formula_renderers`
- `responsive_architecture_panels_stack_when_terminal_is_narrow`
- `progress_renderer_reports_status_text_bar_and_detail_table`
- `rich_ux_prompt_combines_all_visual_status_components`
- `ux_components_are_selected_dynamically_from_prompt_and_failure_text`
- `ux_density_numerically_controls_implicit_progress_components`
- `ux_planner_plan_parser_accepts_llm_style_component_names`
- `narrow_width_table_snapshot_wraps_and_fits_visible_width`
- `pros_cons_shape_uses_requested_width_instead_of_fixed_snapshot_width`
- `quality_report_enforces_width_row_divider_and_architecture_contracts`

## 📁 Project Files

Setup writes:

```text
AGENTS.md
.codexplain/post-response
.codexplain/README.md
.codexplain/config.json
.codexplain/ux-profile.json
.codexplain/app/
```

Repository layout:

```text
bin/codexplain        shell launcher for the Rust CLI
bin/codexplain-codex  shell launcher for Codex wrapping
bin/claudex           legacy compatibility alias
bin/claudex-codex     legacy compatibility alias
rust/codexplain.rs    dependency-free Rust CLI core
target/               ignored Cargo build artifacts
```

## ✅ Verification

```bash
npm test
npm run check
cargo fmt --check
cargo test
cargo build --release
./bin/codexplain build-size
./bin/codexplain compat-check
./bin/codexplain storage-check --min-free-gb 5
./bin/codexplain quality-check --width 88
```
