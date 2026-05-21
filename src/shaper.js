import { classifyPrompt, shouldBackOff } from "./policy.js";
import {
  renderBoxTable,
  renderFlow,
  renderFormulaBox,
  renderIndexedList,
  renderProsConsPanels,
  renderResponsivePanels,
} from "./renderer.js";
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
  return layeredRows({ language, summary, evidence, profile });
}

const ABSTRACTION_ORDER = ["concrete", "implementation", "architecture", "strategy"];

function abstractionIncludes(profile, level) {
  const minIndex = ABSTRACTION_ORDER.indexOf(profile.abstractionRange?.min ?? "concrete");
  const maxIndex = ABSTRACTION_ORDER.indexOf(profile.abstractionRange?.max ?? "architecture");
  const levelIndex = ABSTRACTION_ORDER.indexOf(level);
  return levelIndex >= minIndex && levelIndex <= maxIndex;
}

function wantsLayer(profile, layer) {
  return profile.detailLayers?.includes(layer);
}

function layeredRows({ language, summary, evidence, profile }) {
  const rows = [];
  const ko = language === "ko";
  if (wantsLayer(profile, "tldr")) rows.push(["TLDR", tldr(summary, language)]);
  if (wantsLayer(profile, "summary")) rows.push([ko ? "핵심" : "Core", summary]);
  if (wantsLayer(profile, "concept") && abstractionIncludes(profile, "strategy")) {
    rows.push([
      ko ? "개념" : "Concept",
      ko ? "상위 목표와 사용자 의도를 먼저 잡고, 세부 구현은 필요한 만큼만 내려갑니다." : "Start from the user goal, then descend only as far as needed.",
    ]);
  }
  if (wantsLayer(profile, "mechanism") && abstractionIncludes(profile, "architecture")) {
    rows.push([
      ko ? "작동 원리" : "Mechanism",
      ko ? "입력 의도, 정책 보호, UX 프로필, 렌더링 단계를 분리해 설명합니다." : "Separate intent, policy protection, UX profile, and rendering.",
    ]);
  }
  if (wantsLayer(profile, "architecture") && abstractionIncludes(profile, "architecture")) {
    rows.push([
      ko ? "구조" : "Architecture",
      ko ? "CLI → Policy → Evolution → Shaper → Renderer 순서로 책임을 나눕니다." : "Split responsibility across CLI, policy, evolution, shaper, and renderer.",
    ]);
  }
  if (wantsLayer(profile, "implementation") && abstractionIncludes(profile, "implementation")) {
    rows.push([
      ko ? "구현" : "Implementation",
      ko ? "파일/명령/검증 근거처럼 실행 가능한 세부 정보를 보존합니다." : "Preserve actionable details such as files, commands, and verification evidence.",
    ]);
  }
  if (wantsLayer(profile, "evidence")) rows.push([ko ? "근거" : "Evidence", evidence ?? (ko ? "명시 없음" : "Not stated")]);
  if (wantsLayer(profile, "next-step")) {
    rows.push([
      ko ? "다음 행동" : "Next action",
      ko ? "원하면 더 낮은 구현 레벨이나 더 높은 전략 레벨로 다시 조정할 수 있습니다." : "Ask for a lower implementation level or a higher strategic level if needed.",
    ]);
  }
  return rows.length ? rows : [[ko ? "핵심" : "Core", summary]];
}

function wantsPairedArchitecture(prompt) {
  const text = String(prompt ?? "");
  return /(표.*흐름|흐름.*표|table.*flow|flow.*table|좌우|side[- ]?by[- ]?side|나란히)/iu.test(text);
}

function wantsProsCons(prompt) {
  const text = String(prompt ?? "");
  return /(장단점|장점.*단점|단점.*장점|pros?\s*(?:and|&|\/)\s*cons?|trade[- ]?offs?|JS.*Rust|Rust.*JS)/iu.test(text);
}

function wantsFormula(prompt) {
  return /(수식|공식|equation|formula|math)/iu.test(String(prompt ?? ""));
}

function wantsIndexedList(prompt) {
  return /(1\s*,?\s*2\s*,?\s*3|번호|인덱스|순번|목록|리스트|단계별|numbered|indexed|list)/iu.test(String(prompt ?? ""));
}

function wantsTldr(prompt, intent, profile) {
  const text = String(prompt ?? "");
  if (/(tldr|tl;dr|요약|핵심만|현재\s*상태|status)/iu.test(text)) return true;
  if (profile.detail === "brief") return true;
  return intent === "status";
}

function selectRenderMode({ prompt, intent, requestedStructure, profile }) {
  if (wantsProsCons(prompt)) return "pros-cons";
  if (wantsPairedArchitecture(prompt)) return "paired-architecture";
  if (requestedStructure === "flow") return "flow";
  if (requestedStructure === "table") return "table";
  if (wantsFormula(prompt)) return "formula";
  if (wantsIndexedList(prompt)) return "indexed";
  if (profile.detail === "deep") return "table";
  return wantsTldr(prompt, intent, profile) ? "tldr-prose" : "prose";
}

