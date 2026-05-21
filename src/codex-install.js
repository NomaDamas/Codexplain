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
- Do not show internal mode names, prompt-layer labels, or rewrite mechanics.
- Keep commands, paths, risks, test evidence, and exact technical facts intact.
- Respect project-local Codexplain UX preferences when present, including detail
  level, audience, requested explanation style, terminal color theme, and frame
  style.

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
