# ADR-0005: clap for the aitm command-line interface

- Status: accepted
- Date: 2026-10-28
- Deciders: maintainer

## Context

Task M0-08 (`docs/tasks/M0/08-cli-entrypoint.md`, spec `docs/spec/cli-entrypoint.md`) adds the
clap-based command-line entry point to the `aitm` binary crate (`crates/cli`): `--version`,
`--help`, and the `mcp serve` placeholder.

Forces:

- PLAN section 4 (row "CLI") names clap. PLAN section 6 "CLI" lists `mcp serve` among the subcommands.
  The plugin parameter-schema paragraph says the CLI help is built from the schema by a derive macro.
- AGENTS.md "Prohibitions": a new dependency needs a dependency-auditor review and an accepted ADR. No
  GPL or LGPL.
- ADR-0003 section 2.2 (14-day version rule), section 2.4 (features that add crates need a check), and
  section 3 (cargo-deny rules).
- AGENTS.md "Code style": errors are typed and never panic on user input. PLAN section 3: no silent
  failures. Every CLI error needs a reason and a non-zero exit code.
- AGENTS.md: the CLI surface is public from M0-08. Later changes need a versioned migration and an ADR.

Evidence: dependency-auditor review on 2026-10-28. The graph was measured in a scratch copy outside the
repository with cargo 1.99.1, cargo-deny 0.20.2 (the repository `deny.toml`) and cargo-audit 0.22.1
(RustSec DB commit `40f5be6`). Sources are listed below.

## Decision

1. **Crate and version**: clap 4.6.7, licensed MIT OR Apache-2.0 (we use the Apache-2.0 option; no
   NOTICE entry is needed). Released 2026-09-14, the newest release, not yanked. This satisfies the
   14-day rule (ADR-0003 section 2.2). Rust 1.85 minimum.

2. **Features**: `default-features = false`, with `std`, `help`, `usage`, `error-context`, `suggestions`
   and `derive`.
   - `color` is off (maintainer decision 2026-10-28). It would add 10 crates, including the
     Windows-only `windows-sys` and `windows-link`. Help and error text are plain text.
   - `wrap_help` is off. It would add about 8 crates through `terminal_size` and `rustix`.
   - `env`, `string`, `unicode`, `cargo`, `debug`, `deprecated` and `unstable-*` are off in M0.
   - Enabling any of them needs an approval line in an ADR (ADR-0003 section 2.4).

3. **Declaration**: crate-local in `crates/cli/Cargo.toml`, not in `[workspace.dependencies]`, because
   `aitm` is the only crate that uses clap. Move it to the workspace if a second crate needs it.

   ```toml
   [dependencies]
   clap = { version = "4.6.7", default-features = false, features = ["std", "help", "usage", "error-context", "suggestions", "derive"] }
   ```

4. **Resolved graph**: 11 crates including clap: clap, clap_builder, clap_derive, clap_lex, anstyle,
   strsim, heck, proc-macro2, quote, syn (3.0.9), unicode-ident. All are MIT, Apache-2.0 or
   MIT/Apache-2.0 with Unicode-3.0, all within the ADR-0003 allowlist. `cargo deny check` passes all four
   sections. `cargo audit` reports no vulnerabilities. No `-sys` or native code, no duplicate versions.
   `Cargo.lock` grows by 11 packages and is committed. CI builds with `--locked`.

5. **Upgrades**: a clap bump that adds a crate to the graph needs an approval line in an ADR (ADR-0003
   section 2.5). The clap version is part of the CLI surface, so help and error text changes in a clap
   bump are reviewed in the bump's pull request.

## Alternatives

- **Default features, including `color`**: 21 crates, including Windows-only crates. Colored output is not
  a PLAN requirement. Rejected for M0; can be revisited by ADR.
- **pico-args 0.5.0** (MIT, zero deps): smallest, but no derive and no generated help. PLAN needs help
  generated from the schema. Last release 2022-06-04. Rejected.
- **lexopt 0.3.2** (MIT, zero deps): low-level parser. Help, usage and subcommand routing are hand-written.
  Rejected.
- **argh 0.1.19** (BSD-3-Clause, two normal deps): derive-based. Not named in PLAN; smaller feature set
  and ecosystem; not measured with cargo-deny. Rejected.
- **Hand-rolled parser**: zero deps, but we would own help, usage, error reporting and subcommand routing
  and test them. Rejected.

## Consequences

- Positive: help, usage errors, `--version` and subcommands come from derive macros, matching PLAN. The
  graph stays small and fully allowlisted. No Windows-only or native code is added.
- Negative: 10 new crates to audit and track. `syn` is a compile-time cost. Plain help text, no
  wrapping, and no colored errors until a later ADR.
- Negative: help and error text are part of the CLI surface. A clap upgrade can change them, so such
  changes are reviewed and noted in `CHANGELOG.md`.
- Follow-up (M0-08, platform-engineer): add the dependency line above in `crates/cli/Cargo.toml`, commit
  the updated `Cargo.lock`, and run the local checks. `deny.toml` does not change. No NOTICE entry.

## Sources (accessed 2026-10-28)

- https://crates.io/api/v1/crates/clap, https://crates.io/api/v1/crates/clap/4.6.7 and
  https://crates.io/api/v1/crates/clap/4.6.7/dependencies (versions, dates, license, features)
- https://crates.io/api/v1/crates/{clap_builder,clap_derive,clap_lex,anstyle,anstream,strsim,heck,
  proc-macro2,quote,syn,unicode-ident,windows-sys}/<version> (release dates, licenses)
- https://crates.io/api/v1/crates/{pico-args,lexopt,argh} (alternatives)
- https://github.com/clap-rs/clap (repository, `pushed_at` 2026-10-06, release tags)
- https://github.com/clap-rs/clap/releases/tag/v4.6.7
- https://api.osv.dev/v1/query (crates.io ecosystem, per-crate query): 0 advisories
- RustSec advisory DB via cargo-deny 0.20.2 and cargo-audit 0.22.1 (commit `40f5be6`)
- docs/adr/0003-dependency-license-policy.md, docs/adr/0004-ci-tools-and-actions.md
