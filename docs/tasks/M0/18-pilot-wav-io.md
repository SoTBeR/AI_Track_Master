---
id: M0-18
title: Pilot - WAV read and write in core
milestone: M0
layer: platform
status: todo
depends_on: [M0-07, M0-15]
spec: docs/spec/audio-io-wav.md
agents: [architect, test-author, platform-engineer, dependency-auditor, security-reviewer, code-reviewer]
---

# Pilot - WAV read and write in core

## Goal

First half of the pilot task: WAV reading and writing in `crates/core`, run through the full
spec → test → implementation → review cycle to validate the agent process.

## Scope

- In: spec `docs/spec/audio-io-wav.md`; WAV decode to 64-bit float and encode from 64-bit float in
  `crates/core`, using symphonia for reading and hound for writing after dependency-auditor review and ADRs;
  NOTICE entry for symphonia if MPL-2.0; tests (round-trip, bit depths, sample rates 44.1-192 kHz, malformed
  input, content-based format detection); `CHANGELOG.md` entry.
- Out: FLAC (M1, maintainer decision) and all other formats (M1); session graph and render (M1); dithering on export (M2); project-folder
  write protection beyond what the spec requires for this API (M4).

## Acceptance criteria

- [ ] Reading via symphonia, writing via hound (PLAN section 4 rows "Чтение аудио", "Запись аудио").
- [ ] Integer PCM written and read back is bit-identical (PLAN section 9 M1 acceptance "чтение и запись WAV ... без потерь").
- [ ] Format is detected by content, not extension; malformed or non-audio input returns a typed error with a
      reason, never a panic (PLAN section 6 "Ограничения безопасности"; section 3 "Тихих сбоев нет").
- [ ] Source files are opened read-only and never modified (PLAN section 2 principle 7; section 6).
- [ ] symphonia recorded in NOTICE if MPL-2.0 (PLAN section 4 row "Чтение аудио"; section 2 principle 10).
- [ ] Tests written first, green on all three OSes in CI; ADRs accepted for new dependencies.
- [ ] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 4 (rows "Чтение аудио", "Запись аудио")
- docs/PLAN.md section 6 "Ограничения безопасности"
- docs/PLAN.md section 9 M1 acceptance
- docs/PLAN.md section 14 item 7

## Notes

- Maintainer decision: the pilot stays in M0 (PLAN section 14 item 7). This task covers WAV read/write only;
  FLAC and the rest of audio I/O remain M1 scope.
- Bit depth reduction on export happens with dither, once (PLAN section 7); the spec must say how this API
  behaves until the Dither plugin exists (M2), for example integer output only for already-integer data or
  explicit "no dither" naming.
