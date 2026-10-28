# ADR-0003: Allowed dependencies, licenses and cargo-deny policy

- Status: accepted
- Date: 2026-10-27
- Deciders: maintainer
- Amended: 2026-10-28 by ADR-0004 (dev-only guard uses --target all)

## Context

`docs/PLAN.md` labels this decision "ADR-002" (section 14 item 3: "ADR-002 (допустимые зависимости и
лицензии, настройка cargo-deny)"). Repository ADR numbers follow creation order (ADR-0001), so the
plan's "ADR-002" is this ADR-0003.

Forces from the plan:

- PLAN section 2, principle 10 "Чистая лицензия": only dependencies compatible with Apache-2.0. Crates
  are linked statically, so LGPL and GPL are not allowed; MPL-2.0 is allowed with a NOTICE entry;
  ffmpeg runs as a separate process; license compatibility is checked by cargo-deny.
- PLAN section 4: crate names, versions and licenses of the stack are confirmed in M0 through
  cargo-deny. The stack row "Чтение аудио" already states that symphonia is MPL-2.0 and must be
  recorded in NOTICE; the row "Кодеки" says the license of the specific ffmpeg build must be checked;
  the row "Оптимизация" allows "argmin или собственный поиск в границах".
- PLAN section 11 ("Дополнительно"): Cargo.lock is committed, an SBOM is built for every release,
  cargo-deny checks dependency licenses for Apache-2.0 compatibility; automatic dependency updates
  (Dependabot or Renovate) pass the full check suite. Section 11 style stage: cargo-deny (licenses)
  and cargo-audit (vulnerabilities) in CI.
- PLAN section 12 risk "Лицензии зависимостей": mitigated by cargo-deny in CI, ADRs, ffmpeg as a
  separate process.
- AGENTS.md "Prohibitions": no new dependency without a dependency-auditor review and an accepted
  ADR; no GPL or LGPL; MPL-2.0 with a NOTICE entry; ffmpeg only as a subprocess.

Evidence: a dependency-auditor survey on 2026-10-27 using the crates.io API and a scratch resolve
of the stack crates outside the repository, checked with cargo-deny 0.20.2 and cargo-audit 0.22.1.
The symphonia default-feature crate list was re-checked by the architect the same day against the
crates.io dependency metadata of symphonia 0.6.1 and symphonia-bundle-flac 0.6.1.

## Decision

### 1. License policy

1. **Allowlist** (SPDX identifiers; every crate in the resolved graph, including dev and build
   dependencies, must satisfy its license expression using only these):
   `Apache-2.0`, `MIT`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `Unicode-3.0`,
   `Unicode-DFS-2016`, `CC0-1.0`, `Unlicense`.
2. **Everything else is denied by default**, including GPL-*, LGPL-*, AGPL-*, SSPL, EUPL, CDDL, any
   other copyleft that would require releasing the source of the statically linked binary, and any
   crate without a license expression or license file that cargo-deny can identify.
3. **`OR` expressions** pass when at least one alternative is on the allowlist (the project uses that
   alternative). **`AND` expressions** pass only when every term is allowed or excepted. Example from
   the survey: `r-efi` is `MIT OR Apache-2.0 OR LGPL-2.1-or-later` and is accepted via MIT; an
   expression `X AND LGPL-*` would be rejected.
4. **MPL-2.0** is not on the allowlist. It is accepted only through a per-crate exception that names
   the exact crate, and each excepted crate gets an entry in `NOTICE` (crate name, version range in
   use, license, upstream source URL, statement that the crate is used unmodified). New MPL-2.0
   exceptions are added only through an ADR (a new ADR or an approval line in the ADR of the task
   that adds the crate). Currently approved MPL-2.0 exceptions: the symphonia crates in section 4.
5. **Non-allowlisted permissive licenses** are handled the same way: crate-scoped exception plus an
   ADR. Currently approved: `assert_no_alloc` under `BSD-1-Clause`, dev-dependency only (section 4).
