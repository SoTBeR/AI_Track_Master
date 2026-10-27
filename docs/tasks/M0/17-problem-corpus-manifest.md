---
id: M0-17
title: Start the licensed real-track problem corpus manifest (maintainer)
milestone: M0
layer: platform
status: todo
depends_on: [M0-02]
spec: docs/spec/corpus-manifest.md
agents: [architect, platform-engineer, code-reviewer]
---

# Start the licensed real-track problem corpus manifest (maintainer)

## Goal

Start collecting real tracks with problems under a usable license and track them in a manifest,
so the golden set and detector corpora (M5, M6) have material.

## Scope

- In: spec `docs/spec/corpus-manifest.md` (manifest fields: id, category, source URL, license, license
  evidence, checksum, duration, sample rate, known problems as labels, storage location outside git);
  initial manifest file under `eval/`; a validation check that every entry has a license and checksum;
  `CHANGELOG.md` entry.
- Out: committing audio; the golden-set harness and tolerances; injected-defect corpus (M5).

## Acceptance criteria

- [ ] Collection started for the categories in PLAN section 14 item 5: AI generations, home mixes, clipping,
      over-limited input.
- [ ] Only freely licensed or own material; every entry records its license (PLAN section 8 level 4;
      section 14 item 5 "с допустимой лицензией").
- [ ] Manifests are committed, audio is not (PLAN section 3 "Профили целей": only aggregates go into the repo;
      section 8 level 4 "Манифесты корпуса").
- [ ] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 8 level 4 "Золотой набор"
- docs/PLAN.md section 14 item 5

## Notes

- **Maintainer:** source the tracks and confirm each license; agents cannot judge rights to AI generations.
- Maintainer decision: corpus manifests under `eval/` stay in the platform layer (owner platform-engineer).
