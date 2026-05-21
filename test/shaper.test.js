import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { shapeAnswer } from "../src/shaper.js";

describe("shaper", () => {
  it("keeps strict JSON unchanged", () => {
    assert.equal(
      shapeAnswer({ prompt: "return only valid JSON", response: '{"ok":true}' }),
      '{"ok":true}',
    );
  });

  it("creates Korean concise prose", () => {
    const output = shapeAnswer({
      prompt: "현재 상태를 보기 쉽게 설명해줘",
      response: "구현은 완료됐습니다. 검증은 `npm test`로 통과했습니다. 남은 위험은 없습니다.",
    });
    assert.match(output, /요약하면/);
    assert.match(output, /TLDR/);
    assert.match(output, /`npm test`/);
  });

  it("creates connected flow when requested", () => {
    const output = shapeAnswer({
      prompt: "흐름도로 설명해줘",
      response: "입력을 분석하고 답변을 정리합니다. 검증은 `node --test`로 합니다.",
    });
    assert.match(output, /┌/);
    assert.match(output, /▼/);
  });

  it("uses deep easy explanation preferences", () => {
    const output = shapeAnswer({
      prompt: "초보도 이해하게 자세하지만 쉽게 설명해줘",
      response: "프로필을 읽고 답변 스타일을 정합니다. 검증은 `npm test`로 합니다.",
      uxProfile: { detail: "deep", style: "plain" },
      env: {},
    });
    assert.match(output, /핵심/);
    assert.match(output, /TLDR/);
    assert.match(output, /왜 중요한가/);
    assert.match(output, /`npm test`/);
  });

  it("pairs table and flow when requested", () => {
    const output = shapeAnswer({
      prompt: "표와 흐름을 좌우로 아키텍처 설명해줘",
      response: "CLI가 입력을 받고 policy가 보호합니다. 검증은 npm test로 합니다.",
      width: 110,
      uxProfile: { detail: "deep", style: "plain", theme: "none", frame: "unicode" },
      env: {},
    });
    assert.match(output, /┬/);
    assert.match(output, /TLDR/);
    assert.match(output, /입력/);
    assert.match(output.split("\n")[0], /┐\s+┌/);
  });

  it("stacks paired architecture panels when width is narrow", () => {
    const output = shapeAnswer({
      prompt: "표와 흐름을 좌우로 아키텍처 설명해줘",
      response: "CLI가 입력을 받고 policy가 보호합니다.",
      width: 55,
      uxProfile: { detail: "deep", style: "plain", theme: "none", frame: "unicode" },
      env: {},
    });
    assert.match(output, /┘\n\n┌/);
  });
});
