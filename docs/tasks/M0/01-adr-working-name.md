---
id: M0-01
title: Record the working name and package name (maintainer)
milestone: M0
layer: docs
status: in-review
depends_on: []
spec: none
agents: [architect, code-reviewer]
---

# Record the working name and package name (maintainer)

## Goal

Pick the working name of the project and the package/binary name, and record them in an ADR,
so that the workspace, binary, NOTICE and distribution channels use one name from the start.

## Scope

- In: ADR `docs/adr/NNNN-working-name.md` with the chosen project name, crate/package name and
  binary name, alternatives considered, availability check; index entry in `docs/adr/README.md`.
- Out: renaming anything in code (no code exists yet); distribution setup (M10).

## Acceptance criteria

- [x] Working name and package name chosen by the maintainer (PLAN section 13: "Рабочее название и имя пакета выбираются в M0").
- [x] ADR records the project name, the Cargo package name and the binary name.
- [x] ADR records that the package name is available on the planned install channels
      (crates.io for `cargo install`, Homebrew formula name) as checked on a stated date (PLAN section 4, "Дистрибуция").
- [x] `docs/adr/README.md` index updated.

## Plan references

- docs/PLAN.md section 13 "Допущения по умолчанию"
- docs/PLAN.md section 4 "Технологический стек" (row "Дистрибуция")

## Notes

- **Maintainer decision.** The architect may propose candidates and run the availability check,
  but the maintainer picks the name.
- Blocks M0-02 (NOTICE text) and M0-06 (workspace and binary names).
