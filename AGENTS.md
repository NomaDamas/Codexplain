

<!-- CODEXPLAIN:START -->
# Codexplain Response UX

Shape user-facing answers with a clear, readable, color-aware terminal/chat experience while preserving Codex's coding precision.

Default answer style:
- Start with the outcome or current state, not implementation detail.
- Use concise Korean first when the user writes Korean.
- Use connected Unicode boxes or tables when structure helps scanning.
- Use semantic ANSI colors for labels, risks, success states, artifact names, commands, paths, and next actions when the terminal supports color.
- Use ANSI terminal color by default when Codexplain config asks for `defaultColorOutput: ansi`; for Codex CLI chat output, prefer real ANSI text color over emoji chips or raw HTML spans.
- Respect explanationDepth light/standard/deep, architectureDepth overview/system/internals, and abstractionLevel concrete/architecture/strategy.
- Select renderers dynamically: TLDR prose, progress, tables, flow diagrams, pros/cons, formula boxes, status badges, checklists, risk panels, confidence meters, decision matrices, ETA strips, callouts, and next-action footers.
- When Codexplain is ON in Codex CLI, highlight important terms with sparse semantic ANSI colors. Emojis may be used only as light explanatory supplements, not as the color system.
- Treat UX blocks like tool choices: combine the smallest useful set from prompt, response, profile, and optional planner hints.
- Split explanations by semantic units with active line breaks. If the answer says "two paths", "두 가지", "과정", or "단계", render them as 1. 2. 3. numbered sections instead of one dense paragraph, and leave a blank line between numbered items.
- Architecture, flow, and expansion diagrams should prefer Codexplain renderer-owned boxes before prose. Use a table only when it adds a compact role/decision summary.
- Tables must include row dividers between body rows and must wrap long cell text inside the visible width instead of overflowing.
- Do not hand-draw long Unicode tables from raw model text. If a cell may exceed the terminal width, use Codexplain's width-safe renderer output, a Markdown table, or short per-item boxes so every cell is filled and padded by visible width.
- Keep commands, paths, risks, test evidence, and exact technical facts intact.
- Do not continue an Ouroboros evolve/ralph lineage if drift is detected. Restart with an explicit project-local Seed.

Strict-output safety:
- Do not rewrite JSON, code blocks, diffs, patches, logs, test output, or commit messages when exact formatting matters.
- If exact formatting matters, return the artifact unchanged.

Terminal UX:
- Use connected box-drawing characters such as ┌ ┬ ┐ │ ├ ┼ ┤ └ ┴ ┘.
- Do not use broken pseudo-borders made from repeated hyphens, equals signs, or Korean long vowel marks.
- Do not hand-draw architecture, flow, or expansion diagrams when labels may wrap. Use Codexplain flow/diagram output so box width, connectors, arrows, and branch labels are layout-owned.
- Do not hand-draw long raw box tables when cell text may wrap. Prefer Codexplain width-safe tables, Markdown tables, or short boxes with wrapped rows; every row must be layout-owned, padded, and separated, not manually guessed.
<!-- CODEXPLAIN:END -->
