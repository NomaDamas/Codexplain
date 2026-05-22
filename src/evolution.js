import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { normalizeTheme } from "./theme.js";

export const UX_PROFILE_DIR = ".codexplain";
export const UX_PROFILE_PATH = `${UX_PROFILE_DIR}/ux-profile.json`;
export const LEGACY_UX_PROFILE_PATH = ".claudex/ux-profile.json";

const DETAIL_LEVELS = new Set(["brief", "balanced", "deep"]);
const STYLE_LEVELS = new Set(["plain", "tutorial", "concise", "executive", "technical", "review"]);
const FRAME_STYLES = new Set(["unicode", "ascii"]);
const ABSTRACTION_LEVELS = ["concrete", "implementation", "architecture", "strategy"];
const DETAIL_LAYERS = new Set([
  "tldr",
  "summary",
  "concept",
  "mechanism",
  "architecture",
  "implementation",
  "evidence",
  "next-step",
]);

export const DEFAULT_UX_PROFILE = Object.freeze({
  schemaVersion: 1,
  detail: "balanced",
  style: "plain",
  theme: "ocean",
  frame: "unicode",
  audience: "general",
  preferredStructure: "auto",
  abstractionRange: {
    min: "concrete",
    max: "architecture",
  },
  detailLayers: ["tldr", "summary", "architecture", "implementation", "evidence", "next-step"],
  detailScale: 80,
  uxDensity: 65,
  riskSensitivity: 60,
  explanationMoves: ["tldr", "answer-first", "plain-language", "evidence", "next-step"],
  feedback: {
    positive: 0,
    negative: 0,
    revisions: 0,
    rewardScore: 0,
    signals: [],
  },
});

function cleanString(value, fallback = "") {
  const text = String(value ?? "").trim();
  return text || fallback;
}

function normalizeDetail(value, fallback = "balanced") {
  const text = String(value ?? "").trim().toLowerCase();
  if (["short", "summary", "quick", "간단", "짧게", "요약"].includes(text)) return "brief";
  if (["long", "detailed", "detail", "자세", "상세"].includes(text)) return "deep";
  return DETAIL_LEVELS.has(text) ? text : fallback;
}

function normalizeStyle(value, fallback = "plain") {
  const text = String(value ?? "").trim().toLowerCase();
  if (["easy", "simple", "eli5", "쉽게", "쉬운"].includes(text)) return "plain";
  if (["teach", "teacher", "tutorial", "강의", "튜토리얼"].includes(text)) return "tutorial";
  if (["short", "concise", "간결"].includes(text)) return "concise";
  if (["business", "exec", "executive", "보고"].includes(text)) return "executive";
  if (["code", "dev", "technical", "기술"].includes(text)) return "technical";
  return STYLE_LEVELS.has(text) ? text : fallback;
}

function normalizeStructure(value, fallback = "auto") {
  const text = String(value ?? "").trim().toLowerCase();
  if (["auto", "prose", "flow", "table"].includes(text)) return text;
  if (["표", "비교"].includes(text)) return "table";
  if (["흐름", "흐름도", "단계"].includes(text)) return "flow";
  return fallback;
}

function normalizeFrame(value, fallback = "unicode", env = process.env) {
  const text = String(value ?? "").trim().toLowerCase();
  if (text === "box" || text === "unicode" || text === "utf8" || text === "utf-8") return "unicode";
  if (["ascii", "plain-ascii", "fallback", "non-unicode", "no-unicode"].includes(text)) return "ascii";
  if (text === "auto" || text === "terminal") return terminalSupportsUnicode(env) ? "unicode" : "ascii";
  return FRAME_STYLES.has(text) ? text : fallback;
}

function envFlagEnabled(value) {
  return ["1", "true", "yes", "on"].includes(String(value ?? "").trim().toLowerCase());
}

function terminalSupportsUnicode(env = process.env) {
  if (envFlagEnabled(env.CODEXPLAIN_NO_UNICODE) || envFlagEnabled(env.NO_UNICODE)) return false;
  if (env.TERM === "dumb") return false;
  const locale = [env.LC_ALL, env.LC_CTYPE, env.LANG].find((item) => String(item ?? "").trim());
  if (!locale) return true;
  return /utf-?8/iu.test(String(locale));
}

function normalizeAbstraction(value, fallback = "architecture") {
  const text = String(value ?? "").trim().toLowerCase();
  const aliases = {
    low: "concrete",
    detail: "concrete",
    concrete: "concrete",
    code: "implementation",
    impl: "implementation",
    implementation: "implementation",
    architecture: "architecture",
    architectural: "architecture",
    structure: "architecture",
    high: "strategy",
    strategic: "strategy",
    strategy: "strategy",
  };
  return aliases[text] ?? (ABSTRACTION_LEVELS.includes(text) ? text : fallback);
}

