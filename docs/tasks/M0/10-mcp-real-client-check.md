---
id: M0-10
title: Verify the probe server in real MCP clients and record the result (maintainer)
milestone: M0
layer: docs
status: todo
depends_on: [M0-09]
spec: none
agents: [architect, code-reviewer]
---

# Verify the probe server in real MCP clients and record the result (maintainer)

## Goal

Run the probe binary in two or three real MCP clients over stdio, check the tool call and elicitation,
and record the results and the go/no-go decision on rmcp in an ADR.

## Scope

- In: maintainer runs the probe in these MCP clients on at least one OS: Claude Code, Claude Desktop, and
  VS Code (GitHub Copilot agent mode); the maintainer may substitute clients. Architect writes
  `docs/adr/NNNN-mcp-sdk-rmcp-validation.md` with a client matrix (client, version, OS, date, tool call result,
  elicitation result) and the decision; index entry in `docs/adr/README.md`.
- Out: code changes (if a fix is needed, a new platform task is created); `mcp install` (M4).

## Acceptance criteria

- [ ] The probe server answers a tool call from at least one real client (PLAN section 9 M0 acceptance:
      "пробный MCP-сервер отвечает на вызов инструмента из реального клиента").
- [ ] stdio and elicitation checked in two or three clients (PLAN section 14 item 3; section 9 M0 scope
      "проверка elicitation в реальных клиентах").
- [ ] The tested set includes at least one client that supports elicitation, if any available client does;
      the ADR records, per client, whether elicitation is supported (with client version and test date).
- [ ] If none of the tested clients supports elicitation, the ADR says so explicitly and records the CLI
      plan approval (`plan approve`) as the fallback approval path.
- [ ] For clients without elicitation, the ADR confirms the CLI approval path as the fallback
      (PLAN section 1 "Режимы агента"; section 6 "Режимы автономности"; section 12 row "Клиенты MCP без поддержки elicitation").
- [ ] ADR states whether rmcp is kept or the fallback thin Python layer is needed (PLAN section 12).
- [ ] `docs/adr/README.md` index updated.

## Plan references

- docs/PLAN.md section 1 (row "Режимы агента")
- docs/PLAN.md section 4 (row "MCP-сервер")
- docs/PLAN.md section 9 M0, section 12, section 14 item 3

## Notes

- **Maintainer:** run the clients (agents cannot drive interactive desktop clients); provide client names,
  versions and screenshots or logs. Default client set: Claude Code, Claude Desktop, VS Code (GitHub Copilot
  agent mode); substitutions are recorded in the ADR with the reason.
- Elicitation support differs between clients and changes over time; the ADR records what was observed,
  not what documentation claims.
