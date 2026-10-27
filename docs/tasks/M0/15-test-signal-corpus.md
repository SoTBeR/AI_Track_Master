---
id: M0-15
title: Generate the synthetic test signal corpus (sine, noise, sweep, impulse)
milestone: M0
layer: reference
status: todo
depends_on: [M0-04, M0-07]
spec: docs/spec/test-signals.md
agents: [architect, reference-author, test-author, dependency-auditor, dsp-verifier, code-reviewer]
---

# Generate the synthetic test signal corpus (sine, noise, sweep, impulse)

## Goal

Deterministic Python generators for the basic test signals used by DSP and analysis tests from M0 on.

## Scope

- In: spec `docs/spec/test-signals.md` (signals, parameters, sample rates, bit depth, seeds, file naming);
  one module per generator in `reference/` (sine, white/pink noise, sweep, impulse) using numpy/scipy;
  Python project config for pytest and ruff; pytest tests in `reference/tests/` checking analytic properties
  (frequency, RMS, sweep range, impulse position, seed determinism); `CHANGELOG.md` entry.
- Out: LUFS/EBU test material (M0-16); problem-injected corpus (M5); any Rust code.

## Acceptance criteria

- [ ] Corpus includes sine, noise, sweep and impulse (PLAN section 14 item 5).
- [ ] Generators are plain numpy/scipy, one module per signal, ruff-clean, pytest green in CI
      (PLAN section 4, row "Эталонные прототипы"; section 11 stage "Проверка стиля", "Тесты").
- [ ] Output is deterministic for a given seed and parameter set (PLAN section 2 principle 6).
- [ ] No generated WAV (or other audio) files are committed: tests regenerate signals from the scripts in
      `reference/`; an optional on-disk cache lives in the gitignored `reference/.cache/` (see M0-06), and a stale or
      missing cache only triggers regeneration, never a different result.
- [ ] Sample rates cover the supported range 44.1-192 kHz where the spec requires it (PLAN section 7 "Правила DSP-кода").
- [ ] Each signal's properties are verified against its analytic definition (independent reference: closed-form
      formulas; logarithmic sweep per Farina, AES 108th Convention 2000, if the spec chooses an exponential sweep).
- [ ] Python dependencies reviewed by dependency-auditor and allowed by ADR-0002.
- [ ] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 4 (row "Эталонные прототипы")
- docs/PLAN.md section 7 "Правила DSP-кода"
- docs/PLAN.md section 8 level 1
- docs/PLAN.md section 14 item 5

## Notes

- Maintainer decision: signals are generated deterministically by scripts in `reference/` and not committed
  as WAV; tests regenerate them (cache allowed in a gitignored directory). The cache directory is
  `reference/.cache/`; its root `.gitignore` entry is added by M0-06 (platform), not by reference-author.
