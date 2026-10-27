---
id: M0-05
title: Write the session model spec
milestone: M0
layer: docs
status: todo
depends_on: []
spec: docs/spec/session.md
agents: [architect, code-reviewer]
---

# Write the session model spec

## Goal

Write `docs/spec/session.md`: the session model (tracks, buses, master bus, versions) that core
(M1, M3), engine and server build on.

## Scope

- In: `docs/spec/session.md` from `docs/templates/spec.md`: entities (session, track, bus, master bus,
  routing, version), invariants, versioning and rollback semantics, what is stored in SQLite versus files,
  and a "Public API" section precise enough for test-author.
- Out: implementation (core, M1/M3); MCP tool schemas (M4); storage schema DDL beyond what the model requires.

## Acceptance criteria

- [ ] Spec covers "модель сессии, дорожки, шины, версии" (PLAN section 14 item 2).
- [ ] A session consists of tracks, buses and a master bus; a plain stereo file is a one-track session
      (PLAN section 2 principle 1).
- [ ] Source files are never modified; every change creates a session version with rollback
      (PLAN section 2 principle 7); rollback returns the previous state bit-for-bit (PLAN section 9, M3 acceptance).
- [ ] Processing format is 64-bit float at the source sample rate up to 192 kHz (PLAN sections 2 principle 4, 13).
- [ ] Storage is SQLite plus files (PLAN section 4, row "Хранение"); writes only inside the project folder
      (PLAN section 6 "Ограничения безопасности").
- [ ] Ambiguities are listed as open questions in the spec, not resolved by invention.
- [ ] `docs/spec/README.md` updated if it lists specs.

## Plan references

- docs/PLAN.md section 2 principles 1, 4, 5, 6, 7
- docs/PLAN.md section 4 (row "Хранение")
- docs/PLAN.md section 6 "Ограничения безопасности", tools group "Сессия"
- docs/PLAN.md section 9 M1 and M3
- docs/PLAN.md section 14 item 2

## Notes

- Maximum track count is not fixed yet (M0-13); the spec should reference it as a limit, not invent a number.