6. **ffmpeg** is used only as a separate process. No crate that links or binds ffmpeg/libav* (for
   example `ffmpeg-next`, `ffmpeg-sys-next`, `ffmpeg-sys`) may enter the graph. The license of any
   ffmpeg build that the project distributes or recommends is checked separately (PLAN section 4,
   row "Кодеки") in the task that introduces ffmpeg use.
7. The workspace's own crates are licensed `Apache-2.0` (ADR-0001 names; M0-02 license files).

### 2. Dependency admission rules

1. **New dependency = dependency-auditor review + accepted ADR** (or an explicit approval line in a
   later ADR that names the crate, version and license). This applies to normal, build and dev
   dependencies and to enabling a new Cargo feature that pulls new crates.
2. **Version rule**: when a dependency is added, use the latest stable (non-pre-release,
   non-yanked) release that is at least 14 days old on that day. A newer release may be chosen only
   with a reason recorded in the ADR (for example a security fix). This rule is a maintainer
   decision of this ADR, not a plan requirement: a short delay lets yanks and early regressions
   surface before a release enters the graph (tokio 1.54.0, yanked, is an example).
3. **Sources**: crates.io only. Git, path (outside the workspace) and alternative-registry sources
   are denied; a git source needs its own ADR, which also changes `deny.toml`.
4. **Feature flags**: enabling an optional feature that changes the resolved graph (for example
   symphonia `aiff`, `mp3`, `aac`, `alac`, `isomp4`; rmcp Streamable HTTP transport) requires a
   per-feature license and advisory check in the enabling task, recorded in its ADR. In particular, a
   TLS backend (openssl, ring, aws-lc) is not in today's resolve and would need its own review.
5. **Automatic updates** (Dependabot or Renovate, PLAN section 11) pass the full check suite
   including cargo-deny and cargo-audit. A bump within an already approved crate does not need a new
   ADR; a bump that adds a new crate to the graph, changes a crate's license, or needs a new
   exception does.
6. **The stack table in section 4 is informational.** It is not an approval to add any crate. Each
   crate still gets its own dependency-auditor review and ADR (or an approval line) when a task first
   adds it, and the final confirmation is cargo-deny on the real `Cargo.lock` (task M0-06 onward).

### 3. Rules `deny.toml` must enforce

Written by platform-engineer in task M0-06 against the cargo-deny 0.20 config schema. The list below
is TOML-shaped to fix every value; it is not the final file (comments, ordering and any keys that
cargo-deny requires but that carry no decision are up to M0-06).

