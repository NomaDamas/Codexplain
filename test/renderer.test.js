import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  renderBoxTable,
  renderFlow,
  renderFormulaBox,
  renderIndexedList,
  renderProsConsPanels,
  renderResponsivePanels,
} from "../src/renderer.js";
import { measureWidth } from "../src/text-width.js";
import { stripAnsi } from "../src/theme.js";

describe("renderer", () => {
  it("renders connected Hangul tables", () => {
    const output = renderBoxTable({
      headers: ["영역", "검증"],
      rows: [["계약", "통과"]],
      width: 40,
    });
    assert.match(output, /┌/);
    assert.match(output, /┬/);
    assert.match(output, /┼/);
    assert.match(output, /└/);
    assert.doesNotMatch(output, /-{4,}|={4,}|ㅡ{4,}/);
  });

  it("wraps long cells inside Unicode box tables", () => {
    const output = renderBoxTable({
      headers: ["구분", "내용"],
      rows: [["핵심", "아키텍처 설명이 길어져도 유니코드 박스 표 안에서 줄바꿈됩니다."]],
      width: 42,
    });
    assert.match(output, /┌/);
    assert.match(output, /├/);
    assert.match(output, /└/);
    assert.doesNotMatch(output, /^- /m);
  });

  it("can draw dividers between body rows", () => {
    const output = renderBoxTable({
      headers: ["계층", "역할"],
      rows: [
        ["CLI", "명령 입구"],
        ["Policy", "strict 출력 보호"],
        ["Renderer", "표와 흐름도 출력"],
      ],
      width: 60,
      rowDividers: true,
    });
    assert.equal((output.match(/├/g) ?? []).length, 3);
    assert.match(output, /Policy/);
  });

  it("highlights semantic labels when a color theme is enabled", () => {
    const output = renderBoxTable({
      headers: ["구분", "내용"],
      rows: [["장점", "읽기 쉬움"], ["단점", "색상 미지원 터미널 주의"]],
      width: 60,
      theme: "ocean",
      rowDividers: true,
    });
    assert.match(output, /\u001b\[1;32m장점/);
    assert.match(output, /\u001b\[1;33m단점/);
  });

  it("renders vertical flow with stable connector symbols", () => {
    const output = renderFlow({ steps: ["요청 이해", "핵심 정리", "검증 보존"], width: 40 });
    assert.match(output, /┬/);
    assert.match(output, /┴/);
    assert.match(output, /▼/);
    assert.doesNotMatch(output, /-{4,}|={4,}|ㅡ{4,}/);
  });

  it("can colorize terminal tables and preserve readable text", () => {
    const output = renderBoxTable({
      headers: ["영역", "검증"],
      rows: [["계약", "통과"]],
      width: 40,
      theme: "ocean",
    });
    assert.match(output, /\u001b\[/);
    assert.match(stripAnsi(output), /┌/);
    assert.match(stripAnsi(output), /계약/);
  });

  it("renders ASCII tables and flows when requested", () => {
    const table = renderBoxTable({
      headers: ["Part", "Role"],
      rows: [["CLI", "Input"]],
      width: 40,
      frame: "ascii",
    });
    const flow = renderFlow({ steps: ["CLI", "Policy", "Shaper"], width: 40, frame: "ascii" });
    assert.match(table, /\+[-+]+\+/);
    assert.match(table, /\| CLI/);
    assert.match(flow, /v/);
    assert.doesNotMatch(`${table}\n${flow}`, /┌|┬|┐|│|└|┴|┘|▼/);
  });

  it("wraps long cells instead of falling back from ASCII tables", () => {
    const table = renderBoxTable({
      headers: ["Part", "Role"],
      rows: [["Renderer", "Keeps a long architecture explanation inside an ASCII table."]],
      width: 36,
      frame: "ascii",
    });
    assert.match(table, /\+[-+]+\+/);
    assert.match(table, /\| Renderer/);
    assert.match(table, /architec/);
    assert.match(table, /ture explanation/);
    assert.doesNotMatch(table, /^- /m);
  });

  it("places panels side by side when there is enough width", () => {
    const output = renderResponsivePanels({
      width: 40,
      gap: 2,
      panels: ["┌─┐\n│A│\n└─┘", "┌─┐\n│B│\n└─┘"],
    });
    assert.match(output.split("\n")[0], /┌─┐  ┌─┐/);
  });

  it("stacks panels when width is narrow", () => {
    const output = renderResponsivePanels({
      width: 20,
      gap: 2,
      panels: ["┌────────┐\n│A       │\n└────────┘", "┌────────┐\n│B       │\n└────────┘"],
    });
    assert.match(output, /└────────┘\n\n┌────────┐/);
  });

  it("renders pros and cons as responsive paired panels", () => {
    const output = renderProsConsPanels({
      width: 100,
      left: { title: "JS", pros: ["빠른 수정"], cons: ["런타임 필요"], bestFor: "실험" },
      right: { title: "Rust", pros: ["단일 바이너리"], cons: ["초기 비용"], bestFor: "제품화" },
    });
    assert.match(output, /장점/);
    assert.match(output, /단점/);
    assert.match(output, /├/);
    assert.match(output.split("\n")[0], /┐\s+┌/);
  });

  it("renders formula boxes", () => {
    const output = renderFormulaBox({
      title: "판단식",
      formula: "선택 = f(반복속도, 안정성)",
      notes: ["상황에 따라 가중치가 바뀝니다."],
      width: 70,
    });
    assert.match(output, /수식\/의미/);
    assert.match(output, /선택 = f/);
  });

  it("renders indexed explanations with colored numbers", () => {
    const output = renderIndexedList({
      items: ["첫 번째 설명", "두 번째 설명"],
      width: 60,
      theme: "sunset",
    });
    assert.match(output, /\u001b\[1;38;5;196m1\./);
    assert.match(stripAnsi(output), /1\. │ 첫 번째 설명/);
  });

  it("measures Hangul as wide characters", () => {
    assert.equal(measureWidth("abc 한글"), 8);
  });
});
