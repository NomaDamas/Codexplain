const KOREAN_RE = /[\u3131-\u318e\uac00-\ud7a3]/;

const STRICT_PATTERNS = [
  /\b(?:only|valid|raw)\s+(?:json|yaml|toml|xml|csv)\b/i,
  /\b(?:machine-readable|strict format|exact format|verbatim)\b/i,
  /\b(?:commit message|git commit|diff|patch|test output|logs?)\b/i,
  /(?:JSON만|YAML만|정확한\s*형식|엄격한\s*형식|커밋\s*메시지|로그만|코드만|테스트\s*출력만)/u,
];

const STRUCTURE_PATTERNS = {
  flow: [
    /\b(?:flow|flowchart|sequence|pipeline|step-by-step|lifecycle)\b/i,
    /(?:흐름도|순서도|단계별|과정|파이프라인|라이프사이클)/u,
  ],
  table: [
    /\b(?:table|matrix|compare|comparison|tradeoff|options?)\b/i,
    /(?:표로|비교|장단점|옵션|선택지)/u,
  ],
};

const INTENT_PATTERNS = [
  ["debug", [/\b(?:debug|bug|error|failed|root cause)\b/i, /(?:버그|오류|에러|실패|원인)/u]],
  ["explain", [/\b(?:explain|what is|how does|why)\b/i, /(?:설명|무엇|어떻게|왜|보기\s*쉽게)/u]],
  ["plan", [/\b(?:plan|roadmap|strategy|steps?)\b/i, /(?:계획|전략|로드맵|단계)/u]],
  ["review", [/\b(?:review|audit|findings?)\b/i, /(?:리뷰|검토|감사)/u]],
  ["status", [/\b(?:status|progress|current state)\b/i, /(?:상태|진행|현재)/u]],
];

export function classifyPrompt(prompt) {
  const text = String(prompt ?? "");
  const language = KOREAN_RE.test(text) ? "ko" : "en";
  const intent =
    INTENT_PATTERNS.find(([, patterns]) => patterns.some((pattern) => pattern.test(text)))?.[0] ??
    "answer";
  const structure = STRUCTURE_PATTERNS.table.some((pattern) => pattern.test(text))
    ? "table"
    : STRUCTURE_PATTERNS.flow.some((pattern) => pattern.test(text))
      ? "flow"
      : "prose";

  return { language, intent, structure };
}

export function looksLikeMachineOutput(text) {
  const value = String(text ?? "").trim();
  if (!value) return false;
  if (/^(?:\{[\s\S]*\}|\[[\s\S]*\])$/.test(value)) {
    try {
      JSON.parse(value);
      return true;
    } catch {}
  }
  return [
    /^```[\s\S]*```$/m,
    /^\s*(?:diff --git|@@\s+-\d+|\*\*\* Begin Patch)/m,
    /^\s*(?:PASS|FAIL|ok \d+|not ok \d+|TAP version \d+)/m,
    /^\s*(?:error|warn|info|debug|trace):\s+/im,
    /^\s*(?:feat|fix|docs|style|refactor|test|chore)(?:\([^)]+\))?!?:\s+/m,
  ].some((pattern) => pattern.test(value));
}

export function shouldBackOff({ prompt = "", response = "" } = {}) {
  const promptText = String(prompt ?? "");
  if (STRICT_PATTERNS.some((pattern) => pattern.test(promptText))) return true;
  return looksLikeMachineOutput(response);
}

export function buildGuidance(prompt, uxProfile) {
  const { language, intent, structure } = classifyPrompt(prompt);
  const lines = [
    "Claudex answer guidance:",
    "- Keep the technical facts, commands, paths, risks, and verification status unchanged.",
    "- Start non-artifact explanations with a TLDR.",
    "- Prefer conclusion first, then the smallest useful explanation.",
    "- Do not expose hidden mode names or implementation labels.",
    "- Do not rewrite JSON, code, logs, test output, diffs, or commit messages.",
  ];

  if (language === "ko") {
    lines.push("- Korean-first: use short Korean sentences unless the user asks otherwise.");
  } else {
    lines.push("- Use plain English with compact sentences.");
  }

  if (intent === "debug") lines.push("- Separate cause, impact, fix, and verification.");
  if (intent === "plan") lines.push("- Show goal, constraints, steps, and validation.");
  if (intent === "review") lines.push("- Put findings first and keep severity clear.");
  if (intent === "status") lines.push("- Show current state, next action, and evidence.");
  if (structure === "flow") lines.push("- Use a connected vertical flow only if it reduces scan cost.");
  if (structure === "table") lines.push("- Use a connected box table only if comparison is clearer than prose.");
  if (uxProfile?.detail) lines.push(`- Detail preference: ${uxProfile.detail}.`);
  if (uxProfile?.style) lines.push(`- Style preference: ${uxProfile.style}.`);
  if (uxProfile?.theme) lines.push(`- Terminal color theme: ${uxProfile.theme}.`);
  if (uxProfile?.frame) lines.push(`- Frame style: ${uxProfile.frame}.`);
  if (uxProfile?.audience) lines.push(`- Audience: ${uxProfile.audience}.`);

  return lines.join("\n");
}
