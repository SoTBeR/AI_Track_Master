---
id: M0-02
title: Add LICENSE (Apache-2.0) and NOTICE
milestone: M0
layer: platform
status: in-review
depends_on: [M0-01]
spec: none
agents: [platform-engineer, code-reviewer]
---

# Add LICENSE (Apache-2.0) and NOTICE

## Goal

Put the project under Apache-2.0 with a NOTICE file that will later list MPL-2.0 dependencies.

## Scope

- In: root `LICENSE` (verbatim Apache License 2.0 text), root `NOTICE` (project name from M0-01,
  copyright line, a section reserved for MPL-2.0 dependency attributions), `CHANGELOG.md` entry.
- Out: CONTRIBUTING.md (M0-22); SECURITY.md, CODE_OF_CONDUCT.md (M10 scope);
  any dependency entries (added by the task that introduces the dependency).

## Acceptance criteria

- [x] `LICENSE` contains the unmodified Apache License, Version 2.0 text (PLAN section 1, "Цель проекта"; section 14 item 1).
- [x] `NOTICE` exists, names the project from M0-01 and has a section for MPL-2.0 dependencies
      (PLAN section 2 principle 10: "MPL-2.0 допускается с записью в NOTICE").
- [x] `NOTICE` copyright line reads exactly "Copyright <year> The <project name> Authors", with
      `<project name>` taken from the M0-01 ADR and `<year>` the year of the commit (maintainer decision).
- [x] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 1 (table row "Цель проекта")
- docs/PLAN.md section 2 principle 10 "Чистая лицензия"
- docs/PLAN.md section 14 item 1

## Notes

- Copyright holder line decided by the maintainer: "Copyright <year> The <project name> Authors".
- DCO is used instead of a CLA (PLAN section 11); a short CONTRIBUTING.md documenting it is M0-22.
