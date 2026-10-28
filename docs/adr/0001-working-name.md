# ADR-0001: Working name, package names and binary name

- Status: accepted
- Date: 2026-10-27
- Amended: 2026-10-27 — GitHub repository recorded (sotber/AI_Track_Master)
- Deciders: maintainer

## Context

`docs/PLAN.md` section 13 ("Допущения по умолчанию") states that the working name and the
package name are chosen in M0. The name is needed before anything else is created, because the
Cargo workspace and crate names (task M0-06), the binary name, the `NOTICE` text (task M0-02)
and the distribution channels all embed it, and renaming crates after first publication is
costly.

`docs/PLAN.md` section 4 (row "Дистрибуция") fixes the distribution channels: cargo-dist builds
binaries and installers; the primary channels are `cargo install` and Homebrew; binaries go to
GitHub Releases. The chosen names must therefore be free on crates.io and as a Homebrew formula
name, and must not collide with an existing product in the same domain.

The product is one Rust binary that contains the core, the mastering engine, the MCP server and
the CLI (AGENTS.md, "Product in one paragraph"). Users and MCP client configs type the binary
name often, so it should be short.

## Decision

The maintainer chose the following names.

| Item | Name |
|---|---|
| Project name (docs, README, NOTICE, reports) | AI Track Master |
| Short name | AITM |
| Cargo package prefix | `aitm-` |
| Library crates (one per layer directory under `crates/`) | `aitm-core`, `aitm-dsp`, `aitm-analysis`, `aitm-engine`, `aitm-plugin-sdk`, `aitm-server` |
| Built-in plugin crates (`crates/plugins/<name>/`) | `aitm-plugin-<name>` |
| Eval crate (`eval/` harness code) | `aitm-eval` |
| CLI crate (`crates/cli/`) | published directly as package `aitm` (see "Packaging of the `aitm` crate" below) |
| Published crate for `cargo install` | `aitm` |
| Binary name | `aitm` |
| Homebrew formula name | `aitm` |
| GitHub repository | `sotber/AI_Track_Master` (see "GitHub repository" below) |

Directory names under `crates/` stay as in AGENTS.md (`crates/core`, `crates/cli`, ...); only the
Cargo `package.name` carries the prefix. Directory names are snake_case and package names are
kebab-case: for example `crates/plugins/multiband_comp` has `package.name = "aitm-plugin-multiband-comp"`.
In Rust code the crates are referenced with underscores
(`aitm_core`, `aitm_dsp`, ...), as Cargo normalizes `-` to `_`.

### Packaging of the `aitm` crate

The user-facing install command is `cargo install aitm`, producing the binary `aitm`. The
maintainer's default is that `crates/cli` is published directly as package `aitm` (binary `aitm`),
with no separate `aitm-cli` wrapper crate. Final wiring of the manifest (`package.name`, `[[bin]]`)
is done in task M0-06.

### GitHub repository

