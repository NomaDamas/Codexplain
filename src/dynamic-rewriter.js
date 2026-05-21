import { spawn } from "node:child_process";
import { shapeAnswer } from "./shaper.js";
import { shouldBackOff } from "./policy.js";
import { DEFAULT_UX_PROFILE, buildUxContract, resolveUxProfile } from "./evolution.js";

const DEFAULT_TIMEOUT_MS = 8000;
const DEFAULT_MAX_INPUT_CHARS = 24000;
const DEFAULT_MODEL = "gpt-5.2";

function envFlag(value) {
  const text = String(value ?? "").trim().toLowerCase();
  if (!text) return "auto";
  if (["0", "false", "off", "no"].includes(text)) return "off";
  if (["require", "required", "strict"].includes(text)) return "require";
  return "auto";
}

function clampInput(text, maxChars) {
  const value = String(text ?? "");
  if (value.length <= maxChars) return value;
  return `${value.slice(0, maxChars)}\n\n[Claudex: input truncated for rewrite safety]`;
}

function buildRewritePrompt({ prompt, response, uxProfile = DEFAULT_UX_PROFILE }) {
  const profile = resolveUxProfile({ prompt, profile: uxProfile });
  return [
    "You are Claudex, a post-response readability layer for Codex.",
    "",
    "Rewrite the completed answer for lower cognitive load without changing facts.",
    "",
    buildUxContract(profile),
    "",
    "Hard constraints:",
    "- Preserve commands, file paths, code identifiers, test evidence, risks, dates, and claims exactly.",
    "- Do not add new technical facts, fake verification, or hide uncertainty.",
    "- Do not mention Claudex, hooks, policy, provider, or this rewrite instruction.",
    "- If the user wrote Korean, answer Korean-first with short natural sentences.",
    "- Prefer TLDR / current state / evidence / next step over long process narration.",
    "- Put TLDR before details unless the user requested an exact artifact.",
    "- Use connected Unicode tables or compact diagrams only when they reduce scan cost.",
    "- If showing a table and a flow together, place them side by side only when the terminal width supports it; stack them otherwise.",
    "- Avoid broken ASCII borders, repeated hyphen walls, and decorative filler.",
    "- Return only the rewritten answer.",
    "",
    "Design basis:",
    "- Chunking: group related details so working memory does not have to hold everything at once.",
    "- Signaling: put the most important state first, then evidence.",
    "- Coherence: remove redundant or distracting meta-explanation.",
    "- Spatial integration: when using a diagram/table, keep labels and explanations together.",
    "- Faithfulness: a clear explanation that changes the facts is a failed explanation.",
    "",
    "User prompt:",
    prompt || "(unknown)",
    "",
    "Completed Codex answer:",
    response || "(empty)",
  ].join("\n");
}

function runCommandProvider({ command, payload, timeoutMs }) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, {
      shell: true,
      stdio: ["pipe", "pipe", "pipe"],
      env: process.env,
    });
    let stdout = "";
    let stderr = "";
    const timer = setTimeout(() => {
      child.kill("SIGTERM");
      reject(new Error(`rewrite command timed out after ${timeoutMs}ms`));
    }, timeoutMs);

    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.on("close", (status, signal) => {
      clearTimeout(timer);
      if (status || signal) {
        reject(new Error(stderr.trim() || `rewrite command exited with ${status ?? signal}`));
        return;
      }
      resolve(stdout);
    });
    child.stdin.end(JSON.stringify(payload));
  });
}

function extractOutputText(data) {
  if (typeof data?.output_text === "string") return data.output_text;
  const parts = [];
  for (const item of data?.output ?? []) {
    for (const content of item?.content ?? []) {
      if (typeof content?.text === "string") parts.push(content.text);
    }
  }
  return parts.join("\n");
}

