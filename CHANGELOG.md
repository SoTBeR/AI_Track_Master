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
- clap-based CLI entry point: `aitm --version`, `--help`, `mcp serve` placeholder (exit 1 until M0-09); no arguments prints help with exit 0; exit codes 0 success, 1 runtime error, 2 usage error; ADR-0005.
- Probe MCP server (`aitm mcp serve`, stdio, new crate `aitm-server`): experimental, non-public tools `probe_echo` and `probe_elicit` that validate rmcp, stdio and elicitation (form mode) before M4; legacy-era handshake with protocol versions 2025-06-18 and 2025-11-25; 1 MiB line limit; at most 32 tool calls in flight, client cancellation frees a slot; no file, network or process access; ADR-0006 (rmcp =3.5.1, tokio, serde, schemars).
