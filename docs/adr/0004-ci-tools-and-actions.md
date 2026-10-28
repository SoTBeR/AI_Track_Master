# ADR-0004: Approved CI tools and GitHub Actions

- Status: accepted
- Date: 2026-10-28
- Deciders: maintainer
- Amends: ADR-0003 section 3 (dev-only guard runs with `--target all`)

## Context

Task M0-07 (`docs/tasks/M0/07-ci-matrix.md`) adds the GitHub Actions CI matrix. Its notes require
dependency-auditor to review the CI tools (cargo-nextest, cargo-deny, cargo-audit) and every GitHub
Action used, with the result recorded in an ADR.

Forces:

- PLAN section 4 (row "CI"), section 11 "CI/CD и релизы": builds on Windows, macOS and Linux;
  rustfmt, clippy, cargo-deny, cargo-audit, ruff; cargo nextest and pytest.
- PLAN section 10 "Защитные барьеры": any failing check blocks merge.
- PLAN section 11 ("Дополнительно"): automatic dependency updates (Dependabot or Renovate) pass the
  full check suite.
- ADR-0003: license allowlist, the 14-day version rule (section 2.2), cargo-audit ignore list
  mirroring `[advisories].ignore`, the dev-only guard for `assert_no_alloc` (section 3).
- CI tools and Actions run code with access to the repository checkout and the workflow token, so
  they are a supply-chain surface even though they are never shipped.

Evidence: dependency-auditor review on 2026-10-28 using the crates.io API, the GitHub releases of
each Action, RustSec, OSV and the GitHub Advisory Database (all accessed 2026-10-28), followed by a
code review of the M0-07 workflow draft.

## Decision

### 1. GitHub Actions

Only GitHub-owned Actions are used; no third-party Actions. Each is pinned by full commit SHA (the
version tag is kept as a comment for readability). All are MIT-licensed; no advisories were found in
OSV or the GitHub Advisory Database (accessed 2026-10-28).

| Action | Version (released) | Pinned SHA | License |
|---|---|---|---|
| actions/checkout | v7.0.1 (2026-07-20) | `3d3c42e5aac5ba805825da76410c181273ba90b1` | MIT |
| actions/upload-artifact | v7.0.2 (2026-10-07) | `cf430e030ddbb5b0abf93d22962f4752f3646cd9` | MIT |
| actions/cache | v6.1.0 (2026-06-26) | `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` | MIT |
| actions/setup-python | v7.0.0 (2026-07-20) | `5fda3b95a4ea91299a34e894583c3862153e4b97` | MIT |

Rust is provided by the rustup preinstalled on GitHub-hosted runners together with the repository's
`rust-toolchain.toml`; no toolchain-setup Action is used.

### 2. Cargo tools

Installed in CI with `cargo install --locked --version <exact>`:

| Tool | Version (released) | License (crates.io) | Notes |
|---|---|---|---|
| cargo-deny | 0.20.2 (2026-07-09) | MIT OR Apache-2.0 | OK |
| cargo-audit | 0.22.2 (2026-06-05) | Apache-2.0 OR MIT | OK |
| cargo-nextest | 0.9.150 (2026-10-26) | Apache-2.0 OR MIT | Security exception to the 14-day rule, see below |

- **cargo-nextest 0.9.148 rejected** (the release the 14-day rule would select):
  RUSTSEC-2026-0412 / GHSA-vc25-qg58-w57c, CVSS 7.8, code execution via `--rerun` of a malicious
  recording with `NEXTEST_EXPERIMENTAL_RECORD`; affected `>=0.9.125, <0.9.150`.
- **0.9.150 is younger than 14 days** on the decision date. Per ADR-0003 section 2.2, a newer
  release may be chosen with a recorded reason; the reason is the security fix above. This is a
  maintainer decision.
- The minimum nextest version is also recorded in `.config/nextest.toml` with
  `nextest-version = "0.9.150"` (platform layer, added by M0-07), so local runs with an older,
  vulnerable nextest fail fast.

### 3. Licenses in tool dependency graphs

The dependency graphs of the installed tools contain licenses that are not on the ADR-0003
allowlist: MPL-2.0, LGPL-2.1-or-later (via the `r-efi` OR-choice), BSL-1.0, WTFPL,
CDLA-Permissive-2.0. These are **accepted for build-time use only**: the tools run in CI and are
never linked into or distributed with the project. CI never publishes tool binaries; the only
published artifact is the `aitm` binary, whose graph remains fully governed by ADR-0003 and
`deny.toml`.

### 4. Runner labels

