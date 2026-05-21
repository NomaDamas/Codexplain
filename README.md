# Codexplain

Codexplain is a project-local explanation UX layer for Codex.

It does not replace Codex or depend on a specific GPT version. The goal is to
make Codex answers easier to understand across model versions by adding a
stable explanation contract, terminal-friendly rendering, and feedback-driven
preference tuning.

## 🧭 Index

- [⚡ One-Line Setup](#-one-line-setup)
- [🚀 One-Line Use](#-one-line-use)
- [✨ What It Improves](#-what-it-improves)
- [🧠 Model-Agnostic Goal](#-model-agnostic-goal)
- [👍 RLHF-Lite](#-rlhf-lite)
- [🎛️ CLI](#️-cli)
- [🦀 Rust Core](#-rust-core)
- [💾 Storage Safety](#-storage-safety)
- [📁 Project Files](#-project-files)
- [✅ Verification](#-verification)

## ⚡ One-Line Setup

Install from this repository and enable it in the current project:

```bash
npm install -g github:NomaDamas/Codexplain && codexplain install-codex --local --force
```

If you are inside this repository while developing it:

```bash
npm run on
```

After setup, Codex in that project receives `AGENTS.md` guidance for clearer
answers. Setup is project-local; it does not edit global Codex config.

## 🚀 One-Line Use

Run Codex through Codexplain and locally shape the captured output:

```bash
codexplain-codex --local-shape --prompt "쉽고 자세하게 TLDR와 표/흐름도로 설명해줘" exec "이 프로젝트 아키텍처 설명해줘"
```

Set your preferred style:

```bash
codexplain profile --detail deep --set-style tutorial --theme ocean --frame unicode
```

Control explanation layers and abstraction range:

```bash
codexplain profile \
  --detail deep \
  --abstraction-range concrete:architecture \
  --layers tldr,summary,architecture,implementation,evidence,next-step
```

Use `--theme ocean`, `forest`, or `warm` to make Codexplain terminal output
color-highlight important labels. Use `--theme none` when copy/paste-safe plain
text is more important than visual scanning.

Available color themes include:

```text
none, ocean, forest, warm, sunset, grape, slate, rose, mono
```

Give feedback after an answer:

```bash
codexplain rlhf --rating 5 --comment "이 정도 깊이와 쉬운 말이 좋다"
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
Shaper / dynamic rewriter
    │ TLDR, evidence, next action, table/flow layout
    ▼
Readable terminal answer
```

The result should be:

- TLDR first when the output is explanatory.
- Korean-first when the user writes Korean.
- Short paragraphs instead of scattered process narration.
- Unicode box tables and diagrams when they help scanning.
- Row dividers in dense tables so long architecture lists are easier to track.
- Semantic color highlights for labels such as TLDR, 핵심, 장점, 단점, 위험.
- Adjustable abstraction range: concrete, implementation, architecture, strategy.
- Adjustable detail layers: TLDR, summary, concept, mechanism, architecture,
  implementation, evidence, next-step.
- Dynamic renderer selection: TLDR prose, table, flow, pros/cons panels,
  numbered index lists, and formula boxes.
- Pros/cons and tradeoff questions as comparison panels instead of loose bullets.
- Formula boxes for decision rules or simple math explanations.
- Side-by-side table/flow panels only when terminal width allows it.
- Exact commands, file paths, risks, test evidence, and dates preserved.

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
codexplain guide --prompt "현재 상태를 쉽게 설명해줘"
codexplain shape --prompt "흐름도로 설명해줘" --response "구현은 완료됐습니다."
codexplain shape --prompt "JS와 Rust 장단점을 pros and cons 표와 수식으로 설명해줘" --response "JS는 실험에 좋고 Rust는 제품화에 좋습니다."
codexplain post-response --local-shape --prompt "쉽게 설명해줘"
codexplain feedback --rating 2 --comment "너무 어렵고 설명이 부족해"
codexplain rlhf --rating 5 --comment "이 스타일이 좋아"
```

Dynamic rewriting can use OpenAI or a local command:

```bash
export CODEXPLAIN_DYNAMIC=1
export OPENAI_API_KEY=...
```

```bash
export CODEXPLAIN_DYNAMIC=1
export CODEXPLAIN_REWRITE_COMMAND="node ./my-rewriter.mjs"
```

Legacy `claudex` and `claudex-codex` command names remain as compatibility
aliases, but `codexplain` is the official command.

## 🦀 Rust Core

The primary `codexplain` CLI routes core explanation and terminal rendering
commands through the Rust binary. Node remains as a thin compatibility layer for
npm distribution, project installation, and Codex wrapper integration while the
remaining wrapper pieces are ported.

The Rust core is dependency-free, fast to start, and keeps build output under
ignored `target/`.

```bash
cargo run --bin codexplain -- pros-cons
cargo run --bin codexplain -- formula --frame ascii
cargo run --bin codexplain -- storage-check --min-free-gb 5
cargo run --bin codexplain -- storage-check --min-free-gb 5 --clean
cargo test
```

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
- `dist/` is reported but never removed automatically.
- `node_modules/` is reported but never removed automatically.
- Cleanup only runs when available storage is below the effective threshold.

## 📁 Project Files

Setup writes:

```text
AGENTS.md
.codexplain/post-response.mjs
.codexplain/README.md
.codexplain/config.json
.codexplain/ux-profile.json
```

Repository layout:

```text
bin/codexplain.js        CLI entrypoint
bin/codexplain-codex.js  Codex wrapper
src/policy.js            strict-output protection
src/evolution.js         UX profile and feedback loop
src/shaper.js            deterministic answer shaping
src/renderer.js          Unicode tables, flows, responsive panels
src/dynamic-rewriter.js  provider-backed rewrite layer
rust/codexplain.rs       dependency-free Rust CLI core
```

## ✅ Verification

```bash
npm test
npm run check
cargo fmt --check
cargo test
cargo build --release
node bin/codexplain.js storage-check --min-free-gb 5
```
