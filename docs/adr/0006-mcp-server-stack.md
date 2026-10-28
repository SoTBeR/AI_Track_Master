# ADR-0006: MCP server stack for the probe server (rmcp, tokio, serde, schemars)

- Status: accepted
- Date: 2026-10-28
- Deciders: maintainer (accepted 2026-10-28, including the resolution of Decision 10)
- Amended: 2026-10-28, after code and security review of M0-09 (see "Amendment 2026-10-28")

## Context

Task M0-09 (`docs/tasks/M0/09-mcp-probe-server.md`, spec `docs/spec/mcp-probe.md`) adds the
library crate `crates/server` (package `aitm-server`, ADR-0001) and wires `aitm mcp serve` to it. The
server uses rmcp on tokio over stdio and exposes two experimental probe tools: `probe_echo` (structured
output) and `probe_elicit` (server-to-client elicitation with a definite outcome).

Forces:

- PLAN section 4, row "MCP-сервер": rmcp (official Rust SDK) and tokio, stdio primary. PLAN section 12
  requires a pinned SDK version ("закреплённая версия SDK") and names a fallback (thin Python layer, a
  maintainer decision in M0-10).
- ADR-0003 section 2.1 (dependency-auditor review and an accepted ADR for every new crate, normal, build
  and dev), section 2.2 (latest release at least 14 days old: cutoff 2026-10-14), section 2.4 (every
  feature that adds crates needs a check), section 2.5 (lockfile bumps that add crates need an ADR).
- ADR-0005: crate-local declaration when one crate uses a dependency; exact versions in `Cargo.toml`.
- AGENTS.md: no GPL or LGPL; MPL-2.0 needs NOTICE; no `unsafe` in DSP crates; no bypass.

Evidence: dependency-auditor review on 2026-10-28 with cargo 1.99.1, rustc 1.99.1, cargo-deny 0.20.2
(repository `deny.toml`) and cargo-audit 0.22.1 (RustSec commit `40f5be6`), in a scratch copy outside the
repository. The spike compiled and ran the probe server over stdio with a raw JSON-RPC client, with
accepted, declined, cancelled and unsupported elicitation outcomes. Sources are listed at the end.

## Decision

1. **rmcp 3.5.1, exact pin.** Released 2026-10-05, Apache-2.0. The newest release, 3.8.0 (2026-10-26),
   is younger than 14 days; 3.6.0 to 3.8.0 are excluded by ADR-0003 section 2.2.

   ```toml
   rmcp = { version = "=3.5.1", default-features = false, features = ["server", "macros", "transport-io"] }
   ```

   Enabled features and what they pull: `server` (transport-async-rw, schemars, pastey, uuid), `macros`
   (rmcp-macros), `transport-io` (tokio io-std). Not enabled, and each needs its own ADR approval line
   before enabling (ADR-0003 section 2.4): `elicitation` (adds `url`, `idna` and ICU4X, 28 crates; the
   protocol works without it, see Decision 4), `client`, `base64` (default; not needed), `auth`, `reqwest*`,
   `transport-child-process`, `transport-streamable-http-*`, `which-command`.

2. **tokio 1.53.3, MIT.** 1.54.0 is yanked; 1.54.1 is younger than 14 days.

   ```toml
   tokio = { version = "1.53.3", default-features = false, features = ["rt"] }
   ```

   Amended 2026-10-28: declared as `["rt", "time", "sync"]`, see "Amendment" below.

   Effective features with rmcp: rt, sync, time, macros, io-util, io-std. No `fs`, `net`, `process`
   or `signal` (spec "Security and resource limits").

3. **serde 1.0.229 (`derive`) and schemars 1.2.3 (defaults), both MIT or MIT OR Apache-2.0.** Plugin
   schemas (PLAN section 7) also use them, so the workspace move (ADR-0005) happens with plugin-sdk (M2).

   ```toml
   serde = { version = "1.0.229", features = ["derive"] }
   schemars = "1.2.3"
   ```

4. **Elicitation without the `elicitation` feature.** The probe sends `elicitation/create` with
   `Peer::send_request` and `ServerRequest::ElicitRequest`, and reads `ClientResult::ElicitResult`. Both
   types are ungated in rmcp 3.5.1 (`src/model.rs:3949`, `:4712`). This avoids 28 crates. The typed
   helpers (`create_elicitation`, `elicit_url`) are gated behind the feature; the binary size difference is
   negligible (4.63 MB against 4.58 MB, measured), so the cost is graph size and compile time.

