# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Agent infrastructure: AGENTS.md, CLAUDE.md, Claude Code agents, skills and settings.
- ADR-0001: working name AI Track Master (AITM), `aitm-` crate prefix, binary `aitm`.
- ADR-0002: Rust as the language of core, server and CLI; toolchain Rust stable 1.99.1.
- Session model spec (draft): tracks, buses, master bus, versions and rollback, write-path rules.
- Target profile format spec (draft): versioned TOML profiles with sourced platform values.
- Release quality targets spec (draft): RT-1..RT-5 from PLAN section 8 with measurement and milestones.
- CONTRIBUTING.md: DCO sign-off, Conventional Commits, branch per task.
- ADR-0003: dependency and license policy, cargo-deny rules, stack crate license table.
- Performance targets PT-1..PT-4 (provisional): reference PC, 64 tracks, render budgets.
- LICENSE (Apache-2.0, unmodified canonical text) and NOTICE (copyright and reserved MPL-2.0 attribution section).
- CI workflow (GitHub Actions): fmt, clippy, cargo-deny, cargo-audit, dev-only guard, nextest on Linux/Windows/macOS arm64+x86_64 with binary artifacts, Python checks; required checks listed in `.github/BRANCH_PROTECTION.md`.
- `dev` integration branch: task branches merge into `dev`, `main` changes only through a maintainer PR from `dev`; CI also runs on pushes to `dev`; Claude Code push guard hook allows pushing `dev` only.
- Cargo workspace: `aitm` binary (crates/cli) and `aitm-core` library, pinned toolchain 1.99.1, `deny.toml` per ADR-0003, committed `Cargo.lock`.
