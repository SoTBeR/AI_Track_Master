# ADR-0002: Rust as the language of core, engine, MCP server and CLI

- Status: accepted
- Date: 2026-10-27
- Deciders: maintainer

## Context

`docs/PLAN.md` labels this decision "ADR-001" (section 4, paragraph "Почему Rust (ADR-001)";
section 14 item 3). Repository ADR numbers follow creation order (see ADR-0001), so the plan's
"ADR-001" is this ADR-0002. Quotes of the plan keep the plan's labels.

Forces from the plan:

- The product ships as one executable that contains the core, the mastering engine, the MCP server
  and the CLI, and needs neither Python nor any other runtime to work (PLAN section 2: "MCP-сервер и
  CLI собираются в один исполняемый файл на Rust; для работы не нужны ни Python, ни другие среды
  выполнения"). Heavy renders run in a worker pool behind `catch_unwind`; ffmpeg runs only as a
  separate process (PLAN section 2).
- Most of the code is written by AI agents (PLAN section 1, section 10). Memory errors and data races
  in agent-written code must be caught mechanically, not by review alone.
- Releases target Windows, macOS and Linux from the first release (PLAN section 1; section 4 rows
  "CI" and "Дистрибуция"; section 12 row on platform differences), distributed via cargo-dist,
  `cargo install` and Homebrew.
- A GUI is planned after 1.0; the main candidate is Tauri, which is written in Rust and can use the
  same core directly (PLAN section 13).
- The plan's stack table (PLAN section 4) already assumes Rust stable with a Cargo workspace, rustfft,
  rmcp (the official Rust MCP SDK) with tokio, clap, serde/schemars, symphonia, hound, flacenc,
  rubato, ebur128, rusqlite. Crate names, versions and licenses are confirmed separately in M0 via
  cargo-deny (ADR-0003, task M0-04); this ADR does not approve any dependency.

## Decision

1. Core, dsp, analysis, engine, plugin-sdk, built-in plugins, MCP server and CLI are written in Rust
   as one Cargo workspace and built into one executable (binary name per ADR-0001).
2. Rationale, as stated in PLAN section 4 "Почему Rust (ADR-001)":
   - one language and one executable for core, server and CLI;
   - the compiler rules out memory errors and data races in code written by agents;
   - cross-building and distribution for three operating systems is simpler;
   - a future Tauri GUI can use the core directly.
3. Toolchain assumption (PLAN section 13: "Rust stable последней версии на момент старта"): the
   project starts on **Rust stable 1.99.1** (rustc `1.99.1 (58cf8f9c3 2026-10-13)`; stable channel
   manifest dated 2026-10-15; checked 2026-10-27 at
   https://static.rust-lang.org/dist/channel-rust-stable.toml). Task M0-06 pins this version in
   `rust-toolchain.toml`. Moving to a newer stable later is a normal maintenance change, not a new
   ADR, as long as CI on all three operating systems stays green.
4. Python is a development-only language (PLAN sections 4 and 13): independent reference
   implementations in `reference/` (numpy, scipy, pyloudnorm, pytest, ruff) and eval scripts. Python
   is never part of the shipped product and never a runtime requirement for users.

## Downsides (from PLAN section 4)

- **Fewer ready-made audio libraries.** Part of the algorithms will have to be written in-house.
- **No stable ABI for third-party plugins.** In v1 all plugins are built in and compiled with the
  application; external plugin loading comes later via CLAP or WASM (PLAN section 7).
- **Maturity of the official Rust MCP SDK (rmcp).** The plan states that the official Rust SDK is
  Tier 3 in the MCP SDK table ("поддержка части функций может быть неполной"). Live check on
  2026-10-27: the official SDK list at https://modelcontextprotocol.io/docs/sdk now lists the Rust
  SDK (`modelcontextprotocol/rust-sdk`, crate `rmcp`) as **Tier 1**. Per the tiering page
  https://modelcontextprotocol.io/community/sdk-tiers.md, Tier 1 means 100% conformance-test pass
  rate and complete protocol implementation including optional capabilities such as sampling and
  elicitation; Tier 3 means "Experimental, partially implemented, or specialized". The same page
  says an SDK can be relegated if conformance tests fail continuously for 4 weeks, so the tier is a
  snapshot, not a guarantee. The plan's statement is therefore outdated as of this date; the risk is
  lower than the plan assumed but is not removed, because the conformance suite does not prove
  behavior with the specific MCP clients this product targets (see open questions).

## Mitigations (from PLAN section 12)

| Risk (PLAN section 12) | Mitigation |
|---|---|
| Official Rust MCP SDK less mature than Python and TypeScript SDKs | Probe MCP server on rmcp in M0 with stdio and an elicitation check in two or three real clients (tasks M0-09, M0-10; PLAN section 14 item 3); protocol isolated in its own layer (`crates/server`) so the core API does not depend on rmcp types; pinned SDK version (PLAN section 12 row "Изменения спецификации MCP"); fallback plan: a thin Python layer over the core. |
| Fewer audio libraries; some algorithms written in-house | Independent Python references for verification (`reference/`); early pilot in M0-M1 (tasks M0-18, M0-19); crate selection through ADRs and dependency-auditor review. |
| Subtle DSP errors in agent-written code (the compiler catches memory errors, not math errors) | Independent references, null tests, personal review of key algorithms (PLAN section 12; six verification levels in PLAN section 8). |
| MCP clients without elicitation support | Fallback confirmation path through the CLI, only from an interactive terminal (PLAN section 12). |

The fallback "thin Python layer over the core" would only replace the protocol shell. It must call
the same core and engine API and cannot add a path around the engine or the "do no harm" check
(AGENTS.md, "Core product principle"). Adopting it would require a new ADR, because it changes the
single-binary, no-runtime property of decision 1.

## Alternatives

- **Python for the whole product** (Tier 1 MCP SDK, rich audio/scientific ecosystem): requires a
  runtime on users' machines, contradicting the single-binary goal (PLAN section 2); memory
  safety comes from the runtime, not the compiler, and there is no static data-race checking;
  harder three-OS distribution. Rejected; Python stays for references and eval only.
- **TypeScript/Node for server and CLI with a native DSP core**: two languages and an FFI boundary in
  the main product; embeds or requires a JS runtime. Rejected on the two-language and FFI cost and
  on the plan's rationale (one language, one executable, PLAN section 4).
- **C++ for core and DSP**: mature audio ecosystem, but no compiler-checked memory safety for
  agent-written code and harder cross-OS builds. Rejected on the plan's main rationale.

## Consequences

- Positive: one codebase and one executable; memory safety and data-race freedom enforced by the
  compiler (with `#![forbid(unsafe_code)]` in DSP crates per AGENTS.md); one distribution pipeline
  (cargo-dist) for three OSes; a Tauri GUI after 1.0 can reuse the core without a protocol hop.
- Negative: more in-house DSP, which raises the importance of independent references and the
  verification levels of PLAN section 8; no third-party plugin loading in v1; dependence on rmcp
  for the MCP layer.
- Follow-up: M0-04 (ADR-0003) defines the dependency and license policy and cargo-deny setup;
  M0-06 pins Rust 1.99.1 in `rust-toolchain.toml`; M0-09 and M0-10 build the rmcp probe server and
  check stdio and elicitation in real clients, and record the rmcp version used.
- Rules out: shipping a Python or other runtime with the product; loading external binary plugins
  in v1.

## Open questions

- The plan (section 4, section 12) describes the Rust MCP SDK as Tier 3, but the official SDK page
  lists it as Tier 1 on 2026-10-27. Should `docs/PLAN.md` be updated to reflect this? This ADR does
  not change the plan and keeps all mitigations from PLAN section 12 in force regardless of tier.
- Should the "Rust MCP SDK maturity" risk probability in PLAN section 12 (currently "Средняя") be
  revisited after tasks M0-09 and M0-10 report results?

## Sources (accessed 2026-10-27)

- https://modelcontextprotocol.io/docs/sdk (official SDK list with tiers; Rust SDK listed as Tier 1)
- https://modelcontextprotocol.io/community/sdk-tiers.md (tier definitions, conformance requirements,
  relegation rules)
- https://crates.io/api/v1/crates/rmcp (repository `https://github.com/modelcontextprotocol/rust-sdk/`;
  latest stable 3.8.0, for context only; the version is fixed in M0-09/M0-10)
- https://static.rust-lang.org/dist/channel-rust-stable.toml (current stable: 1.99.1)
