---
id: M0-13
title: Fix the render-time budget and maximum track count (maintainer)
milestone: M0
layer: docs
status: todo
depends_on: [M0-12]
spec: docs/spec/release-targets.md
agents: [architect, code-reviewer]
---

# Fix the render-time budget and maximum track count (maintainer)

## Goal

Record the target render time for a 10-minute track in maximum-quality mode and the maximum track count,
based on measurements on real projects, in the release targets spec.

## Scope

- In: maintainer provides measurements from real projects (track counts, durations, hardware); architect
  adds the budget, the maximum track count, the reference hardware and the measurement method to
  `docs/spec/release-targets.md`.
- Out: benchmarks of our renderer (it does not exist yet); performance work (M9).

## Acceptance criteria

- [ ] Render-time budget for a 10-minute track and maximum track count are recorded with the measurement basis
      (PLAN section 13: "фиксируются в M0 по измерениям на реальных проектах"; section 14 item 6).
- [ ] The track limit is consistent with M8 acceptance (a project of 32 or more tracks renders within the M0
      budget) and the budget with M9 acceptance (10-minute track in maximum-quality mode within target time)
      (PLAN section 9 M8, M9).
- [ ] Reference hardware for the budget is stated, since time depends on it.

## Plan references

- docs/PLAN.md section 1 (row "Приоритет": quality over speed)
- docs/PLAN.md section 9 M8 and M9 acceptance
- docs/PLAN.md section 13, section 14 item 6

## Notes

## Maintainer decisions (2026-10-27)

Our renderer does not exist yet, so the numbers are provisional targets on a defined worst-case benchmark,
not measurements; they are refined in M9 by ADR. The budget is a performance target, not a cut-off: when the
engine hits its iteration/time limits (PLAN section 3 stage 6, section 6 limits), it returns the best result
that passed "do no harm" (or the baseline) with a warning, never an unverified one.

- **Reference hardware:** Intel Core i7 12th generation, 32 GB RAM, Windows 11.
- **Maximum track count:** 64.
- **Mastering benchmark (worst case):** 10-minute stereo track, 48 kHz, maximum-quality profile, every chain
  block enabled, maximum optimization iterations, 8x oversampling on nonlinear blocks.
  Budget: ≤ 10 min (not slower than real time). Variant at 96 kHz: ≤ 20 min.
- **Mixing benchmark (worst case, for M8):** 10-minute project with 64 tracks, 48 kHz, full mix chain plus
  mastering. Budget: ≤ 20 min. M8 acceptance (32+ tracks within budget) is checked against this.
- Record these in `docs/spec/release-targets.md` with the benchmark definitions above.
