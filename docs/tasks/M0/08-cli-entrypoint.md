---
id: M0-08
title: Add the clap-based CLI entry point with `mcp serve`
milestone: M0
layer: platform
status: done
depends_on: [M0-07]
spec: docs/spec/cli-entrypoint.md
agents: [architect, test-author, platform-engineer, dependency-auditor, code-reviewer]
---

# Add the clap-based CLI entry point with `mcp serve`

## Goal

Give the single binary a clap command structure with `--version`, `--help` and an `mcp serve`
subcommand placeholder, so the probe MCP server (M0-09) runs from the same binary.

## Scope

- In: spec `docs/spec/cli-entrypoint.md`; clap added to `crates/cli` after dependency-auditor review and ADR;
  `--version`, `--help`, `mcp serve` (returns a clear "not available" error until M0-09); tests; `CHANGELOG.md` entry.
- Out: all other CLI commands from PLAN section 6 (M3 and later); MCP server code (M0-09).

## Acceptance criteria

- [x] CLI is built on clap (PLAN section 4, row "CLI").
- [x] `mcp serve` exists as the subcommand name from PLAN section 6 "CLI".
- [x] Errors are reported with a reason and a non-zero exit code, never a panic (AGENTS.md "Code style";
      PLAN section 3 "Тихих сбоев нет").
- [x] Accepted ADR for the clap dependency (AGENTS.md "Prohibitions"; PLAN section 2 principle 10).
- [x] Tests written first pass on all three OSes in CI.
- [x] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 4 (row "CLI")
- docs/PLAN.md section 6 "CLI"
- docs/PLAN.md section 9 M0

## Notes

- This is the first public CLI surface; later changes need a versioned migration and an ADR (AGENTS.md).
