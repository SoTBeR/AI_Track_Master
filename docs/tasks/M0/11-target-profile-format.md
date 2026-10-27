---
id: M0-11
title: Specify the target profile format
milestone: M0
layer: docs
status: done
depends_on: []
spec: docs/spec/target-profiles.md
agents: [architect, code-reviewer]
---

# Specify the target profile format

## Goal

Define the versioned data format for platform delivery targets and material profiles stored in `profiles/`,
so the engine (M6) and MCP resources (M4) read targets as data, never from code.

## Scope

- In: `docs/spec/target-profiles.md`: file format and location, schema version and migration rule,
  platform profile fields (integrated loudness target, true-peak ceiling, delivery formats, source URL and
  access date for every platform value), material profile fields (aggregated smoothed-spectrum tonal target
  curves and dynamic targets per material type, bounds within which intent and reference may shift targets),
  validation rules, and a "Public API" section for the loader.
- Out: actual platform values and profile files (M6, engine); loader implementation (M6); tonal curves from
  a reference corpus (later); choosing a parser crate (ADR when the loader task starts).

## Acceptance criteria

- [x] Profiles are versioned data in `profiles/`, not code (PLAN section 3 "Профили целей"; section 8 last paragraph).
- [x] Every platform value carries its source (official documentation URL) and access date, and the spec
      requires checking current documentation on update (PLAN section 8; section 13 last item).
- [x] Material profiles hold only aggregated curves, never audio (PLAN section 3 "Профили целей").
- [x] The format expresses bounded target shifts for intent and reference (PLAN section 3 stage 4).
- [x] The format can be exposed as an MCP resource "цели по платформам" (PLAN section 6 "Ресурсы и промпты").
- [x] Example snippets in the spec use clearly fake placeholder values, not real platform numbers.
- [x] `docs/spec/README.md` updated if it lists specs.

## Plan references

- docs/PLAN.md section 3 stage 4 and "Профили целей"
- docs/PLAN.md section 6 "Ресурсы и промпты"
- docs/PLAN.md section 8 (last paragraph)
- docs/PLAN.md section 9 M0 ("формат профилей целей") and M6 ("профили целей")
- docs/PLAN.md section 13

## Notes

- File syntax (TOML, JSON, YAML) is a decision; if it is architectural, record it in an ADR in the same task.
- Decision (fix round 1): the syntax ADR is deferred to M6, to be written before the profile loader task
  (together with the parser-crate ADR). The spec keeps TOML 1.0 as the proposal, marked "pending ADR in M6".
