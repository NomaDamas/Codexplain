# Explanation UX Research Notes

This note tracks which explanation methods are already represented in
Codexplain and which research-backed ideas should be discussed before adding.

## Sources Checked

- Vaswani et al., **Attention Is All You Need** (2017): establishes attention as
  a signal-routing mechanism and motivates sparse visual emphasis for the most
  decision-relevant tokens.
  Source: https://arxiv.org/abs/1706.03762
- Wei et al., **Chain-of-Thought Prompting Elicits Reasoning in Large Language
  Models** (2022): supports stepwise intermediate reasoning for complex tasks.
  Source: https://arxiv.org/abs/2201.11903
- Ouyang et al., **Training language models to follow instructions with human
  feedback** (2022): supports preference feedback loops and user-intent
  alignment.
  Source: https://arxiv.org/abs/2203.02155
- Bai et al., **Constitutional AI: Harmlessness from AI Feedback** (2022):
  supports critique/revision loops and principle-based self-improvement.
  Source: https://arxiv.org/abs/2212.08073
- Liang et al., **Can large language models provide useful feedback on research
  papers?** (2023): compares GPT-4 feedback with human reviewers across Nature
  family journals and ICLR, motivating rubric-like feedback surfaces.
  Source: https://arxiv.org/abs/2310.01783
- Zhu et al., **Using Reinforcement Learning to Train Large Language Models to
  Explain Human Decisions** (ICLR 2026): supports outcome-rewarded explanation
  generation.
  Source: https://openreview.net/forum?id=coJPBEZ9Te
- Nam et al., **Learning to summarize user information for personalized
  reinforcement learning from human feedback** (ICLR 2026): supports
  user-specific preference summaries for personalized response shaping.
  Source: https://openreview.net/forum?id=Ar078WR3um
- Lee et al., **The CoT Encyclopedia** (ICLR 2026): supports bottom-up
  categorization of reasoning strategies and contrastive rubrics.
  Source: https://mlanthology.org/iclr/2026/lee2026iclr-cot/
- Wu and Barez, **Query Circuits** (submitted to ICLR 2026): supports local,
  input-level explanation and faithfulness checks. Treat as a candidate idea,
  not a stable accepted baseline unless its venue status changes.
  Source: https://openreview.net/forum?id=DBoGyuahIX

## Already Implemented

- Attention-like salience: semantic ANSI roles highlight commands, paths,
  risks, artifacts, status, and key labels without coloring every word.
- Stepwise structure: indexed renderer turns "two paths", "process", and
  "steps" prompts into numbered sections.
- Feedback loop: `feedback` and `rlhf` commands store user ratings/comments for
  preference evolution.
- Principle-based safety: strict-output policy preserves exact JSON, code,
  diffs, logs, and test output.
- Personalized controls: profile stores theme, frame, 3-stage depth,
  abstraction level, UX density, and custom styles.
- Rubric-like feedback: progress, risk, confidence, decision matrix, and
  cause-effect reports expose the reason for an answer shape.
- Width-safe rendering: tables are layout-owned, visibly measured, wrapped, and
  checked by `quality-check`.

## Candidate Additions To Discuss

- Preference summary memory: derive a compact "user explanation preference
  summary" from accumulated feedback, inspired by personalized RLHF.
- Explanation rubric mining: cluster accepted/rejected answer styles into
  reusable rubrics, inspired by CoT Encyclopedia-style bottom-up categories.
- Outcome-based explanation score: after feedback, score explanations by
  clarity, usefulness, correctness preservation, and next-action quality.
- Faithfulness contract: add checks that rendered explanations do not invent
  facts outside the original answer unless marked as inference.
- Peer-review mode: add Nature/ICLR-style review cards with strengths,
  weaknesses, missing evidence, actionability, and confidence.

