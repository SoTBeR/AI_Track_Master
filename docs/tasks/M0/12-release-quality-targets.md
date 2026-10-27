---
id: M0-12
title: Record the release quality targets
milestone: M0
layer: docs
status: todo
depends_on: []
spec: docs/spec/release-targets.md
agents: [architect, code-reviewer]
---

# Record the release quality targets

## Goal

Write down the release quality targets fixed in M0 so M6 and the M7 quality gate measure against them.

## Scope

- In: `docs/spec/release-targets.md` with the quality targets, how each is measured (which verification level
  from PLAN section 8), and placeholders for the render-time budget and maximum track count filled by M0-13.
- Out: the render budget values (M0-13); listening test protocol details (M7); golden-set tolerances (later).

## Acceptance criteria

- [ ] Records, as in PLAN section 8 "Целевые показатели релиза": integrated loudness within ±0.5 LU of target;
      true peak not above target in all formats, including lossy; no golden-set track worse than the baseline
      (loudness normalization + limiter) in blind evaluation; statistically significant preference over the
      baseline; aim: not worse than external automatic mastering systems.
- [ ] States that the targets are fixed in M0 and refined in M7, and that M7 acceptance checks them (PLAN section 9 M7).
- [ ] Each target names its measurement level (PLAN section 8 levels 2, 4, 6).
- [ ] Nothing beyond PLAN section 8 is added as a target; ambiguities (for example, statistical test and
      significance level) are listed as open questions.

## Plan references

- docs/PLAN.md section 8 "Метрики и оценка качества"
- docs/PLAN.md section 9 M0 acceptance ("целевые показатели записаны") and M7
- docs/PLAN.md section 14 item 6