function jsRustProsCons(language) {
  if (language === "ko") {
    return {
      left: {
        title: "JS / Node",
        pros: ["빠른 수정", "provider 연동 쉬움", "문자열/JSON 처리 편함", "CLI 실험 비용 낮음"],
        cons: ["단일 바이너리 배포 약함", "런타임 의존성 필요", "장기 core 안정성은 Rust보다 약함"],
        bestFor: "설명 UX, 프롬프트, 피드백 루프를 빠르게 실험할 때",
      },
      right: {
        title: "Rust",
        pros: ["단일 바이너리", "빠른 시작 속도", "낮은 메모리", "강한 타입 안정성", "터미널 렌더러에 적합"],
        cons: ["초기 구현 비용 큼", "provider/JSON 실험은 JS보다 무거움", "UX 템플릿을 코드에 묶으면 반복이 느려짐"],
        bestFor: "기능이 굳은 뒤 CLI core와 renderer를 제품화할 때",
      },
    };
  }
  return {
    left: {
      title: "JS / Node",
      pros: ["Fast edits", "Easy provider integration", "Convenient string/JSON handling", "Low CLI experiment cost"],
      cons: ["Weaker single-binary story", "Needs runtime dependency", "Less strict long-term core safety"],
      bestFor: "Fast iteration on explanation UX, prompts, and feedback loops",
    },
    right: {
      title: "Rust",
      pros: ["Single binary", "Fast startup", "Low memory", "Strong typing", "Good fit for terminal rendering"],
      cons: ["Higher initial cost", "Provider/JSON experiments are heavier", "Hard-coded UX slows iteration"],
      bestFor: "Productizing a stable CLI core and renderer",
    },
  };
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
  const renderMode = selectRenderMode({ prompt, intent, requestedStructure, profile });

  if (renderMode === "pros-cons") {
    const { left, right } = jsRustProsCons(language);
    const comparison = renderProsConsPanels({
      left,
      right,
      width,
      theme: profile.theme,
      frame: profile.frame,
    });
    const formula = wantsFormula(prompt)
      ? `\n\n${renderFormulaBox({
          title: language === "ko" ? "판단식" : "Decision",
          formula: language === "ko" ? "선택 = f(반복속도, 배포형태, 안정성, 유지보수)" : "choice = f(iteration, distribution, safety, maintenance)",
          notes: language === "ko"
            ? ["초기에는 반복속도 가중치가 크고, 제품화 단계에서는 배포/안정성 가중치가 커집니다."]
            : ["Early work weights iteration higher; productized CLIs weight distribution and safety higher."],
          width,
          theme: profile.theme,
          frame: profile.frame,
        })}`
      : "";
    return `${comparison}${formula}`;
  }

  if (renderMode === "paired-architecture") {
    const leftWidth = Math.max(42, Math.floor(width * 0.58));
    const rightWidth = Math.max(24, width - leftWidth - 3);
    const table = renderBoxTable({
      width: leftWidth,
      theme: profile.theme,
      frame: profile.frame,
      headers: language === "ko" ? ["구분", "내용"] : ["Part", "Content"],
      rows,
      rowDividers: true,
    });
    const flow = renderFlow({
      width: rightWidth,
      theme: profile.theme,
      frame: profile.frame,
      steps: architectureSteps(language, summary, evidence),
    });
    return renderResponsivePanels({ panels: [table, flow], width, gap: 3 });
  }

  if (renderMode === "flow") {
    return renderFlow({
      width,
      theme: profile.theme,
      frame: profile.frame,
      steps: language === "ko"
        ? ["결론", summary, evidence ?? "검증 정보 없음"]
        : ["Conclusion", summary, evidence ?? "No verification evidence found"],
    });
  }

  if (renderMode === "table") {
    return renderBoxTable({
      width,
      theme: profile.theme,
      frame: profile.frame,
      headers: language === "ko" ? ["구분", "내용"] : ["Part", "Content"],
      rows,
      rowDividers: true,
    });
  }

  if (renderMode === "formula") {
    return renderFormulaBox({
      title: language === "ko" ? "핵심식" : "Formula",
      formula: language === "ko" ? "설명 품질 = f(명확성, 구조, 근거, 다음 행동)" : "explanation_quality = f(clarity, structure, evidence, next_action)",
      notes: [summary],
      width,
      theme: profile.theme,
      frame: profile.frame,
    });
  }

  if (renderMode === "indexed") {
    const items = splitSentences(summary).length ? splitSentences(summary) : [summary];
    return renderIndexedList({
      items,
      width,
      theme: profile.theme,
      frame: profile.frame,
    });
  }

  const prefix = prosePrefix(language, intent, profile);
  const tldrLabel = language === "ko" ? "TLDR: " : "TLDR: ";
  if (renderMode === "tldr-prose") {
    return [
      colorize(tldrLabel, "heading", profile.theme) + tldr(summary, language),
      colorize(prefix, "heading", profile.theme) + summary,
      evidence,
    ].filter(Boolean).join("\n");
  }
  return [colorize(prefix, "heading", profile.theme) + summary, evidence].filter(Boolean).join("\n");
}
