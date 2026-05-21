import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { initProject } from "../src/project-init.js";

describe("project init", () => {
  it("creates only project-local Claudex adapter files", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "claudex-init-"));
    try {
      const written = await initProject({ cwd });
      assert.deepEqual(written, [".claudex/post-response.mjs", ".claudex/README.md"]);

      const adapter = await readFile(join(cwd, ".claudex/post-response.mjs"), "utf8");
      assert.match(adapter, /CLAUDEX_BIN/);
      assert.doesNotMatch(adapter, /global/i);
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("prints original input when the adapter cannot run claudex", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "claudex-init-"));
    try {
      await initProject({ cwd });
      const adapterPath = join(cwd, ".claudex/post-response.mjs");
      const input = '{"prompt":"설명해줘","response":"원문"}';
      const result = spawnSync(process.execPath, [adapterPath], {
        cwd,
        input,
        encoding: "utf8",
        env: { ...process.env, CLAUDEX_BIN: "definitely-missing-claudex" },
      });
      assert.equal(result.status, 0);
      assert.equal(result.stdout, input);
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("does not trim successful adapter output", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "claudex-init-"));
    try {
      await initProject({ cwd });
      const fakeBin = join(cwd, "fake-claudex.mjs");
      await writeFile(
        fakeBin,
        "#!/usr/bin/env node\nprocess.stdout.write('  {\"ok\":true}\\n');\n",
        { mode: 0o755 },
      );
      const result = spawnSync(process.execPath, [join(cwd, ".claudex/post-response.mjs")], {
        cwd,
        input: '{"prompt":"return only valid JSON","response":"  {\\"ok\\":true}\\n"}',
        encoding: "utf8",
        env: { ...process.env, CLAUDEX_BIN: fakeBin },
      });
      assert.equal(result.status, 0);
      assert.equal(result.stdout, '  {"ok":true}\n');
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });
});
