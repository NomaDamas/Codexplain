# Codexplain Architecture

Codexplain is not a replacement model and not a second cloud service. It is a
local presentation boundary around Codex answers.

## Capability Map

```text
 Capability Boundary                     Responsibility
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Activation Boundary                      Chooses session-only, project-local, or
                                          managed global guidance.
───────────────────────────────────────  ─────────────────────────────────────────────
 Execution Boundary                       Runs Codex or receives a completed answer
                                          without changing stderr or exit status.
───────────────────────────────────────  ─────────────────────────────────────────────
 Strict Safety Boundary                   Lets exact JSON, code, diff, logs, tests,
                                          patches, and commit messages pass through.
───────────────────────────────────────  ─────────────────────────────────────────────
 Preference Boundary                      Resolves depth, abstraction, theme, color
                                          policy, and custom explanation styles.
───────────────────────────────────────  ─────────────────────────────────────────────
 UX Planning Boundary                     Selects TLDR, table, flow, progress, risk,
                                          quote, toggle, checklist, or next action.
───────────────────────────────────────  ─────────────────────────────────────────────
 Rendering Boundary                       Wraps by visible width, inserts row
                                          dividers, and applies semantic ANSI color.
───────────────────────────────────────  ─────────────────────────────────────────────
 Rollback Boundary                        Removes only Codexplain-managed blocks,
                                          shims, configs, and adapter files.
```

## Runtime Flow

```text
┌───────────────────────┐
│ User / Codex request  │
└───────────┬───────────┘
────────────▼────────────
┌───────────┴───────────┐
│ Scope Resolver        │
│ session/project/global│
└───────────┬───────────┘
────────────▼────────────
┌───────────┴───────────┐
│ Codex Runner          │
│ stdout/stderr/exit    │
└───────────┬───────────┘
────────────▼────────────
┌───────────┴───────────┐
│ Strict Safety Policy  │
│ exact artifacts pass  │
└───────────┬───────────┘
────────────▼────────────
┌───────────┴───────────┐
│ Profile Resolver      │
│ depth/theme/styles    │
└───────────┬───────────┘
────────────▼────────────
┌───────────┴───────────┐
│ Renderer Selector     │
│ useful UX blocks only │
└───────────┬───────────┘
────────────▼────────────
┌───────────┴───────────┐
│ Terminal Renderer     │
│ layout/color/wrapping │
└───────────────────────┘
```

## Explanation Rule

Architecture answers must start with functional responsibility and abstraction
level, and they must include a renderer-owned diagram before prose. File names
are supporting evidence, not the architecture itself.

```text
 Required visual order
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 1. TLDR / capability boundary
────────────────────────────────────────────────────────────
 2. Boxed runtime or component diagram
────────────────────────────────────────────────────────────
 3. Optional capability map or second diagram
────────────────────────────────────────────────────────────
 4. Row-divided table for roles, tradeoffs, or evidence
```

```text
 Preferred                              Avoid
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 "Strict Safety Boundary preserves..." "rust/codexplain.rs has..."
────────────────────────────────────  ───────────────────────────────────
 "Rendering Boundary wraps cells..."   "README.md says..."
────────────────────────────────────  ───────────────────────────────────
 "Rollback Boundary removes managed..." ".codexplain contains..."
```

## Safety Contract

Codexplain may change explanatory presentation. It must not rewrite exact
technical artifacts.

```text
 Artifact Type                Policy
━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 JSON                         Preserve exactly
───────────────────────────  ─────────────────────────────────────────────
 Code blocks                   Preserve exactly
───────────────────────────  ─────────────────────────────────────────────
 Diff / patch                  Preserve exactly
───────────────────────────  ─────────────────────────────────────────────
 Logs / test output            Preserve exactly
───────────────────────────  ─────────────────────────────────────────────
 Explanatory prose             Shape with renderer-owned UX
```

## Implementation Evidence

The architecture above is implemented by the Rust CLI core, shell shims, and
project-local `.codexplain/` state. Those files are implementation evidence for
the capability boundaries, not the primary explanation model.
