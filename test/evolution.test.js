import assert from "node:assert/strict";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import {
  buildUxContract,
  buildRlhfSummary,
  evolveUxProfileFromFeedback,
  loadProjectUxProfile,
  resolveUxProfile,
  saveProjectUxProfile,
} from "../src/evolution.js";

describe("evolution profile", () => {
  it("infers easy detailed explanations from prompt signals", () => {
    const profile = resolveUxProfile({
      prompt: "초보도 이해하게 자세하지만 쉽게 설명해줘",
      profile: { detail: "brief", style: "technical" },
      env: {},
    });
    assert.equal(profile.detail, "deep");
    assert.equal(profile.style, "plain");
  });

  it("applies terminal theme from environment", () => {
    const profile = resolveUxProfile({
      prompt: "설명해줘",
      profile: { theme: "none" },
      env: { CLAUDEX_THEME: "forest" },
    });
    assert.equal(profile.theme, "forest");
  });

  it("applies ASCII frame from environment", () => {
    const profile = resolveUxProfile({
      prompt: "아키텍처로 설명해줘",
      profile: { frame: "unicode" },
      env: { CLAUDEX_FRAME: "ascii" },
    });
    assert.equal(profile.frame, "ascii");
  });

  it("evolves profile from feedback without storing raw answer text", () => {
    const profile = evolveUxProfileFromFeedback(
      { detail: "balanced", style: "technical", feedback: { signals: [] } },
      { rating: 2, comment: "너무 어렵고 설명이 부족해. 더 쉽게 자세히." },
    );
    assert.equal(profile.detail, "deep");
    assert.equal(profile.style, "plain");
    assert.equal(profile.feedback.negative, 1);
    assert.equal(profile.feedback.rewardScore, -1);
    assert.equal(profile.feedback.signals[0].signal, "needs-more-detail");
    assert.doesNotMatch(JSON.stringify(profile), /설명이 부족/);
  });

  it("loads and saves project-local ux profile", async () => {
    const cwd = await mkdtemp(join(tmpdir(), "claudex-ux-"));
    try {
      await saveProjectUxProfile({ detail: "deep", style: "tutorial" }, { cwd });
      const profile = await loadProjectUxProfile({ cwd });
      assert.equal(profile.detail, "deep");
      assert.equal(profile.style, "tutorial");
      assert.match(buildUxContract(profile), /Adaptive explanation contract/);
      assert.match(buildRlhfSummary(profile), /Preference reward/);
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("includes TLDR in the default explanation moves", () => {
    const profile = resolveUxProfile({ prompt: "설명해줘", env: {} });
    assert.match(profile.explanationMoves.join(","), /tldr/);
    assert.match(buildUxContract(profile), /TLDR/);
  });
});
