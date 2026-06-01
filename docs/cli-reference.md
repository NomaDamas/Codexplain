# CLI Reference

This document keeps command details out of the README.

## 1️⃣ Shape Existing Text

```bash
codexplain shape \
  --prompt "Explain with a TLDR and table" \
  --response "DONE: tests PASS. Risk: JSON/code/diff output must stay exact."
```

Useful flags:

```text
 Flag                         Purpose
━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 --width 88                   target terminal width
───────────────────────────  ───────────────────────────────────────
 --theme ocean                semantic color theme
───────────────────────────  ───────────────────────────────────────
 --chat-color                 ANSI-friendly Codex CLI output
───────────────────────────  ───────────────────────────────────────
 --color-output html          HTML span output for HTML-capable hosts
───────────────────────────  ───────────────────────────────────────
 --emoji-cues                 enable active semantic emoji cues
───────────────────────────  ───────────────────────────────────────
 --no-emoji-cues              disable emoji cues
```

## 2️⃣ Profile and Settings

```bash
codexplain settings-ui
codexplain profile --detail deep --set-style tutorial --theme ocean
codexplain profile --explanation-depth deep --architecture-depth internals
codexplain profile --ux-density 70 --risk-sensitivity 80
```

Depth controls:

```text
explanation-depth  light | standard | deep
architecture-depth overview | system | internals
abstraction-level  concrete | architecture | strategy
```

## 3️⃣ Color Control

```bash
codexplain color on
codexplain color off
codexplain color status
codexplain color rules
```

Color policy is semantic-sparse:

```text
 Role                   Meaning
━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 success                PASS, DONE, APPROVED, completed
─────────────────────  ───────────────────────────────────────
 warning                risk, warning, drift, regression
─────────────────────  ───────────────────────────────────────
 danger                 FAIL, blocked, unsafe, OOM, error
─────────────────────  ───────────────────────────────────────
 command/path/artifact  commands, paths, JSON/code/diff/log/test
```

## 4️⃣ Custom Styles

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

## 5️⃣ TUI Adapter

```bash
codexplain tui-color on
codexplain tui-color full
codexplain tui-color off
codexplain tui-color status

codexplain tui-adapter status
codexplain tui-adapter build
codexplain tui-adapter off
```

The stock Codex TUI renders assistant messages inside its own ratatui renderer.
Full in-place TUI recoloring therefore requires the project-local patched Codex
binary managed by the adapter.

## 6️⃣ Quality and Compatibility

```bash
cargo fmt --check
cargo test
./bin/codexplain quality-check --width 88
./bin/codexplain compat-check
./bin/codexplain storage-check --min-free-gb 5
```
