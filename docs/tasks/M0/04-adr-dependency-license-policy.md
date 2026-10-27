---
id: M0-04
title: Write ADR-0002 allowed dependencies, licenses and cargo-deny policy
milestone: M0
layer: docs
status: todo
depends_on: [M0-03]
spec: none
agents: [architect, dependency-auditor, code-reviewer]
---

# Write ADR-0002 allowed dependencies, licenses and cargo-deny policy

## Goal

Fix the license policy (what may be linked, what needs NOTICE, what is banned) and the rules that
`deny.toml` must encode, and record the license status of the crates named in the plan's stack.

## Scope

- In: `docs/adr/0002-dependency-license-policy.md`; index entry in `docs/adr/README.md`.
  The ADR defines: allowed license list, MPL-2.0 handling (NOTICE entry), banned licenses (GPL, LGPL,
  any copyleft requiring source release of the static binary), ffmpeg only as a subprocess,
  advisory policy (cargo-audit / cargo-deny advisories), the rule "new dependency = dependency-auditor review + ADR",
  and the policy for Python reference dependencies.
- In: a table of the stack crates from PLAN section 4 (rmcp, tokio, clap, serde, schemars, symphonia, hound,
  flacenc, rubato, ebur128, rustfft, rusqlite, argmin, assert_no_alloc, proptest, criterion) with license as
  published on crates.io on a stated date, produced by dependency-auditor.
- Out: writing `deny.toml` itself (M0-06, platform); adding any crate to the workspace.

## Acceptance criteria

- [ ] Policy matches PLAN section 2 principle 10: Apache-2.0-compatible only; no LGPL/GPL (static linking);
      MPL-2.0 allowed with a NOTICE entry; ffmpeg only as a separate process; compatibility checked by cargo-deny.
- [ ] ADR states the rules `deny.toml` must enforce (licenses, bans, advisories, sources) precisely enough for
      platform-engineer to write it without further decisions.
- [ ] Stack crate table lists crate, license, source URL and access date; any crate whose license conflicts
      with the policy is flagged as an open question (PLAN section 4: "Названия крейтов, их версии и лицензии
      подтверждаются в M0 через cargo-deny").
- [ ] ADR notes that Cargo.lock is committed and an SBOM is produced per release (PLAN section 11).
- [ ] `docs/adr/README.md` index updated.

## Plan references

- docs/PLAN.md section 2 principle 10 "Чистая лицензия"
- docs/PLAN.md section 4 "Технологический стек"
- docs/PLAN.md section 11 "CI/CD и релизы" (additional items)
- docs/PLAN.md section 14 item 3

## Notes

- The table is informational; each crate still gets its own audit and ADR when a task first adds it
  (AGENTS.md "Prohibitions"). Final confirmation happens when cargo-deny runs on the real Cargo.lock.
- symphonia is MPL-2.0 per PLAN section 4; verify the current license, do not assume.
