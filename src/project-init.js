import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const POST_RESPONSE_ADAPTER = `#!/usr/bin/env node
import { spawnSync } from "node:child_process";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(Buffer.from(chunk));
const input = Buffer.concat(chunks).toString("utf8");

if (!input.trim()) process.exit(0);

const bin = process.env.CLAUDEX_BIN || "claudex";
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

const README = `# Claudex Local Adapter

This directory is project-local.

Use this adapter when a host can pipe a completed answer into a post-response
command. The command is safe-by-default: if dynamic rewriting fails, the
original answer is printed unchanged by the adapter.

\`\`\`bash
node .claudex/post-response.mjs
\`\`\`

To force deterministic local shaping without a dynamic provider:

\`\`\`bash
CLAUDEX_LOCAL_SHAPE=1 node .claudex/post-response.mjs
\`\`\`

Input can be JSON:

\`\`\`json
{"prompt":"흐름도로 설명해줘","response":"구현은 완료됐습니다."}
\`\`\`

Or plain text with \`CLAUDEX_PROMPT\` set.

Dynamic rewriting is enabled by provider configuration:

\`\`\`bash
export OPENAI_API_KEY=...
export CLAUDEX_DYNAMIC=1
\`\`\`

For a local or custom model command, set \`CLAUDEX_REWRITE_COMMAND\`. The command
receives JSON on stdin and must print the rewritten answer on stdout.

Adaptive explanation preferences, including terminal color theme and ASCII frame
style, are stored project-locally in
\`.claudex/ux-profile.json\` when users run \`claudex profile\` or
\`claudex feedback\`.
`;

export function localAdapterFiles() {
  return {
    ".claudex/post-response.mjs": POST_RESPONSE_ADAPTER,
    ".claudex/README.md": README,
  };
}

export async function initProject({ cwd = process.cwd(), force = false } = {}) {
  const files = localAdapterFiles();
  await mkdir(join(cwd, ".claudex"), { recursive: true });

  const written = [];
  for (const [relativePath, content] of Object.entries(files)) {
    const target = join(cwd, relativePath);
    await writeFile(target, content, { flag: force ? "w" : "wx", mode: relativePath.endsWith(".mjs") ? 0o755 : 0o644 });
    written.push(relativePath);
  }

  return written;
}
