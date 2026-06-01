# Codexplain Features

This document lists the major user-facing capabilities. README shows the short
promotional version; this file keeps the fuller catalog.

## Renderer UX

Codexplain is English-first for global open-source defaults. It mirrors the
user's language when the user writes in Korean or another language.

```text
Feature                         What It Does
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 TLDR                             Starts explanatory answers with the point
───────────────────────────────  ───────────────────────────────────────────
 Width-safe table                 Wraps cells and inserts every row divider
───────────────────────────────  ───────────────────────────────────────────
 Flow diagram                     Uses renderer-owned boxes and connectors
───────────────────────────────  ───────────────────────────────────────────
 Pros/cons                        Compares options without dense prose
───────────────────────────────  ───────────────────────────────────────────
 Formula box                      Shows simple scoring or tradeoff equations
───────────────────────────────  ───────────────────────────────────────────
 Cause/effect report              Separates root cause from visible symptom
───────────────────────────────  ───────────────────────────────────────────
 Progress UI                      Reports phase, bar, evidence, and next step
───────────────────────────────  ───────────────────────────────────────────
 Notion-style blocks              Adds quote, divider, toggle, and callout cues
───────────────────────────────  ───────────────────────────────────────────
 Semantic color                   Highlights only meaningful states and refs
```

## Settings UI

`codexplain settings-ui` is dependency-free and terminal-native. It presents
settings by capability, not file path.

```text
 Setting Area              Choices
━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Explanation depth         light, standard, deep
────────────────────────  ────────────────────────────────────────────────
 Architecture view         overview, system, internals
────────────────────────  ────────────────────────────────────────────────
 Abstraction level         concrete, architecture, strategy
────────────────────────  ────────────────────────────────────────────────
 Theme                     ocean, forest, warm, sunset, grape, slate, rose,
                           mono, none
────────────────────────  ────────────────────────────────────────────────
 UX density                35, 65, 90 presets for how many UX blocks appear
────────────────────────  ────────────────────────────────────────────────
 Scope                     session, project-local, global guidance, off
```

## Custom Explanation Styles

Teams can add explanation patterns without changing Rust code.

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

Custom styles are stored under `.codexplain/styles/` and remain project-local
unless the user explicitly manages a global guidance block.

## Color Policy

Codexplain uses semantic-sparse color. Highlighting is a task-relevance cue,
not a decoration layer.

```text
 Role                    Intended Meaning
━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 border                  Table, box, and flow structure
──────────────────────  ───────────────────────────────────────────────
 heading                 TLDR and structural section labels
──────────────────────  ───────────────────────────────────────────────
 success                 PASS, DONE, APPROVED, completed
──────────────────────  ───────────────────────────────────────────────
 warning                 risk, warning, drift, regression, required
──────────────────────  ───────────────────────────────────────────────
 danger                  FAIL, blocked, unsafe, OOM, error
──────────────────────  ───────────────────────────────────────────────
 command/path/artifact    exact commands, paths, JSON/code/diff/log/test
```

The rule is intentionally strict: color should improve attention, not make the
whole answer rainbow-colored. Ordinary nouns such as "renderer", "architecture",
"Codexplain", and "TUI" are not colored inside normal prose unless they appear
as structural labels.

### Highlight Criteria

The current highlight policy is a conservative baseline, not a claim of final
optimality. It highlights meaning-bearing tokens only when they affect user
attention or copy/paste safety.

```text
 Signal group              Why it gets color
━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Outcome                   PASS, FAIL, DONE, blocked, approved
────────────────────────  ─────────────────────────────────────────────
 Risk                      warning, drift, regression, unsafe, OOM
────────────────────────  ─────────────────────────────────────────────
 Exact artifact            JSON, code, diff, patch, log, test output
────────────────────────  ─────────────────────────────────────────────
 Action                    commands, flags, install/off/check commands
────────────────────────  ─────────────────────────────────────────────
 Location                  paths, config files, managed state
```

The next improvement should be a context score: color only the highest-signal
terms per section, suppress repeated terms, and prefer outcome/risk/action over
generic technical nouns.

## Quality Gates

```bash
cargo fmt --check
cargo test
./bin/codexplain quality-check --width 88
./bin/codexplain compat-check
./bin/codexplain storage-check --min-free-gb 5
```

The quality check fails on overflowing renderer output, missing body row
dividers, broken flow connectors, sparse architecture diagrams, and missing
numbered decomposition for two-path/process answers.

## Row Divider Enforcement

Codexplain-rendered tables always separate body rows. If a non-strict answer
contains a hand-drawn Unicode table with a header divider but no body row
dividers, Codexplain post-processing inserts row separators between adjacent
body rows.

```text
 Before                                 After
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 │ row 1 │ value │                    │ row 1 │ value │
 │ row 2 │ value │                    ├───────┼───────┤
                                      │ row 2 │ value │
```

Strict artifacts are still preserved. JSON, code, diffs, patches, logs, tests,
and commit messages bypass this table repair path.
