#!/usr/bin/env node
import { basename } from "node:path";
import { readFile } from "node:fs/promises";
import {
  rewriteAnswerDynamic,
  dynamicRewriteConfigured,
  buildGuidance,
  buildRlhfSummary,
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

function commandName() {
  const name = basename(process.argv[1] ?? "codexplain").replace(/\.js$/, "");
  return name || "codexplain";
}

function brandName(name = commandName()) {
  return name.startsWith("claudex") ? "Claudex (legacy alias for Codexplain)" : "Codexplain";
}

function usage(name = commandName()) {
  const codexName = name.startsWith("claudex") ? "claudex-codex" : "codexplain-codex";
  return [
    brandName(name),
    "",
    "Usage:",
    `  ${name} guide --prompt <text>`,
    `  ${name} shape --prompt <text> --response <text>`,
    `  ${name} shape --prompt <text> --response-file <path>`,
    `  ${name} shape --dynamic --prompt <text> --response <text>`,
    `  ${name} shape --width <columns> --prompt <text> --response <text>`,
    `  ${name} post-response --prompt <text> [--local-shape]`,
    `  ${name} profile --show`,
    `  ${name} profile --set-style <plain|tutorial|concise|executive|technical|review>`,
    `  ${name} profile --detail <brief|balanced|deep>`,
    `  ${name} profile --theme <${themeNames().join("|")}>`,
    `  ${name} profile --frame <unicode|ascii>`,
    `  ${name} profile --abstraction <concrete|implementation|architecture|strategy>`,
    `  ${name} profile --abstraction-range <concrete:strategy>`,
    `  ${name} profile --layers <tldr,summary,architecture,implementation,evidence,next-step>`,
    `  ${name} feedback --rating <1-5> --comment <text>`,
    `  ${name} rlhf --rating <1-5> --comment <text>`,
    `  ${name} init --local [--force]`,
    `  ${name} install-codex --local [--force]`,
    `  ${name} codex --prompt <text> [codex exec args...]`,
    `  ${codexName} --prompt <text> [codex exec args...]`,
    `  ${name} demo`,
    "",
    "Stdin:",
    `  echo '<answer>' | ${name} shape --prompt '현재 상태 보기 쉽게'`,
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
    process.env.CODEXPLAIN_WIDTH ??
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
  const cliName = commandName();

  if (!command || command === "--help" || command === "-h") {
    console.log(usage(cliName));
    return;
  }

  if (command === "demo") {
    console.log(renderDemo());
    return;
  }

  if (command === "init") {
    if (!args.includes("--local")) {
      console.error("Codexplain only supports project-local init. Pass --local.");
      process.exitCode = 2;
      return;
    }

    try {
      const written = await initProject({ force: args.includes("--force") });
      console.log(`Created ${written.join(", ")}`);
    } catch (error) {
      if (error?.code === "EEXIST") {
        console.error("Local Codexplain files already exist. Re-run with --force to overwrite.");
        process.exitCode = 1;
        return;
      }
      throw error;
    }
    return;
  }

  if (command === "install-codex") {
    if (!args.includes("--local")) {
      console.error("Codexplain only installs Codex integration project-locally. Pass --local.");
      process.exitCode = 2;
      return;
    }

    try {
      const written = await installCodexProject({ force: args.includes("--force") });
      console.log(`Installed project-local Codex UX: ${written.join(", ")}`);
    } catch (error) {
      if (error?.code === "EEXIST") {
        console.error("Local Codexplain files already exist. Re-run with --force to overwrite.");
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
      abstractionRange:
        readArg("--abstraction-range", args) ??
        (readArg("--abstraction", args)
          ? { min: current.abstractionRange?.min ?? "concrete", max: readArg("--abstraction", args) }
          : current.abstractionRange),
      detailLayers: readArg("--layers", args) ?? current.detailLayers,
    };
    const shouldWrite = [
      "--set-style",
      "--detail",
      "--theme",
      "--frame",
      "--audience",
      "--structure",
      "--abstraction",
      "--abstraction-range",
      "--layers",
    ].some((item) => args.includes(item));
    const profile = shouldWrite ? await saveProjectUxProfile(next) : current;
    console.log(JSON.stringify(profile, null, 2));
    return;
  }

  if (command === "feedback" || command === "rlhf") {
    const current = await loadProjectUxProfile();
    const next = evolveUxProfileFromFeedback(current, {
      rating: readArg("--rating", args),
      comment: readArg("--comment", args) ?? "",
      detail: readArg("--detail", args),
      style: readArg("--style", args),
    });
    const saved = await saveProjectUxProfile(next);
    if (command === "rlhf") {
      console.log(`Updated ${UX_PROFILE_PATH}\n${buildRlhfSummary(saved)}`);
    } else {
      console.log(`Updated ${UX_PROFILE_PATH}: detail=${saved.detail}, style=${saved.style}`);
    }
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
    const fallbackPrompt = prompt || process.env.CODEXPLAIN_PROMPT || process.env.CLAUDEX_PROMPT || "";

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
    if (
      !dynamicRewriteConfigured(process.env) &&
      (args.includes("--local-shape") || process.env.CODEXPLAIN_LOCAL_SHAPE || process.env.CLAUDEX_LOCAL_SHAPE)
    ) {
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
    console.error(usage(cliName));
    process.exitCode = 2;
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
