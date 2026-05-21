import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";
import { describe, it } from "node:test";
import { parseCodexWrapperArgs } from "../src/codex-runner.js";

function withoutRewriteEnv(extra = {}) {
  const env = { ...process.env };
  delete env.OPENAI_API_KEY;
  delete env.CLAUDEX_REWRITE_COMMAND;
  delete env.CLAUDEX_DYNAMIC;
  return { ...env, ...extra };
}

async function withFakeCodex(callback) {
  const cwd = await mkdtemp(join(tmpdir(), "claudex-codex-"));
  try {
    const codexPath = join(cwd, "codex");
    await writeFile(
      codexPath,
      "#!/usr/bin/env node\nprocess.stdout.write(process.env.FAKE_CODEX_OUTPUT || '');\n",
      { mode: 0o755 },
    );
    return await callback(cwd);
  } finally {
    await rm(cwd, { recursive: true, force: true });
  }
}

describe("codex runner", () => {
  it("extracts Claudex prompt without swallowing Codex args", () => {
    assert.deepEqual(parseCodexWrapperArgs(["--prompt", "흐름도로 설명해줘", "exec", "작업해"]), {
      prompt: "흐름도로 설명해줘",
      codexArgs: ["exec", "작업해"],
    });
  });

  it("leaves captured Codex stdout unchanged when rewrite is not configured", async () => {
    await withFakeCodex(async (cwd) => {
      const original = "작업이 완료됐습니다. 검증은 `npm test`로 했습니다.";
      const result = spawnSync(
        process.execPath,
        [join(process.cwd(), "bin/claudex-codex.js"), "--prompt", "설명해줘", "exec", "상태 확인"],
        {
          encoding: "utf8",
          env: withoutRewriteEnv({
            FAKE_CODEX_OUTPUT: original,
            PATH: `${cwd}${delimiter}${process.env.PATH ?? ""}`,
          }),
        },
      );

      assert.equal(result.status, 0);
      assert.equal(result.stdout, original);
      assert.equal(result.stderr, "");
    });
  });

  it("invokes dynamic rewriting for captured Codex stdout when rewrite is configured", async () => {
    await withFakeCodex(async (cwd) => {
      const result = spawnSync(
        process.execPath,
        [join(process.cwd(), "bin/claudex-codex.js"), "--prompt", "설명해줘", "exec", "상태 확인"],
        {
          encoding: "utf8",
          env: withoutRewriteEnv({
            CLAUDEX_DYNAMIC: "1",
            CLAUDEX_REWRITE_COMMAND: `${process.execPath} -e "process.stdin.resume(); process.stdin.on('end',()=>process.stdout.write('TLDR: 작업 완료'))"`,
            FAKE_CODEX_OUTPUT: "작업이 완료됐습니다.",
            PATH: `${cwd}${delimiter}${process.env.PATH ?? ""}`,
          }),
        },
      );

      assert.equal(result.status, 0);
      assert.equal(result.stdout, "TLDR: 작업 완료");
      assert.equal(result.stderr, "");
    });
  });
});
