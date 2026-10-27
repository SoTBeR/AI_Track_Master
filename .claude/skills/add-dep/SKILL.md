---
name: add-dep
description: 'Audit a proposed new dependency via the dependency-auditor agent and turn its draft into a proposed ADR.'
argument-hint: <crate> [purpose]
disable-model-invocation: true
---

# Dependency audit: $ARGUMENTS

1. Delegate to **dependency-auditor**: audit crate `$0` for purpose "$ARGUMENTS" (license incl. transitive, maintenance,
   security, native deps, alternatives, version at least two weeks old). Return recommendation, evidence and an ADR draft.
2. If the recommendation is not "reject", run `/adr` steps with the draft: delegate to **architect** to write the ADR
   (`docs/adr/NNNN-dependency-<crate>.md`, status `proposed`) from the auditor's draft, and add an MPL-2.0 NOTICE note if relevant.
3. Report to the maintainer: recommendation, license summary, ADR path. Do not edit `Cargo.toml`; the dependency is
   used only after the maintainer accepts the ADR.