```toml
[graph]
all-features = true   # feature-gated crates (symphonia aiff/mp3/aac, rmcp HTTP) are license-checked too
# targets: not set (default = all targets; see open question on the target filter).
# exclude-dev: not set (default false): dev and build dependencies are included in every check.

[licenses]
allow = [
  "Apache-2.0", "MIT", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib",
  "Unicode-3.0", "Unicode-DFS-2016", "CC0-1.0", "Unlicense",
]
confidence-threshold = 0.93
# Exact crate names; no wildcards. New entries only via ADR.
exceptions = [
  { crate = "symphonia",              allow = ["MPL-2.0"] },
  { crate = "symphonia-core",         allow = ["MPL-2.0"] },
  { crate = "symphonia-metadata",     allow = ["MPL-2.0"] },
  { crate = "symphonia-common",       allow = ["MPL-2.0"] },
  { crate = "symphonia-bundle-flac",  allow = ["MPL-2.0"] },
  { crate = "symphonia-codec-adpcm",  allow = ["MPL-2.0"] },
  { crate = "symphonia-codec-pcm",    allow = ["MPL-2.0"] },
  { crate = "symphonia-codec-vorbis", allow = ["MPL-2.0"] },
  { crate = "symphonia-format-mkv",   allow = ["MPL-2.0"] },
  { crate = "symphonia-format-ogg",   allow = ["MPL-2.0"] },
  { crate = "symphonia-format-riff",  allow = ["MPL-2.0"] },
  { crate = "assert_no_alloc",        allow = ["BSD-1-Clause"] },  # dev-dependency only, see 4
]
# These exceptions are pre-approved by this ADR and written into deny.toml in M0-06 now.
# Until the crates are actually in the graph, cargo-deny reports `license-not-encountered` /
# unmatched-exception warnings (also for allowlisted licenses nothing uses yet); these warnings
# are acceptable. An exception pre-approves only the license: each crate still needs its own
# dependency-auditor review and ADR approval line when a task first adds it.

[advisories]
# Vulnerabilities always fail (cargo-deny 0.20 has no switch to soften them).
unmaintained = "all"
unsound = "all"
yanked = "deny"
# Every ignore entry: { id = "RUSTSEC-YYYY-NNNN", reason = "<why> (ADR-NNNN)" }.
# An entry without an id, a reason and an ADR reference is not allowed. Empty in M0.
ignore = []

[bans]
multiple-versions = "warn"     # skip list revisited after M0; see Consequences
wildcards = "deny"
allow-wildcard-paths = true    # exempts path deps of private (publish = false) crates and dev-deps only
# Best-effort list of crates that link or bind ffmpeg/libav*; the dependency-auditor review is
# the primary control, this list is a backstop.
deny = [
  { crate = "ffmpeg-next" }, { crate = "ffmpeg-sys-next" }, { crate = "ffmpeg-sys" },
  { crate = "ffmpeg-the-third" }, { crate = "ffmpeg-sys-the-third" },
  { crate = "rsmpeg" }, { crate = "rusty_ffmpeg" }, { crate = "ac-ffmpeg" },
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

Additional CI rules (M0-06 / CI task, platform layer):

- `cargo deny check` (all four sections) and `cargo audit` run on every pull request and fail the
  build on error. cargo-audit uses the same ignore list as `[advisories].ignore`, with the same
  id + reason + ADR requirement.
- Wildcard rule for workspace crates. cargo-deny's `allow-wildcard-paths = true` exempts a path
  dependency without a version only in private crates (and dev-dependencies); in a publishable crate
  it is still reported as a wildcard (cargo-deny docs; failure reproduced by code review on a public
  crate). Therefore M0-06 must:
  - give every internal path dependency an explicit `version = "..."` equal to the workspace version
    (for example `aitm-core = { path = "../core", version = "<workspace version>" }`), and
  - library crates in `aitm`'s normal dependency graph (`aitm-core`, `aitm-dsp`, `aitm-analysis`,
    `aitm-engine`, `aitm-plugin-sdk`, `aitm-plugin-*`, `aitm-server`) stay publishable, because
    `cargo install aitm` requires them on crates.io (PLAN section 4 "Дистрибуция"; ADR-0001);
    `publish = false` is set only on crates outside that graph (e.g. `aitm-eval`, test-only crates).
    Versioned path dependencies alone satisfy `wildcards = "deny"` in publishable crates.
  `wildcards = "deny"` stays as is; `allow-wildcard-paths = true` stays for the private crates and
  dev-dependencies.
- A dev-only guard for crates excepted as dev-only (today `assert_no_alloc`): CI runs
  `cargo tree -p aitm -e normal,build -i assert_no_alloc --locked --target all`. Pass: the crate is absent from
  the normal/build graph, which cargo reports as "package ID specification ... did not match any
  packages" with exit code 101; the CI step must treat exactly this outcome (or empty output) as
  success. Fail: the command prints a non-empty dependency path from the shipped binary to the crate.
  Any other error also fails the step. cargo-deny cannot express "dev-only", so this separate check is
  required.
- `Cargo.lock` is committed to the repository (PLAN section 11) and CI builds with `--locked`.
  `--locked` is a maintainer decision of this ADR, not a plan requirement: it makes CI fail instead
  of silently re-resolving, so the checked graph is exactly the committed one.
- An SBOM is produced for every release (PLAN section 11); the format and tool are chosen in the
  release task (M10) with its own ADR if a new tool is needed.

### 4. Stack crate table (snapshot 2026-10-27)

Source: crates.io API (`https://crates.io/api/v1/crates/<name>`), accessed 2026-10-27. "Version"
is the version that the 14-day rule selects on that date; it is recorded for the later ADRs, not
pinned here.

