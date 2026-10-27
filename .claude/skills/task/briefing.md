# Agent briefs for /task

A subagent starts with no knowledge of this conversation. Each brief must be self-contained.

## Common header (every brief)

```
Task file: <path>   Layer: <layer>   Branch: <branch>
Spec: <path or none>
Plan sections: docs/PLAN.md §<n> ...
Read AGENTS.md first. Stay inside: <allowed paths>.
Acceptance criteria:
- <copied from the task/spec>
Return: summary, files changed, check results (one line each), open questions.
```

## Per agent

- **architect (spec)**: "Write `<spec path>` from docs/templates/spec.md for this task. Public API must be exact enough for tests.
  Name the independent reference. Add an ADR if the task makes an architectural decision."
- **reference-author**: "Implement the reference for `<algorithm>` in `reference/<area>/`, with pytest against `<standard/values>`.
  Do not read `crates/`. Provide fixture generation for test-author at `<fixture path>`."
- **test-author**: "Write tests for every acceptance criterion against the spec's Public API, using fixtures from `<path>`.
  Do not read implementation code. Run them and show they fail for the right reason."
- **executor** (dsp-engineer / engine-engineer / platform-engineer): "Make the tests in `<test paths>` pass. Do not modify tests,
  fixtures or `reference/`. No new dependencies (approved crates: `<list from ADRs or none>`). Update CHANGELOG. Run local checks."
- **fix round**: "Round <k>/3. Fix these findings: <level, claim, evidence>. Same constraints as before. Report what changed per finding."
- **reviewers** (code-reviewer, dsp-verifier, security-reviewer, eval-runner): "Review `git diff main...HEAD` for this task.
  Findings with level blocker|major|minor and evidence. Do not edit anything."

## Executor and reviewers by layer

| Layer | Executor | Reference | Reviewers (besides code-reviewer) |
|---|---|---|---|
| dsp | dsp-engineer | yes | dsp-verifier |
| analysis | dsp-engineer | yes | dsp-verifier |
| plugins | dsp-engineer | yes | dsp-verifier, eval-runner |
| engine | engine-engineer | if new math | eval-runner (+ dsp-verifier if new math) |
| platform | platform-engineer | no | security-reviewer if server, I/O, paths or CLI approval |
| docs | architect | no | none (code-reviewer checks consistency with the plan) |
| reference | reference-author | n/a | dsp-verifier |

A new dependency anywhere: run `/add-dep` first; the dependency must be in an accepted ADR before the executor uses it.
