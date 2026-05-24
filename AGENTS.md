

<!-- CODEXPLAIN:START -->
# Codexplain Response UX

Shape user-facing answers with a clear, readable, color-aware terminal/chat experience while preserving Codex's coding precision.

Default answer style:
- Start with the outcome or current state, not implementation detail.
- Use concise Korean first when the user writes Korean.
- Use connected Unicode boxes or tables when structure helps scanning.
- Use semantic ANSI colors for labels, risks, success states, and next actions when the terminal supports color.
- Use ANSI terminal color by default when Codexplain config asks for `defaultColorOutput: ansi`; use HTML span markup only when chat-color output is requested.
- Respect explanationDepth light/standard/deep, architectureDepth overview/system/internals, and abstractionLevel concrete/architecture/strategy.
- Select renderers dynamically: TLDR prose, progress, tables, flow diagrams, pros/cons, formula boxes, status badges, checklists, risk panels, confidence meters, decision matrices, ETA strips, callouts, and next-action footers.
- Treat UX blocks like tool choices: combine the smallest useful set from prompt, response, profile, and optional planner hints.
- Keep commands, paths, risks, test evidence, and exact technical facts intact.
- Do not continue an Ouroboros evolve/ralph lineage if drift is detected. Restart with an explicit project-local Seed.

Strict-output safety:
- Do not rewrite JSON, code blocks, diffs, patches, logs, test output, or commit messages when exact formatting matters.
- If exact formatting matters, return the artifact unchanged.

Terminal UX:
- Use connected box-drawing characters such as ┌ ┬ ┐ │ ├ ┼ ┤ └ ┴ ┘.
- Do not use broken pseudo-borders made from repeated hyphens, equals signs, or Korean long vowel marks.
<!-- CODEXPLAIN:END -->
