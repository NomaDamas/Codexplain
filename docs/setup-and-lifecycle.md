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

Bare `/codexplain` is a toggle. If Codexplain is enabled, it disables the
project-local integration. If Codexplain is disabled, it enables it.

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

## 6️⃣ Auto Update

When the project-local shim is active, `codex` startup can check GitHub releases
best-effort. Network failures are skipped so Codex still opens.

Disable for one command:

```bash
CODEXPLAIN_AUTO_UPDATE=off codex
```