Matrix labels (documented GitHub-hosted labels,
https://docs.github.com/en/actions/reference/runners/github-hosted-runners, accessed 2026-10-28):

- `ubuntu-24.04` (Linux x86_64)
- `windows-2025` (Windows x86_64)
- `macos-15` (macOS arm64)
- `macos-15-intel` (macOS x86_64)

`macos-15-intel` is GitHub's last Intel macOS image. No official retirement date was found on
2026-10-28; the maintainer watches the `actions/runner-images` announcements and plans an
alternative for macOS x86_64 builds (for example cross-compilation) before retirement.

### 5. CI rules decided in code review

1. **Dev-only guard uses `--target all`**: the command becomes
   `cargo tree -p aitm -e normal,build -i assert_no_alloc --locked --target all`, so a normal
   dependency gated on `cfg(windows)` or `cfg(target_os = "macos")` is caught even when the guard
   runs on Linux. Pass/fail semantics are unchanged from ADR-0003 section 3. ADR-0003 is amended
   accordingly.
2. **cargo-audit ignore entries mirror `deny.toml` `[advisories].ignore`** exactly, either as
   `--ignore` flags in the workflow or in `.cargo/audit.toml`, each with the same id + reason + ADR
   reference required by ADR-0003.
3. **clippy runs on Linux only in M0.** Lint on the other OSes when OS-specific code
   (`cfg(windows)`, `cfg(target_os = "macos")`) appears.
4. **No build cache yet.** Add caching (via the approved `actions/cache`) when build time grows;
   that change needs no new ADR as long as it uses an Action from section 1.

### 6. Update policy

- Every bump of a CI tool or Action needs a repeat check: RustSec/OSV/GitHub advisories, release age
  (14-day rule or a recorded exception), license, and a dependency-auditor note (an approval line in
  a later ADR, or an amendment of this one). Adding a new Action or tool needs a new ADR.
- `cargo audit` in CI checks `Cargo.lock` only; it does **not** cover the installed tools, which is
  why the manual check above is required.
- No Dependabot or Renovate yet (follow-up per PLAN section 11; out of scope for M0-07).

## Alternatives

- **Third-party toolchain and install Actions** (for example a Rust toolchain setup Action or a
  prebuilt-binary installer): faster setup, but each adds a non-GitHub publisher with access to the
  workflow token and must be audited and re-audited. Rejected; preinstalled rustup plus
  `rust-toolchain.toml` and `cargo install --locked` are sufficient.
- **Pin Actions by version tag** instead of SHA: simpler to read and bump, but tags are mutable and
  can be moved to malicious commits. Rejected.
- **Prebuilt tool binaries** downloaded from GitHub releases: faster than `cargo install`, but needs
  checksum management per OS and a separate trust path. Rejected for M0; may be revisited with its
  own ADR if install time becomes a problem.
- **cargo-nextest 0.9.148** (complies with the 14-day rule): affected by RUSTSEC-2026-0412.
  Rejected.
- **`cargo test` instead of nextest**: no new tool, but PLAN section 11 names nextest. Kept only as a
  local fallback (AGENTS.md "Commands").
- **Apply the ADR-0003 allowlist to tool graphs**: would forbid all three tools. Rejected; the
  allowlist governs what is linked into and shipped with `aitm`.

## Consequences

- Positive: a small, fully GitHub-owned and SHA-pinned Action set; exact tool versions make CI
  reproducible; the known nextest vulnerability is excluded both in CI and locally via
  `nextest-version`; the dev-only guard now covers OS-gated dependencies.
- Negative: `cargo install` of each tool runs on a tool-cache miss (cached per OS and exact version via `actions/cache`); there is no build cache yet;
  bumps are manual until Dependabot/Renovate is configured; clippy does not yet see OS-specific code
  on Windows/macOS.
- Risk: `macos-15-intel` retirement would remove the native macOS x86_64 build.
- Follow-up: M0-07 (platform) adds `.config/nextest.toml` with `nextest-version = "0.9.150"` and the
  cargo-audit ignore mirroring; a later task configures Dependabot or Renovate (PLAN section 11);
  add a build cache when build time grows; extend clippy to Windows/macOS when OS-specific code
  appears; M0-15 adds `reference/requirements.txt` with pinned versions for the CI Python job.
- Rules out: unpinned or tag-pinned Actions; third-party Actions without a new ADR; unversioned
  `cargo install` of CI tools; publishing tool binaries from CI.

## Sources (accessed 2026-10-28)

- https://github.com/actions/checkout/releases, https://github.com/actions/upload-artifact/releases,
  https://github.com/actions/cache/releases, https://github.com/actions/setup-python/releases
- https://crates.io/api/v1/crates/cargo-deny, https://crates.io/api/v1/crates/cargo-audit,
  https://crates.io/api/v1/crates/cargo-nextest
- https://rustsec.org/advisories/RUSTSEC-2026-0412, https://github.com/advisories/GHSA-vc25-qg58-w57c
- https://osv.dev (Actions and tools)
- https://docs.github.com/en/actions/reference/runners/github-hosted-runners
