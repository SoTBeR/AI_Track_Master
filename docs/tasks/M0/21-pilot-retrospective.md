---
id: M0-21
title: Record the pilot retrospective and M0 acceptance status (maintainer)
milestone: M0
layer: docs
status: todo
depends_on: [M0-10, M0-12, M0-13, M0-18, M0-19, M0-20]
spec: none
agents: [architect, code-reviewer]
---

# Record the pilot retrospective and M0 acceptance status (maintainer)

## Goal

Record what the pilot showed about the agent process before the rest is built on it, and confirm each
M0 acceptance item with evidence.

## Scope

- In: a short ADR `docs/adr/NNNN-agent-process-after-pilot.md` (or "no change" if the process holds):
  what worked, what failed (wrong tests, review misses, oversized diffs), process changes adopted;
  a checklist of M0 acceptance items with links to CI runs, M0-10 ADR and the release targets spec.
- Out: changing `.claude/` or `AGENTS.md` (follow-up tasks if needed).

## Acceptance criteria

- [ ] Pilot result recorded (PLAN section 14 item 7: "Результат пилота покажет, насколько процесс с агентами работает").
- [ ] M0 acceptance items each linked to evidence: green CI on three OSes; one binary builds; probe MCP server answered a
      tool call from a real client; targets recorded (PLAN section 9 M0 acceptance).
- [ ] `docs/adr/README.md` index updated if an ADR is written.

## Plan references

- docs/PLAN.md section 9 M0 acceptance
- docs/PLAN.md section 10 "Цикл задачи", "Антипаттерны"
- docs/PLAN.md section 14 item 7

## Notes

- **Maintainer:** provides the judgment on how the process worked; the architect writes it down.
