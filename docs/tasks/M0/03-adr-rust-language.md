---
id: M0-03
title: Write ADR-0002 Rust as the language of core, server and CLI
milestone: M0
layer: docs
status: todo
depends_on: []
spec: none
agents: [architect, code-reviewer]
---

# Write ADR-0002 Rust as the language of core, server and CLI

## Goal

Record the decision to write core, engine, MCP server and CLI in Rust as one binary, with its
consequences and risks, so later sessions do not relitigate it.

## Scope

- In: `docs/adr/0002-rust-language.md` from `docs/templates/adr.md`; index entry in `docs/adr/README.md`.
- Out: dependency/license policy (M0-04); MCP SDK verification results (M0-10).

## Acceptance criteria

- [ ] ADR states the decision and rationale from PLAN section 4 "Почему Rust (ADR-001)": one language and one
      executable, compiler-checked memory safety and data races, simpler cross-builds for three OSes,
      future Tauri GUI reusing the core.
- [ ] ADR states the downsides from the same paragraph: fewer audio libraries, no stable ABI for third-party
      plugins, rmcp is Tier 3 in the MCP SDK conformance table (cite the current table URL and access date).
- [ ] ADR records the toolchain assumption "Rust stable, latest at project start" with the concrete version
      (PLAN section 13) and Python as development-only for references and eval scripts.
- [ ] ADR lists the mitigations from PLAN section 12 for the Rust-specific risks (probe server and elicitation
      check in M0, isolated protocol layer, fallback thin Python layer over the core; Python references).
- [ ] `docs/adr/README.md` index updated.

## Plan references

- docs/PLAN.md section 4 "Технологический стек", paragraph "Почему Rust (ADR-001)"
- docs/PLAN.md section 12 "Риски" (rows on the Rust MCP SDK and audio libraries)
- docs/PLAN.md section 13 "Допущения по умолчанию"
- docs/PLAN.md section 14 item 3

## Notes

- The MCP SDK tier must be checked against the live conformance page, not from memory.
- Numbering: repository ADR numbers follow creation order in `docs/adr/` (ADR-0001 is the working
  name, M0-01). The plan's "ADR-001" (Rust) = ADR-0002 and the plan's "ADR-002" (dependencies and
  licenses) = ADR-0003. Quotes of `docs/PLAN.md` keep the plan's labels.
