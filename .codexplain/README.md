# Codexplain Local Adapter

This directory is project-local and Rust-only at runtime. The default output mode is ANSI terminal color for explanation surfaces.

To route this project's terminal Codex calls through Codexplain, activate the local shim:

```bash
source .codexplain/activate
codex exec "이 프로젝트 아키텍처를 표와 흐름도로 설명해줘"
```

The shim only prepends `.codexplain/bin` in the current shell. `codexplain on --local` builds the project-local patched Codex TUI binary only when it is missing. `codexplain uninstall-codex --local` removes the shim files and the managed AGENTS.md block.

Color can be toggled without uninstalling Codexplain:

```bash
codexplain color on
codexplain color off
codexplain color status
codexplain color rules
```

Open the project-local status control surface or install local app launchers:

```bash
codexplain statusbar status
codexplain install-app
```

`codexplain statusbar on|off|set` controls power, theme, color output,
expression mode, and the three explanation depth levels without touching
unrelated global Codex settings.

Validate project-local OMX/harness compatibility without mutating settings:

```bash
codexplain compat-check
```

`codex exec` and `codex review` can be post-processed with Codexplain ANSI text color. Interactive Codex TUI is passed through to the real Codex process with color env (`CLICOLOR_FORCE`, `FORCE_COLOR`, `COLORTERM`). Assistant-message recoloring inside ratatui requires the project-local patched Codex renderer.

When this shim is active, `codex` startup performs a best-effort GitHub release
check. If a newer Codexplain release exists and this repo is on a branch with no
user-code changes, the shim runs `git pull --ff-only` and rebuilds the release
binary before starting Codex. Dirty Codexplain-managed local adapter files do
not block the check; unrelated dirty files still do. It never blocks Codex
startup on network failure. Disable it for one command with
`CODEXPLAIN_AUTO_UPDATE=off codex`.

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
codexplain tui-adapter apply
codexplain tui-adapter build
```

`tui-adapter on` uses restrained semantic highlighting by default. Use
`tui-adapter full` only when you explicitly want stronger recoloring. If no
patched binary is present, it exits successfully and reports the fallback:
exec/review shaping still works, while interactive TUI assistant-message
recoloring needs a project-local patched Codex binary. `tui-adapter build`
applies `patches/codex-tui-assistant-color.patch` and
`patches/codex-tui-codexplain-slash.patch` to the ignored project-local upstream
clone and builds only the project-local patched Codex binary.

The shim routes to `.codexplain/state/codex-upstream/codex-rs/target/release/codex` or `.codexplain/state/codex-upstream/codex-rs/target/debug/codex` when that binary exists and `tuiAssistantColor` is enabled.

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
codexplain style add research-card --trigger "연구 카드" --renderers "tldr,table,formula" --description "배경, 근거, 한계, 다음 행동을 분리한다." --tone "research" --example "연구 카드로 이 설계를 설명해줘"
codexplain style add problem-diagnosis --trigger "왜 안됨" --renderers "problem-diagnosis" --description "문제 원인과 해결책을 결론부터 말하고 근거, 해결 흐름, 질문-답으로 자연스럽게 내려가며 정리한다." --tone "direct" --example "왜 안되고 있는지 문제와 해결책을 설명해줘"
codexplain style list
codexplain style preview research-card
codexplain style remove research-card
```
