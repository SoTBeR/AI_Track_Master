# Spec: Release quality targets

- Status: draft
- Layer: docs
- Task: docs/tasks/M0/12-release-quality-targets.md (quality targets),
  docs/tasks/M0/13-render-budget-track-limit.md (performance targets)
- Plan: docs/PLAN.md section 1 (row "Приоритет"), section 3 stage 6, section 6 ("Ограничения
  безопасности"), section 8 ("Целевые показатели релиза"), section 9 (M0, M6, M7, M8, M9),
  section 13, section 14 item 6

## What

The release quality targets for 1.0, as fixed in M0. For each target this spec states what is
measured, at which verification level of PLAN section 8 it is measured, on which material, and at
which milestone it is checked. The targets are fixed in M0 and refined in M7; M7 acceptance
("quality gate") checks that RT-1 to RT-4 are met. RT-5 is an aim and is reported, not gating,
pending open question Q7. The performance targets PT-1 to PT-4 (render-time budget, maximum
track count, reference hardware) are provisional, refined in M9, and checked by M8 and M9
acceptance (see "Performance targets").

This spec defines targets, not tools. It adds no requirement beyond PLAN section 8. Numbers the plan
does not give (statistical test, significance level, listener count, measurement tolerances) are
not invented here: they are marked "fixed in M7" or listed under open questions.

## Why

The product has no human in the loop at run time (PLAN section 8, intro), so quality must be
established during development against fixed, measurable targets. M0 acceptance requires that the
release targets are written down ("целевые показатели записаны", PLAN section 9 M0; section 14
item 6). M6 measures the objective part against them, and the M7 quality gate requires RT-1 to
RT-4 to be met before mixing (M8) starts (PLAN section 9 intro: "сведение начинается только после
ворот качества M7"); RT-5 is reported pending Q7.

## Public API

None. This spec defines no Rust API, MCP schema or CLI surface. It is the contract that the eval
harness (golden set, level 4), the delivery checks and the M7 listening-test protocol measure
against. Target identifiers `RT-1` ... `RT-5` below are document labels for traceability in tasks,
eval reports and the M7 results; they are not a schema.

## Terms

- **Target** (loudness, true peak): the delivery target taken from the selected platform profile in
  `profiles/` (PLAN section 3 stage 4, "Профили целей"). This spec gives no platform values; they are
  versioned data, verified against current platform documentation when the profiles are written
  (PLAN section 8, last paragraph; section 13).
- **Baseline**: loudness normalization plus limiter, at the same delivery target; our result is
  compared with it at matched loudness (PLAN section 3 stage 7; section 8 level 6).
- **All formats** (draft rule, see Q9): for golden-set runs, the format set is the delivery
  formats listed in the selected platform profile (the "delivery formats" field defined by
  docs/tasks/M0/11-target-profile-format.md, spec docs/spec/target-profiles.md), not whatever a
  single run happens to request. If the profile lists no lossy format (e.g. CD), one lossy format
  (MP3, AAC or OGG, written via ffmpeg, PLAN section 13) is added on top for golden-set runs, so
  RT-1 and RT-2 cannot be passed by requesting WAV only. A lossy output is measured after encode
  and decode, as the engine's own delivery stage does (PLAN section 3 stage 8; section 8 level 2).
- **Golden set**: the corpus manifests of PLAN section 8 level 4 (synthetic, freely licensed, own
  material, AI generations, home mixes, problem input).
- **Blind evaluation**: the MUSHRA-like test of PLAN section 8 level 6: hidden reference and anchor,
  all variants loudness-matched, statistical significance tested, listener consistency controlled.

## Behavior

See "Targets" below: each target states its measurement, material, check milestone and pass rule.

## Targets

### RT-1. Integrated loudness within ±0.5 LU of target

- Statement: the integrated loudness of every delivered output is within ±0.5 LU of the profile's
  loudness target.
- Measurement: level 2 (measurements), integrated loudness per ITU-R BS.1770 as implemented by the
  LoudnessMeter (M1, validated against EBU Tech 3341 / 3342 signals).
- Material: every golden-set track (level 4), for every format in the golden-set
  format set (see Terms, "All formats"; open question Q1 on lossy formats).
- Checked: M6 acceptance ("на золотом наборе все треки попадают в цели громкости ... во всех
  форматах") and M7 quality gate.
- Pass: |measured − target| ≤ 0.5 LU for every track and format. No track is exempt.

### RT-2. True peak not above target in all formats, including lossy

- Statement: the true peak of every delivered output, in every format including lossy formats
  after encode and decode, is not above the profile's true-peak target.
- Measurement: level 2 (measurements), true peak per ITU-R BS.1770 (true-peak annex) as
  implemented by the LoudnessMeter; for lossy formats, measured on the decoded signal (PLAN
  section 3 stage 8; section 8 level 2 "в том числе после кодирования в форматы с потерями").
- Material: every golden-set track (level 4), every format in the golden-set format set
  (see Terms, "All formats").
- Checked: M6 acceptance ("... и true peak во всех форматах") and M7 quality gate. M2 acceptance
  (limiter does not exceed the true-peak target on the corpus) is an earlier component-level check,
  not this release target.
- Pass: measured true peak ≤ target for every track and format. No tolerance above the target is
  granted by the plan (see open question Q2 on meter tolerance).

### RT-3. No golden-set track worse than the baseline in blind evaluation

- Statement: for no golden-set track is our result rated worse than the baseline in blind
  evaluation.
- Measurement: level 6 (blind listening), result versus baseline, loudness-matched.
- Material: golden-set tracks (level 4); selection for listening is an open question (Q4).
- Checked: M7 quality gate. M6 checks the objective counterpart only: "ни один результат не хуже
  базового варианта по объективным метрикам" (levels 2 and 4), which is the engine's "do no harm"
  criterion and is a precondition, not a substitute, for RT-3.
- Pass: definition of "worse" for a single track (statistical criterion, threshold) is fixed in M7
  (Q3, Q5).
- Note: when the engine falls back to the baseline (PLAN section 3 stage 7), the delivered output
  is the baseline itself, so the track passes RT-3 by definition. Whether fallbacks count against
  RT-4, and whether a fallback rate is reported, is open question Q6.

### RT-4. Statistically significant preference over the baseline

- Statement: across the blind evaluation, listeners prefer our result over the baseline with
  statistical significance.
- Measurement: level 6 (blind listening), significance tested statistically, listener consistency
  controlled.
- Material: the golden-set tracks used in the blind evaluation.
- Checked: M7 quality gate.
- Pass: statistical test, significance level, minimum listeners and listener-screening rule are
  fixed in M7 (Q3, Q5).

### RT-5. Aim: not worse than external automatic mastering systems

- Statement: as an aim ("ориентир"), our result is not worse than other automatic mastering systems
  used as external references.
- Measurement: level 6 (blind listening), our result versus external systems, loudness-matched.
- Checked: M7 (comparison with external systems is in M7 scope). Whether RT-5 is a gate condition
  or only reported is open question Q7; the plan words it as an aim, so this spec treats it as
  reported, not blocking, until M7 decides.

## Measurement levels summary

| Target | Level (PLAN section 8) | Objective check | Blind check |
|---|---|---|---|
| RT-1 loudness ±0.5 LU | 2 on 4 | M6, M7 | n/a |
| RT-2 true peak, all formats | 2 on 4 | M6, M7 | n/a |
| RT-3 none worse than baseline | 6 on 4 (objective counterpart: 2, 4) | M6 (objective) | M7 |
| RT-4 significant preference | 6 | n/a | M7 |
| RT-5 aim vs external systems | 6 | n/a | M7 |

Level 5 (agent eval runs) is not a separate release target; see PLAN section 8 level 5.

## Refined in M7

PLAN section 8 states the targets are refined in M7, and M7 calibrates objective weights and
"do no harm" thresholds from listening results. Refinement happens via ADR in M7 (PLAN section 13
allows changes by ADR), with this spec updated through a dedicated task. M7 fixes:

- the blind-test protocol: listeners, screening and consistency control, stimuli, loudness-matching
  method, anchor, and which golden-set tracks are listened to;
- the statistical test and significance level for RT-3 and RT-4, and the per-track definition of
  "worse" for RT-3;
- the status of RT-5 (gate or reported aim);
- any change to the RT-1 and RT-2 tolerances, with reasons.

Recalibrating the "do no harm" thresholds is allowed; no path may bypass the "do no harm" check
(AGENTS.md). M7 results must be documented and reproducible (PLAN section 9 M7 acceptance).

## Performance targets

Recorded by docs/tasks/M0/13-render-budget-track-limit.md from the maintainer decisions of
2026-10-27 (PLAN section 13: "Целевое время рендера и максимальное число дорожек фиксируются в M0";
section 14 item 6). Labels `PT-1` ... `PT-4` are document labels for traceability, not a schema.

### Basis and status

- **Provisional.** PLAN section 13 asks for values based on measurements of real projects. Our
  renderer does not exist yet, so these numbers are provisional targets on a defined worst-case
  benchmark, not measurements. They are refined in M9 by ADR (PLAN section 13 allows changes by
  ADR), with this spec updated through a dedicated task.
- **Quality over speed.** PLAN section 1 (row "Приоритет"): offline processing, render time is
  secondary. The budget never justifies a lower-quality default.
- **A target, not a cut-off.** The budget is a performance target for development and acceptance.
  It is not a time limit that truncates a job. The engine's own limits (optimization loop bounded by
  iterations and time, PLAN section 3 stage 6; execution-time and render-count limits, PLAN section
  6 "Ограничения безопасности") are separate. When the engine hits those limits it returns the best
  result that passed "do no harm", or the baseline, with a warning in the report; it never returns
  an unverified result and never presents a partial result as finished (AGENTS.md). Exceeding the
  budget is a performance defect to be fixed, not a reason to skip any check.

### PT-1. Reference hardware

Intel Core i7 12th generation, 32 GB RAM, Windows 11. All budgets below are wall-clock times on
this machine. Budget compliance is measured only on the reference machine (see "How checked").

### PT-2. Maximum track count

64 tracks per session. This is the largest session size for which the budgets apply and which the
mixing benchmark (PT-4) exercises. Behavior for sessions above 64 tracks is open question Q11.

### PT-3. Mastering benchmark (worst case)

- Input: one 10-minute stereo track (a one-track session).
- Settings: maximum-quality profile; every chain block of PLAN section 3 stage 5 enabled (no block
  skipped for lack of a reason); maximum number of optimization iterations; 8x oversampling on all
  nonlinear blocks (upper end of the 4-8x range in PLAN section 7).
- How "every block enabled" is reached: through benchmark material that gives the engine a reason
  for every block, or through a test-only fixture in the benchmark harness. Never through a
  user-facing CLI flag, MCP parameter or config option: engine planning stays authoritative (PLAN
  section 3 stage 5, "Блоки, для которых нет причин, не включаются"), and the benchmark job still
  goes through the "do no harm" check. Which of the two is used is part of open question Q12.
- Budgets:

| Variant | Sample rate | Budget (wall clock) |
|---|---|---|
| PT-3a | 48 kHz | ≤ 10 min (not slower than real time) |
| PT-3b | 96 kHz | ≤ 20 min |

### PT-4. Mixing benchmark (worst case, M8)

- Input: a 10-minute project with 64 tracks (PT-2), 48 kHz.
- Settings: the full mixing chain (defined in M8) plus mastering. Mastering "at the PT-3 settings"
  (maximum quality, every block, maximum iterations, 8x oversampling) is this spec's interpretation
  of the maintainer decision "full mix chain plus mastering", to be confirmed (open question Q12).
- Budget: mix plus mastering ≤ 20 min (wall clock).

### Measurement method (draft)

- Time is the wall-clock duration of one complete engine job, from job start through delivery and
  the final report, run from the CLI without an agent (so model and client latency are excluded;
  same engine path as MCP, AGENTS.md "Determinism").
- The run is a release build on the reference machine with no other heavy load.
- Benchmark material, delivery format set, number of repetitions and the statistic used (single
  run, median, worst) are not fixed by the plan or the maintainer decisions: open questions Q12,
  Q13.

### How checked

- **M8 acceptance** (PLAN section 9 M8): "проект из 32 и более дорожек рендерится в пределах
  бюджета времени из M0" is checked against PT-4: a project of 32 or more tracks, up to the PT-2
  maximum of 64, completes mix plus mastering within the PT-4 budget on the reference machine.
  The 64-track case is the worst-case benchmark; meeting it covers 32 or more.
- **M9 acceptance** (PLAN section 9 M9): "рендер 10-минутного трека в режиме максимального качества
  укладывается в целевое время" is checked against PT-3a (and PT-3b) on the reference machine.
- **CI.** CI runners are not the reference hardware, so CI does not check the absolute budgets.
  Proposed: CI only tracks performance regressions of the benchmarks relative to `main` (same
  runner class, same tool versions, deltas reported). Whether the benchmarks run in CI at all, the
  regression threshold, and block versus warn are open question Q14. Budget compliance is measured
  on the reference machine and recorded in the M8 and M9 results.

## Listening panel — see M0-14

Placeholder. The listeners for the M7 blind tests are recruited by
docs/tasks/M0/14-listening-panel.md (PLAN section 14 item 6). Panel size and screening rules
for RT-3 and RT-4 remain open (Q5) and are fixed in M7.

## Reference

- ITU-R BS.1770 (integrated loudness, true peak); revision pinned by the LoudnessMeter spec.
- EBU Tech 3341 and EBU Tech 3342 (meter conformance signals, M1 acceptance).
- ITU-R BS.1534 (MUSHRA method), basis for the "MUSHRA-like" level 6 test.
- Independent Python reference in `reference/`: integrated loudness via pyloudnorm; true peak per
  ITU-R BS.1770 Annex 2 (pyloudnorm has no true-peak function). Expected values and signals are
  defined by docs/spec/loudness-reference.md (future, produced by
  docs/tasks/M0/16-loudness-reference-set.md).

## Acceptance criteria

- [ ] RT-1 to RT-5 are recorded as in PLAN section 8 "Целевые показатели релиза", with no
      additional targets.
- [ ] States that targets are fixed in M0, refined in M7, and checked by M7 acceptance.
- [ ] Each target names its measurement level (2, 4, 6) and the milestone that checks it.
- [ ] Numbers not given by the plan are marked "fixed in M7" or listed as open questions.
- [ ] Contains a placeholder for the listening panel (M0-14).
- [ ] Performance targets state the reference hardware (PT-1), the maximum track count of 64
      (PT-2), the mastering worst-case benchmark with budgets ≤ 10 min at 48 kHz and ≤ 20 min at
      96 kHz (PT-3), and the mixing worst-case benchmark with a budget of ≤ 20 min for 64 tracks,
      10 minutes, 48 kHz, mix plus mastering (PT-4).
- [ ] States that the values are provisional, not measurements, and are refined in M9 by ADR.
- [ ] States that the budget is a performance target, not a cut-off: on engine limits the result is
      the best one that passed "do no harm", or the baseline, with a warning; never unverified.
- [ ] Track limit is consistent with M8 acceptance (32+ tracks within budget, checked by PT-4) and
      the budget with M9 acceptance (10-minute track in maximum-quality mode, checked by PT-3).
- [ ] States that CI does not check the absolute budgets (CI regression tracking against `main` is
      proposed, Q14) and that budget compliance is measured on the reference machine.

## Tests

No unit tests. Consumers: the eval harness checks RT-1 and RT-2 on the golden set in every format
(M6); the M7 listening-test results document RT-3 to RT-5. Performance: a benchmark for PT-3 and
PT-4 (task, owner and milestone assigned when planned; it must exist by M8 acceptance) is
proposed to run in CI as a regression check against `main` (Q14); PT-4 is checked on the reference machine for M8 acceptance and PT-3 for M9
acceptance, with results recorded in the milestone results.

## Out of scope

- Implementation of the benchmarks and performance work (M8, M9).
- Listening-test protocol details (M7).
- Per-track golden-set metric tolerances and score (later, dedicated task).
- Platform loudness and true-peak values (profiles, verified against current platform docs).
- Problem-detector recall and precision thresholds (level 3, M5).

## Open questions

- Q1. Does the ±0.5 LU loudness target apply to lossy formats after decode, or only to lossless
  outputs? PLAN section 8 attaches "во всех форматах" to true peak; M6 acceptance applies "во всех
  форматах" to both loudness and true peak. This draft follows M6 (all formats).
- Q2. True peak "not above target": strict comparison, or with a stated meter tolerance (true-peak
  meters have finite accuracy)? Plan gives no tolerance.
- Q3. Statistical test and significance level for RT-3 and RT-4 (fixed in M7).
- Q4. Are all golden-set tracks blind-evaluated, or a representative subset? "Ни один трек золотого
  набора" suggests all; listening cost may require a subset.
- Q5. Minimum number of listeners and the listener-screening rule (fixed in M7; listeners are
  recruited in M0, see "Listening panel — see M0-14").
- Q6. A baseline fallback passes RT-3 by definition. Do fallback tracks count against RT-4 (they
  cannot be preferred over themselves), and should the fallback rate be reported in M7 results?
- Q7. Is RT-5 a gate condition of M7 or a reported aim only? Which external systems are used?
- Q8. Profiles without a loudness target (if any, e.g. a delivery format with only a peak ceiling):
  is RT-1 not applicable for them?
- Q9. Confirm the draft "all formats" rule: golden-set runs use the platform profile's delivery
  formats and always include at least one lossy format (MP3, AAC or OGG). Which lossy format(s)
  and encoder settings are mandatory?
- Q10. PLAN section 13 asks for values "по измерениям на реальных проектах"; the maintainer fixed
  provisional targets on a worst-case benchmark instead because no renderer exists yet. Confirm that
  M9 refines them from measurements of real projects (this draft assumes so).
- Q11. Sessions above 64 tracks: rejected at intake with a clear reason, or accepted with a warning
  that the budget does not apply? Does "track" count mono and stereo tracks alike? Does the limit
  also bound buses?
- Q12. Benchmark material for PT-3 and PT-4 (synthetic, golden-set, own material) and the delivery
  format set of the benchmark job (one lossless format, or all formats of a profile including lossy
  encode/decode re-check). How the PT-3 "every block enabled" condition is reached: material that
  gives a reason for every block, or a test-only fixture in the benchmark harness (never a
  user-facing CLI/MCP flag or config). Whether PT-4 mastering uses the PT-3 settings, as this
  draft interprets the maintainer decision.
- Q13. Number of benchmark repetitions and the statistic compared with the budget (single run,
  median, or worst of N).
- Q14. Proposed CI performance tracking: do the benchmarks run in CI at all (cost, noisy shared
  runners)? If so, what regression threshold relative to `main` (percentage), and does a
  regression block the merge or only warn? CI never checks the absolute budgets.
- Q15. Determinism versus the engine's time limits (PLAN section 3 stage 6, "ограничен по ...
  времени"; PLAN section 6, "Лимиты: ... время выполнения"): if a wall-clock limit ends the
  optimization loop, the delivered result can depend on machine speed, contradicting "same input +
  same intent = same output" (AGENTS.md). Options:
  (a) bound the loop deterministically by iterations/renders; the wall-clock limit is only a safety
  stop, reported as such in the warning;
  (b) additionally, when the wall-clock safety stop fires, return the deterministic baseline (which
  still passes the delivery checks) or fail with a report, instead of a speed-dependent best-so-far.
  In either option the safety time limit must sit well above the PT-3/PT-4 budgets so it never
  fires in the benchmarks. Draft proposal: (a) is primary, i.e. the deterministic bound by
  iterations/renders governs normal runs; the choice of behavior on a safety stop ((a) or (b)) is
  for the maintainer.
