# Setup and Lifecycle

Codexplain is project-local by default. It should be easy to turn on, easy to
turn off, and safe to remove without damaging unrelated Codex settings.

## 1️⃣ Project-Local Setup

```bash
npm install -g codexplain
codexplain install-codex --local --force
```

Homebrew:

```bash
brew tap NomaDamas/Codexplain https://github.com/NomaDamas/Codexplain
brew install codexplain
```

Inside this repository during development:

```bash
npm run on
```

Project-local setup manages only Codexplain-owned state:

```text
 Managed item                 Purpose
━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 .codexplain/bin/codex        project-local Codex shim
───────────────────────────  ───────────────────────────────────────
 .codexplain/activate         current-shell activation helper
───────────────────────────  ───────────────────────────────────────
 .codexplain/post-response    post-response shaping helper
───────────────────────────  ───────────────────────────────────────
 .codexplain/config.json      local color/TUI settings
───────────────────────────  ───────────────────────────────────────
 .codexplain/harness-adapter.json
                              shared harness adapter manifest
───────────────────────────  ───────────────────────────────────────
 .codexplain/harnesses/*      target-specific post-response shims
───────────────────────────  ───────────────────────────────────────
 AGENTS.md managed block      local Codexplain response guidance
```

## 2️⃣ Automatic Activation

`codexplain on --local` installs a managed zsh auto-activation block for the
exact project root. New zsh shells automatically route `codex` through the
project-local shim when they start inside the project or `cd` into it.

If you need activation only for the already-open shell:

```bash
source ./.codexplain/activate
```

The auto-activation block is reversible. Leaving the project restores the
previous `codex` alias when one existed.

## 3️⃣ Clean Uninstall

```bash
codexplain uninstall-codex --local
```

To also remove the local UX profile:

```bash
codexplain uninstall-codex --local --remove-profile
```

`off` and uninstall remove only Codexplain-managed files, shell blocks, and
guidance blocks. They do not remove user-authored Codex settings.

## 4️⃣ Global Guidance

Global Codex guidance is explicit opt-in:

```bash
codexplain install-codex --global --force
codexplain uninstall-codex --global
```

Use global mode only when you want Codexplain guidance outside one repository.

## 5️⃣ Slash Bridge

When Codexplain guidance reaches the agent, these commands map to local control:

```text
/codexplain
/codexplain on
/codexplain off
/codexplain status
/codexplain settings
```

Bare `/codexplain` is a toggle. If Codexplain UX is enabled, it disables the
managed guidance/color UX but keeps the native slash bridge installed. If
Codexplain UX is disabled, it enables the project-local guidance and renderer
settings again. Use `codexplain off --local` only when you want strict uninstall.

If the Codex host intercepts unknown slash commands before the prompt reaches
the agent, use the direct CLI fallback:

```bash
codexplain slash on
codexplain slash off
codexplain slash status
codexplain slash toggle
codexplain slash settings
```

Native `/codexplain` inside the full-screen Codex TUI requires the project-local
patched TUI adapter because upstream Codex owns the slash-command registry.

## 6️⃣ Harness Adapter

Codexplain can expose one project-local intervention surface for multiple agent
harnesses:

```bash
codexplain harness-adapter init
codexplain harness-adapter status
codexplain harness-adapter envelope --target oh-my-codex
codexplain harness-adapter envelope --target lazycodex
codexplain harness-adapter envelope --target gajae-code
```

Supported harness targets:

| Target | Integration boundary |
| --- | --- |
| `oh-my-codex` | adapt envelope, probe, and status surfaces |
| `lazycodex` | Codex hook command output boundaries |
| `gajae-code` | assistant-message render boundary before Markdown rendering |

Each target gets a stable post-response command:

```bash
.codexplain/harnesses/<target>/post-response
```

Pipe assistant text, or JSON containing `prompt` and `response`, into that
command. Strict artifacts remain protected by the same Codexplain preservation
rules used by `codexplain post-response`.

Turn only the harness adapter off or on without uninstalling Codexplain:

```bash
codexplain harness-adapter off
codexplain harness-adapter on
codexplain harness-adapter off --target lazycodex
codexplain harness-adapter on --target gajae-code
codexplain slash harness off lazycodex
codexplain slash harness on gajae-code
codexplain slash harness status all
```

When a target is off, only that target shim passes stdin through unchanged. Other
enabled harness shims continue to apply Codexplain explanation shaping.
The slash form is equivalent to the CLI form and works through the patched
Codex TUI `/codexplain` bridge.

## 7️⃣ Auto Update

When the project-local shim is active, `codex` startup can check GitHub releases
best-effort. The development `bin/codexplain` wrapper uses the same best-effort
check when the checkout is clean. Network failures are skipped so Codex and
Codexplain still open.

Disable for one command:

```bash
CODEXPLAIN_AUTO_UPDATE=off codex
```
