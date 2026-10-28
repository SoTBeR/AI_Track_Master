---
id: M0-07
title: Set up the CI matrix on Windows, macOS and Linux
milestone: M0
layer: platform
status: in-review
depends_on: [M0-06]
spec: none
agents: [platform-engineer, dependency-auditor, code-reviewer]
---

# Set up the CI matrix on Windows, macOS and Linux

## Goal

GitHub Actions workflow that builds the workspace, runs lints, tests and license/vulnerability checks,
and builds the probe binary on the three target OSes on every pull request.

## Scope

- In: `.github/workflows/ci.yml` with jobs: rustfmt, clippy (warnings as errors), cargo-deny, cargo-audit,
  build + `cargo nextest run --workspace` on the matrix, binary artifact upload per OS;
  a Python job running `ruff check reference/` and `python3 -m pytest reference/` that is skipped while
  `reference/` does not exist; `CHANGELOG.md` entry.
- Out: Miri/ASan/TSan jobs (no unsafe code yet; PLAN section 11 lists them for unsafe modules), cargo-fuzz
  schedule, metric/golden-set jobs, MCP integration job (M4), eval runs.
- Out of scope (M10 / release work, maintainer decision): cargo-dist release builds, Dependabot/Renovate, SBOM.

## Acceptance criteria

- [x] Matrix: Windows, macOS arm64 and x86_64, Linux x86_64 (PLAN section 11, stage "Сборка").
- [x] Stages present: rustfmt, clippy with warnings as errors, cargo-deny, cargo-audit; ruff for Python references
      (PLAN section 11, stage "Проверка стиля"); cargo nextest and pytest (stage "Тесты").
- [ ] CI is green on all three OSes on `main` and on a test PR; the built binary is available as an artifact
      per OS (PLAN section 9 M0 acceptance; section 14 item 4).
- [x] Any failing job blocks merge (required checks configured or documented for the maintainer to enable;
      PLAN section 10 "Защитные барьеры").
- [x] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 4 (row "CI")
- docs/PLAN.md section 9 M0 acceptance
- docs/PLAN.md section 10 "Защитные барьеры"
- docs/PLAN.md section 11 "CI/CD и релизы"
- docs/PLAN.md section 14 item 4

## Notes

- dependency-auditor reviews the CI tools (cargo-nextest, cargo-deny, cargo-audit) and every third-party
  GitHub Action used; record the result in an ADR if any is new.
- Maintainer: needs a GitHub repository with Actions enabled and branch protection with required checks.
