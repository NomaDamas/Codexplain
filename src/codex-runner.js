import { spawn } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { dynamicRewriteConfigured, rewriteAnswerDynamic } from "./dynamic-rewriter.js";
import { loadProjectUxProfile, resolveUxProfile } from "./evolution.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

function run(command, args, options = {}) {
  return new Promise((resolve) => {
    const { input, ...spawnOptions } = options;
    const child = spawn(command, args, {
      ...spawnOptions,
      env: { ...process.env, ...spawnOptions.env },
    });
    let stdout = "";
    let stderr = "";

    child.stdout?.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr?.on("data", (chunk) => {
      stderr += chunk;
    });
    if (input != null) {
      child.stdin?.end(input);
    }
    child.on("error", (error) => resolve({ error, status: 1, stdout, stderr }));
    child.on("close", (status) => resolve({ status: status ?? 1, stdout, stderr }));
  });
}

async function shapeWithRust({ prompt, response, env }) {
  const projectDir = env.CODEXPLAIN_PROJECT_DIR || process.cwd();
  const result = await run(
    process.execPath,
    [join(root, "bin", "codexplain.js"), "shape", "--prompt", prompt || ""],
    {
      cwd: root,
      input: response,
      stdio: ["pipe", "pipe", "pipe"],
      env: { ...env, CODEXPLAIN_PROJECT_DIR: projectDir },
    },
  );
  if (result.status !== 0) {
    if (result.stderr) process.stderr.write(result.stderr);
    return response;
  }
  return result.stdout;
}

export function parseCodexWrapperArgs(args) {
  const list = [...args];
  const promptIndex = list.indexOf("--prompt");
  const prompt = promptIndex >= 0 ? list[promptIndex + 1] ?? "" : "";
  if (promptIndex >= 0) list.splice(promptIndex, 2);
  const localShape = list.includes("--local-shape");
  if (localShape) list.splice(list.indexOf("--local-shape"), 1);
  return { prompt, codexArgs: list, localShape };
}

export async function runCodexWithClaudex({ args, codexArgs: parsedArgs, prompt = "", localShape = false, env = process.env } = {}) {
  const argsToRun = parsedArgs ?? args;
  const codexArgs = argsToRun?.length ? argsToRun : ["exec", prompt].filter(Boolean);
  const result = await run("codex", codexArgs, { stdio: ["inherit", "pipe", "pipe"], env });

  if (result.stderr) process.stderr.write(result.stderr);
  if (result.stdout) {
    if (dynamicRewriteConfigured(env) || localShape || env.CODEXPLAIN_LOCAL_SHAPE || env.CLAUDEX_LOCAL_SHAPE) {
      const effectivePrompt = prompt || codexArgs.join(" ");
      const shaped = dynamicRewriteConfigured(env)
        ? await rewriteAnswerDynamic({
            prompt: effectivePrompt,
            response: result.stdout,
            uxProfile: resolveUxProfile({
              prompt: effectivePrompt,
              profile: await loadProjectUxProfile(),
              env,
            }),
            env,
          })
        : await shapeWithRust({ prompt: effectivePrompt, response: result.stdout, env });
      process.stdout.write(shaped);
    } else {
      process.stdout.write(result.stdout);
    }
  }

  return result.status;
}
