# Codexplain Local Adapter

This directory is project-local and Rust-only at runtime. The default output mode is ANSI terminal color for explanation surfaces.

To route this project's terminal Codex calls through Codexplain, activate the local shim:

```bash
source .codexplain/activate
codex exec "이 프로젝트 아키텍처를 표와 흐름도로 설명해줘"
```

The shim only prepends `.codexplain/bin` in the current shell. `codexplain uninstall-codex --local` removes the shim files and the managed AGENTS.md block.

Color can be toggled without uninstalling Codexplain:

```bash
codexplain color on
codexplain color off
codexplain color status
```

`codex exec` and `codex review` can be post-processed with Codexplain ANSI text color. Interactive Codex TUI is passed through to the real Codex process with best-effort color env (`CLICOLOR_FORCE`, `FORCE_COLOR`, `COLORTERM`), but native assistant-message recoloring inside ratatui requires Codex renderer support.

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

Custom explanation styles:

```bash
codexplain style add research-card --trigger "연구 카드" --renderers "tldr,table,formula" --description "배경, 근거, 한계, 다음 행동을 분리한다."
codexplain style list
codexplain style remove research-card
```
