# Codexplain

Codexplain is a project-local explanation UX layer for Codex.

It does not replace Codex or depend on a specific GPT version. The goal is to
make Codex answers easier to understand across model versions by adding a
stable explanation contract, terminal-friendly rendering, and feedback-driven
preference tuning.

## One-Line Setup

Install from this repository and enable it in the current project:

```bash
npm install -g github:NomaDamas/Codexplain && codexplain install-codex --local --force
```

If you are inside this repository while developing it:

```bash
npm run on
```

After setup, Codex in that project receives `AGENTS.md` guidance for clearer
answers. The setup is project-local; it does not edit global Codex config.

## One-Line Use

Run Codex through Codexplain and locally shape the captured output:

```bash
codexplain-codex --local-shape --prompt "쉽고 자세하게 TLDR와 표/흐름도로 설명해줘" exec "이 프로젝트 아키텍처 설명해줘"
```

Set your preferred style:

```bash
codexplain profile --detail deep --set-style tutorial --theme ocean --frame unicode
```

Give feedback after an answer:

```bash
codexplain rlhf --rating 5 --comment "이 정도 깊이와 쉬운 말이 좋다"
```

## What It Improves

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
- Side-by-side table/flow panels only when terminal width allows it.
- Exact commands, file paths, risks, test evidence, and dates preserved.

## Model-Agnostic Goal

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

## RLHF-Lite

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

## CLI

```bash
codexplain demo
codexplain guide --prompt "현재 상태를 쉽게 설명해줘"
codexplain shape --prompt "흐름도로 설명해줘" --response "구현은 완료됐습니다."
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

## Project Files

Setup writes:

```text
AGENTS.md
.codexplain/post-response.mjs
.codexplain/README.md
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
```

## Verification

```bash
npm test
npm run check
```
