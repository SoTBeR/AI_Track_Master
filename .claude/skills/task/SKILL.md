---
name: task
description: 'Run the full cycle for one task file - branch, spec, reference, tests first, implementation, local checks, parallel reviews, fix rounds, signed commit. Never merges.'
argument-hint: <path to docs/tasks/<M#>/NN-slug.md>
disable-model-invocation: true
---

# Task cycle: $ARGUMENTS

Orchestrate from the main session. Each agent gets a full brief built from `${CLAUDE_SKILL_DIR}/briefing.md`
and returns a short summary. Do not paste full logs into the conversation; keep the context compact.

0. Read the task file. Check `depends_on` tasks are `done` or `in-review`; otherwise stop and tell the maintainer.
   Note the layer and the `agents` list. Ensure the working tree is clean.
1. **Branch**: `git switch -c task/<M#>-<NN>-<slug>` from `main` (or switch to it if it exists). Set task `status: in-progress`.
2. **Spec**: if `spec` is not `none` and the file is missing, delegate to **architect**. Show the maintainer any open questions it returns
   and wait if they block the work.
3. **Reference** (layers dsp, analysis, plugins, or if listed): delegate to **reference-author**.
4. **Tests first** (any code layer): delegate to **test-author**. It must report that the new tests fail. If they pass already, stop and ask.
5. **Implement**: delegate to the layer owner (AGENTS.md "Layers and owners"). Docs-only tasks: architect.
   If the executor reports a wrong test or reference, stop and ask the maintainer; do not let anyone "fix" the test silently.
6. **Local checks** in the main session, output trimmed (`2>&1 | tail -n 40`):
   fmt, clippy, tests, `cargo deny check`, pytest if `reference/` changed (commands in AGENTS.md; skip what does not exist yet and say so).
   Failures go back to the executor as in step 8.
7. **Reviews in parallel** (one message, several Agent calls):
   - code-reviewer: always;
   - dsp-verifier: layers dsp, analysis, plugins;
   - security-reviewer: server, core I/O, filesystem paths, CLI approval;
   - eval-runner: engine, plugins (once the eval harness exists).
   Also verify that the executor's diff did not touch `tests/`, `reference/` or other layers (`git diff --stat main...HEAD`).
8. **Fix rounds**: send blockers (and majors worth fixing) with evidence to the executor, then re-run steps 6-7 for the affected reviewers.
   At most 3 rounds. If blockers remain after round 3, stop and ask the maintainer.
9. **Commit**: update `CHANGELOG.md` if not done, set task `status: in-review`, then
   check `git status` (no stray or secret files), stage the task's files, and `git commit -s` with a Conventional Commit subject and a body containing:
   spec path, check results (one line each), metric deltas, reviewer verdicts, open questions.
   Do not push and do not merge into `main`.
10. Report to the maintainer (in Russian): what was done, checks, findings left (minor), open questions, the branch name.
