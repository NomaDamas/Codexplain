import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { CODEX_GUIDANCE, installCodexProject } from "../src/codex-install.js";

describe("codex install", () => {
  it("installs project-local Codex guidance and adapter files", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "claudex-codex-"));
    try {
      const written = await installCodexProject({ cwd });
      assert.deepEqual(written, [".claudex/post-response.mjs", ".claudex/README.md", "AGENTS.md"]);

      const agents = await readFile(join(cwd, "AGENTS.md"), "utf8");
      assert.match(agents, /Claudex Response UX/);
      assert.match(agents, /Do not show internal mode names/);

      const adapter = await readFile(join(cwd, ".claudex/post-response.mjs"), "utf8");
      assert.match(adapter, /claudex/);
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("replaces an existing Claudex block without deleting other AGENTS content", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "claudex-agents-"));
    try {
      await writeFile(join(cwd, "AGENTS.md"), `Project rules\n\n${CODEX_GUIDANCE}\n\nKeep this line.\n`);
      await installCodexProject({ cwd });

      const agents = await readFile(join(cwd, "AGENTS.md"), "utf8");
      assert.match(agents, /Project rules/);
      assert.match(agents, /Keep this line/);
      assert.equal((agents.match(/Claudex Response UX/g) ?? []).length, 1);
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });
});
