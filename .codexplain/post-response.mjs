#!/usr/bin/env node
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
