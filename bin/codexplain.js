#!/usr/bin/env node
import { existsSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const commandName = args[0] ?? "demo";
const projectDir = process.env.CODEXPLAIN_PROJECT_DIR || process.cwd();
const jsCompatibilityCommands = new Set([
  "init",
  "install-codex",
  "codex",
  "feedback",
  "rlhf",
  "guide",
]);

if (jsCompatibilityCommands.has(commandName)) {
  const result = spawnSync(process.execPath, [join(root, "bin", "claudex.js"), ...args], {
    cwd: process.cwd(),
    stdio: "inherit",
    env: { ...process.env, CODEXPLAIN_PROJECT_DIR: projectDir },
  });
  process.exitCode = result.status ?? (result.error ? 1 : 0);
  if (result.error) console.error(result.error.message);
  process.exit();
}

const candidates = [
  join(root, "target", "release", "codexplain"),
  join(root, "target", "debug", "codexplain"),
];

const source = join(root, "rust", "codexplain.rs");
const sourceMtime = statSync(source).mtimeMs;
const binary = candidates.find((path) => {
  if (!existsSync(path)) return false;
  return statSync(path).mtimeMs >= sourceMtime;
});
const command = binary ?? "cargo";
const commandArgs = binary ? args : ["run", "--quiet", "--bin", "codexplain", "--", ...args];
const result = spawnSync(command, commandArgs, {
  cwd: binary ? process.cwd() : root,
  stdio: "inherit",
  env: { ...process.env, CODEXPLAIN_PROJECT_DIR: projectDir },
});

if (result.error) {
  console.error(result.error.message);
  process.exitCode = 1;
} else {
  process.exitCode = result.status ?? 0;
}
