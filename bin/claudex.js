#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import {
  rewriteAnswerDynamic,
  dynamicRewriteConfigured,
  buildGuidance,
  renderDemo,
  initProject,
  installCodexProject,
  buildUxContract,
  evolveUxProfileFromFeedback,
  loadProjectUxProfile,
  parseCodexWrapperArgs,
  resolveUxProfile,
  runCodexWithClaudex,
  saveProjectUxProfile,
  themeNames,
  UX_PROFILE_PATH,
} from "../src/index.js";

function usage() {
  return [
    "Claudex",
    "",
    "Usage:",
    "  claudex guide --prompt <text>",
    "  claudex shape --prompt <text> --response <text>",
    "  claudex shape --prompt <text> --response-file <path>",
    "  claudex shape --dynamic --prompt <text> --response <text>",
    "  claudex shape --width <columns> --prompt <text> --response <text>",
    "  claudex post-response --prompt <text> [--local-shape]",
    "  claudex profile --show",
    "  claudex profile --set-style <plain|tutorial|concise|executive|technical|review>",
    "  claudex profile --detail <brief|balanced|deep>",
    `  claudex profile --theme <${themeNames().join("|")}>`,
    "  claudex profile --frame <unicode|ascii>",
    "  claudex feedback --rating <1-5> --comment <text>",
    "  claudex init --local [--force]",
    "  claudex install-codex --local [--force]",
    "  claudex codex --prompt <text> [codex exec args...]",
    "  claudex demo",
    "",
    "Stdin:",
    "  echo '<answer>' | claudex shape --prompt '현재 상태 보기 쉽게'",
  ].join("\n");
}

function readArg(name, args) {
  const index = args.indexOf(name);
  if (index < 0) return null;
  const value = args[index + 1];
  if (!value || value.startsWith("--")) return "";
  return value;
}

function readWidth(args) {
  const value =
    readArg("--width", args) ??
    process.env.CLAUDEX_WIDTH ??
    process.stdout?.columns ??
    process.env.COLUMNS ??
    "80";
  const width = Number(value);
  return Number.isFinite(width) && width > 0 ? width : 80;
}

async function readStdin() {
  if (process.stdin.isTTY) return "";
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(Buffer.from(chunk));
  return Buffer.concat(chunks).toString("utf8");
}