Resolved by the maintainer (2026-10-27): the repository is https://github.com/sotber/AI_Track_Master,
owned by the user account `sotber`, private for now. Its name `AI_Track_Master` differs from the
planned `ai-track-master`; the existing name is kept. GitHub resolves owner and repository names
case-insensitively, so `github.com/sotber/ai_track_master` reaches the same repository; the
underscores are not interchangeable with hyphens, so `github.com/sotber/ai-track-master` does not.
Update 2026-10-28: the account is now spelled `SoTBeR` (https://github.com/SoTBeR/AI_Track_Master)
and the repository is public.
Renaming to `ai-track-master` is optional and, if done, should happen before the repository is made
public (GitHub redirects renamed repositories, but published links, badges and cargo-dist/Homebrew
metadata are easier to get right once). Crate, binary and formula names above are unaffected.

### Availability check (2026-10-27)

Method: HTTP GET of the registry APIs; HTTP 404 means the name is not registered. The method was
validated the same day against known entries (`https://crates.io/api/v1/crates/serde` returned the
crate; `https://formulae.brew.sh/api/formula/ripgrep.json` returned the formula).

| Name | crates.io (`/api/v1/crates/<name>`) | Homebrew formula (`/api/formula/<name>.json`) | Homebrew cask (`/api/cask/<name>.json`) |
|---|---|---|---|
| `aitm` | 404, free | 404, free | 404, free |
| `ai-track-master` | 404, free | 404, free | 404, free |
| `aitm-core` | 404, free | n/a | n/a |
| `aitm-dsp` | 404, free | n/a | n/a |
| `aitm-engine` | 404, free | n/a | n/a |
| `aitm-server` | 404, free | n/a | n/a |
| `aitm-cli` | 404, free | n/a | n/a |

(`aitm-cli` was checked while it was still a candidate package name; under the default above it is
not used.)

A crates.io search for `aitm` (`https://crates.io/api/v1/crates?q=aitm`) returned only unrelated
crates whose names contain the substring (`waitmap`, `futures-waitmap`); no crate uses an `aitm`
prefix. `aitm-analysis`, `aitm-plugin-sdk`, `aitm-eval` and `aitm-plugin-*` were not checked individually;
given no `aitm*` crate exists, they are presumed free and must be re-checked before first publish.

Product-name clash search (web search and GitHub repository search, 2026-10-27):

- No software product named "AI Track Master" was found. Nearby names in the same domain exist
  (for example "AI Master" by Exonic UK, a mastering plugin; aimastering.com / phaselimiter;
  LANDR; eMastered; Kits.ai "AI Mastering"). They are different names; no trademark search was
  performed (deferred to release 1.0, see Consequences).
- GitHub has no repository named exactly `ai-track-master`; similar names exist
  (`Matticusnicholas/Ai-Track-Masterer`, `eriklau/TrackMaster-AI`, `CHAIMAABDELAZIZ/AI-Master-track`
  and others), none of them a published mastering product. GitHub repository names are unique only
  per owner; the repository now exists as `sotber/AI_Track_Master` (see "GitHub repository"), so
  there is no clash under that owner.
- "AITM" / "AiTM" is widely used in information security for "adversary-in-the-middle" phishing
  (Microsoft Security blog, CISA/CIS advisories, many vendor glossaries), and in machine learning
  for the "AITM" multi-task learning framework (KDD 2021; GitHub `xidongbo/AITM`). These are not
  software products in the audio domain and do not block the name; see Consequences.

These results are a snapshot. crates.io and Homebrew have no reservation mechanism, so the names
remain free only until someone publishes them.

## Alternatives

- **Binary name `ai-track-master`** (matching the originally planned GitHub repository name): unambiguous and free on all
  checked channels, but long to type in shells and in MCP client configuration. Rejected in favor
  of `aitm`; it remains a fallback if `aitm` is taken before first publication.
- **Published crate and Homebrew formula `ai-track-master`** with binary `aitm`: avoids the
  acronym's search noise on crates.io and Homebrew, but makes the install name differ from the
  command name, which confuses users. Rejected; same fallback role as above.
- **Unprefixed crate names** (`core`, `dsp`, `engine`, ...): not publishable on crates.io (taken or
  too generic) and ambiguous in dependency trees. Rejected.

## Consequences

- Positive: one consistent name across the workspace, binary, NOTICE and distribution channels from
  the start; short command `aitm`; all checked names were free on 2026-10-27.
- Negative: "AITM" collides in web search with the security term "adversary-in-the-middle" and the
  ML framework of the same acronym. Expect search noise; README, crate descriptions and the
  Homebrew formula description should always say "AI Track Master" and "audio mastering" so search
  engines and users can disambiguate. This is a known cost, not a blocker.
- Negative: names are not reserved. Maintainer default: no placeholder publication or other name
  reservation on crates.io before M10. Availability must be re-checked (same method) immediately
  before the first publication to crates.io and Homebrew (M10). If a name is taken by then, a new
  ADR supersedes this one, using the `ai-track-master` fallback above.
- Trademark: maintainer default is to defer a trademark search for "AI Track Master" / "AITM"
  until release 1.0.
- Homebrew: the formula name `aitm` applies to the project's own tap generated by cargo-dist and to
  any later submission to homebrew-core; homebrew-core acceptance rules are out of scope here (M10).
- Follow-up: task M0-02 uses "AI Track Master" in `NOTICE`; task M0-06 applies the crate names and
  the binary name in the workspace and wires `crates/cli` as package `aitm`.
- Follow-up for task M0-20: the eval crate's package name is `aitm-eval`, so the AGENTS.md command
  `cargo run -p eval -- golden --compare main` becomes `cargo run -p aitm-eval -- golden --compare main`.
  AGENTS.md is not edited by this ADR.
- ADR numbering: ADR numbers follow creation order in `docs/adr/`, not the labels used in
  `docs/PLAN.md`. This ADR is 0001; the plan's "ADR-001" (Rust, task M0-03) = ADR-0002 and the
  plan's "ADR-002" (dependencies and licenses, task M0-04) = ADR-0003.

## Open questions

- None open. Resolved 2026-10-27: GitHub owner is the user account `sotber`; repository
  `sotber/AI_Track_Master` (private for now), existing name kept; renaming to `ai-track-master`
  before going public is optional (see "GitHub repository").

## Sources (accessed 2026-10-27)

- https://crates.io/api/v1/crates/aitm, https://crates.io/api/v1/crates/ai-track-master,
  https://crates.io/api/v1/crates/aitm-core (and `aitm-dsp`, `aitm-engine`, `aitm-server`, `aitm-cli`)
- https://crates.io/api/v1/crates?q=aitm
- https://formulae.brew.sh/api/formula/aitm.json, https://formulae.brew.sh/api/cask/aitm.json
- https://formulae.brew.sh/api/formula/ai-track-master.json, https://formulae.brew.sh/api/cask/ai-track-master.json
- https://api.github.com/search/repositories?q=ai-track-master+in:name
- https://api.github.com/search/repositories?q=aitm+in:name
- https://www.cisecurity.org/insights/blog/ctas-using-adversary-in-the-middle-aitm-phishing-attacks
- https://gearspace.com/threads/ai-master-fully-automatic-mastering-plugin.1323336/
