# Codexplain Features

This document lists the major user-facing capabilities. README shows the short
promotional version; this file keeps the fuller catalog.

## Renderer UX

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
  --trigger "연구 카드" \
  --renderers "tldr,table,formula" \
  --tone "research" \
  --description "배경, 근거, 한계, 다음 행동을 분리한다." \
  --example "연구 카드로 이 설계를 설명해줘"

codexplain style list
codexplain style preview research-card
codexplain style remove research-card
```

Custom styles are stored under `.codexplain/styles/` and remain project-local
unless the user explicitly manages a global guidance block.

## Color Policy

Codexplain uses semantic-sparse color.

```text
 Role                    Intended Meaning
━━━━━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 border                  Table, box, and flow structure
──────────────────────  ───────────────────────────────────────────────
 heading                 TLDR, architecture, renderer, important labels
──────────────────────  ───────────────────────────────────────────────
 success                 completed, passed, possible, preserved
──────────────────────  ───────────────────────────────────────────────
 warning                 needed, running, workaround, hook
──────────────────────  ───────────────────────────────────────────────
 danger                  failed, error, impossible, not visible
──────────────────────  ───────────────────────────────────────────────
 command/path/artifact    exact commands, paths, JSON/code/diff/log/test
```

The rule is intentionally strict: color should improve attention, not make the
whole answer rainbow-colored.

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