function normalizeAbstractionRange(value, fallback = DEFAULT_UX_PROFILE.abstractionRange) {
  if (value && typeof value === "object" && !Array.isArray(value)) {
    const min = normalizeAbstraction(value.min, fallback.min);
    const max = normalizeAbstraction(value.max, fallback.max);
    return orderAbstractionRange(min, max);
  }
  const text = String(value ?? "").trim().toLowerCase();
  if (!text) return orderAbstractionRange(fallback.min, fallback.max);
  if (text === "full" || text === "all") return { min: "concrete", max: "strategy" };
  const [minRaw, maxRaw] = text.split(/[:.]{2}|:|,/u).map((item) => item?.trim()).filter(Boolean);
  const min = normalizeAbstraction(minRaw, fallback.min);
  const max = normalizeAbstraction(maxRaw ?? minRaw, fallback.max);
  return orderAbstractionRange(min, max);
}

function orderAbstractionRange(min, max) {
  const minIndex = ABSTRACTION_LEVELS.indexOf(min);
  const maxIndex = ABSTRACTION_LEVELS.indexOf(max);
  if (minIndex <= maxIndex) return { min, max };
  return { min: max, max: min };
}


function normalizeControl(value, fallback) {
  const number = Number(value);
  if (!Number.isFinite(number)) return fallback;
  return Math.max(0, Math.min(100, Math.round(number)));
}

function normalizeDetailLayers(value, fallback = DEFAULT_UX_PROFILE.detailLayers) {
  const values = Array.isArray(value)
    ? value
    : String(value ?? "")
        .split(/[,|]/u)
        .map((item) => item.trim())
        .filter(Boolean);
  const normalized = values
    .map((item) => {
      const text = String(item ?? "").trim().toLowerCase();
      if (["next", "action", "next-action"].includes(text)) return "next-step";
      if (["arch", "structure"].includes(text)) return "architecture";
      if (["impl", "code"].includes(text)) return "implementation";
      return text;
    })
    .filter((item) => DETAIL_LAYERS.has(item));
  return normalized.length ? uniqueList(normalized) : [...fallback];
}

function uniqueList(values) {
  return [...new Set(values.map((item) => cleanString(item)).filter(Boolean))];
}

export function sanitizeUxProfile(profile = {}) {
  const feedback = profile.feedback && typeof profile.feedback === "object" ? profile.feedback : {};
  return {
    schemaVersion: 1,
    detail: normalizeDetail(profile.detail, DEFAULT_UX_PROFILE.detail),
    style: normalizeStyle(profile.style, DEFAULT_UX_PROFILE.style),
    theme: normalizeTheme(profile.theme, DEFAULT_UX_PROFILE.theme),
    frame: normalizeFrame(profile.frame, DEFAULT_UX_PROFILE.frame),
    audience: cleanString(profile.audience, DEFAULT_UX_PROFILE.audience),
    preferredStructure: normalizeStructure(profile.preferredStructure, DEFAULT_UX_PROFILE.preferredStructure),
    abstractionRange: normalizeAbstractionRange(profile.abstractionRange, DEFAULT_UX_PROFILE.abstractionRange),
    detailLayers: normalizeDetailLayers(profile.detailLayers, DEFAULT_UX_PROFILE.detailLayers),
    detailScale: normalizeControl(profile.detailScale, DEFAULT_UX_PROFILE.detailScale),
    uxDensity: normalizeControl(profile.uxDensity, DEFAULT_UX_PROFILE.uxDensity),
    riskSensitivity: normalizeControl(profile.riskSensitivity, DEFAULT_UX_PROFILE.riskSensitivity),
    explanationMoves: uniqueList(
      Array.isArray(profile.explanationMoves)
        ? profile.explanationMoves
        : DEFAULT_UX_PROFILE.explanationMoves,
    ),
    feedback: {
      positive: Number(feedback.positive || 0),
      negative: Number(feedback.negative || 0),
      revisions: Number(feedback.revisions || 0),
      rewardScore: Number(feedback.rewardScore || 0),
      signals: Array.isArray(feedback.signals) ? feedback.signals.slice(-12) : [],
    },
  };
}

export async function loadProjectUxProfile({ cwd = process.cwd() } = {}) {
  try {
    const raw = await readFile(join(cwd, UX_PROFILE_PATH), "utf8");
    return sanitizeUxProfile(JSON.parse(raw));
  } catch {
    try {
      const raw = await readFile(join(cwd, LEGACY_UX_PROFILE_PATH), "utf8");
      return sanitizeUxProfile(JSON.parse(raw));
    } catch {
      return sanitizeUxProfile(DEFAULT_UX_PROFILE);
    }
  }
}