| Crate | Version (released) | License (crates.io) | Source | Status / notes |
|---|---|---|---|---|
| rmcp | 3.5.1 (2026-10-05) | Apache-2.0 | https://github.com/modelcontextprotocol/rust-sdk | OK. 3.6.0-3.8.0 younger than 14 days. Streamable HTTP feature not resolved; check when enabled. |
| tokio | 1.53.3 (2026-10-13) | MIT | https://github.com/tokio-rs/tokio | OK. Exactly 14 days old; 1.54.0 is yanked; 1.54.1 (2026-10-21) is younger than 14 days. |
| clap | 4.6.7 (2026-09-14) | MIT OR Apache-2.0 | https://github.com/clap-rs/clap | OK |
| serde | 1.0.229 (2026-07-18) | MIT OR Apache-2.0 | https://github.com/serde-rs/serde | OK |
| schemars | 1.2.3 (2026-10-12) | MIT | https://github.com/GREsau/schemars | OK |
| symphonia | 0.6.1 (2026-08-13) | MPL-2.0 | https://github.com/pdeljanov/Symphonia | OK via MPL-2.0 exceptions + NOTICE. Default features (`adpcm`, `flac`, `mkv`, `ogg`, `pcm`, `vorbis`, `wav`, `all-meta`, `opt-simd`) pull 11 MPL-2.0 crates: `symphonia`, `-core`, `-metadata`, `-common`, `-bundle-flac`, `-codec-adpcm`, `-codec-pcm`, `-codec-vorbis`, `-format-mkv`, `-format-ogg`, `-format-riff`. |
| hound | 3.5.1 (2023-09-25) | Apache-2.0 | https://github.com/ruuda/hound | OK on license. Maintenance risk: no release since 2023. |
| flacenc | 0.5.1 (2025-12-18) | Apache-2.0 | https://github.com/yotarok/flacenc-rs | OK |
| rubato | 5.0.1 (2026-10-01) | MIT OR Apache-2.0 | https://github.com/HEnquist/rubato | OK |
| ebur128 | 0.1.10 (2024-10-26) | MIT | https://github.com/sdroege/ebur128 | OK. Pure Rust; 0.1.11 younger than 14 days. |
| rustfft | 6.4.1 (2025-09-18) | MIT OR Apache-2.0 | https://github.com/ejmahler/RustFFT | OK |
| rusqlite | 0.40.2 (2026-08-08) | MIT | https://github.com/rusqlite/rusqlite | OK. `libsqlite3-sys` is MIT; with feature `bundled` it compiles SQLite (public domain) from source. |
| argmin | 0.11.0 (2025-09-28) | MIT OR Apache-2.0 | https://github.com/argmin-rs/argmin | **Not approved now.** Pulls `paste` (build-time proc-macro) with advisory RUSTSEC-2024-0436 (unmaintained), which fails `unmaintained = "all"`. Deferred to M6, see below. |
| assert_no_alloc | 1.1.2 (2021-08-03) | BSD-1-Clause | https://github.com/Windfisch/rust-assert-no-alloc | Approved as dev-dependency only, crate-scoped BSD-1-Clause exception. Last release 2021: maintenance risk. |
| proptest (dev) | 1.12.0 (2026-10-10) | MIT OR Apache-2.0 | https://github.com/proptest-rs/proptest | OK |
| criterion (dev) | 0.8.2 (2026-02-04) | Apache-2.0 OR MIT | https://github.com/criterion-rs/criterion.rs | OK |

