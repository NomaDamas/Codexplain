import { spawn } from "node:child_process";
import { dynamicRewriteConfigured, rewriteAnswerDynamic } from "./dynamic-rewriter.js";
import { loadProjectUxProfile, resolveUxProfile } from "./evolution.js";

function run(command, args, options = {}) {
  return new Promise((resolve) => {
    const child = spawn(command, args, {
      ...options,
      env: { ...process.env, ...options.env },
    });
    let stdout = "";
    let stderr = "";

    child.stdout?.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr?.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", (error) => resolve({ error, status: 1, stdout, stderr }));
    child.on("close", (status) => resolve({ status: status ?? 1, stdout, stderr }));
  });
}

export function parseCodexWrapperArgs(args) {
  const list = [...args];
  const promptIndex = list.indexOf("--prompt");
  const prompt = promptIndex >= 0 ? list[promptIndex + 1] ?? "" : "";
  if (promptIndex >= 0) list.splice(promptIndex, 2);
  return { prompt, codexArgs: list };
}

export async function runCodexWithClaudex({ args, prompt = "", env = process.env } = {}) {
  const codexArgs = args?.length ? args : ["exec", prompt].filter(Boolean);
  const result = await run("codex", codexArgs, { stdio: ["inherit", "pipe", "pipe"], env });

  if (result.stderr) process.stderr.write(result.stderr);
  if (result.stdout) {
    if (dynamicRewriteConfigured(env)) {
      const storedProfile = await loadProjectUxProfile();
      const uxProfile = resolveUxProfile({ prompt: prompt || codexArgs.join(" "), profile: storedProfile, env });
      const shaped = await rewriteAnswerDynamic({
        prompt: prompt || codexArgs.join(" "),
        response: result.stdout,
        uxProfile,
        env,
      });
      process.stdout.write(shaped);
    } else {
      process.stdout.write(result.stdout);
    }
  }

  return result.status;
}
