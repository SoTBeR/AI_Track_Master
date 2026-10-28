---
id: M0-09
title: Build the probe MCP server on rmcp over stdio with an elicitation probe
milestone: M0
layer: platform
status: done
depends_on: [M0-08]
spec: docs/spec/mcp-probe.md
agents: [architect, test-author, platform-engineer, dependency-auditor, security-reviewer, code-reviewer]
---

# Build the probe MCP server on rmcp over stdio with an elicitation probe

## Goal

A minimal MCP server in `crates/server`, started by `<binary> mcp serve`, that answers a tool call over
stdio and exercises MCP elicitation, to validate rmcp before M4 builds on it.

## Scope

- In: spec `docs/spec/mcp-probe.md`; `crates/server` crate; rmcp and tokio (plus serde/schemars if rmcp
  requires them) after dependency-auditor review and ADRs; one probe tool returning a structured result;
  one probe tool that requests elicitation from the client and reports whether the client supports it,
  accepted, declined or cancelled; an automated stdio test driving the server; `CHANGELOG.md` entry.
- Out: the real tool set, job queue, autonomy modes, path protection, audit log (M4);
  Streamable HTTP transport (optional per PLAN section 4); testing in real clients (M0-10, maintainer).

## Acceptance criteria

- [x] Server uses rmcp with tokio, stdio transport (PLAN section 4, row "MCP-сервер").
- [x] `<binary> mcp serve` starts it from the single binary (PLAN section 2 "Модель процессов"; section 6 "CLI").
- [ ] An automated test over stdio initializes a session, lists tools and gets a valid result from the probe tool
      on all three OSes in CI (PLAN section 9 M0 acceptance).
- [x] The elicitation probe reports a definite outcome for clients with and without elicitation support;
      a client without support gets an explicit result, not a hang or silent failure (PLAN section 4;
      section 6 "Режимы автономности"; section 3 "Тихих сбоев нет").
- [x] The probe tools perform no file writes and read no files (PLAN section 2 principle 9).
- [x] Probe tools are explicitly experimental and non-public: every probe tool name carries the `probe_` prefix,
      and the spec and tool descriptions state that they are removable without notice. Removing them in M4
      therefore requires no versioned schema migration and no ADR (AGENTS.md "Prohibitions" covers public schemas only).
- [x] rmcp version is pinned (PLAN section 12, row "Изменения спецификации MCP": "закреплённая версия SDK").
- [x] Accepted ADRs for each new dependency (AGENTS.md "Prohibitions").
- [x] `CHANGELOG.md` "Unreleased" updated.

## Plan references

- docs/PLAN.md section 4 (row "MCP-сервер"), paragraph "Почему Rust (ADR-001)"
- docs/PLAN.md section 6 "Режимы автономности", "CLI"
- docs/PLAN.md section 9 M0
- docs/PLAN.md section 12 (rows on the MCP SDK and MCP spec changes)
- docs/PLAN.md section 14 item 3

## Notes

- Maintainer decision: probe tools use the `probe_` prefix (for example `probe_echo`, `probe_elicit`) and are
  documented in `docs/spec/mcp-probe.md` as experimental, non-public and removable. They are not part of the
  public MCP schema, so their removal in M4 is not a schema change.
- If rmcp cannot do elicitation, stop and report; the fallback (thin Python layer, PLAN section 12) is a
  maintainer decision recorded in M0-10.