export async function saveProjectUxProfile(profile, { cwd = process.cwd() } = {}) {
  const safeProfile = sanitizeUxProfile(profile);
  await mkdir(join(cwd, UX_PROFILE_DIR), { recursive: true });
  await writeFile(join(cwd, UX_PROFILE_PATH), `${JSON.stringify(safeProfile, null, 2)}\n`);
  return safeProfile;
}

function promptSignals(prompt) {
  const text = String(prompt ?? "");
  const signals = {};
  if (/(자세|상세|구체|깊게|deep|detail|detailed)/iu.test(text)) signals.detail = "deep";
  if (/(간단|짧게|요약|brief|concise|short)/iu.test(text)) signals.detail = "brief";
  if (/(쉽게|쉬운|초보|비전공|eli5|plain|simple)/iu.test(text)) signals.style = "plain";
  if (/(강의|튜토리얼|단계별|teach|tutorial)/iu.test(text)) signals.style = "tutorial";
  if (/(경영진|보고용|executive|business)/iu.test(text)) signals.style = "executive";
  if (/(기술적으로|내부 구조|technical|implementation)/iu.test(text)) signals.style = "technical";
  if (/(표|비교|matrix|table)/iu.test(text)) signals.preferredStructure = "table";
  if (/(흐름도|순서도|flow|pipeline)/iu.test(text)) signals.preferredStructure = "flow";
  if (/(낮은\s*레벨|구체|코드\s*레벨|concrete|low-level)/iu.test(text)) {
    signals.abstractionRange = { min: "concrete", max: "implementation" };
  }
  if (/(아키텍처|구조|architecture|system design)/iu.test(text)) {
    signals.abstractionRange = { min: "implementation", max: "architecture" };
  }
  if (/(전략|비즈니스|상위\s*레벨|strategy|high-level)/iu.test(text)) {
    signals.abstractionRange = { min: "architecture", max: "strategy" };
  }
  return signals;
}

export function resolveUxProfile({ prompt = "", profile = DEFAULT_UX_PROFILE, env = process.env } = {}) {
  const base = sanitizeUxProfile(profile);
  const resolved = {
    ...base,
    detail: normalizeDetail(env.CODEXPLAIN_DETAIL ?? env.CLAUDEX_DETAIL, base.detail),
    style: normalizeStyle(env.CODEXPLAIN_STYLE ?? env.CLAUDEX_STYLE, base.style),
    theme: normalizeTheme(env.CODEXPLAIN_THEME ?? env.CODEXPLAIN_COLOR ?? env.CLAUDEX_THEME ?? env.CLAUDEX_COLOR, base.theme),
    frame: normalizeFrame(env.CODEXPLAIN_FRAME ?? env.CLAUDEX_FRAME, base.frame, env),
    audience: cleanString(env.CODEXPLAIN_AUDIENCE ?? env.CLAUDEX_AUDIENCE, base.audience),
    preferredStructure: normalizeStructure(env.CODEXPLAIN_STRUCTURE ?? env.CLAUDEX_STRUCTURE, base.preferredStructure),
    abstractionRange: normalizeAbstractionRange(
      env.CODEXPLAIN_ABSTRACTION_RANGE ?? env.CLAUDEX_ABSTRACTION_RANGE ?? {
        min: env.CODEXPLAIN_ABSTRACTION_MIN ?? env.CLAUDEX_ABSTRACTION_MIN ?? base.abstractionRange.min,
        max: env.CODEXPLAIN_ABSTRACTION_MAX ?? env.CLAUDEX_ABSTRACTION_MAX ?? env.CODEXPLAIN_ABSTRACTION ?? env.CLAUDEX_ABSTRACTION ?? base.abstractionRange.max,
      },
      base.abstractionRange,
    ),
    detailLayers: normalizeDetailLayers(env.CODEXPLAIN_LAYERS ?? env.CLAUDEX_LAYERS, base.detailLayers),
    detailScale: normalizeControl(env.CODEXPLAIN_DETAIL_SCALE, base.detailScale),
    uxDensity: normalizeControl(env.CODEXPLAIN_UX_DENSITY, base.uxDensity),
    riskSensitivity: normalizeControl(env.CODEXPLAIN_RISK_SENSITIVITY, base.riskSensitivity),
  };
  return sanitizeUxProfile({ ...resolved, ...promptSignals(prompt) });
}

