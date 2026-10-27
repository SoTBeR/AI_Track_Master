# Spec: Release quality targets

- Status: draft
- Layer: docs
- Task: docs/tasks/M0/12-release-quality-targets.md
- Plan: docs/PLAN.md section 8 ("Целевые показатели релиза"), section 9 (M0, M6, M7, M8, M9),
  section 13, section 14 item 6

## What

The release quality targets for 1.0, as fixed in M0. For each target this spec states what is
measured, at which verification level of PLAN section 8 it is measured, on which material, and at
which milestone it is checked. The targets are fixed in M0 and refined in M7; M7 acceptance
("quality gate") checks that RT-1 to RT-4 are met. RT-5 is an aim and is reported, not gating,
pending open question Q7.

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

## Performance targets — see M0-13

Placeholder. The render-time budget for a 10-minute track in maximum-quality mode, the maximum
track count, the reference hardware and the benchmark definitions are recorded here by
docs/tasks/M0/13-render-budget-track-limit.md (PLAN section 13; section 14 item 6; checked by
M8 and M9 acceptance).

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
- [ ] Contains placeholders for performance targets (M0-13) and the listening panel (M0-14).

## Tests

No unit tests. Consumers: the eval harness checks RT-1 and RT-2 on the golden set in every format
(M6); the M7 listening-test results document RT-3 to RT-5.

## Out of scope

- Render-time budget and track limit values (M0-13).
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
