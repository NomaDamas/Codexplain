import { access, appendFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { constants } from "node:fs";
import { join } from "node:path";
import { initProject } from "./project-init.js";

const START = "<!-- CODEXPLAIN:START -->";
const END = "<!-- CODEXPLAIN:END -->";
const LEGACY_START = "<!-- CLAUDEX:START -->";
const LEGACY_END = "<!-- CLAUDEX:END -->";

export const CODEX_GUIDANCE = `${START}
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
- For progress updates, place a concise status label above the progress bar,
  then show a compact checkpoint table with current state, percentage, and next
  action.
- Use richer terminal UX components when they improve scanning: status badges,
  checklists, risk panels, confidence meters, diff summary cards, decision
  matrices, ETA strips, attention callouts, and next-action footers.
- Choose the explanation renderer dynamically: TLDR prose for status/summary,
  progress reports with a short status line and progress bar for work tracking,
  tables for comparison, pros/cons panels for tradeoffs, numbered lists for
  ordered steps, flow diagrams for process, formula boxes for math-like
  decision rules, and compact status UX cards for work-state answers.
- When the user asks for multiple useful formats, combine them instead of
  forcing a single renderer. For example, architecture + tradeoff + formula
  requests should produce table/flow architecture context, pros/cons comparison,
  and a formula box when terminal space allows. Treat these blocks like
  tool-calling choices: select the smallest useful set from prompt and response
  signals, not every component by default.
- Do not show internal mode names, prompt-layer labels, or rewrite mechanics.
- Keep commands, paths, risks, test evidence, and exact technical facts intact.
- Respect project-local Codexplain UX preferences when present, including detail
  level, audience, requested explanation style, terminal color theme, and frame
  style.
- Respect project-local storage safety preferences in
  \`.codexplain/config.json\`, including \`storageCheck.minFree.value\`, when
  invoking \`codexplain storage-check\`.
- Do not continue an Ouroboros evolve/ralph lineage if drift is detected.
  Restart with an explicit project-local Seed and verify the first events still
  match this repository before letting automation mutate files.
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
${END}`;

async function exists(path) {
  try {
    await access(path, constants.F_OK);
    return true;
  } catch {
    return false;
  }
}

function replaceBlock(text, block) {
  const startIndex = text.indexOf(START) >= 0 ? text.indexOf(START) : text.indexOf(LEGACY_START);
  const endMarker = text.indexOf(END) >= 0 ? END : LEGACY_END;
  const endIndex = text.indexOf(endMarker);
  if (startIndex >= 0 && endIndex > startIndex) {
    return `${text.slice(0, startIndex).trimEnd()}\n\n${block}\n${text.slice(endIndex + endMarker.length).trimStart()}`;
  }
  return `${text.trimEnd()}\n\n${block}\n`;
}

export async function installCodexProject({ cwd = process.cwd(), force = false } = {}) {
  const written = await initProject({ cwd, force });
  const agentsPath = join(cwd, "AGENTS.md");

  if (await exists(agentsPath)) {
    const current = await readFile(agentsPath, "utf8");
    if (current.includes(START)) {
      await writeFile(agentsPath, replaceBlock(current, CODEX_GUIDANCE));
    } else {
      await appendFile(agentsPath, `\n\n${CODEX_GUIDANCE}\n`);
    }
  } else {
    await writeFile(agentsPath, `${CODEX_GUIDANCE}\n`);
  }

  await mkdir(join(cwd, ".codexplain"), { recursive: true });
  return [...written, "AGENTS.md"];
}
