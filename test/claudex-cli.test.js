import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";

function parseKeyValueOutput(stdout) {
  return Object.fromEntries(
    stdout
      .trim()
      .split("\n")
      .map((line) => line.split(/=(.*)/s).slice(0, 2)),
  );
}

function withoutRewriteEnv(extra = {}) {
  const env = { ...process.env };
  delete env.OPENAI_API_KEY;
  delete env.CODEXPLAIN_REWRITE_COMMAND;
  delete env.CODEXPLAIN_DYNAMIC;
  delete env.CODEXPLAIN_LOCAL_SHAPE;
  delete env.CLAUDEX_REWRITE_COMMAND;
  delete env.CLAUDEX_DYNAMIC;
  delete env.CLAUDEX_LOCAL_SHAPE;
  return { ...env, ...extra };
}

describe("claudex cli post-response", () => {
  it("exposes the codexplain command alias", () => {
    const result = spawnSync(process.execPath, [join(process.cwd(), "bin/codexplain.js"), "demo"], {
      encoding: "utf8",
      env: withoutRewriteEnv(),
    });

    assert.equal(result.status, 0);
    assert.match(result.stdout, /JS \/ Node|설명 품질/);
  });

  it("documents storage-check in Rust CLI help with its stable output contract", () => {
    const result = spawnSync(process.execPath, [join(process.cwd(), "bin/codexplain.js"), "--help"], {
      encoding: "utf8",
      env: withoutRewriteEnv(),
    });

    assert.equal(result.status, 0);
    assert.match(result.stdout, /codexplain storage-check \[--min-free-gb 5\] \[--clean\]/);
    assert.match(result.stdout, /Storage-check output contract:/);
    assert.match(result.stdout, /contract=codexplain\.storage-check\.v1/);
    assert.equal(result.stderr, "");
  });

  it("routes storage-check through the Rust core with parseable key-value output", () => {
    const result = spawnSync(
      process.execPath,
      [join(process.cwd(), "bin/codexplain.js"), "storage-check", "--min-free-gb", "0"],
      {
        encoding: "utf8",
        env: withoutRewriteEnv(),
      },
    );
    const fields = parseKeyValueOutput(result.stdout);

    assert.equal(result.status, 0);
    assert.deepEqual(result.stdout.trim().split("\n").map((line) => line.split("=", 1)[0]), [
      "contract",
      "free_gb",
      "min_free_gb",
      "effective_min_free_gb",
      "target_mb",
      "dist_mb",
      "node_modules_mb",
      "result",
      "message",
      "status",
    ]);
    assert.equal(fields.contract, "codexplain.storage-check.v1");
    assert.match(fields.free_gb, /^\d+\.\d{2}$/);
    assert.equal(fields.min_free_gb, "0");
    assert.equal(fields.effective_min_free_gb, "0");
    assert.match(fields.target_mb, /^\d+\.\d$/);
    assert.match(fields.dist_mb, /^\d+\.\d$/);
    assert.match(fields.node_modules_mb, /^\d+\.\d$/);
    assert.equal(fields.result, "pass");
    assert.match(fields.message, /pass: free_gb \d+\.\d{2} meets effective_min_free_gb 0/);
    assert.equal(fields.status, "ok");
    assert.equal(result.stderr, "");
  });

  it("resolves the default, configured, and invalid storage-check thresholds", () => {
    const defaultDir = mkdtempSync(join(tmpdir(), "codexplain-storage-default-"));
    const configuredDir = mkdtempSync(join(tmpdir(), "codexplain-storage-config-"));
    const invalidDir = mkdtempSync(join(tmpdir(), "codexplain-storage-invalid-"));
    try {
      mkdirSync(join(configuredDir, ".codexplain"));
      writeFileSync(
        join(configuredDir, ".codexplain", "config.json"),
        JSON.stringify({ storageCheck: { minFree: { value: 1, unit: "gb" } } }),
      );
      mkdirSync(join(invalidDir, ".codexplain"));
      writeFileSync(
        join(invalidDir, ".codexplain", "config.json"),
        JSON.stringify({ storageCheck: { minFree: { value: "large", unit: "tb" } } }),
      );

      const defaulted = spawnSync(process.execPath, [join(process.cwd(), "bin/codexplain.js"), "storage-check"], {
        encoding: "utf8",
        env: withoutRewriteEnv({ CODEXPLAIN_PROJECT_DIR: defaultDir }),
      });
      const configured = spawnSync(process.execPath, [join(process.cwd(), "bin/codexplain.js"), "storage-check"], {
        encoding: "utf8",
        env: withoutRewriteEnv({ CODEXPLAIN_PROJECT_DIR: configuredDir }),
      });
      const invalid = spawnSync(process.execPath, [join(process.cwd(), "bin/codexplain.js"), "storage-check"], {
        encoding: "utf8",
        env: withoutRewriteEnv({ CODEXPLAIN_PROJECT_DIR: invalidDir }),
      });

      assert.equal(defaulted.status, 0);
      assert.equal(parseKeyValueOutput(defaulted.stdout).min_free_gb, "5");
      assert.equal(parseKeyValueOutput(defaulted.stdout).effective_min_free_gb, "5");
      assert.match(parseKeyValueOutput(defaulted.stdout).message, /effective_min_free_gb 5/);
      assert.equal(configured.status, 0);
      assert.equal(parseKeyValueOutput(configured.stdout).min_free_gb, "1");
      assert.equal(parseKeyValueOutput(configured.stdout).effective_min_free_gb, "1");
      assert.match(parseKeyValueOutput(configured.stdout).message, /effective_min_free_gb 1/);
      assert.equal(invalid.status, 0);
      assert.equal(parseKeyValueOutput(invalid.stdout).min_free_gb, "5");
      assert.equal(parseKeyValueOutput(invalid.stdout).effective_min_free_gb, "5");
      assert.match(parseKeyValueOutput(invalid.stdout).message, /effective_min_free_gb 5/);
    } finally {
      rmSync(defaultDir, { recursive: true, force: true });
      rmSync(configuredDir, { recursive: true, force: true });
      rmSync(invalidDir, { recursive: true, force: true });
    }
  });

  it("reports missing storage directories as zero and keeps cleanup target-only", () => {
    const projectDir = mkdtempSync(join(tmpdir(), "codexplain-storage-missing-"));
    try {
      const result = spawnSync(
        process.execPath,
        [join(process.cwd(), "bin/codexplain.js"), "storage-check", "--min-free-gb", "999999", "--clean"],
        {
          encoding: "utf8",
          env: withoutRewriteEnv({ CODEXPLAIN_PROJECT_DIR: projectDir }),
        },
      );
      const fields = parseKeyValueOutput(result.stdout);

      assert.equal(result.status, 0);
      assert.equal(fields.contract, "codexplain.storage-check.v1");
      assert.match(fields.free_gb, /^\d+\.\d{2}$/);
      assert.equal(fields.min_free_gb, "999999");
      assert.equal(fields.effective_min_free_gb, "999999");
      assert.equal(fields.target_mb, "0.0");
      assert.equal(fields.dist_mb, "0.0");
      assert.equal(fields.node_modules_mb, "0.0");
      assert.equal(fields.result, "fail");
      assert.match(fields.message, /fail: free_gb \d+\.\d{2} is below effective_min_free_gb 999999/);
      assert.equal(fields.status, "low-space");
      assert.equal(fields.cleaned, "target_already_absent");
      assert.equal(result.stderr, "");
    } finally {
      rmSync(projectDir, { recursive: true, force: true });
    }
  });

  it("preserves dist and node_modules when --clean removes build artifacts", () => {
    const projectDir = mkdtempSync(join(tmpdir(), "codexplain-storage-clean-preserve-"));
    try {
      mkdirSync(join(projectDir, "target", "debug"), { recursive: true });
      mkdirSync(join(projectDir, "target", ".fingerprint", "codexplain"), { recursive: true });
      mkdirSync(join(projectDir, "dist", "assets"), { recursive: true });
      mkdirSync(join(projectDir, "node_modules", "@scope", "pkg"), { recursive: true });
      writeFileSync(join(projectDir, "target", "debug", "codexplain"), "cargo debug binary");
      writeFileSync(join(projectDir, "target", ".fingerprint", "codexplain", "dep"), "cargo fingerprint");
      writeFileSync(join(projectDir, "dist", "assets", "bundle.js"), "compiled web bundle");
      writeFileSync(join(projectDir, "node_modules", "@scope", "pkg", "package.json"), "{}");

      const result = spawnSync(
        process.execPath,
        [join(process.cwd(), "bin/codexplain.js"), "storage-check", "--min-free-gb", "999999", "--clean"],
        {
          encoding: "utf8",
          env: withoutRewriteEnv({ CODEXPLAIN_PROJECT_DIR: projectDir }),
        },
      );
      const fields = parseKeyValueOutput(result.stdout);

      assert.equal(result.status, 0);
      assert.equal(fields.status, "low-space");
      assert.equal(fields.cleaned, "target");
      assert.equal(fields.suggested_cleanup, undefined);
      assert.equal(existsSync(join(projectDir, "target")), false);
      assert.equal(existsSync(join(projectDir, "dist", "assets", "bundle.js")), true);
      assert.equal(existsSync(join(projectDir, "node_modules", "@scope", "pkg", "package.json")), true);
      assert.equal(result.stderr, "");
    } finally {
      rmSync(projectDir, { recursive: true, force: true });
    }
  });

  it("uses the configured threshold when --clean is requested without a CLI threshold", () => {
    const projectDir = mkdtempSync(join(tmpdir(), "codexplain-storage-clean-config-"));
    try {
      mkdirSync(join(projectDir, ".codexplain"));
      mkdirSync(join(projectDir, "target"));
      mkdirSync(join(projectDir, "dist"));
      mkdirSync(join(projectDir, "node_modules"));
      writeFileSync(join(projectDir, "target", "artifact.tmp"), "cargo artifact");
      writeFileSync(join(projectDir, "dist", "bundle.js"), "regenerated bundle");
      writeFileSync(join(projectDir, "node_modules", "package.txt"), "installed dependency");
      writeFileSync(
        join(projectDir, ".codexplain", "config.json"),
        JSON.stringify({ storageCheck: { minFree: { value: 999999, unit: "gb" } } }),
      );

      const result = spawnSync(process.execPath, [join(process.cwd(), "bin/codexplain.js"), "storage-check", "--clean"], {
        encoding: "utf8",
        env: withoutRewriteEnv({ CODEXPLAIN_PROJECT_DIR: projectDir }),
      });
      const fields = parseKeyValueOutput(result.stdout);

      assert.equal(result.status, 0);
      assert.equal(fields.min_free_gb, "999999");
      assert.equal(fields.effective_min_free_gb, "999999");
      assert.equal(fields.status, "low-space");
      assert.equal(fields.cleaned, "target");
      assert.equal(existsSync(join(projectDir, "target")), false);
      assert.equal(existsSync(join(projectDir, "dist", "bundle.js")), true);
      assert.equal(existsSync(join(projectDir, "node_modules", "package.txt")), true);
      assert.equal(result.stderr, "");
    } finally {
      rmSync(projectDir, { recursive: true, force: true });
    }
  });

  it("does not clean target when free space meets the effective threshold", () => {
    const projectDir = mkdtempSync(join(tmpdir(), "codexplain-storage-clean-gate-"));
    try {
      mkdirSync(join(projectDir, "target"));
      writeFileSync(join(projectDir, "target", "artifact.tmp"), "cargo artifact");

      const result = spawnSync(
        process.execPath,
        [join(process.cwd(), "bin/codexplain.js"), "storage-check", "--min-free-gb", "0", "--clean"],
        {
          encoding: "utf8",
          env: withoutRewriteEnv({ CODEXPLAIN_PROJECT_DIR: projectDir }),
        },
      );
      const fields = parseKeyValueOutput(result.stdout);

      assert.equal(result.status, 0);
      assert.equal(fields.result, "pass");
      assert.equal(fields.status, "ok");
      assert.equal(fields.cleaned, undefined);
      assert.equal(existsSync(join(projectDir, "target", "artifact.tmp")), true);
      assert.equal(result.stderr, "");
    } finally {
      rmSync(projectDir, { recursive: true, force: true });
    }
  });

  it("keeps storage-check available from the legacy claudex entrypoint", () => {
    const help = spawnSync(process.execPath, [join(process.cwd(), "bin/claudex.js"), "--help"], {
      encoding: "utf8",
      env: withoutRewriteEnv(),
    });
    const result = spawnSync(
      process.execPath,
      [join(process.cwd(), "bin/claudex.js"), "storage-check", "--min-free-gb", "0"],
      {
        encoding: "utf8",
        env: withoutRewriteEnv(),
      },
    );

    assert.equal(help.status, 0);
    assert.match(help.stdout, /claudex storage-check \[--min-free-gb 5\] \[--clean\]/);
    assert.equal(result.status, 0);
    assert.match(result.stdout, /contract=codexplain\.storage-check\.v1/);
    assert.match(result.stdout, /status=ok/);
  });

  it("routes legacy post-response through the Rust core when rewrite is not configured", () => {
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
    assert.match(result.stdout, /요약하면/);
    assert.match(result.stdout, /작업이 완료됐습니다/);
    assert.match(result.stdout, /`npm test`/);
    assert.equal(result.stderr, "");
  });

  it("shapes codexplain post-response through the Rust core", () => {
    const result = spawnSync(
      process.execPath,
      [join(process.cwd(), "bin/codexplain.js"), "post-response", "--prompt", "설명해줘", "--width", "80"],
      {
        input: JSON.stringify({
          prompt: "설명해줘",
          response: "작업이 완료됐습니다. 검증은 `npm test`로 했습니다.",
        }),
        encoding: "utf8",
        env: withoutRewriteEnv(),
      },
    );

    assert.equal(result.status, 0);
    assert.match(result.stdout, /요약하면/);
    assert.match(result.stdout, /`npm test`/);
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

  it("prints a project-local profile with abstraction controls", () => {
    const cwd = mkdtempSync(join(tmpdir(), "codexplain-abstraction-"));
    try {
      const result = spawnSync(
        process.execPath,
        [
          join(process.cwd(), "bin/codexplain.js"),
          "profile",
          "--abstraction-range",
          "implementation:strategy",
          "--layers",
          "tldr,summary,architecture,evidence",
        ],
        {
          cwd,
          encoding: "utf8",
          env: withoutRewriteEnv(),
        },
      );

      assert.equal(result.status, 0);
      assert.match(result.stdout, /"min": "implementation"/);
      assert.match(result.stdout, /"max": "strategy"/);
      assert.match(result.stdout, /"architecture"/);
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
