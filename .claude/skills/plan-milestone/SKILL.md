---
name: plan-milestone
description: 'Break a milestone from docs/PLAN.md section 9 into ordered task files in docs/tasks/<M#>/ via the architect agent.'
argument-hint: <M#>
disable-model-invocation: true
---

# Plan milestone $0

1. Check `docs/tasks/$0/`. If task files already exist, show them and ask the maintainer whether to extend or replan; do not overwrite.
2. Delegate to the **architect** agent with this brief (fill in the milestone):
   - Read `AGENTS.md`, `docs/PLAN.md` section 9 milestone $0 (scope and acceptance), plus sections 10 and 14 when $0 is M0,
     and any section the milestone depends on.
   - Create ordered tasks `docs/tasks/$0/NN-<kebab-slug>.md` from `docs/templates/task.md`, `NN` = execution order from `01`.
   - Each task: goal, exactly one layer (see AGENTS.md "Layers and owners"), `depends_on`, acceptance criteria quoted or derived
     from the plan with section references, `spec` path (or `none`), `agents` needed (executor by layer; reference-author and
     dsp-verifier for dsp/analysis/plugins; security-reviewer for server/I/O/paths; eval-runner for engine/plugins; code-reviewer always).
   - Small tasks (one branch, one reviewable diff). Together they must cover every deliverable and acceptance item of $0, no more.
   - Items that need the maintainer (choices, accounts, real MCP clients, listeners, recordings) become tasks or notes marked "maintainer".
   - Do not write specs or ADRs yet; `/task` does that.
   - Return: a table `NN | slug | layer | depends_on | agents`, a coverage map (plan item → task), open questions.
3. Show the maintainer the table, coverage map and open questions. Do not start any task.
