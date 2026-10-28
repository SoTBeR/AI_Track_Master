@AGENTS.md

Project plan: `docs/PLAN.md` (Russian). Talk to the maintainer in Russian; write all repository files in English.

## Agents (`.claude/agents/`)

- architect: splits milestones into tasks, writes specs and ADRs; edits only `docs/`.
- reference-author: independent Python references in `reference/`; never reads Rust code.
- test-author: tests first, from spec and references; never reads the implementation.
- dsp-engineer: implements `dsp`, `analysis`, `plugin-sdk`, `plugins`.
- engine-engineer: implements `engine` and `profiles/`.
- platform-engineer: implements `core`, `server`, `cli`, eval harness, CI, distribution.
- dsp-verifier: independently checks DSP math; read-only.
- code-reviewer: reviews a diff against spec and AGENTS.md; read-only.
- security-reviewer: write paths, input validation, MCP tools, limits; read-only.
- eval-runner: golden set and eval scenarios vs dev; read-only.
- dependency-auditor: license, maintenance, alternatives for a crate; drafts an ADR.

## Skills

- `/plan-milestone <M#>`: architect breaks a milestone into `docs/tasks/<M#>/`.
- `/task <path>`: full task cycle: branch, spec, reference, tests, implementation, checks, reviews, commit.
- `/new-plugin <Name> <category>`: scaffold a plugin from templates.
- `/adr <title>`: new numbered ADR.
- `/add-dep <crate>`: dependency audit and ADR draft.
- `/review-diff`: run the relevant reviewers on the current diff.
- `/eval`: golden set and eval scenarios compared with dev.
- `/progress`: milestone progress and next step.
- Knowledge (preloaded into agents): `dsp-rules`, `mastering-domain`, `mcp-design`, `quality-gates`.

## Main loop

`/plan-milestone M#` → `/task docs/tasks/M#/NN-slug.md` (repeat per task; merged into `dev` and pushed) →
maintainer reviews and merges a pull request `dev` -> `main`.

Orchestration happens in the main session. Subagents get the whole task in one brief
and return short summaries. Never give a subagent the `Agent` tool. Keep the main context compact.
