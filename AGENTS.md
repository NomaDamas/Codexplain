<!-- CODEXPLAIN:START -->
# Codexplain Response UX

Shape user-facing answers with a clear, readable, color-aware terminal/chat experience while preserving Codex's coding precision.

Default answer style:
- Start with the outcome or current state, not implementation detail.
- Use English by default for global open-source usage.
- Mirror the user's language when the user writes in Korean or another non-English language.
- Use connected Unicode boxes or tables when structure helps scanning.
- Use semantic ANSI colors for labels, risks, success states, artifact names, commands, paths, and next actions when the terminal supports color.
- Color policy is semantic-sparse: use color as a meaning signal, not decoration. Keep one dominant structure color, one key/accent color, and state colors only for success/warning/danger/reference terms.
- Use ANSI terminal color by default when Codexplain config asks for `defaultColorOutput: ansi`; for Codex CLI chat output, prefer real ANSI text color over decorative emoji chips or raw HTML spans, but do not suppress semantic emoji cues.
- Respect explanationDepth light/standard/deep, architectureDepth overview/system/internals, and abstractionLevel concrete/architecture/strategy.
- Select renderers dynamically: TLDR prose, progress, tables, flow diagrams, pros/cons, formula boxes, status badges, checklists, risk panels, confidence meters, decision matrices, ETA strips, callouts, Notion-style toggle/quote/divider blocks, and next-action footers.
- Emoji-first chat policy: when `emojiCues` is true and exact artifact formatting is not required, prefer compact emoji-labeled sections/items over Markdown-heavy headings, dense bullets, or tables.
- When Codexplain is ON in Codex CLI, use semantic emoji cues actively as section markers and status markers. Use them for TLDR/summary, success, warning, danger, evidence, fix/action, architecture, progress, settings, docs, and next steps while preserving exact artifacts unchanged.
- Emoji cues are part of the explanation UX, not decoration. Prefer one meaningful emoji at the start of each semantic section or numbered item: 🧭 overview/architecture, ✅ success, ⚠️ warning/risk, 🚨 danger/failure, 🔎 evidence/inspection, 🛠️ fix/action, 📌 note, 📚 docs/research, 🎛️ settings, 🚀 next step.
- Pair emojis with text labels so meaning survives no-emoji fonts and screen readers. Do not replace commands, paths, JSON, code, diffs, logs, tests, or patches with emojis.
- Markdown is a transport and fallback format, not the primary explanation style. Use it for exact code fences, file links, small tables, or renderer fallback only when it improves clarity.
- Treat UX blocks like tool choices: combine the smallest useful set from prompt, response, profile, and optional planner hints.
- Split explanations by semantic units with active line breaks. If the answer says "two paths", "두 가지", "과정", or "단계", render them as compact 1. 2. 3. numbered sections. Do not put blank lines inside one numbered item; if an item has multiple details, use short bullet-style sublines under that item.
- Use indentation as a meaning boundary: continuation lines align under the content column, not under the number marker; do not add decorative vertical bars to numbered lists.
- Architecture, structure, system design, component, flow, and expansion answers must include a renderer-owned diagram first. Use tables as a second visual layer when they add role/decision summaries.
- Architecture/project explanations must explain by capability boundary and runtime responsibility first, not by file list. Mention files only as supporting evidence after the conceptual structure is clear.
- Architecture/project explanations should create a visible "wow point": TLDR first, at least one boxed diagram second, then optional capability map and wide-divider table using ━ for the header rule and ─ between rows.
- Tables must include row dividers between body rows and must wrap long cell text inside the visible width instead of overflowing.
- Every visible table row must be separated. Do not produce a dense table where many `│ ... │` body rows appear back-to-back without `├...┤` or `─` row separators.
- Do not hand-draw long Unicode tables from raw model text. If a cell may exceed the terminal width, use Codexplain's width-safe renderer output, a Markdown table, or short per-item boxes so every cell is filled and padded by visible width.
- Process answers should use short numbered sections, with one idea per item and bullet-style sublines for multiple details.
- Problem-diagnosis answers should be answer-first, then flow downward from evidence/reason to fix and the user's exact question. In Korean, use natural labels such as 결론, 왜 그런가, 해결 흐름, 네 질문, 답 instead of forcing rigid English labels. In English, Conclusion/Evidence/Fix/Your question/Answer is acceptable when it improves scanning.
- Keep commands, paths, risks, test evidence, and exact technical facts intact.
- Do not continue an Ouroboros evolve/ralph lineage if drift is detected. Restart with an explicit project-local Seed.

Strict-output safety:
- Do not rewrite JSON, code blocks, diffs, patches, logs, test output, or commit messages when exact formatting matters.
- If exact formatting matters, return the artifact unchanged.

Terminal UX:
- Use connected box-drawing characters such as ┌ ┬ ┐ │ ├ ┼ ┤ └ ┴ ┘.
- Do not use broken pseudo-borders made from repeated hyphens, equals signs, or Korean long vowel marks.
- Do not hand-draw architecture, flow, or expansion diagrams when labels may wrap. Use Codexplain flow/diagram output so box width, connectors, arrows, and branch labels are layout-owned.
- Flow connectors must not rely on leading spaces alone. Prefer renderer-owned full-width connector rails so arrows stay aligned in Codex CLI, terminal scrollback, and chat transcripts.
- Do not hand-draw long raw box tables when cell text may wrap. Prefer Codexplain width-safe tables, Markdown tables, or short boxes with wrapped rows; every row must be layout-owned, padded, and separated, not manually guessed.
- For long tool transcripts such as Explored/Ran/Read, summarize the macro phase first instead of listing every micro event.
- Use blank lines between semantic sections so the user can scan without reading a wall of text.
- Slash control bridge: when a user asks exactly `/codexplain`, run `./bin/codexplain slash toggle` to switch project-local Codexplain UX on/off while preserving the native slash bridge. When a user asks exactly `/codexplain on`, run `./bin/codexplain slash on` for project-local activation. When a user asks exactly `/codexplain off`, run `./bin/codexplain slash off` to disable Codexplain UX but keep the installed `/codexplain` control available. When a user asks `/codexplain status`, run `./bin/codexplain slash status`. When a user asks `/codexplain harness ...`, run `./bin/codexplain slash harness ...` to control per-harness Codexplain adapters. Do not treat these as prose requests.
- Settings control bridge: if the user asks to change Codexplain explanation depth, abstraction level, architecture depth, color, emoji, density, or explanation settings, run `./bin/codexplain settings-ui` when an interactive terminal is available. The bare `codexplain` command also opens this settings UI.
<!-- CODEXPLAIN:END -->
