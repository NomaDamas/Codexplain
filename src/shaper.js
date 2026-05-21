import { classifyPrompt, shouldBackOff } from "./policy.js";
import { renderBoxTable, renderFlow, renderResponsivePanels } from "./renderer.js";
import { DEFAULT_UX_PROFILE, resolveUxProfile } from "./evolution.js";
import { colorize } from "./theme.js";

function splitSentences(text) {
  return String(text ?? "")
    .replace(/\s+/g, " ")
    .split(/(?<=[.!?。！？])\s+|\n+/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function compact(text, limit = 4) {
  const sentences = splitSentences(text);
  if (sentences.length <= limit) return sentences.join(" ");
  return sentences.slice(0, limit).join(" ");
}

function tldr(text, language) {
  const summary = compact(text, 1);
  if (!summary) return language === "ko" ? "핵심 요약 없음" : "No summary available";
  return summary;
}

function evidenceLine(response, language) {
  const text = String(response ?? "");
  const commands = [...text.matchAll(/`([^`]+)`/g)].map((match) => match[1]);
  const urls = [...text.matchAll(/https?:\/\/[^\s)]+/g)].map((match) => match[0].replace(/[.,;:!?]+$/u, ""));
  const paths = [...text.matchAll(/(?:^|[\s(["'])((?:\.{1,2}|~)?\/[A-Za-z0-9._~/-]+|[A-Za-z0-9._-]+\/[A-Za-z0-9._~/-]+)/g)]
    .map((match) => match[1].replace(/[.,;:!?]+$/u, ""));
  const evidence = [...new Set([...commands, ...urls, ...paths])];
  if (!evidence.length) return null;
  return language === "ko"
    ? `검증/근거: ${evidence.map((item) => `\`${item}\``).join(", ")}`
    : `Evidence: ${evidence.map((item) => `\`${item}\``).join(", ")}`;
}

function summaryLimit(language, profile) {
  if (profile.detail === "brief") return language === "ko" ? 2 : 3;
  if (profile.detail === "deep") return language === "ko" ? 8 : 9;
  return language === "ko" ? 4 : 5;
}

function prosePrefix(language, intent, profile) {
  if (language !== "ko") {
    if (profile.style === "tutorial") return "Step by step: ";
    if (profile.style === "executive") return "Bottom line: ";
    return intent === "debug" ? "Root cause first: " : "In short: ";
  }
  if (profile.style === "tutorial") return "쉽게 풀면, ";
  if (profile.style === "executive") return "핵심만 말하면, ";
  if (profile.style === "technical") return "기술적으로 정리하면, ";
  return intent === "debug" ? "원인부터 정리하면, " : "요약하면, ";
}

function deepRows({ language, summary, evidence, profile }) {
  if (profile.detail !== "deep") return null;
  return language === "ko"
    ? [
        ["TLDR", tldr(summary, language)],
        ["핵심", summary],
        ["왜 중요한가", "먼저 결론을 잡고, 필요한 배경만 이어서 설명합니다."],
        ["근거", evidence ?? "명시 없음"],
        ["다음 행동", "원하면 더 짧게, 더 기술적으로, 또는 단계별로 다시 조정할 수 있습니다."],
      ]
    : [
        ["TLDR", tldr(summary, language)],
        ["Core", summary],
        ["Why it matters", "Start with the conclusion, then add only the context needed to act."],
        ["Evidence", evidence ?? "Not stated"],
        ["Next action", "Ask for a shorter, more technical, or step-by-step version if needed."],
      ];
}

function wantsPairedArchitecture(prompt) {
  const text = String(prompt ?? "");
  return /(표.*흐름|흐름.*표|table.*flow|flow.*table|좌우|side[- ]?by[- ]?side|나란히)/iu.test(text);
}

function architectureSteps(language, summary, evidence) {
  return language === "ko"
    ? ["입력", "정책 검사", "UX 프로필", "답변 재구성", evidence ? "근거 보존" : "출력"]
    : ["Input", "Policy check", "UX profile", "Response shaping", evidence ? "Evidence kept" : "Output"];
}

export function shapeAnswer({ prompt, response, width = 80, uxProfile = DEFAULT_UX_PROFILE, env = process.env }) {
  if (shouldBackOff({ prompt, response })) return String(response ?? "");

  const { language, intent, structure } = classifyPrompt(prompt);
  const profile = resolveUxProfile({ prompt, profile: uxProfile, env });
  const requestedStructure = profile.preferredStructure === "auto" ? structure : profile.preferredStructure;
  const summary = compact(response, summaryLimit(language, profile));
  const evidence = evidenceLine(response, language);
  const rows = deepRows({ language, summary, evidence, profile }) ?? [
    ["TLDR", tldr(summary, language)],
    [language === "ko" ? "요약" : "Summary", summary],
    [language === "ko" ? "근거" : "Evidence", evidence ?? (language === "ko" ? "명시 없음" : "Not stated")],
  ];

  if (wantsPairedArchitecture(prompt)) {
    const leftWidth = Math.max(42, Math.floor(width * 0.58));
    const rightWidth = Math.max(24, width - leftWidth - 3);
    const table = renderBoxTable({
      width: leftWidth,
      theme: profile.theme,
      frame: profile.frame,
      headers: language === "ko" ? ["구분", "내용"] : ["Part", "Content"],
      rows,
    });
    const flow = renderFlow({
      width: rightWidth,
      theme: profile.theme,
      frame: profile.frame,
      steps: architectureSteps(language, summary, evidence),
    });
    return renderResponsivePanels({ panels: [table, flow], width, gap: 3 });
  }

  if (requestedStructure === "flow") {
    return renderFlow({
      width,
      theme: profile.theme,
      frame: profile.frame,
      steps: language === "ko"
        ? ["결론", summary, evidence ?? "검증 정보 없음"]
        : ["Conclusion", summary, evidence ?? "No verification evidence found"],
    });
  }

  if (requestedStructure === "table" || profile.detail === "deep") {
    return renderBoxTable({
      width,
      theme: profile.theme,
      frame: profile.frame,
      headers: language === "ko" ? ["구분", "내용"] : ["Part", "Content"],
      rows,
    });
  }

  const prefix = prosePrefix(language, intent, profile);
  const tldrLabel = language === "ko" ? "TLDR: " : "TLDR: ";
  return [
    colorize(tldrLabel, "heading", profile.theme) + tldr(summary, language),
    colorize(prefix, "heading", profile.theme) + summary,
    evidence,
  ].filter(Boolean).join("\n");
}
