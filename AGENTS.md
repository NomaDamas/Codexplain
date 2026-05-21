

<!-- CODEXPLAIN:START -->
# Codexplain Response UX

For this repository only, shape user-facing answers with a clear, Claude-like
reading experience while preserving Codex's coding precision.

Default answer style:
- Start with the outcome or current state, not implementation detail.
- Use concise Korean first when the user writes Korean.
- Prefer short paragraphs over code-heavy explanations.
- Use connected Unicode boxes or tables when structure helps scanning.
- When the active terminal supports ANSI color and the project theme is not
  none, color-highlight important labels such as TLDR, 핵심, 장점, 단점, 위험,
  and next action. Never rely on color as the only carrier of meaning.
- In dense tables, include row dividers so each row boundary is visible.
- Choose the explanation renderer dynamically: TLDR prose for status/summary,
  tables for comparison, pros/cons panels for tradeoffs, numbered lists for
  ordered steps, flow diagrams for process, and formula boxes for math-like
  decision rules.
- Do not show internal mode names, prompt-layer labels, or rewrite mechanics.
- Keep commands, paths, risks, test evidence, and exact technical facts intact.
- Respect project-local Codexplain UX preferences when present, including detail
  level, audience, requested explanation style, terminal color theme, and frame
  style.
- Respect abstraction range preferences such as concrete..implementation,
  implementation..architecture, or architecture..strategy.

Strict-output safety:
- Do not rewrite JSON, code blocks, diffs, patches, logs, test output, or commit
  messages when the user asks for an exact artifact.
- If exact formatting matters, return the artifact unchanged.

Terminal UX:
- Use connected box-drawing characters such as ┌ ┬ ┐ │ ├ ┼ ┤ └ ┴ ┘.
- Do not use broken pseudo-borders made from repeated hyphens, equals signs, or
  Korean long vowel marks.
<!-- CODEXPLAIN:END -->
