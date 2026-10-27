---
id: M0-06
title: Create the Cargo workspace, deny.toml and the single binary crate
milestone: M0
layer: platform
status: todo
depends_on: [M0-01, M0-02, M0-04]
spec: none
agents: [test-author, platform-engineer, code-reviewer]
---

# Create the Cargo workspace, deny.toml and the single binary crate

## Goal

Create the Cargo workspace with the platform crates and one binary that builds, plus `deny.toml`
encoding ADR-0002, so CI (M0-07) has something to build and check.

## Scope

- In: root `Cargo.toml` (workspace, shared lints: clippy warnings as errors), committed `Cargo.lock`,
  `rust-toolchain.toml` pinned to the version in ADR-0001, `deny.toml` per ADR-0002, `crates/core` (empty
  library with doc comment), `crates/cli` (binary named per M0-01 that exits 0), a smoke test that the
  binary runs; root `.gitignore` covering `target/` and the generated-audio cache directory `reference/.cache/`
  used by M0-15 and M0-16 (generated test audio is never committed); `CHANGELOG.md` entry.
- Out: crates of other layers (`dsp`, `analysis`, `engine`, `plugin-sdk`, `plugins`) — each is created by the
  first task in its layer; `crates/server` (M0-09); any third-party crate (none in this task).

## Acceptance criteria

- [ ] `cargo build --workspace` produces one binary with the name from M0-01 (PLAN section 2 "Модель процессов";
      section 9 M0 acceptance "собирается один бинарник").
- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, tests and
      `cargo deny check` pass locally (PLAN section 11, stage "Проверка стиля").
- [ ] `deny.toml` encodes every rule ADR-0002 lists for cargo-deny (PLAN section 2 principle 10).
- [ ] `Cargo.lock` is committed (PLAN section 11, additional items).
- [ ] No third-party dependencies are added.
- [ ] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 2 "Модель процессов", principle 10
- docs/PLAN.md section 4 (rows "Язык и сборка", "Качество кода")
- docs/PLAN.md section 5 "Структура репозитория"
- docs/PLAN.md section 9 M0, section 14 item 4

## Notes

- Creating empty crates for other layers here would cross layers (AGENTS.md); they are added later by their owners.
