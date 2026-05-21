import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { CODEX_GUIDANCE, installCodexProject } from "../src/codex-install.js";

describe("codex install", () => {
  it("installs project-local Codex guidance and adapter files", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "codexplain-codex-"));
    try {
      const written = await installCodexProject({ cwd });
      assert.deepEqual(written, [
        ".codexplain/post-response.mjs",
        ".codexplain/README.md",
        ".codexplain/config.json",
        "AGENTS.md",
      ]);

      const agents = await readFile(join(cwd, "AGENTS.md"), "utf8");
      assert.match(agents, /Codexplain Response UX/);
      assert.match(agents, /Do not show internal mode names/);
      assert.match(agents, /storageCheck\.minFree\.value/);

      const adapter = await readFile(join(cwd, ".codexplain/post-response.mjs"), "utf8");
      assert.match(adapter, /codexplain/);

      const config = JSON.parse(await readFile(join(cwd, ".codexplain/config.json"), "utf8"));
      assert.deepEqual(config.storageCheck.minFree, { value: 5, unit: "gb" });
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("replaces an existing Codexplain block without deleting other AGENTS content", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "codexplain-agents-"));
    try {
      await writeFile(join(cwd, "AGENTS.md"), `Project rules\n\n${CODEX_GUIDANCE}\n\nKeep this line.\n`);
      await installCodexProject({ cwd });

      const agents = await readFile(join(cwd, "AGENTS.md"), "utf8");
      assert.match(agents, /Project rules/);
      assert.match(agents, /Keep this line/);
      assert.equal((agents.match(/Codexplain Response UX/g) ?? []).length, 1);
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });
});