5. **Protocol version and era.** M0-09 is a legacy-era server (spec "Protocol era and versions"). rmcp's
   default `supported_protocol_versions` is overridden to `[2025-06-18, 2025-11-25]`. Without this override,
   2026-07-28 per-request `_meta` requests are answered as stateless (verified in the spike), so the
   override is part of this decision.

6. **Logging.** No logging crate in M0. rmcp uses `tracing` macros, which are non-optional and already in
   the graph. No subscriber is installed, so stdout carries only JSON-RPC and stderr stays empty
   (spec "Process lifecycle"). A subscriber (tracing-subscriber 0.3.23, MIT, 2026-03-13, +4 crates:
   lazy_static, sharded-slab, thread_local) is deferred to M4 with its own approval line.

7. **serde_json 1.0.154, MIT OR Apache-2.0, as `[dev-dependencies]` of crates/cli.** It is already in the
   normal graph through rmcp, so this adds no crate. Tests speak raw JSON-RPC (spec "Tests").

   ```toml
   # crates/cli/Cargo.toml
   [dev-dependencies]
   serde_json = "1.0.154"
   ```

8. **Internal dependency.** `crates/cli` depends on `aitm-server` through a versioned path dependency
   (ADR-0003 wildcard rule). `aitm-server` stays publishable.

   ```toml
   # crates/cli/Cargo.toml
   aitm-server = { path = "../server", version = "0.1.0" }
   ```

9. **Transitive pins (14-day rule).** `cargo update` otherwise selects six crates younger than 14 days:
   rmcp-macros 3.8.0, tokio 1.54.1, uuid 1.29.0, libc 0.2.191, cc 1.10.0, find-msvc-tools 0.1.15. These are
   pinned in `Cargo.lock` to rmcp-macros 3.5.1, tokio 1.53.3, uuid 1.28.0, libc 0.2.190, cc 1.8.0,
   find-msvc-tools 0.1.14. Any lockfile regeneration must reapply them (ADR-0003 section 2.5):

   ```sh
   cargo update -p rmcp-macros --precise 3.5.1
   cargo update -p tokio --precise 1.53.3
   cargo update -p uuid --precise 1.28.0
   cargo update -p libc --precise 0.2.190
   cargo update -p cc --precise 1.8.0
   cargo update -p find-msvc-tools --precise 0.1.14
   ```

10. **`server/discover` before `initialize` (resolved, maintainer 2026-10-28).** rmcp 3.5.1 answers
    `server/discover` received before `initialize` with JSON-RPC error `-32602`. This is hard-wired in
    rmcp above the `ServerHandler`; overriding the handler does not change it. The maintainer accepted
    `-32602` instead of the `-32601` first drafted in the spec. `-32602` is outside the range reserved
    for modern MCP errors (`-32099..=-32020`), so dual-era clients still classify the server as legacy
    and fall back to `initialize`. No transport wrapper intercepts the request. Spec test 24 asserts
    `-32602` and that a following `initialize` succeeds.

### Amendment 2026-10-28 (code and security review of M0-09; maintainer decisions)

- **schemars** stays declared as in Decision 3, but in M0 the probe tool schemas are hand-written JSON;
  direct use of schemars starts with plugin-sdk (M2).
- **tokio `time` and `sync`** are used directly by `aitm-server` (elicitation timeout, in-flight cap,
  input-end grace), so `crates/server/Cargo.toml` declares them explicitly:
  `tokio = { version = "1.53.3", default-features = false, features = ["rt", "time", "sync"] }`.
  rmcp already enables both transitively, so the resolved graph and `Cargo.lock` do not change.
- **In-flight cap:** at most 32 `tools/call` in flight (spec `docs/spec/mcp-probe.md`, "Concurrency"),
  implemented with `std::sync::atomic::AtomicUsize` and an RAII slot; a client cancellation frees the
  slot at once; no new crate.
- **No rewriting of client messages:** stdin is wrapped only to count line bytes and record read errors
  (spec "Implementation requirements"). rmcp 3.5.1 limitations (no response to a `tools/call` with non-object
  `arguments`; no response to requests with non-standard ids) are accepted and recorded in the spec;
  revisit on an rmcp upgrade or in M4.

## Alternatives

- **rmcp with the `elicitation` feature.** Typed helpers, +28 crates. Rejected for M0; revisit in M4.
- **rust-mcp-sdk 2.0.0 (MIT, 2026-08-27) or turbomcp 3.6.0 (MIT, 2026-10-01).** Active community SDKs,
  not spiked. Rejected: not official; rmcp is the SDK the PLAN names and is Tier 1 on the official page.