Transitive graph of the scratch resolve: 246 crates. Notable items:

- `unicode-ident` requires `Unicode-3.0` (on the allowlist).
- `r-efi` is `MIT OR Apache-2.0 OR LGPL-2.1-or-later`, accepted via MIT.
- No openssl, ring or aws-lc in the resolve.
- Duplicate versions (warn only): `bitflags`, `getrandom`, `hashbrown`, `r-efi`, `rand_core`, `syn`.
- No license conflicts other than MPL-2.0 (symphonia, handled by exceptions) and BSD-1-Clause
  (`assert_no_alloc`, handled by a dev-only exception).

Decisions on specific crates:

- **symphonia**: MPL-2.0 confirmed on crates.io for 0.6.1; the 11 crates above are excepted by
  name and each gets a NOTICE entry (M0-02 / the task that first adds symphonia). The list must be
  re-confirmed against `cargo deny list` on the real `Cargo.lock`; any extra or missing symphonia
  crate is corrected through an amendment of this ADR, not by a wildcard.
- **assert_no_alloc**: BSD-1-Clause is a permissive license compatible with Apache-2.0, but it is not
  on the allowlist; the exception is scoped to this crate and to dev use, because the crate is never
  linked into the shipped binary (enforced by the CI guard in section 3).
- **argmin**: not approved. PLAN section 4 allows "argmin или собственный поиск в границах"; the
  choice (argmin plus an advisory ignore entry for RUSTSEC-2024-0436 with reason and ADR reference,
  or an in-house bounded search) is made in the M6 engine optimization task with its own ADR.
- **rusqlite**: recommended with feature `bundled`, so the same SQLite version is used on all three
  operating systems (needs a C compiler in CI). Final decision in the M3 storage task ADR.

### 5. Python reference dependencies

- numpy, scipy and pyloudnorm (plus pytest and ruff as tools) are allowed for `reference/` and eval
  scripts only; they are never shipped and never a runtime requirement (ADR-0002 decision 4).
- The same license policy applies: allowlist above, no GPL/LGPL/AGPL; anything else needs an ADR.
  New Python dependencies need a dependency-auditor review and an ADR, like Rust crates.
- Licenses on PyPI (`https://pypi.org/pypi/<name>/json`, accessed 2026-10-27): pyloudnorm 0.2.0
  `MIT`; scipy 1.18.1 BSD-3-Clause-style text in the `license` field (no SPDX expression); numpy 2.5.4
  `BSD-3-Clause AND 0BSD AND MIT AND Zlib AND CC0-1.0`. `0BSD` (a bundled component of numpy) is not
  on the allowlist; numpy is approved by name for `reference/` with this expression (0BSD is a
  public-domain-equivalent permissive license).
- Python versions are pinned in the reference task that sets up `reference/` (not here).

## Alternatives

- **Dynamic linking of LGPL libraries** (for example libav*, libsndfile): LGPL would be satisfiable
  with dynamic linking, but it breaks the single static executable with no runtime requirements
  (PLAN section 2, ADR-0002) and complicates distribution on three OSes. Rejected.
- **Allow any OSI-approved license**: simpler config, but admits GPL/LGPL/AGPL and other copyleft,
  which conflicts with static linking under Apache-2.0 (PLAN section 2 principle 10). Rejected.
- **ffmpeg for all decoding and encoding**: one tool for every format, but it makes ffmpeg a hard
  runtime requirement, ties the license to a specific ffmpeg build, and moves core I/O out of the
  tested Rust code. Rejected; ffmpeg stays an optional separate process for formats the Rust stack
  does not cover (PLAN section 4, rows "Запись аудио" and "Кодеки").
- **Allow MPL-2.0 globally on the allowlist**: less config, but any new MPL-2.0 crate would enter
  silently without a NOTICE entry. Rejected in favor of per-crate exceptions.
