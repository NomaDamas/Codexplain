import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { buildGuidance, classifyPrompt, shouldBackOff } from "../src/policy.js";

describe("policy", () => {
  it("classifies Korean flow prompts", () => {
    assert.deepEqual(classifyPrompt("단계별 흐름도로 보기 쉽게 설명해줘"), {
      language: "ko",
      intent: "explain",
      structure: "flow",
    });
  });

  it("backs off strict and machine-readable outputs", () => {
    assert.equal(shouldBackOff({ prompt: "return only valid JSON", response: "hi" }), true);
    assert.equal(shouldBackOff({ prompt: "explain", response: '{"ok":true}' }), true);
    assert.equal(shouldBackOff({ prompt: "explain", response: "PASS test/example.test.js" }), true);
  });

  it("builds guidance without exposing implementation labels", () => {
    const guidance = buildGuidance("현재 상태 보기 쉽게 설명해줘");
    assert.match(guidance, /Korean-first/);
    assert.doesNotMatch(guidance, /activated/i);
  });
});