- **mcp-core 0.1.50, async-mcp 0.1.3, mcp-sdk 0.0.3.** Last released 2025. Rejected as stale.
- **Hand-rolled JSON-RPC over stdio with serde_json.** No new crates beyond serde_json. Rejected for M0:
  negotiation, schemas, cancellation, elicitation and concurrency, across five spec revisions, would all be
  ours to maintain. Remains the isolated-layer fallback if rmcp churn becomes too costly, decided separately.
- **Python thin layer (PLAN section 12 fallback).** Breaks the single binary (PLAN section 2). Last resort.
- **Transport wrapper answering `server/discover` with `-32601`** (Decision 10, option b). Rejected by the
  maintainer: extra protocol code for no change in client behavior.

## Consequences

- Positive: official SDK with 41 million downloads and a Tier 1 listing; all 74 new registry crates are on
  the allowlist (MIT, Apache-2.0, MIT/Apache-2.0, Unlicense OR MIT); cargo-deny passes all four sections;
  cargo-audit reports 0 vulnerabilities (88 packages); no `-sys` or native code, and no HTTP, TLS or auth
  crates; no `unsafe` in our crates (`unsafe_code = "forbid"`).
- Positive: elicitation is verified end to end on the stdio path, with capability detection and all three
  client actions.
- Negative: rmcp had three major releases in four months (1.0.0 2026-03-03, 2.0.0 2026-06-29, 3.0.0
  2026-07-28); upgrades need a new ADR. The lockfile needs the six pins in Decision 9 after every update.
- Negative: 74 new crates, one duplicate (`syn` 2.0.119 next to 3.0.9, cargo-deny warning). Linux release
  build time 6.6 s to 43.8 s on the measured machine; release binary 0.99 MB to 4.63 MB.
- Negative: `chrono` enters through rmcp's schemars integration; this cannot be avoided with the `server`
  feature. rmcp's build script runs `git config core.hooksPath` only in a workspace layout (verified by
  reading the source).
- Negative: the spec requires the following that rmcp does not do by default (implementation work, no new
  crate): protocol version override (Decision 5), EOF before `initialize` must exit 0 (rmcp reports an
  error), a 1 MiB line limit (rmcp's stdio path is unbounded), and read errors must not look like EOF.
  The spec section "Implementation requirements" lists them.
- Follow-ups: update the PLAN tier wording after the live page is confirmed; M10 release task bundles
  third-party Apache-2.0 license texts (rmcp's crate ships no LICENSE file; the upstream LICENSE is a
  transitional MIT/Apache-2.0 notice; both are allowlisted); M4 decides the logging subscriber and the
  `elicitation` feature, each with its own approval line; modern-era (2026-07-28) support is revisited
  after M0-10.
- Rules out: enabling `elicitation`, `client`, `base64`, auth, HTTP, reqwest or child-process features
  without an approval line; a logging subscriber without an approval line; `rmcp` versions other than the
  pin without a new ADR.

## Sources (accessed 2026-10-28)

- crates.io API: https://crates.io/api/v1/crates/rmcp (versions, dates, downloads, owners);
  https://crates.io/api/v1/crates/rmcp/3.5.1 and .../3.5.1/dependencies (features, dependencies, MSRV
  1.88); https://static.crates.io/crates/rmcp/rmcp-3.5.1.crate (source inspected);
  https://crates.io/api/v1/crates/<name>/<version> for the 74 new crates (license, created_at, yanked).
- GitHub API: https://api.github.com/repos/modelcontextprotocol/rust-sdk (stars, open issues, pushed_at,
  license field NOASSERTION); https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/LICENSE.
- MCP SDK tiers: https://modelcontextprotocol.io/docs/sdk (Rust: Tier 1);
  https://modelcontextprotocol.io/community/sdk-tiers (criteria).
- MCP specification 2026-07-28 changelog: https://modelcontextprotocol.io/specification/2026-07-28/changelog
  (stateless `_meta`, `server/discover`, MRTR, removed `ping`, deprecated Roots, Sampling and Logging);
  https://modelcontextprotocol.io/specification/2026-07-28/basic/lifecycle (legacy, modern and dual-era
  compatibility; reserved error range).
- Community crates (crates.io metadata only): rust-mcp-sdk, turbomcp, mcp-core, async-mcp, mcp-sdk.
- Tools: cargo 1.99.1, rustc 1.99.1, cargo-deny 0.20.2 (`cargo deny --locked check`, `cargo deny list`),
  cargo-audit 0.22.1 (`cargo audit`, RustSec DB commit `40f5be6`, 1381 advisories).
- Repository: docs/adr/0003-dependency-license-policy.md, docs/adr/0005-clap-cli.md, docs/PLAN.md sections
  4, 12, 14, docs/spec/mcp-probe.md, docs/tasks/M0/09-mcp-probe-server.md.