- **Blanket `multiple-versions = "deny"`**: cleaner graph, but the survey already shows six
  duplicates from upstream crates that the project cannot fix in M0. Rejected for now; revisited
  after M0.

## Consequences

- Positive: license compliance and advisories are checked mechanically on every pull request; every
  non-allowlisted license is visible as a named exception tied to an ADR; NOTICE stays complete for
  MPL-2.0 code; Cargo.lock plus a per-release SBOM make every shipped graph reproducible and
  auditable.
- Negative: each new crate (and many feature flags) costs an audit and an ADR; per-crate symphonia
  exceptions must be maintained when symphonia restructures its crates; `unmaintained = "all"` will
  block some otherwise useful crates (argmin today) until an ignore entry with an ADR is accepted.
- Negative: two crates with maintenance risk (hound, assert_no_alloc). hound is a runtime
  dependency; its status is re-checked in the task that first adds it.
- Follow-up: M0-06 writes `deny.toml` from section 3 (including the pre-approved exceptions), adds
  versioned internal path dependencies (and `publish = false` only outside `aitm`'s dependency
  graph), commits `Cargo.lock`, and adds cargo-deny, cargo-audit and the dev-only guard to CI. M0-02 / the task that adds symphonia writes the NOTICE
  entries. M3 storage task decides rusqlite `bundled`. M6 optimization task decides argmin vs
  in-house search. M10 release task chooses the SBOM tool and format. After M0, revisit
  `multiple-versions` and a skip list.
- Rules out: GPL/LGPL/AGPL and other copyleft in the binary; linking ffmpeg; git or non-crates.io
  sources without an ADR; advisory ignores without id, reason and ADR; wildcard version
  requirements on registry crates.

## Open questions

- **assert_no_alloc maintenance**: last release 2021. If it stays unmaintained (or gets an
  advisory), replace it with an in-house allocation trap (a test-only global allocator)? Default: keep
  it as dev-only for now and revisit when the first `process` allocation test is written (M2).
- **argmin**: argmin with an advisory ignore for RUSTSEC-2024-0436, or in-house bounded search?
  Deferred to the M6 optimization task ADR.
- **rusqlite `bundled`**: recommended; confirm in the M3 storage task ADR (adds a C compiler
  requirement to CI on all three OSes).
- **Target filter**: default is to check all targets (no `[graph].targets`), which keeps crates such as
  `r-efi` (UEFI) in the license check. Restrict to the release triples (Windows, macOS arm64/x86_64,
  Linux x86_64/arm64) instead? Default kept unless the maintainer decides otherwise.
- **0BSD**: numpy's expression includes `0BSD`. Add `0BSD` to the general allowlist (Rust and
  Python), or keep the named numpy approval for `reference/` only? Default: named approval only.
- **hound**: no release since 2023. Accept for WAV writing in the task that first adds it, or
  evaluate an alternative there?
- The survey brief described pyloudnorm as BSD; PyPI lists it as `MIT` (2026-10-27). Both are
  allowed; recorded here as MIT.

## Sources (accessed 2026-10-27)

- https://crates.io/api/v1/crates/<name> for every crate in the table in section 4 (license,
  versions, release dates, repository)
- https://crates.io/api/v1/crates/symphonia/0.6.1 and
  https://crates.io/api/v1/crates/symphonia/0.6.1/dependencies (default features, sub-crates)
- https://crates.io/api/v1/crates/symphonia-bundle-flac/0.6.1/dependencies (`symphonia-common`)
- https://rustsec.org/advisories/RUSTSEC-2024-0436 (`paste` unmaintained)
- https://pypi.org/pypi/pyloudnorm/json, https://pypi.org/pypi/numpy/json,
  https://pypi.org/pypi/scipy/json
- Tools used by dependency-auditor: cargo-deny 0.20.2, cargo-audit 0.22.1 (scratch resolve outside
  the repository)
