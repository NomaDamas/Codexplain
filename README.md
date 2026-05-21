# Claudex

> Codex keeps the coding strength. Claudex rewrites the final answer before it
> is shown, so the user sees a lower-friction explanation UX instead of raw
> process-heavy output.

<p align="center">
  <strong>Dynamic answer rewriting. Connected terminal boxes. Strict-format safety.</strong>
  <br>
  <em>A package that makes Codex speak with a calmer, clearer explanation UX.</em>
</p>

## 🧭 Index

| Go to | Why it matters |
| --- | --- |
| ✨ [What Claudex Does](#-what-claudex-does) | Product purpose |
| ⚡ [Dynamic Layer](#-dynamic-layer) | How post-response generation works |
| 🧠 [Explanation Contract](#-explanation-contract) | How answers should feel |
| 🔬 [Research Basis](#-research-basis) | Why the UX is shaped this way |
| 🖼️ [Terminal UX](#%EF%B8%8F-terminal-ux) | Connected boxes and flows |
| 🚀 [Install](#-install) | Add Claudex to a Codex project |
| 🤖 [Codex Integration](#-codex-integration) | Make Codex answer through Claudex |
| 🧩 [CLI](#-cli) | `guide`, `shape`, `post-response`, and `demo` |
| 🔌 [Adapter](#-adapter) | Hook-ready without global side effects |
| 🧪 [Verification](#-verification) | Tests and identity checks |
| 🗺️ [Roadmap](#%EF%B8%8F-roadmap) | Next build steps |

## ✨ What Claudex Does

Claudex is a dynamic readability layer for Codex responses.

It is built for the case where Codex is strong at coding but its explanations
feel too code-heavy, too scattered, or visually hard to scan. Claudex lets Codex
finish the answer first, then rewrites the presentation layer before the user
sees it:

- Korean-first concise prose for Korean prompts.
- Conclusion before implementation details.
- Connected Unicode tables and flows when they make structure clearer.
- No broken pseudo-ASCII borders.
- TLDR/current-state/evidence/next-step framing when it reduces cognitive load.
- Backoff for JSON, code, logs, test output, diffs, patches, and commit messages.
- Dynamic provider support through OpenAI or any custom local rewrite command.
- Adaptive UX profile support for user-selected explanation depth and style.
- Feedback-based evolution that stores preference signals, not raw answers.
- Failure-safe fallback to deterministic shaping or the original adapter input.
- No hidden-prompt copying, scraping, or model distillation.
- No repository ships pre-installed hooks; installation writes into the target
  project only when the user runs the install command.

## ⚡ Dynamic Layer

Claudex is not limited to fixed rules. The intended runtime path is:

```text
Codex completes answer
       │
       ▼
Claudex post-response hook
       │
       ├─ strict artifact? return unchanged
       ├─ dynamic provider available? rewrite for readability
       ├─ provider failed/timeout/empty? safe fallback
       ▼
User sees concise, structured answer
```

Dynamic rewriting is enabled by configuration:

```bash
export OPENAI_API_KEY=...
export CLAUDEX_DYNAMIC=1
```

Or use a local/custom model command:

```bash
export CLAUDEX_REWRITE_COMMAND="node ./my-rewriter.mjs"
export CLAUDEX_DYNAMIC=1
```

The command receives JSON on stdin:

```json
{
  "prompt": "사용자 요청",
  "response": "완성된 Codex 답변",
  "instruction": "Claudex rewrite contract"
}
```

It must print only the rewritten answer to stdout.

Important behavior:

- Claudex does not call Codex a second time by default.
- If `OPENAI_API_KEY` is set, Claudex can call the configured OpenAI model for
  the rewrite.
- If `CLAUDEX_REWRITE_COMMAND` is set, Claudex delegates rewriting to that
  command instead.
- If no provider is configured, automatic post-response integrations leave the
  completed answer unchanged; the deterministic shaper remains available through
  `claudex shape`.
- If a configured provider fails, Claudex falls back to the deterministic local
  shaper.
- If the adapter itself errors, it prints the original input so Codex is not
  broken by the hook.

## 🧠 Explanation Contract

Claudex uses a simple contract:

```text
User prompt
    │
    ▼
Completed Codex answer
    │
    ▼
Safety + rewrite decision
    │
    ├─ strict artifact → return unchanged
    │
    ├─ dynamic rewrite → TLDR / evidence / diagram when useful
    │
    └─ fallback → deterministic shaping or original input
    │
    ▼
Human-scannable answer
```

The layer should feel natural. It should not print labels like “style layer
activated” or explain that it is rewriting the answer.

## 🧬 Adaptive UX Evolution

Claudex can keep a project-local explanation profile in `.claudex/ux-profile.json`.
The profile controls how much detail to include and what style to use:

```bash
claudex profile --set-style tutorial
claudex profile --detail deep
claudex profile --theme ocean
claudex profile --frame unicode
```

Available styles:

```text
plain      easy language for general users
tutorial   step-by-step teaching
concise    compact, high-signal answers
executive  outcome, impact, risk, decision
technical  implementation-aware explanations
review     findings, severity, evidence, fixes
```

Color themes:

```text
none    no ANSI color, safest for logs and copy/paste
ocean   blue/cyan terminal grouping
forest  green terminal grouping
warm    red/yellow terminal grouping
mono    grayscale emphasis
```

Frame styles:

```text
unicode  connected box drawing, best visual density
ascii    +---+ tables and v arrows, safest for plain terminals and copy/paste
```

`unicode` is the default. It keeps both architecture diagrams and tables in the
same connected box-drawing language:

```text
┌─────────────┬────────────────────┐
│ 구분        │ 내용               │
├─────────────┼────────────────────┤
│ 핵심        │ 결론 먼저 설명     │
└─────────────┴────────────────────┘
```

Users can also influence the current answer directly through the prompt:

```text
초보도 이해하게 자세하지만 쉽게 설명해줘
핵심만 짧게 말해줘
구현 관점에서 기술적으로 설명해줘
```

After an answer, feedback can evolve the profile:

```bash
claudex feedback --rating 2 --comment "너무 어렵고 설명이 부족해"
```

That feedback nudges the next profile toward easier and deeper explanations. It
stores only compact preference signals such as `needs-more-detail`; it does not
store the original answer text.

## 🔬 Research Basis

Claudex's UX contract follows a few research-backed principles rather than
copying another product's hidden prompt:

- Cognitive load theory: reduce working-memory pressure with chunking,
  signaling, and removing irrelevant detail.
- Multimedia learning: use diagrams/tables only when they integrate related
  information better than prose; avoid decorative visuals.
- Human-AI explanation safety: preserve uncertainty and evidence so a clearer
  answer does not become a more persuasive false answer.
- LLM reliability work: do not turn fluent presentation into a claim of extra
  correctness; verification evidence must remain explicit.

## 🖼️ Terminal UX

Flow example:

```text
┌────────────┐
│ 요청 이해  │
└──────┬─────┘
       │
       ▼
┌──────┴─────┐
│ 핵심 정리  │
└──────┬─────┘
       │
       ▼
┌──────┴─────┐
│ 검증 보존  │
└────────────┘
```

Table example:

```text
┌────────┬────────────────┐
│ 영역   │ 답변 기준      │
├────────┼────────────────┤
│ 내용   │ 기술 사실 보존 │
│ 표현   │ 짧고 스캔 가능 │
└────────┴────────────────┘
```

## 🚀 Install

Install the package, then connect it to the Codex project where you want the
answer UX to change:

```bash
npm install -g claudex
cd /path/to/your/codex-project
claudex install-codex --local
```

That command does not modify global Codex config. It prepares the current
project so Codex gets Claudex response guidance and optional post-response
shaping.

For non-interactive Codex runs, use the wrapper directly:

```bash
claudex-codex --prompt "흐름도로 설명해줘" exec "현재 구현 상태를 설명해줘"
```

Or equivalently:

```bash
claudex codex --prompt "흐름도로 설명해줘" exec "현재 구현 상태를 설명해줘"
```

## 🤖 Codex Integration

Claudex makes Codex speak in the improved UX through two surfaces:

```text
Codex prompt
    │
    ├─ interactive project session
    │      └─ AGENTS.md response contract guides Codex's natural answer
    │
    └─ captured/non-interactive output
           └─ Claudex post-response layer rewrites presentation
```

Project install writes these files into the target project:

```text
AGENTS.md
.claudex/post-response.mjs
.claudex/README.md
```

What this means:

- Interactive Codex in that project is instructed to answer with Claudex's
  concise, Korean-first, connected-box style.
- `codex exec` style output can be captured and dynamically rewritten by
  `claudex-codex`.
- Host tools that support post-response commands can call
  `.claudex/post-response.mjs`.
- Exact artifacts are protected and returned unchanged.

## 🧩 CLI

Run from this repository:

```bash
node bin/claudex.js demo
```

After installing or linking the package, use `claudex` directly.

Generate guidance for a prompt:

```bash
claudex guide --prompt "현재 상태를 보기 쉽게 설명해줘"
```

Shape an answer:

```bash
claudex shape \
  --prompt "현재 상태를 보기 쉽게 설명해줘" \
  --response "구현은 완료됐습니다. 검증은 \`npm test\`로 통과했습니다."
```

For paired table/flow explanations, Claudex reads the available terminal width
and places panels side by side when there is enough room. When the terminal is
narrow, it stacks the same panels vertically to avoid hard-to-read wrapping:

```bash
claudex shape --width 120 \
  --prompt "표와 흐름을 좌우로 아키텍처 설명해줘" \
  --response "CLI가 입력을 받고 policy가 보호합니다."
```

Width detection order:

```text
--width → CLAUDEX_WIDTH → process.stdout.columns → COLUMNS → 80
```

Use local shaping in a post-response pipe without a dynamic provider:

```bash
echo '{"prompt":"쉽게 설명해줘","response":"구현은 완료됐습니다."}' \
  | claudex post-response --local-shape
```

Manage adaptive explanation preferences:

```bash
claudex profile --show
claudex profile --set-style plain
claudex profile --detail deep
claudex profile --theme ocean
claudex profile --frame unicode
claudex feedback --rating 5 --comment "이 정도 설명이 좋아"
```

Shape an answer through the dynamic layer:

```bash
CLAUDEX_DYNAMIC=1 \
CLAUDEX_REWRITE_COMMAND="node ./my-rewriter.mjs" \
claudex shape --dynamic \
  --prompt "현재 상태를 보기 쉽게 설명해줘" \
  --response "구현은 완료됐습니다. 검증은 \`npm test\`로 통과했습니다."
```

Pipe an answer through stdin:

```bash
echo "구현은 완료됐습니다. 검증은 \`npm test\`로 통과했습니다." \
  | claudex shape --prompt "흐름도로 설명해줘"
```

Render the built-in UX demo:

```bash
claudex demo
```

## 🔌 Adapter

To prepare one project for post-response shaping, run inside that project:

```bash
claudex init --local
```

This creates only:

```text
.claudex/post-response.mjs
.claudex/README.md
```

If a host tool can pipe a completed answer into a command, point it at:

```bash
node .claudex/post-response.mjs
```

The adapter accepts JSON:

```json
{"prompt":"흐름도로 설명해줘","response":"구현은 완료됐습니다. 검증은 `npm test`로 통과했습니다."}
```

Or plain text with `CLAUDEX_PROMPT` set:

```bash
CLAUDEX_PROMPT="흐름도로 설명해줘" node .claudex/post-response.mjs
```

If `claudex` is unavailable, the adapter prints the original input unchanged
instead of breaking the host tool.

## 🧪 Verification

Run:

```bash
npm test
npm run check
```

Current checks cover:

- Korean prompt classification.
- Strict-format backoff.
- Connected table and flow rendering.
- TLDR-first explanations.
- Responsive paired panels: side by side when wide, stacked when narrow.
- Hangul visible-width handling.
- Dynamic rewrite provider success/failure/timeout fallback.
- Project-local adapter generation.
- Project-local Codex guidance installation.
- Public identity guard against inherited project branding.

## 🗺️ Roadmap

- Add snapshot tests for more Korean final-answer shapes.
- Add provider-specific examples for local LLM servers.
- Add terminal-width probes for macOS/Linux shells.
- Add a small visual docs page showing good and bad answer shapes.
- Track official Codex hook schemas if a future version exposes direct
  response-replacement hooks.

## 📁 Project Layout

| Path | Purpose |
| --- | --- |
| `bin/claudex.js` | CLI entrypoint |
| `bin/claudex-codex.js` | Codex wrapper that shapes captured output |
| `src/policy.js` | Intent detection and strict-format backoff |
| `src/renderer.js` | Connected tables and flows |
| `src/text-width.js` | Hangul-aware terminal width handling |
| `src/shaper.js` | Deterministic answer shaping |
| `src/dynamic-rewriter.js` | Dynamic post-response rewrite layer |
| `src/codex-install.js` | Target-project Codex integration installer |
| `src/codex-runner.js` | `codex exec` wrapper integration |
| `test/` | Node test suite |

## 📜 License

MIT
