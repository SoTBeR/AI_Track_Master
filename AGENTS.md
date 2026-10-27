# AGENTS.md

Rules for every AI agent working in this repository. The full project plan is
`docs/PLAN.md` (written in Russian); it is the source of truth for requirements.
Do not invent requirements that are not in the plan or in a spec under `docs/spec/`.
If the plan is ambiguous, stop and ask the maintainer.

## Product in one paragraph

An open-source (Apache-2.0) application for autonomous mastering and mixing of
stereo tracks, stems and multitrack projects. It ships as one Rust binary that
contains the core, the mastering engine, an MCP server (rmcp) and a CLI (clap).
An AI agent drives it over MCP, but **the agent cannot hear audio**: it only states
intent (platform, character, genre, reference) and reads reports. All sound
parameters are computed and verified by the built-in deterministic mastering engine.

## Core product principle (non-negotiable)

- The engine computes and verifies every processing parameter (plan sections 2 and 3).
- "Do no harm": the result must not be worse than the input, or than the baseline
  (loudness normalization + limiter), by objective metrics at matched loudness.
  On failure the engine weakens processing; as a last resort it returns the baseline
  with a warning in the report.
- **No code path may let an agent, MCP tool, CLI flag or config bypass the
  "do no harm" check.** Every export goes through it. Low-level tools exist for
  fine work, but they cannot skip verification.
- No silent failures: every error returns a report with a reason; a partial result
  is never presented as finished.
- Determinism: same input + same intent = same output in any MCP client, with any
  model, and from the CLI without an agent (cross-platform differences only within
  stated tolerances).

## Architecture

The system is layered. Interfaces are thin shells over one core API.

- **core**: session graph (tracks, buses, master bus; a stereo file is a one-track
  session), audio I/O, offline 64-bit float render, storage (SQLite + files),
  non-destructive session versions.
- **dsp**: filters, oversampling, FFT utilities.
- **analysis**: loudness, spectrum, dynamics, stereo, transients, sections, problem detectors.
- **engine**: intake checks, analysis, diagnosis, target selection, chain planning,
  optimization loop, "do no harm" check, delivery, report.
- **plugin-sdk / plugins**: the `Plugin` trait, parameter schemas, built-in plugins
  (each its own crate). No external plugin loading in v1.
- **server**: MCP server on rmcp, job queue, autonomy modes, write-path protection, audit log.
- **cli**: clap commands mirroring MCP tools, plan approval.
- **eval**: metrics, corpus manifests, agent run harness, blind listening tests.

Heavy renders run in a worker pool behind `catch_unwind`. ffmpeg runs only as a
separate process. Target profiles (platforms, material) are versioned data in
`profiles/`, never hard-coded.

## Directory map

```
AGENTS.md, CLAUDE.md     agent rules
LICENSE, NOTICE          Apache-2.0 (NOTICE lists MPL-2.0 deps)
CHANGELOG.md             Keep a Changelog format, "Unreleased" on top
Cargo.toml, deny.toml    workspace, license/vulnerability policy   (M0)
docs/PLAN.md             project plan
docs/adr/                architecture decision records, NNNN-slug.md
docs/spec/               short module specs
docs/tasks/<M#>/         milestone task files, NN-slug.md
docs/templates/          spec, task and ADR templates
crates/core|dsp|analysis|engine|plugin-sdk|server|cli/
crates/plugins/<name>/   one crate per built-in plugin
profiles/                platform targets and material profiles (data)
eval/                    corpus manifests, metrics, scenarios, listening tests
reference/               independent Python reference implementations + pytest
.github/workflows/       CI
.claude/                 agents, skills, settings for Claude Code
```

## Layers and owners

One task touches exactly one layer. Owner = the only agent allowed to write that layer's code.

| Layer | Paths | Owner |
|---|---|---|
| dsp | `crates/dsp/` | dsp-engineer |
| analysis | `crates/analysis/` | dsp-engineer |
| plugins | `crates/plugin-sdk/`, `crates/plugins/` | dsp-engineer |
| engine | `crates/engine/`, `profiles/` | engine-engineer |
| platform | `crates/core/`, `crates/server/`, `crates/cli/`, `eval/` harness code, `.github/`, root Cargo/deny/dist config | platform-engineer |
| docs | `docs/` | architect |
| reference | `reference/` | reference-author |
| tests | `crates/*/tests/`, `crates/plugins/*/tests/`, `reference/tests/` | test-author |

## Commands

Placeholders until the workspace exists; M0 will finalize them. Summarize long output
(`2>&1 | tail -n 40`) instead of pasting it.

```sh
cargo fmt --all --check                                   # format
cargo clippy --workspace --all-targets -- -D warnings     # lint
cargo nextest run --workspace                             # tests (fallback: cargo test --workspace)
cargo deny check                                          # licenses, bans, advisories
cargo audit                                               # vulnerabilities
python3 -m pytest reference/                              # reference implementations
ruff check reference/                                     # Python lint
cargo run -p eval -- golden --compare main                # golden set (TBD in M0)
```

"Local checks" means: fmt, clippy, tests, cargo-deny, and pytest when `reference/` changed.

## Code style and naming

- Rust stable, latest edition; `rustfmt` defaults; clippy warnings are errors.
- Crates `snake_case` directories; types `UpperCamelCase`; functions and modules `snake_case`.
- Units in names where ambiguous: `gain_db`, `freq_hz`, `time_ms`, `level_lufs`, `peak_dbtp`.
- Public items have doc comments; DSP code cites the standard or paper it implements.
- Errors: typed (`thiserror`-style) in libraries, never `panic!` on user input. No `unwrap()`
  outside tests except on proven invariants with a comment.
- Python references: plain numpy/scipy/pyloudnorm, `ruff`-clean, one module per algorithm.
- English for all code, comments, docs and commit messages.

## Prohibitions

- No allocations, locks or system calls in `process` (checked with an allocation trap such as `assert_no_alloc`).
- No `unsafe` without a written justification and Miri tests; DSP crates use `#![forbid(unsafe_code)]`.
- No new dependency without a dependency-auditor review and an accepted ADR.
- No GPL or LGPL dependencies (static linking). MPL-2.0 allowed with a NOTICE entry. ffmpeg only as a subprocess.
- No change to public MCP schemas (or CLI surface) without a versioned migration and an ADR.
- No edits across several layers in one task.
- No bypass of the engine or the "do no harm" check (see above).
- Executors never modify tests or reference implementations; if a test looks wrong, stop and report.
- No rewriting whole modules; no tests fitted to the implementation.
- Never modify or delete source audio files; writes only inside the project folder.

## Definition of done

- Spec exists in `docs/spec/` and acceptance criteria are met.
- Tests written first from spec/reference, now passing; for DSP: null, block-size, NaN/Inf, frequency-response tests.
- Public API and behavior documented; ADR added for any architectural decision.
- `CHANGELOG.md` "Unreleased" updated.
- All local checks green; required reviewers report no blockers; metric deltas reported for engine/plugins.

## Git

- One branch per task: `task/<M#>-<NN>-<slug>`. Small changes.
- Conventional Commits (`feat(dsp): ...`, `fix(server): ...`, `docs(adr): ...`, `test(analysis): ...`).
- Every commit signed off for DCO: `git commit -s`.
- Never force push. Never rewrite published history.
- Never merge into `main`; the maintainer merges after review.
