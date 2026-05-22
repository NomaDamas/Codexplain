import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const POST_RESPONSE_ADAPTER = `#!/usr/bin/env node
import { spawnSync } from "node:child_process";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(Buffer.from(chunk));
const input = Buffer.concat(chunks).toString("utf8");

if (!input.trim()) process.exit(0);

const bin = process.env.CODEXPLAIN_BIN || process.env.CLAUDEX_BIN || "codexplain";
const result = spawnSync(bin, ["post-response"], {
  input,
  encoding: "utf8",
  env: process.env,
});

if (result.error || result.status || result.signal) {
  process.stdout.write(input);
} else {
  process.stdout.write(result.stdout);
}
`;

const README = `# Codexplain Local Adapter

This directory is project-local.

Use this adapter when a host can pipe a completed answer into a post-response
command. The command is safe-by-default: if dynamic rewriting fails, the
original answer is printed unchanged by the adapter.

\`\`\`bash
node .codexplain/post-response.mjs
\`\`\`

To force deterministic local shaping without a dynamic provider:

\`\`\`bash
CODEXPLAIN_LOCAL_SHAPE=1 node .codexplain/post-response.mjs
\`\`\`

Input can be JSON:

\`\`\`json
{"prompt":"흐름도로 설명해줘","response":"구현은 완료됐습니다."}
\`\`\`

Or plain text with \`CODEXPLAIN_PROMPT\` set.

Dynamic rewriting is enabled by provider configuration:

\`\`\`bash
export OPENAI_API_KEY=...
export CODEXPLAIN_DYNAMIC=1
\`\`\`

For a local or custom model command, set \`CODEXPLAIN_REWRITE_COMMAND\`. The command
receives JSON on stdin and must print the rewritten answer on stdout.

Adaptive explanation preferences, including terminal color theme and ASCII frame
style, are stored project-locally in
\`.codexplain/ux-profile.json\` when users run \`codexplain profile\`,
\`codexplain feedback\`, or \`codexplain rlhf\`.

Renderer selection is compositional. If the prompt asks for architecture,
tradeoff, and formula views together, Codexplain should combine table/flow
architecture context, pros/cons comparison, and a formula box instead of
dropping later requested formats. Progress prompts should render a short status
line, progress bar, and checkpoint table so users can scan current state before
reading details. Rich work-state prompts may add status badges, checklists, risk
panels, confidence meters, diff summary cards, decision matrices, ETA strips,
attention callouts, and next-action footers. Select these blocks like
tool-calling choices from prompt and response signals instead of showing every
component by default. The selector supports explicit rules, numeric score
thresholds, and optional planner hints through CODEXPLAIN_UX_PLAN or
CODEXPLAIN_UX_PLANNER_COMMAND.

Storage safety preferences are stored in \`.codexplain/config.json\`. Adjust
\`storageCheck.minFree.value\` to override the project-local threshold used by
\`codexplain storage-check\`; the \`--min-free-gb\` CLI flag still wins for one
command invocation.

Ouroboros automation must stay project-local. If an evolve or ralph run starts
emitting acceptance criteria from another project, cancel it and restart with an
explicit Seed for this repository before allowing further file mutations.
`;

const CONFIG = `{
  "schemaVersion": 1,
  "storageCheck": {
    "minFree": {
      "value": 5,
      "unit": "gb"
    }
  }
}
`;

export function localAdapterFiles() {
  return {
    ".codexplain/post-response.mjs": POST_RESPONSE_ADAPTER,
    ".codexplain/README.md": README,
    ".codexplain/config.json": CONFIG,
  };
}

export async function initProject({ cwd = process.cwd(), force = false } = {}) {
  const files = localAdapterFiles();
  await mkdir(join(cwd, ".codexplain"), { recursive: true });

  const written = [];
  for (const [relativePath, content] of Object.entries(files)) {
    const target = join(cwd, relativePath);
    await writeFile(target, content, { flag: force ? "w" : "wx", mode: relativePath.endsWith(".mjs") ? 0o755 : 0o644 });
    written.push(relativePath);
  }

  return written;
}
