# Codexplain Local Adapter

This directory is project-local and Rust-only at runtime.

Use this adapter when a host can pipe a completed answer into a post-response command:

```bash
.codexplain/post-response --prompt "흐름도로 설명해줘"
```

Input may be plain text or JSON with `prompt` and `response` fields. The Rust core preserves exact JSON, code, diffs, logs, and test output when strict formatting matters.

Explanation depth uses 3-stage controls:

```text
explanationDepth light/standard/deep
architectureDepth overview/system/internals
abstractionLevel concrete/architecture/strategy
```

UX selection combines explicit rules, score thresholds, and optional planner hints through `CODEXPLAIN_UX_PLAN` or `CODEXPLAIN_UX_PLANNER_COMMAND`.

For chat or Markdown hosts that render inline HTML, pass `--chat-color` or
`--color-output html`. Codexplain converts ANSI theme colors into `<span>` tags
and leaves strict JSON, code, diffs, logs, and test output unchanged.
