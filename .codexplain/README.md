# Codexplain Local Adapter

This directory is project-local and Rust-only at runtime. The default output mode is ANSI terminal color for explanation surfaces.

To route this project's terminal Codex calls through Codexplain, activate the local shim:

```bash
source .codexplain/activate
codex exec "이 프로젝트 아키텍처를 표와 흐름도로 설명해줘"
```

The shim only prepends `.codexplain/bin` in the current shell. `codexplain uninstall-codex --local` removes the shim files and the managed AGENTS.md block.

On/off is reversible by design:

- `codexplain on --local` adds only Codexplain-managed project files and the
  managed `CODEXPLAIN:START` block.
- `codexplain off --local` removes only those managed files and blocks.
- `codexplain tui-color off` disables only the project-local TUI color adapter
  mode and does not remove unrelated Codex settings.

Activation scopes:

```bash
codexplain on --project
codexplain on --global
codexplain on --session
```

`--project` changes only this repository's managed Codexplain files. `--global`
changes only the managed guidance block under `CODEX_HOME`. `--session` prints
the `source .codexplain/activate` command because a child process cannot mutate
the parent shell directly.

Color can be toggled without uninstalling Codexplain:

```bash
codexplain color on
codexplain color off
codexplain color status
```

Open the Rust-only settings UI or install local app launchers:

```bash
codexplain statusbar status
codexplain settings-ui
codexplain install-app
```

`codexplain statusbar on|off|set` is the project-local control surface for
power, theme, color output, expression mode, and the three explanation depth
levels.

Validate project-local OMX/harness compatibility without mutating settings:

```bash
codexplain compat-check
```

`codex exec` and `codex review` can be post-processed with Codexplain ANSI text color. Interactive Codex TUI is passed through to the real Codex process with color env (`CLICOLOR_FORCE`, `FORCE_COLOR`, `COLORTERM`). Assistant-message recoloring inside ratatui requires the project-local patched Codex renderer.

Project-local interactive TUI assistant color can be toggled without touching global Codex settings:

```bash
codexplain tui-color on
codexplain tui-color full
codexplain tui-color off
codexplain tui-color status
```

Adapter status and rollback details are available through:

```bash
codexplain tui-adapter status
codexplain tui-adapter on
codexplain tui-adapter full
codexplain tui-adapter off
```

`tui-adapter on` is an alias for the existing `full` enable behavior. It does not clone or build upstream Codex. If no patched binary is present, it exits
successfully and reports the fallback: exec/review shaping still works, while
interactive TUI assistant-message recoloring needs a project-local patched Codex
binary.

The local adapter defaults `tuiAssistantColor` to `full`, so activating Codexplain and opening a new Codex TUI session shows assistant-message color immediately. The shim routes to `.codexplain/state/codex-upstream/codex-rs/target/release/codex` or `.codexplain/state/codex-upstream/codex-rs/target/debug/codex` when that binary exists and `tuiAssistantColor` is enabled.

Use this adapter when a host can pipe a completed answer into a post-response command:

```bash
.codexplain/post-response --prompt "흐름도로 설명해줘"
```

Input may be plain text or JSON with `prompt` and `response` fields. The Rust core preserves exact JSON, code, diffs, logs, and test output when strict formatting matters.

Explanation depth uses 3-stage controls:

```text
explanationDepth light/standard/deep
architectureDepth overview/system/internals
abstractionLevel concrete/architecture/strategy
```

UX selection combines explicit rules, score thresholds, and optional planner hints through `CODEXPLAIN_UX_PLAN` or `CODEXPLAIN_UX_PLANNER_COMMAND`.
Cause-effect prompts such as `원인-결과`, `인과`, `cause-effect`, or `왜 ... 그래서` render as 원인/결과/대응 reports.
Workflow prompts such as `workflow`, `progress block`, `개발 워크플로우`,
`하네스`, or `사용자 정의 워크플로우` add a shared progress block with type,
phase, percent, completed/current/next steps, evidence, and trigger source.

Tables are renderer-owned. If text may be long, use the Codexplain renderer,
Markdown tables, or short boxes instead of hand-drawing raw Unicode tables. The
Rust renderer wraps by visible width, fills each cell with padding, and inserts
body row dividers.

Architecture, flow, and expansion diagrams are renderer-owned too. If labels
may wrap, use Codexplain flow/diagram output so box width, arrows, connectors,
and branch labels are measured instead of hand-drawn.

Numbered index lists stay compact. If one numbered item needs multiple details,
use bullet-style sublines instead of blank lines inside the item.

Notion-style static blocks are available as supplemental UX components. Prompts
with hints such as `notion`, `toggle`, `quote`, `divider`, `노션`, `토글`,
`인용`, or `구분선` can add toggle-style summaries, quote bars, dividers,
callouts, and checklists without replacing the primary renderer.

Renderer quality can be checked locally:

```bash
codexplain quality-check --width 88
```

Custom explanation styles:

```bash
codexplain style add research-card --trigger "연구 카드" --renderers "tldr,table,formula,cause-effect" --description "배경, 근거, 한계, 다음 행동을 분리한다."
codexplain style list
codexplain style remove research-card
```
