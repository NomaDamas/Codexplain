import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";

function withoutRewriteEnv(extra = {}) {
  const env = { ...process.env };
  delete env.OPENAI_API_KEY;
  delete env.CODEXPLAIN_REWRITE_COMMAND;
  delete env.CODEXPLAIN_DYNAMIC;
  delete env.CODEXPLAIN_LOCAL_SHAPE;
  delete env.CLAUDEX_REWRITE_COMMAND;
  delete env.CLAUDEX_DYNAMIC;
  return { ...env, ...extra };
}

describe("claudex cli post-response", () => {
  it("exposes the codexplain command alias", () => {
    const result = spawnSync(process.execPath, [join(process.cwd(), "bin/codexplain.js"), "demo"], {
      encoding: "utf8",
      env: withoutRewriteEnv(),
    });

    assert.equal(result.status, 0);
    assert.match(result.stdout, /TLDR|요약|핵심/);
  });

  it("prints the original response unchanged when rewrite is not configured", () => {
    const original = "작업이 완료됐습니다. 검증은 `npm test`로 했습니다.";
    const result = spawnSync(
      process.execPath,
      [join(process.cwd(), "bin/claudex.js"), "post-response", "--prompt", "설명해줘"],
      {
        input: JSON.stringify({ prompt: "설명해줘", response: original }),
        encoding: "utf8",
        env: withoutRewriteEnv(),
      },
    );

    assert.equal(result.status, 0);
    assert.equal(result.stdout, original);
    assert.equal(result.stderr, "");
  });

  it("invokes dynamic rewriting for post-response input when rewrite is configured", () => {
    const result = spawnSync(
      process.execPath,
      [join(process.cwd(), "bin/claudex.js"), "post-response", "--prompt", "설명해줘"],
      {
        input: JSON.stringify({ prompt: "설명해줘", response: "작업이 완료됐습니다." }),
        encoding: "utf8",
        env: withoutRewriteEnv({
          CODEXPLAIN_DYNAMIC: "1",
          CODEXPLAIN_REWRITE_COMMAND: `${process.execPath} -e "process.stdin.resume(); process.stdin.on('end',()=>process.stdout.write('TLDR: 작업 완료'))"`,
        }),
      },
    );

    assert.equal(result.status, 0);
    assert.equal(result.stdout, "TLDR: 작업 완료");
    assert.equal(result.stderr, "");
  });

  it("can locally shape post-response output when explicitly requested", () => {
    const original = "작업이 완료됐습니다. 검증은 `npm test`로 했습니다.";
    const result = spawnSync(
      process.execPath,
      [join(process.cwd(), "bin/claudex.js"), "post-response", "--prompt", "쉽게 설명해줘", "--local-shape"],
      {
        input: JSON.stringify({ prompt: "쉽게 설명해줘", response: original }),
        encoding: "utf8",
        env: withoutRewriteEnv(),
      },
    );

    assert.equal(result.status, 0);
    assert.match(result.stdout, /요약하면|핵심/);
    assert.match(result.stdout, /`npm test`/);
    assert.equal(result.stderr, "");
  });

  it("records feedback as a project-local ux profile", () => {
    const cwd = mkdtempSync(join(tmpdir(), "codexplain-cli-"));
    try {
      const result = spawnSync(
        process.execPath,
        [join(process.cwd(), "bin/claudex.js"), "feedback", "--rating", "2", "--comment", "너무 어렵고 설명이 부족해"],
        {
          cwd,
          encoding: "utf8",
          env: withoutRewriteEnv(),
        },
      );

      assert.equal(result.status, 0);
      assert.match(result.stdout, /detail=deep/);
      assert.match(result.stdout, /style=plain/);
    } finally {
      rmSync(cwd, { recursive: true, force: true });
    }
  });

  it("prints a project-local profile with a theme", () => {
    const cwd = mkdtempSync(join(tmpdir(), "codexplain-theme-"));
    try {
      const result = spawnSync(
        process.execPath,
        [join(process.cwd(), "bin/claudex.js"), "profile", "--theme", "ocean"],
        {
          cwd,
          encoding: "utf8",
          env: withoutRewriteEnv(),
        },
      );

      assert.equal(result.status, 0);
      assert.match(result.stdout, /"theme": "ocean"/);
    } finally {
      rmSync(cwd, { recursive: true, force: true });
    }
  });

  it("prints a project-local profile with an ASCII frame", () => {
    const cwd = mkdtempSync(join(tmpdir(), "codexplain-frame-"));
    try {
      const result = spawnSync(
        process.execPath,
        [join(process.cwd(), "bin/claudex.js"), "profile", "--frame", "ascii"],
        {
          cwd,
          encoding: "utf8",
          env: withoutRewriteEnv(),
        },
      );

      assert.equal(result.status, 0);
      assert.match(result.stdout, /"frame": "ascii"/);
    } finally {
      rmSync(cwd, { recursive: true, force: true });
    }
  });

  it("records RLHF-lite reward feedback", () => {
    const cwd = mkdtempSync(join(tmpdir(), "codexplain-rlhf-"));
    try {
      const result = spawnSync(
        process.execPath,
        [join(process.cwd(), "bin/codexplain.js"), "rlhf", "--rating", "5", "--comment", "이 설명 방식이 좋아"],
        {
          cwd,
          encoding: "utf8",
          env: withoutRewriteEnv(),
        },
      );

      assert.equal(result.status, 0);
      assert.match(result.stdout, /Preference reward: positive/);
      assert.match(result.stdout, /rewardScore: 2/);
    } finally {
      rmSync(cwd, { recursive: true, force: true });
    }
  });
});
