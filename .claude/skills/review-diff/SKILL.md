---
name: review-diff
description: 'Run the relevant read-only reviewers in parallel on the current diff and produce a combined report.'
argument-hint: "[base branch, default main]"
disable-model-invocation: true
---

# Review current diff

Base: `$ARGUMENTS` if given, otherwise `main`.

1. Collect scope: `git diff --stat <base>...HEAD` and `git diff --stat` (uncommitted). If empty, say so and stop.
   Map changed paths to layers (AGENTS.md "Layers and owners"); find the task file and spec from the branch name if possible.
2. Launch in parallel (one message, several Agent calls), each with the diff range, task/spec paths and the layer:
   - code-reviewer: always;
   - dsp-verifier: changes in `crates/dsp`, `crates/analysis`, `crates/plugin-sdk`, `crates/plugins`, `reference/`;
   - security-reviewer: changes in `crates/server`, `crates/core` I/O or storage, filesystem paths, CLI plan approval;
   - eval-runner: changes in `crates/engine`, `crates/plugins`, `profiles/`.
3. Combined report (in Russian to the maintainer): findings grouped blocker → major → minor, deduplicated, each with
   reviewer, evidence (file:line / command). Also flag multi-layer diffs and executor edits to tests or `reference/`.
   End with a verdict: merge-ready or not. Fix nothing.