async function runOpenAIProvider({ apiKey, model, input, timeoutMs }) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch("https://api.openai.com/v1/responses", {
      method: "POST",
      signal: controller.signal,
      headers: {
        "content-type": "application/json",
        authorization: `Bearer ${apiKey}`,
      },
      body: JSON.stringify({
        model,
        input,
        store: false,
        text: {
          format: { type: "text" },
          verbosity: "concise",
        },
      }),
    });
    if (!response.ok) {
      const errorText = await response.text().catch(() => "");
      throw new Error(`OpenAI rewrite failed: ${response.status} ${errorText}`.trim());
    }
    return extractOutputText(await response.json());
  } finally {
    clearTimeout(timer);
  }
}

function isUsableRewrite(value) {
  const text = String(value ?? "").trim();
  return text.length > 0;
}

function unique(values) {
  return [...new Set(values.filter(Boolean))];
}

export function extractProtectedElements(text) {
  const value = String(text ?? "");
  const matches = [
    ...value.matchAll(/`([^`\n]+)`/g),
    ...value.matchAll(/https?:\/\/[^\s)]+/g),
    ...value.matchAll(/(?:^|[\s(["'])((?:\.{1,2}|~)?\/[A-Za-z0-9._~/-]+|[A-Za-z0-9._-]+\/[A-Za-z0-9._~/-]+)/g),
  ];
  return unique(
    matches
      .map((match) => match[1] ?? match[0])
      .map((item) => item.trim().replace(/[.,;:!?]+$/u, "")),
  );
}

export function preservesProtectedElements({ original, rewritten }) {
  const output = String(rewritten ?? "");
  return extractProtectedElements(original).every((item) => output.includes(item));
}

export function dynamicRewriteConfigured(env = process.env) {
  return Boolean(env.CLAUDEX_REWRITE_COMMAND || env.OPENAI_API_KEY);
}

export async function rewriteAnswerDynamic({
  prompt = "",
  response = "",
  uxProfile = DEFAULT_UX_PROFILE,
  mode = envFlag(process.env.CLAUDEX_DYNAMIC),
  model = process.env.CLAUDEX_MODEL || DEFAULT_MODEL,
  timeoutMs = Number(process.env.CLAUDEX_TIMEOUT_MS || DEFAULT_TIMEOUT_MS),
  maxInputChars = Number(process.env.CLAUDEX_MAX_INPUT_CHARS || DEFAULT_MAX_INPUT_CHARS),
  width = Number(process.env.CLAUDEX_WIDTH || process.stdout?.columns || process.env.COLUMNS || 80),
  env = process.env,
} = {}) {
  const original = String(response ?? "");
  if (!original.trim()) return "";
  if (shouldBackOff({ prompt, response: original })) return original;
  const profile = resolveUxProfile({ prompt, profile: uxProfile, env });

  const deterministic = () => {
    const shaped = shapeAnswer({ prompt, response: original, uxProfile: profile, width, env });
    return preservesProtectedElements({ original, rewritten: shaped }) ? shaped : original;
  };
  if (mode === "off") return deterministic();

  const input = buildRewritePrompt({
    prompt: clampInput(prompt, maxInputChars),
    response: clampInput(original, maxInputChars),
    uxProfile: profile,
  });
  const payload = { prompt, response: original, uxProfile: profile, instruction: input };

  try {
    let rewritten = "";
    if (env.CLAUDEX_REWRITE_COMMAND) {
      rewritten = await runCommandProvider({
        command: env.CLAUDEX_REWRITE_COMMAND,
        payload,
        timeoutMs,
      });
    } else if (env.OPENAI_API_KEY) {
      rewritten = await runOpenAIProvider({
        apiKey: env.OPENAI_API_KEY,
        model,
        input,
        timeoutMs,
      });
    } else if (mode === "require") {
      return deterministic();
    } else {
      return deterministic();
    }

    const trimmed = String(rewritten ?? "").trim();
    return isUsableRewrite(trimmed) && preservesProtectedElements({ original, rewritten: trimmed })
      ? trimmed
      : deterministic();
  } catch {
    return deterministic();
  }
}

export const CLAUDEX_REWRITE_PROMPT_CONTRACT = buildRewritePrompt;
