---
id: M0-20
title: Finalize AGENTS.md commands and definition of done
milestone: M0
layer: platform
status: todo
depends_on: [M0-07, M0-15]
spec: none
agents: [platform-engineer, code-reviewer]
---

# Finalize AGENTS.md commands and definition of done

## Goal

Replace the placeholder command list in `AGENTS.md` with the commands that actually work after the workspace,
CI and reference project exist, and close the gaps against PLAN section 10.

## Scope

- In: `AGENTS.md` "Commands" section: real fmt, clippy, nextest, cargo-deny, cargo-audit, pytest and ruff
  commands matching CI; the eval/golden-set command marked "not available until the eval harness task";
  remove "Placeholders until the workspace exists"; "Definition of done" mentions green CI;
  `CHANGELOG.md` entry.
- Out: any other rule change in AGENTS.md (needs the maintainer).

## Acceptance criteria

- [ ] AGENTS.md lists working build, test, lint and eval-run commands (PLAN section 10 "AGENTS.md": "команды сборки,
      тестов, линтеров и запуска eval"); every listed command runs successfully or is explicitly marked as not yet available.
- [ ] The eval/golden-set line reads "not available until the eval harness task" and notes: measurement harness
      planned with M1 (first analyzers); golden-set scoring by M6 (PLAN section 9 M6 acceptance).
- [ ] Commands match those used in CI (M0-07).
- [ ] Definition of done includes green CI (PLAN section 10 "AGENTS.md": "зелёный CI").
- [ ] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 10 "AGENTS.md"
- docs/PLAN.md section 11
- docs/PLAN.md section 14 item 2

## Notes

- Maintainer decision: no golden-set harness in M0. No M1/M6 tasks are created here; the note in AGENTS.md
  only points to where the harness is planned.
- Existing AGENTS.md already covers architecture, directory map, style, prohibitions and definition of done
  from PLAN section 10; only the commands are placeholders and "green CI" is implicit.
