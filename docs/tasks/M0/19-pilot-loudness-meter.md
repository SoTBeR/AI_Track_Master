---
id: M0-19
title: Pilot - first LoudnessMeter in analysis (LUFS, LRA, true peak)
milestone: M0
layer: analysis
status: todo
depends_on: [M0-07, M0-16]
spec: docs/spec/loudness-meter.md
agents: [architect, reference-author, test-author, dsp-engineer, dependency-auditor, dsp-verifier, code-reviewer]
---

# Pilot - first LoudnessMeter in analysis (LUFS, LRA, true peak)

## Goal

Second half of the pilot task: the first LoudnessMeter analyzer in `crates/analysis`, verified against the
independent reference set from M0-16, run through the full spec → test → implementation → review cycle.

## Scope

- In: spec `docs/spec/loudness-meter.md`; `crates/analysis` crate with `#![forbid(unsafe_code)]`; integrated,
  short-term and momentary loudness, LRA and true peak on 64-bit float buffers, using ebur128 after
  dependency-auditor review and ADR; tests against the M0-16 expected values, plus NaN/Inf input, block-size
  independence and sample-rate coverage; `CHANGELOG.md` entry.
- Out: the LoudnessMeter `Plugin` wrapper (needs plugin-sdk, M2); the full EBU 3341/3342 suite (M1); file reading (tests use the reference
  signals per the spec); other analyzers (M1).

## Acceptance criteria

- [ ] Measures LUFS, LRA and true peak per BS.1770 (PLAN section 4 row "Громкость"; section 7 "Состав плагинов").
- [ ] Results match the M0-16 expected values within the tolerances cited from EBU Tech 3341/3342
      (independent reference: ITU-R BS.1770, EBU Tech 3341, EBU Tech 3342, cross-checked with pyloudnorm)
      for this M0 subset (stereo, analytically generated cases only):
      - EBU Tech 3341, integrated loudness: cases 1, 2, 3, 4, 5 (stereo 1 kHz sine sequences;
        expected -23.0 / -33.0 LUFS per the document, tolerance as published).
      - EBU Tech 3341, true peak: cases 15, 16, 17, 18, 19 (stereo sines at fs/4, fs/6, fs/8 with phase offsets;
        expected -6.0 dBTP for 15-18 and +3.0 dBTP for 19, tolerance +0.2/-0.4 dB per the document).
      - EBU Tech 3342, LRA: cases 1, 2, 3, 4 (stereo 1 kHz sine level sequences; expected LRA per the document,
        tolerance as published).
      The spec author confirms each case number, signal description, expected value and tolerance against the
      current Tech 3341 / Tech 3342 revision and cites the clause; any mismatch with this list is reported to the
      maintainer, not silently substituted.
- [ ] The full Tech 3341 / 3342 suite (including multichannel, short-term/momentary cases and cases that need
      EBU's original files) is not required here; it remains the M1 acceptance (PLAN section 9 M1).
- [ ] Result does not depend on block size; NaN/Inf input is rejected or reported, never propagated silently
      (PLAN section 7 "Правила DSP-кода"; section 8 level 1).
- [ ] Sample rates 44.1-192 kHz supported (PLAN section 7).
- [ ] dsp-verifier reports no blockers; the maintainer personally reviews true peak (PLAN section 10
      "Что проверяете лично").
- [ ] Tests written first, green on all three OSes in CI; ADR accepted for ebur128.
- [ ] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 4 (row "Громкость")
- docs/PLAN.md section 7 "Правила DSP-кода", "Состав плагинов"
- docs/PLAN.md section 9 M1 acceptance
- docs/PLAN.md section 10 "Цикл задачи", "Что проверяете лично"
- docs/PLAN.md section 14 item 7

## Notes

- **Maintainer:** personal review of true-peak math is required by the plan.
- Maintainer decision: the pilot stays in M0 (PLAN section 14 item 7) and is accepted on the representative
  subset above; the full EBU 3341/3342 pass is M1 acceptance.
- Maintainer decision: this task delivers a plain analysis function/type in `crates/analysis` (no `Plugin`
  trait, no parameter schema). The LoudnessMeter `Plugin` wrapper (PLAN section 7) is built in M2, after
  plugin-sdk exists, and wraps this API without changing its results.
- The subset values above come from a secondary listing of Tech 3341 v4; they are a checklist for the spec,
  not a substitute for the document.
