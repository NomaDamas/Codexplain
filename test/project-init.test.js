import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { initProject } from "../src/project-init.js";

describe("project init", () => {
  it("creates only project-local Codexplain adapter files", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "codexplain-init-"));
    try {
      const written = await initProject({ cwd });
      assert.deepEqual(written, [
        ".codexplain/post-response.mjs",
        ".codexplain/README.md",
        ".codexplain/config.json",
      ]);

      const adapter = await readFile(join(cwd, ".codexplain/post-response.mjs"), "utf8");
      assert.match(adapter, /CODEXPLAIN_BIN/);
      assert.doesNotMatch(adapter, /global/i);

      const config = JSON.parse(await readFile(join(cwd, ".codexplain/config.json"), "utf8"));
      assert.deepEqual(config.storageCheck.minFree, { value: 5, unit: "gb" });

      const readme = await readFile(join(cwd, ".codexplain/README.md"), "utf8");
      assert.match(readme, /storageCheck\.minFree\.value/);
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("prints original input when the adapter cannot run codexplain", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "codexplain-init-"));
    try {
      await initProject({ cwd });
      const adapterPath = join(cwd, ".codexplain/post-response.mjs");
      const input = '{"prompt":"설명해줘","response":"원문"}';
      const result = spawnSync(process.execPath, [adapterPath], {
        cwd,
        input,
        encoding: "utf8",
        env: { ...process.env, CODEXPLAIN_BIN: "definitely-missing-codexplain" },
      });
      assert.equal(result.status, 0);
      assert.equal(result.stdout, input);
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("does not trim successful adapter output", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "codexplain-init-"));
    try {
      await initProject({ cwd });
      const fakeBin = join(cwd, "fake-codexplain.mjs");
      await writeFile(
        fakeBin,
        "#!/usr/bin/env node\nprocess.stdout.write('  {\"ok\":true}\\n');\n",
        { mode: 0o755 },
      );
      const result = spawnSync(process.execPath, [join(cwd, ".codexplain/post-response.mjs")], {
        cwd,
        input: '{"prompt":"return only valid JSON","response":"  {\\"ok\\":true}\\n"}',
        encoding: "utf8",
        env: { ...process.env, CODEXPLAIN_BIN: fakeBin },
      });
      assert.equal(result.status, 0);
      assert.equal(result.stdout, '  {"ok":true}\n');
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("allows generated project config to override the storage-check threshold", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "codexplain-init-storage-"));
    try {
      await initProject({ cwd });
      await writeFile(
        join(cwd, ".codexplain", "config.json"),
        JSON.stringify({ storageCheck: { minFree: { value: 2, unit: "gb" } } }),
      );

      const result = spawnSync(process.execPath, [join(process.cwd(), "bin/codexplain.js"), "storage-check"], {
        cwd,
        encoding: "utf8",
        env: { ...process.env, CODEXPLAIN_PROJECT_DIR: cwd },
      });

      assert.equal(result.status, 0);
      assert.match(result.stdout, /^min_free_gb=2$/m);
      assert.match(result.stdout, /^effective_min_free_gb=2$/m);
      assert.match(result.stdout, /^message=.*effective_min_free_gb 2$/m);
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });
});