async function main() {
  const [command, ...args] = process.argv.slice(2);

  if (!command || command === "--help" || command === "-h") {
    console.log(usage());
    return;
  }

  if (command === "demo") {
    console.log(renderDemo());
    return;
  }

  if (command === "init") {
    if (!args.includes("--local")) {
      console.error("Claudex only supports project-local init. Pass --local.");
      process.exitCode = 2;
      return;
    }

    try {
      const written = await initProject({ force: args.includes("--force") });
      console.log(`Created ${written.join(", ")}`);
    } catch (error) {
      if (error?.code === "EEXIST") {
        console.error("Local Claudex files already exist. Re-run with --force to overwrite.");
        process.exitCode = 1;
        return;
      }
      throw error;
    }
    return;
  }

  if (command === "install-codex") {
    if (!args.includes("--local")) {
      console.error("Claudex only installs Codex integration project-locally. Pass --local.");
      process.exitCode = 2;
      return;
    }

    try {
      const written = await installCodexProject({ force: args.includes("--force") });
      console.log(`Installed project-local Codex UX: ${written.join(", ")}`);
    } catch (error) {
      if (error?.code === "EEXIST") {
        console.error("Local Claudex files already exist. Re-run with --force to overwrite.");
        process.exitCode = 1;
        return;
      }
      throw error;
    }
    return;
  }

  if (command === "codex") {
    const parsed = parseCodexWrapperArgs(args);
    process.exitCode = await runCodexWithClaudex(parsed);
    return;
  }

  if (command === "profile") {
    const current = await loadProjectUxProfile();
    const next = {
      ...current,
      style: readArg("--set-style", args) ?? current.style,
      detail: readArg("--detail", args) ?? current.detail,
      theme: readArg("--theme", args) ?? current.theme,
      frame: readArg("--frame", args) ?? current.frame,
      audience: readArg("--audience", args) ?? current.audience,
      preferredStructure: readArg("--structure", args) ?? current.preferredStructure,
    };
    const shouldWrite = ["--set-style", "--detail", "--theme", "--frame", "--audience", "--structure"].some((item) =>
      args.includes(item),
    );
    const profile = shouldWrite ? await saveProjectUxProfile(next) : current;
    console.log(JSON.stringify(profile, null, 2));
    return;
  }

  if (command === "feedback") {
    const current = await loadProjectUxProfile();
    const next = evolveUxProfileFromFeedback(current, {
      rating: readArg("--rating", args),
      comment: readArg("--comment", args) ?? "",
      detail: readArg("--detail", args),
      style: readArg("--style", args),
    });
    const saved = await saveProjectUxProfile(next);
    console.log(`Updated ${UX_PROFILE_PATH}: detail=${saved.detail}, style=${saved.style}`);
    return;
  }

  const prompt = readArg("--prompt", args) ?? "";
  if (command !== "post-response" && !prompt.trim()) {
    console.error("Missing required --prompt text.");
    process.exitCode = 2;
    return;
  }

  if (command === "guide") {
    const storedProfile = await loadProjectUxProfile();
    const uxProfile = resolveUxProfile({ prompt, profile: storedProfile });
    console.log(`${buildGuidance(prompt, uxProfile)}\n\n${buildUxContract(uxProfile)}`);
    return;
  }

  if (command === "shape") {
    const responseFile = readArg("--response-file", args);
    const response =
      readArg("--response", args) ??
      (responseFile ? await readFile(responseFile, "utf8") : await readStdin());

    if (!response.trim()) {
      console.error("Missing response text. Pass --response, --response-file, or stdin.");
      process.exitCode = 2;
      return;
    }

    const storedProfile = await loadProjectUxProfile();
    const uxProfile = resolveUxProfile({ prompt, profile: storedProfile });
    const shaped = await rewriteAnswerDynamic({
      prompt,
      response,
      uxProfile,
      width: readWidth(args),
      mode: args.includes("--dynamic") ? "auto" : undefined,
    });
    process.stdout.write(shaped);
    return;
  }

  if (command === "post-response") {
    const input = await readStdin();
    if (!input.trim()) return;
    const fallbackPrompt = prompt || process.env.CLAUDEX_PROMPT || "";

    let payload;
    try {
      payload = JSON.parse(input);
    } catch {
      payload = { prompt: fallbackPrompt, response: input };
    }

    const response = payload.response ?? payload.answer ?? payload.text ?? input;
    const storedProfile = await loadProjectUxProfile();
    const uxProfile = resolveUxProfile({
      prompt: payload.prompt ?? payload.userPrompt ?? fallbackPrompt,
      profile: payload.uxProfile ?? storedProfile,
    });
    if (!dynamicRewriteConfigured(process.env) && (args.includes("--local-shape") || process.env.CLAUDEX_LOCAL_SHAPE)) {
      const shaped = await rewriteAnswerDynamic({
        prompt: payload.prompt ?? payload.userPrompt ?? fallbackPrompt,
        response,
        uxProfile,
        width: readWidth(args),
        mode: "off",
      });
      process.stdout.write(shaped);
      return;
    }
    if (!dynamicRewriteConfigured(process.env)) {
      process.stdout.write(response);
      return;
    }

    const shaped = await rewriteAnswerDynamic({
      prompt: payload.prompt ?? payload.userPrompt ?? fallbackPrompt,
      response,
      uxProfile,
      width: readWidth(args),
    });
    process.stdout.write(shaped);
    return;
  }

  console.error(`Unknown command: ${command}`);
  console.error("");
  console.error(usage());
  process.exitCode = 2;
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
