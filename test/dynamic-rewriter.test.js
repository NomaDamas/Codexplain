import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  extractProtectedElements,
  preservesProtectedElements,
  rewriteAnswerDynamic,
} from "../src/dynamic-rewriter.js";

describe("dynamic rewriter", () => {
  it("falls back to deterministic shaping when dynamic is off", async () => {
    const output = await rewriteAnswerDynamic({
      mode: "off",
      prompt: "현재 상태를 보기 쉽게 설명해줘",
      response: "구현은 완료됐습니다. 검증은 `npm test`로 통과했습니다.",
      env: {},
    });
    assert.match(output, /요약하면/);
    assert.match(output, /`npm test`/);
  });

  it("keeps strict artifacts unchanged", async () => {
    const output = await rewriteAnswerDynamic({
      mode: "auto",
      prompt: "return only valid JSON",
      response: '  {"ok":true}\n',
      env: {
        CODEXPLAIN_REWRITE_COMMAND: `${process.execPath} -e "process.stdin.resume(); process.stdin.on('end',()=>console.log('rewritten'))"`,
      },
    });
    assert.equal(output, '  {"ok":true}\n');
  });

  it("uses a custom command provider when configured", async () => {
    const output = await rewriteAnswerDynamic({
      mode: "auto",
      prompt: "설명해줘",
      response: "작업이 완료됐습니다.",
      env: {
        CODEXPLAIN_REWRITE_COMMAND: `${process.execPath} -e "process.stdin.resume(); process.stdin.on('end',()=>console.log('TLDR: 작업 완료'))"`,
      },
    });
    assert.equal(output, "TLDR: 작업 완료");
  });

  it("falls back when the command provider fails", async () => {
    const output = await rewriteAnswerDynamic({
      mode: "auto",
      prompt: "설명해줘",
      response: "작업이 완료됐습니다. 검증은 `npm test`로 했습니다.",
      env: {
        CODEXPLAIN_REWRITE_COMMAND: `${process.execPath} -e "process.exit(7)"`,
      },
    });
    assert.match(output, /요약하면/);
    assert.match(output, /`npm test`/);
  });

  it("falls back when the command provider times out", async () => {
    const output = await rewriteAnswerDynamic({
      mode: "auto",
      prompt: "설명해줘",
      response: "작업이 완료됐습니다.",
      timeoutMs: 20,
      env: {
        CODEXPLAIN_REWRITE_COMMAND: `${process.execPath} -e "setTimeout(()=>{}, 1000)"`,
      },
    });
    assert.match(output, /요약하면/);
  });

  it("falls back when a rewrite drops protected inline artifacts", async () => {
    const response = [
      "첫 번째 문장입니다.",
      "두 번째 문장입니다.",
      "세 번째 문장입니다.",
      "네 번째 문장입니다.",
      "검증은 `npm test`로 했고 자세한 내용은 src/dynamic-rewriter.js에 있습니다.",
    ].join(" ");
    const output = await rewriteAnswerDynamic({
      mode: "auto",
      prompt: "설명해줘",
      response,
      env: {
        CODEXPLAIN_REWRITE_COMMAND: `${process.execPath} -e "process.stdin.resume(); process.stdin.on('end',()=>console.log('검증 완료'))"`,
      },
    });
    assert.match(output, /요약하면/);
    assert.match(output, /`npm test`/);
    assert.match(output, /src\/dynamic-rewriter\.js/);
  });

  it("extracts and checks protected elements", () => {
    const original = "Run `npm test`, inspect src/index.js, and read https://example.com/docs.";
    const elements = extractProtectedElements(original);
    assert.deepEqual(elements, ["npm test", "https://example.com/docs", "src/index.js"]);
    assert.equal(
      preservesProtectedElements({
        original,
        rewritten: "Use `npm test`, src/index.js, and https://example.com/docs.",
      }),
      true,
    );
    assert.equal(
      preservesProtectedElements({
        original,
        rewritten: "Use the tests and docs.",
      }),
      false,
    );
  });
});