function inferFeedbackSignal(comment) {
  const text = String(comment ?? "");
  const inferred = {};
  const signals = [];
  if (/(자세|상세|구체|부족|얕|more detail|too shallow)/iu.test(text)) {
    inferred.detail = "deep";
    signals.push("needs-more-detail");
  }
  if (/(길|장황|너무 많|짧게|요약|too long|shorter)/iu.test(text)) {
    inferred.detail = "brief";
    inferred.style = "concise";
    signals.push("needs-less-detail");
  }
  if (/(어렵|쉽게|비전공|초보|plain|simpler|confusing)/iu.test(text)) {
    inferred.style = "plain";
    signals.push("needs-simpler-language");
  }
  if (/(단계|튜토리얼|예시|example|step)/iu.test(text)) {
    inferred.style = "tutorial";
    signals.push("needs-teaching-steps");
  }
  if (/(기술|코드|구현|technical|implementation)/iu.test(text)) {
    inferred.style = "technical";
    signals.push("needs-technical-depth");
  }
  return { ...inferred, signal: signals[0] ?? "general-feedback" };
}

export function evolveUxProfileFromFeedback(profile, { rating, comment = "", detail, style } = {}) {
  const next = sanitizeUxProfile(profile);
  const score = Number(rating);
  const inferred = inferFeedbackSignal(comment);

  if (Number.isFinite(score) && score >= 4) next.feedback.positive += 1;
  if (Number.isFinite(score) && score <= 2) next.feedback.negative += 1;
  next.feedback.revisions += 1;
  next.feedback.rewardScore += Number.isFinite(score) ? score - 3 : 0;

  next.detail = normalizeDetail(detail, inferred.detail || next.detail);
  next.style = normalizeStyle(style, inferred.style || next.style);
  next.feedback.signals = [
    ...next.feedback.signals,
    {
      at: new Date().toISOString(),
      rating: Number.isFinite(score) ? score : null,
      signal: inferred.signal,
    },
  ].slice(-12);

  return sanitizeUxProfile(next);
}

export function buildRlhfSummary(profile = DEFAULT_UX_PROFILE) {
  const resolved = sanitizeUxProfile(profile);
  const reward =
    resolved.feedback.rewardScore > 0
      ? "positive"
      : resolved.feedback.rewardScore < 0
        ? "needs-adjustment"
        : "neutral";
  return [
    `Preference reward: ${reward}`,
    `- revisions: ${resolved.feedback.revisions}`,
    `- positive: ${resolved.feedback.positive}`,
    `- negative: ${resolved.feedback.negative}`,
    `- rewardScore: ${resolved.feedback.rewardScore}`,
    `- next detail: ${resolved.detail}`,
    `- numeric controls: detailScale=${resolved.detailScale}, uxDensity=${resolved.uxDensity}, riskSensitivity=${resolved.riskSensitivity}`,
    `- next style: ${resolved.style}`,
    `- abstraction range: ${resolved.abstractionRange.min}..${resolved.abstractionRange.max}`,
  ].join("\n");
}

export function buildUxContract(profile = DEFAULT_UX_PROFILE) {
  const resolved = sanitizeUxProfile(profile);
  const depth =
    resolved.detail === "deep"
      ? "Give enough context to be useful, but keep each paragraph short."
      : resolved.detail === "brief"
        ? "Compress aggressively and omit nonessential background."
        : "Use a balanced explanation with the conclusion first.";
  const style =
    {
      plain: "Use plain language, define jargon in context, and prefer familiar words.",
      tutorial: "Teach step by step, moving from the user's goal to the mechanism.",
      concise: "Use minimal prose and high-signal bullets only when useful.",
      executive: "State outcome, impact, risk, and decision points first.",
      technical: "Keep implementation details explicit while preserving readability.",
      review: "Put findings, severity, evidence, and fixes before summary.",
    }[resolved.style] ?? "Use plain language.";

  return [
    "Adaptive explanation contract:",
    `- Detail level: ${resolved.detail}. ${depth}`,
    `- Style: ${resolved.style}. ${style}`,
    `- Terminal color theme: ${resolved.theme}. Use color only for visual grouping, never to encode the only meaning.`,
    `- Frame style: ${resolved.frame}. Use ASCII frames when copy/paste safety matters.`,
    `- Audience: ${resolved.audience}.`,
    `- Abstraction range: ${resolved.abstractionRange.min}..${resolved.abstractionRange.max}. Stay inside that explanation level range.`,
    `- Detail layers: ${resolved.detailLayers.join(", ")}.`,
    `- Numeric controls: detailScale ${resolved.detailScale}/100, uxDensity ${resolved.uxDensity}/100, riskSensitivity ${resolved.riskSensitivity}/100.`,
    "- Prefer answer-first structure, then why it matters, evidence, and next action.",
    "- In terminal output, use ANSI color when the active theme is not none; use colors to highlight labels, risks, and key terms.",
    "- Start explanatory answers with a TLDR when the output is not an exact artifact.",
    "- Keep related visuals spatially close: pair tables and flows side by side only when width allows; otherwise stack them.",
    "- Use examples or analogies only when they reduce confusion.",
    "- Preserve exact commands, paths, dates, risks, and verification evidence.",
    "- Do not store or repeat private answer text as feedback memory.",
  ].join("\n");
}
