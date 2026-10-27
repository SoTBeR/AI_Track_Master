---
id: M0-22
title: Add a short CONTRIBUTING.md (DCO, Conventional Commits, branch per task)
milestone: M0
layer: platform
status: in-review
depends_on: [M0-02]
spec: none
agents: [platform-engineer, code-reviewer]
---

# Add a short CONTRIBUTING.md (DCO, Conventional Commits, branch per task)

## Goal

Give human and agent contributors one short page with the contribution rules that apply from the first
commit, pointing to `AGENTS.md` for everything else.

## Scope

- In: root `CONTRIBUTING.md` covering DCO sign-off (`git commit -s`, `Signed-off-by` line), Conventional Commits,
  one branch per task (`task/<M#>-<NN>-<slug>`), maintainer-only merges into `main`, and a pointer to `AGENTS.md`
  (layers, prohibitions, definition of done); `CHANGELOG.md` entry.
- Out: SECURITY.md, CODE_OF_CONDUCT.md, issue templates and the full contributor documentation (M10);
  any change to `AGENTS.md`; CI enforcement of DCO (a separate task if the maintainer wants it).

## Acceptance criteria

- [x] `CONTRIBUTING.md` states that contributions use DCO sign-off instead of a CLA and shows `git commit -s`
      (PLAN section 11: "Для вкладов используется DCO (подпись Signed-off-by) вместо CLA").
- [x] It states the Conventional Commits format and the branch-per-task naming exactly as in `AGENTS.md` "Git",
      without restating or contradicting other AGENTS.md rules (it links to them instead).
- [x] It states that contributions are under Apache-2.0 (PLAN section 1, row "Цель проекта"; LICENSE from M0-02).
- [x] The document is short (about one screen) and in English.
- [x] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 1 (row "Цель проекта")
- docs/PLAN.md section 11 (DCO)
- docs/PLAN.md section 9 M10 (full CONTRIBUTING is listed there; this task is the short M0 version)
- AGENTS.md "Git"

## Notes

- Maintainer decision: a short CONTRIBUTING.md moves into M0; SECURITY.md and CODE_OF_CONDUCT.md stay in M10.
  The M10 task extends this file rather than replacing it.
