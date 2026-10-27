# Spec: Target profile format (platform delivery targets and material profiles)

- Status: draft
- Layer: docs (format); loader implemented later in engine (M6)
- Task: docs/tasks/M0/11-target-profile-format.md
- Plan: docs/PLAN.md section 3 (stage 4 "Выбор целей", "Профили целей"), section 6 ("Ресурсы и промпты"),
  section 8 (last paragraph), section 9 (M0, M6), section 13 (last item)

## What

The versioned on-disk data format for the two kinds of target profile stored under `profiles/`:

- **Platform profiles**: delivery targets of one platform or medium: integrated loudness target, true-peak
  ceiling, delivery formats. Every platform value names its source (documentation URL) and the date that
  source was accessed.
- **Material profiles**: tonal and dynamic targets for one material type: aggregated statistics of smoothed
  spectra (curves only, never audio), dynamic ranges, and the bounds within which agent intent and a
  reference track may shift these targets.

This spec also fixes the Rust "Public API" of the loader that the engine (M6) and the MCP server (M4
resource "targets per platform") use. The loader is implemented in M6; this spec is the contract.

Everything not required by `docs/PLAN.md` is a proposal listed in "Proposed defaults" (PD-n) and needs
maintainer confirmation. The rules below reference those items.

## Why

- PLAN section 3 "Профили целей": platform targets and material profiles are versioned data in `profiles/`,
  not code; tonal targets are statistics of smoothed spectra of a reference corpus per material type; only
  aggregated curves are committed, never audio.
- PLAN section 3 stage 4: delivery targets come from the platform profile; tonal and dynamic targets adapt to
  the material; intent and reference shift targets **within defined bounds**. The bounds must therefore be data.
- PLAN section 6 "Ресурсы и промпты": "targets per platform" is an MCP resource, stored as updatable data.
- PLAN section 8 (last paragraph) and section 13 (last item): platforms change their values, so values are
  checked against current platform documentation when they are added or updated.

## Files and layout

Syntax: **TOML 1.0** (https://toml.io/en/v1.0.0), UTF-8, LF line endings only (a CR byte anywhere is an error,
`LineEnding`), `schema_version` as the first key. The repository must pin LF for profile files
(`.gitattributes`: `profiles/** text eol=lf`, added by the platform task that creates `profiles/`, and a CI
check), so Windows checkouts with `core.autocrlf` do not break loading. This is a proposal **pending ADR in
M6** (syntax ADR deferred to M6, before the loader task; see the task file Notes and open question 1). Wherever
this spec says "TOML" (syntax errors, `inf`/`nan` literals, line/column), that assumption is pending the same ADR.

```
profiles/
  platforms/<platform-id>.toml     one platform profile per file
  materials/<material-id>.toml     one material profile per file
```

- File stem must equal the profile's `id` field.
- Ids follow the id rule (PD-1).
- Only `*.toml` files directly inside `platforms/` and `materials/` are profiles. Any other entry in those two
  directories (subdirectory, other extension, symlink) is an error, so stray files are never silently ignored.
  Other files under `profiles/` (for example a `README.md`) are ignored.
- Size limits per file and per set: PD-2.
- Profiles contain only numbers, short strings, dates and URLs. No audio, no base64 blobs, no paths to audio.

## Versioning

Two independent version numbers:

1. **`schema_version`** (integer, required, first key of every file): version of this format. Current value: `1`.
   - The loader accepts exactly the versions it has a migration for; v1 supports only `1`.
     Any other value is `ProfileError::UnsupportedSchemaVersion`.
   - Any change to the format (new field, removed field, changed unit or meaning) bumps `schema_version`.
     Unknown fields are rejected (see Validation), so there are no "silent additive" changes.
   - A bump requires: an ADR; an in-code migration from every still-supported older version to the new one;
     migration of all files in `profiles/` in the same task; a CHANGELOG entry. Because the MCP resource exposes
     this format, a bump is also a public MCP schema change (AGENTS.md prohibition: versioned migration + ADR).
2. **`revision`** (integer in [1, 2^32 − 1], required): data revision of one profile. Incremented by exactly 1
   whenever any value in that file changes. Two files with the same `id` and `revision` but different content are
   a process error (checked by review / CI, not by the loader).

Consumers that produce output from a profile (engine report, MCP resource) record `id`, `revision` and
`schema_version` of every profile used, so a result is traceable to the exact target data (determinism:
same input + same intent + same profile revisions = same output).

## Provenance

### Sources table (platform profiles)

Each platform profile has a `[[sources]]` array. Each entry:

| Field | Type | Rule |
|---|---|---|
| `id` | string | Unique within the file; id rule (PD-1). |
| `kind` | string enum | `"official_documentation"`, or `"standard"` (published standard, for example a disc format standard; the `standard` kind is a proposal, PD-14). |
| `url` | string | Absolute `https://` URL with a non-empty host. |
| `accessed` | string | Access date, ISO 8601 calendar date `YYYY-MM-DD` (RFC 3339 `full-date`), a valid calendar date. |
| `title` | string | Non-empty, length limit PD-10: page or document title as shown by the source. |
| `excerpt` | string, optional | Length limit PD-10: the exact sentence(s) the value was taken from. |

### Sourced values

Every platform value is a table `{ value = <number or string>, source = "<source id>" }`.
A sourced value whose `source` does not name an entry in `[[sources]]` is an error. A platform value without
a `source` key is an error. There is no way to write a platform value without provenance. Every source must be
referenced by at least one value (PD-9).

### Engineering bounds (not platform values)

Values that are project decisions rather than platform facts (the loudness shift bounds of a platform profile;
all shift bounds of a material profile) carry a `rationale` string (non-empty, length limit PD-10) instead of a
source, and optionally `adr`. They must never be presented as platform values.

| Field | Type | Rule |
|---|---|---|
| `rationale` | string | Required in every engineering-bound table. Non-empty, length limit PD-10. |
| `adr` | string, optional | Format `ADR-NNNN` (regex `^ADR-[0-9]{4}$`), the number of an ADR in `docs/adr/`. The loader validates the format only; it does **not** check that the ADR file exists (profiles may be loaded where the repository is absent). Existence is checked by review / CI (open question 4). |

### Update rule (process requirement, applies to every source kind)

- A platform value may be added or changed only after reading the current version of its source (official
  platform documentation, or the published standard for `kind = "standard"`) in the same task. The `accessed`
  date must be the date of that reading; the `url` must be the page or document actually read.
- Values from model memory, forums or third-party articles are not allowed for any source kind.
- Changing a value without re-reading the source, or bumping `accessed` without re-reading, is not allowed.
- The loader does **not** read the system clock and never warns about "stale" dates (that would make loading and
  reports time-dependent and non-deterministic). Staleness checks are a separate tool/CI concern (open question 4).

## Platform profile fields

| Key | Type | Unit | Rule |
|---|---|---|---|
| `schema_version` | integer | | `1` |
| `kind` | string | | `"platform"` |
| `id` | string | | Profile id = file stem. |
| `revision` | integer | | [1, 2^32 − 1] |
| `name` | string | | Display name, non-empty, length limit PD-10. |
| `description` | string, optional | | Length limit PD-10. |
| `[[sources]]` | array of tables | | >= 1 entry, see Provenance. |
| `loudness.integrated_lufs` | sourced number | LUFS (ITU-R BS.1770 integrated loudness) | Finite, range PD-13. Target integrated loudness the engine aims at for this platform. |
| `loudness_shift.min_lu`, `loudness_shift.max_lu` | number | LU | Engineering bound: how far intent may move the integrated target. `min_lu <= 0 <= max_lu`, finite, range PD-13. |
| `loudness_shift.rationale`, `loudness_shift.adr` | string | | See "Engineering bounds". |
| `true_peak.ceiling_dbtp` | sourced number | dBTP (ITU-R BS.1770 true peak) | Finite, range PD-13. Hard ceiling for every delivered file, including after lossy encode/decode. |
| `[[delivery_formats]]` | array of tables | | >= 1 entry, no duplicates (see below). |

Each `[[delivery_formats]]` entry:

| Key | Type | Unit | Rule |
|---|---|---|---|
| `codec` | sourced string enum | | One of `"wav"`, `"flac"`, `"aiff"`, `"mp3"`, `"aac"`, `"ogg_vorbis"` (PLAN section 13 formats). |
| `sample_rate_hz` | sourced integer | Hz | Allowed set PD-3. |
| `sample_format` | sourced string enum, PCM codecs only | | `wav`/`aiff`: `"int16"`, `"int24"`, `"float32"`; `flac`: `"int16"`, `"int24"`. Required for PCM codecs, forbidden for lossy codecs. |
| `bitrate_kbps` | sourced integer, lossy codecs only | kbit/s | Range PD-4. Required for `mp3`, `aac`, `ogg_vorbis`; forbidden for PCM codecs. |
| `channels` | sourced integer | | `2` (PD-11). |

- **Duplicate delivery formats:** two entries are duplicates iff their `codec`, `sample_rate_hz`,
  `sample_format` and `bitrate_kbps` and `channels` **values** are equal; `source` ids are ignored in the comparison.
- A codec is lossy iff it is `mp3`, `aac` or `ogg_vorbis` (derived, not stored). Lossy formats are encoded,
  decoded and re-checked by the engine (PLAN section 3 stage 8); the profile cannot turn that off.
- **A profile does not have to list a lossy format.** `delivery_formats` lists only what the platform documents
  (sourced). Profiles must not add lossy entries without a source to satisfy quality checks. For golden-set runs,
  the eval harness adds one lossy check format on top when the profile lists none; that format is defined by
  docs/spec/release-targets.md (Terms, "All formats", and its open question Q9), not by the profile.

**What a platform profile cannot express (by design):** shifting or disabling the true-peak ceiling; disabling
the post-codec re-check; disabling or weakening the "do no harm" check; any engine switch. Unknown keys are
rejected, so such keys cannot be added without a schema bump, an ADR and review.

### Platform example (all values are fake placeholders)

```toml
# EXAMPLE ONLY. Every number, URL and date below is a placeholder, NOT a real platform value.
schema_version = 1
kind = "platform"
id = "example-platform"
revision = 1
name = "Example Platform (placeholder)"

[[sources]]
id = "loudness-page"
kind = "official_documentation"
url = "https://example.com/placeholder/loudness"
accessed = "2000-01-01"
title = "Placeholder loudness page"
excerpt = "Placeholder sentence."

[loudness]
integrated_lufs = { value = -19.25, source = "loudness-page" }   # placeholder

[loudness_shift]
min_lu = -2.5     # placeholder engineering bound
max_lu = 1.5      # placeholder engineering bound
rationale = "Placeholder rationale."

[true_peak]
ceiling_dbtp = { value = -2.75, source = "loudness-page" }        # placeholder

[[delivery_formats]]
codec = { value = "flac", source = "loudness-page" }
sample_rate_hz = { value = 48000, source = "loudness-page" }
sample_format = { value = "int24", source = "loudness-page" }
channels = { value = 2, source = "loudness-page" }
```

## Material profile fields

| Key | Type | Unit | Rule |
|---|---|---|---|
| `schema_version` | integer | | `1` |
| `kind` | string | | `"material"` |
| `id` | string | | Profile id = file stem. |
| `revision` | integer | | [1, 2^32 − 1] |
| `name` | string | | Non-empty, length limit PD-10. |
| `description` | string, optional | | Length limit PD-10. |
| `tags` | array of strings | | Genre / material-type labels (PD-12): count limit PD-12, each matching the id rule, unique. |
| `origin.kind` | string enum | | `"corpus"` (computed from a reference corpus) or `"provisional"` (hand-authored before a corpus exists). |
| `origin.corpus_manifest` | string | | Required iff `corpus`, forbidden otherwise: repository-relative `/`-separated path of the corpus manifest under `eval/` (no `..` segment, not absolute). |
| `origin.track_count` | integer | | Required iff `corpus`, forbidden otherwise: [1, 2^32 − 1]. |
| `origin.computed` | string | | Required iff `corpus`, forbidden otherwise: date `YYYY-MM-DD`. |
| `origin.tool` | string | | Required iff `corpus`, forbidden otherwise: name and version of the tool, length limit PD-10. |
| `origin.rationale` | string | | Required iff `provisional`, forbidden otherwise: non-empty, length limit PD-10. |

### Tonal target (`[tonal]`)

Aggregated statistics of smoothed, perceptually weighted spectra (PLAN section 3 stage 2 and "Профили целей").
The choice of statistics (10th percentile, median, 90th percentile) is a proposal (PD-15).

| Key | Type | Unit | Rule |
|---|---|---|---|
| `tonal.analysis_method` | string | | Id and version of the spectrum analysis that produced and must consume the curve, for example `"smoothed-spectrum/v1"` (defined by the analysis spec, M1/M5). Non-empty, length limit PD-10, characters `[a-z0-9-./]`. The engine must refuse to compare a curve against a spectrum computed with a different method (error, not silent). |
| `tonal.freq_hz` | array of numbers | Hz | Point count PD-5; finite; strictly increasing; each in the frequency range PD-13. |
| `tonal.p10_db` | array of numbers | dB (relative) | Same length as `freq_hz`, finite, range PD-13. 10th percentile across the corpus. |
| `tonal.median_db` | array of numbers | dB (relative) | Same length, same range. Target curve. |
| `tonal.p90_db` | array of numbers | dB (relative) | Same length, same range. 90th percentile. |

Per point `p10_db[i] <= median_db[i] <= p90_db[i]`. Curves are relative levels; the normalization reference
(for example level-matched to 0 dB average) is part of `analysis_method`, not of this format. The point-count cap
and the numeric-arrays-only rule make it impossible to store audio in a profile.

### Dynamic target (`[dynamics]`)

| Key | Type | Unit | Rule |
|---|---|---|---|
| `dynamics.plr_db.min`, `.max` | number | dB (peak-to-loudness ratio, true peak dBTP minus integrated LUFS) | Finite, range PD-13, `min <= max`. |
| `dynamics.lra_lu.min`, `.max` | number | LU (EBU Tech 3342 loudness range) | Finite, range PD-13, `min <= max`. |

### Shift bounds (`[shift_bounds.intent]`, `[shift_bounds.reference]`, `[shift_bounds.combined]`)

Engineering bounds for how far agent intent, a reference track, and both together may move the tonal and
dynamic targets of this material profile (PLAN section 3 stage 4; section 3 "Роль агента": "warmer", "louder",
"gentler on dynamics" are bounded shifts). The shape (tilt + bands + PLR offset) and the `combined` table are
proposals (PD-7). All three tables are required and have the same shape:

| Key | Type | Unit | Rule |
|---|---|---|---|
| `tilt_db_per_octave.min`, `.max` | number | dB/octave | Tilt around `tilt_pivot_hz`. Finite, range PD-13. |
| `tilt_pivot_hz` | number | Hz | Finite, range PD-13. Same value in all three tables (PD-8). |
| `[[...bands]]` | array of tables | | Per-band level offset bounds; count PD-6. |
| `bands[].name` | string | | Id rule, unique within the table. |
| `bands[].low_hz`, `bands[].high_hz` | number | Hz | Finite, `low_hz < high_hz`, both in the frequency range PD-13. |
| `bands[].min_db`, `bands[].max_db` | number | dB | Finite, range PD-13. |
| `plr_offset_db.min`, `.max` | number | dB | Finite, range PD-13. |
| `rationale`, `adr` | string | | See "Engineering bounds". |

- Every `min`/`max` pair satisfies `min <= 0 <= max` (no shift is always allowed).
- **Band matching (PD-8):** bands are matched across the three tables by `name`. A band with the same name in two
  tables must have identical `low_hz` and `high_hz`. Every band present in `intent` or `reference` must also be
  present in `combined`. A band absent from a table allows zero shift from that source in that band.
- Semantics for the engine (M6): intent shifts are clamped to `intent`, reference shifts to `reference`, their sum
  to `combined`. Loudness shifts come from the platform profile's `loudness_shift`, never from a material profile.
  The true-peak ceiling is never shiftable. Shifted targets still pass the full optimization loop and the
  "do no harm" check.

### Material example (all values are fake placeholders)

```toml
# EXAMPLE ONLY. All numbers are placeholders, not measured statistics.
schema_version = 1
kind = "material"
id = "example-material"
revision = 1
name = "Example material (placeholder)"
tags = ["example"]

[origin]
kind = "provisional"
rationale = "Placeholder until a reference corpus exists."

[tonal]
analysis_method = "smoothed-spectrum/v1"
freq_hz   = [31.5, 63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0]
p10_db    = [-9.0, -8.0, -7.0, -6.0, -5.0, -5.0, -6.0, -7.0, -8.0, -9.0]
median_db = [-6.0, -5.0, -4.0, -3.0, -2.0, -2.0, -3.0, -4.0, -5.0, -6.0]
p90_db    = [-3.0, -2.0, -1.0,  0.0,  1.0,  1.0,  0.0, -1.0, -2.0, -3.0]

[dynamics]
plr_db = { min = 5.5, max = 11.5 }
lra_lu = { min = 2.5, max = 9.5 }

[shift_bounds.intent]
tilt_db_per_octave = { min = -0.25, max = 0.25 }
tilt_pivot_hz = 1000.0
plr_offset_db = { min = -1.5, max = 1.5 }
rationale = "Placeholder."
[[shift_bounds.intent.bands]]
name = "low"
low_hz = 20.0
high_hz = 150.0
min_db = -1.5
max_db = 1.5

[shift_bounds.reference]
tilt_db_per_octave = { min = -0.5, max = 0.5 }
tilt_pivot_hz = 1000.0
plr_offset_db = { min = -2.0, max = 2.0 }
rationale = "Placeholder."

[shift_bounds.combined]
tilt_db_per_octave = { min = -0.5, max = 0.5 }
tilt_pivot_hz = 1000.0
plr_offset_db = { min = -2.0, max = 2.0 }
rationale = "Placeholder."
[[shift_bounds.combined.bands]]
name = "low"
low_hz = 20.0
high_hz = 150.0
min_db = -2.0
max_db = 2.0
```

## Proposed defaults

Design choices that `docs/PLAN.md` does not fix. Each is an architect proposal for maintainer confirmation;
changing one before acceptance of this spec needs no ADR, afterwards it is a format change (schema bump).

| Id | Choice | Proposed value |
|---|---|---|
| PD-1 | Id rule (profile ids, source ids, band names, tags) | `^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$` (lowercase ASCII, digits, inner hyphens, 1..=64 chars) |
| PD-2 | Size limits | file <= 256 KiB (262144 bytes); <= 256 profiles per kind |
| PD-3 | Allowed sample rates | 44100, 48000, 88200, 96000, 176400, 192000 Hz (PLAN section 13: up to 192 kHz) |
| PD-4 | Lossy bitrate range | [32, 512] kbit/s |
| PD-5 | Tonal curve point count | 8..=256 points |
| PD-6 | Bands per shift-bounds table | 0..=16 |
| PD-7 | Shift shape | tilt (dB/oct around a pivot) + named bands (dB) + PLR offset (dB); three tables `intent`, `reference`, `combined` |
| PD-8 | Cross-table rules | same `tilt_pivot_hz` in all three tables; band matching by name as specified above |
| PD-9 | Unused sources | error `UnusedSource` |
| PD-10 | String length limits (Unicode scalar values) | `name` <= 100; `description` <= 1000; `title` <= 200; `excerpt` <= 500; `rationale` <= 500; `origin.tool` <= 100; `analysis_method` <= 64; URL <= 2048; `corpus_manifest` <= 256 |
| PD-11 | Channels | only `2` in v1 |
| PD-12 | Tags | field `tags`, 0..=32 entries, id rule, unique |
| PD-13 | Value ranges (inclusive, sanity only, not targets) | integrated loudness [-40.0, -5.0] LUFS; `loudness_shift` each in [-12.0, 12.0] LU; true-peak ceiling [-12.0, 0.0] dBTP; frequency [10.0, 24000.0] Hz; curve levels [-120.0, 60.0] dB; PLR [0.0, 30.0] dB; LRA [0.0, 40.0] LU; tilt [-3.0, 3.0] dB/oct; `tilt_pivot_hz` [100.0, 10000.0]; band offset [-12.0, 12.0] dB; PLR offset [-12.0, 12.0] dB |
| PD-14 | Source kind `standard` | allowed in addition to `official_documentation` |
| PD-15 | Tonal statistics | 10th percentile, median, 90th percentile per frequency point |
| PD-16 | Empty set | a `ProfileSet` with zero profiles is valid; a missing profile is reported by the consumer |

## Validation

The loader is strict and all-or-nothing: if any check fails, `load_dir` / `from_files` returns an error and no
partial `ProfileSet` (no silent failures, AGENTS.md). Only the first error is returned.

### Error conventions

- `path` in every error is the profile's path relative to the profiles root, always `/`-separated on every OS
  (for example `platforms/example-platform.toml`); never absolute, never `\`. Exception: `RootNotFound` carries
  the root as given by the caller.
- `field` is a dotted path with zero-based indices, e.g. `delivery_formats[0].bitrate_kbps`,
  `shift_bounds.intent.bands[1].min_db`. For a top-level problem without a field (e.g. syntax) there is no `field`.
- `line` and `column` (in `Syntax`) are 1-based; column counts Unicode scalar values.

### Order of checks ("first error")

All ordering by path is byte-wise lexicographic order of the relative `/`-separated path.

1. **Set level, before reading contents**, in this order:
   1. `RootNotFound` (`load_dir` only);
   2. one pass over all entries in path order: `UnexpectedEntry` (including `platforms` or `materials` existing
      as a non-directory, reported with path `platforms` / `materials`), `DuplicatePath` (`from_files`; reported
      at the second occurrence in input order);
   3. `TooManyProfiles`, checked after the per-path pass (it is not tied to one path); `materials` before
      `platforms`.
2. **Per file**, files in path order (so all `materials/...` before `platforms/...`). Within a file, phases in
   this order; the first failing phase reports its first error:
   1. size (`FileTooLarge`), then UTF-8 (`NotUtf8`), then line endings (`LineEnding`, first CR byte);
   2. syntax (`Syntax`);
   3. header, in this order: `schema_version` (`MissingField` / `SchemaVersionNotFirst` / `WrongType` /
      `UnsupportedSchemaVersion`); then `kind` (`MissingField`; `WrongType`; a string outside
      {`platform`, `material`} -> `NotAllowed`; a valid kind in the wrong directory -> `KindMismatch`).
      On any header failure nothing else is reported;
   4. structure: `UnknownField`, `MissingField`, `WrongType`, `NotAllowed` (enum strings). The document is
      traversed depth-first: within each table, entries are visited in document order (a nested table at the
      position of its first key or header) and present-field errors are reported as they are met; when all
      entries of a table, including its nested tables, have been visited, that table's missing-field errors are
      reported. So **missing-field errors of a table come after all present-field errors of that same table,
      including its nested tables**; the top-level table is the whole file, so its missing fields come last.
      Several missing fields of one table are ordered as listed in this spec's field tables;
   5. per-value rules (`NonFinite`, `OutOfRange`, `IntegerOutOfRange`, `InvalidString`, `InvalidId`,
      `InvalidDate`, `InvalidUrl`, `NotAllowed` for integer sets), in document order;
   6. cross-field rules, in this order: `IdMismatch`, empty `sources`/`delivery_formats` (`MissingField`),
      sources, uniqueness, bounds, curves, format rules, origin (including required-iff `MissingField`).

### Rule to error table

| Rule | Error variant |
|---|---|
| `load_dir` root does not exist or is not a directory | `RootNotFound` |
| Root or subdirectory unreadable | `Io` |
| Entry in `platforms/` or `materials/` that is not a regular `*.toml` file; `platforms` or `materials` is a regular file (not a directory); `from_files` rel_path not `platforms/<stem>.toml` or `materials/<stem>.toml` | `UnexpectedEntry` |
| Same rel_path twice in `from_files` | `DuplicatePath` |
| More than PD-2 profiles of one kind | `TooManyProfiles` |
| File (or `from_files` content) larger than PD-2 | `FileTooLarge` |
| Content not valid UTF-8 | `NotUtf8` |
| CR byte present (CRLF or lone CR line endings) | `LineEnding` |
| Invalid TOML, including duplicate keys | `Syntax` |
| `schema_version` missing | `MissingField` (field `schema_version`) |
| `schema_version` present but not the first key of the file | `SchemaVersionNotFirst` |
| `schema_version` not an integer | `WrongType` |
| `schema_version` integer other than supported | `UnsupportedSchemaVersion` |
| `kind` missing / not a string | `MissingField` / `WrongType` (phase 3) |
| `kind` string outside {`platform`, `material`} | `NotAllowed` (phase 3) |
| Valid `kind` in the wrong directory | `KindMismatch` (phase 3) |
| Unknown key at any level | `UnknownField` |
| Required key missing (incl. `source` in a sourced value) | `MissingField` (phase 4) |
| Required-iff key missing (`origin` keys required by `origin.kind`) | `MissingField` (phase 6) |
| Key forbidden by `origin.kind` present (e.g. `track_count` with `provisional`) | `InvalidOrigin` (phase 6) |
| Wrong TOML type (e.g. float in integer field, string in number field, table expected) | `WrongType` |
| Unknown enum string (`codec`, `sample_format`, `origin.kind`, source `kind`) | `NotAllowed` |
| `sample_rate_hz` or `channels` integer that does not fit its Rust type (`u32` / `u16`, incl. negative) | `IntegerOutOfRange` |
| `sample_rate_hz` in type range but not in PD-3; `channels` in type range but != 2 | `NotAllowed` |
| `nan`, `inf`, `-inf` | `NonFinite` |
| Float outside its range (PD-13) | `OutOfRange` |
| Integer outside its range, including values that do not fit the Rust type (`revision`, `track_count`, `bitrate_kbps` PD-4, negative values) | `IntegerOutOfRange` |
| String empty where non-empty required, or longer than PD-10 | `InvalidString` |
| Id, source id, band name or tag violates PD-1; `analysis_method` characters | `InvalidId` |
| Bad date | `InvalidDate` |
| URL not `https://` absolute with host, or longer than PD-10 | `InvalidUrl` |
| `id` != file stem | `IdMismatch` |
| Duplicate source id | `DuplicateSource` |
| `source` names no source | `UnknownSource` |
| Source never referenced (PD-9) | `UnusedSource` |
| Duplicate tag; duplicate band name in one table; duplicate delivery format | `Duplicate` |
| `sources` or `delivery_formats` empty (phase 6) | `MissingField` (field `sources` / `delivery_formats`) |
| Too many tags (PD-12) or bands (PD-6) | `TooMany` |
| `min > max`; `0` not in `[min, max]` where required; `low_hz >= high_hz`; `tilt_pivot_hz` differs between tables; band mismatch across tables (PD-8) | `InvalidBounds` |
| Curve lengths differ; point count outside PD-5; `freq_hz` not strictly increasing; `p10 > median` or `median > p90` | `InvalidCurve` |
| `sample_format` on lossy codec or missing on PCM; `bitrate_kbps` on PCM or missing on lossy; `flac` with `float32` | `FormatRule` |
| `corpus_manifest` absolute, with `..`, or not under `eval/` | `InvalidOrigin` |
| `adr` not matching `^ADR-[0-9]{4}$` | `InvalidString` |

## Public API

Crate `aitm-engine` (`aitm_engine`), module `profiles`. Dependencies `serde` and `schemars` (PLAN section 4,
"Схемы") are approved through task M0-04 (dependency and license policy) or a dedicated dependency ADR; the TOML
parser crate needs its own ADR in M6.

### Validated construction (no bypass)

- `PlatformProfile` and `MaterialProfile` have **private fields and read-only getters**. The only ways to obtain
  one are the loader functions working on bytes: `parse_platform_profile`, `parse_material_profile`,
  `ProfileSet::load_dir`, `ProfileSet::from_files`. **No profile type implements `Deserialize`**; JSON is
  output-only (for the MCP resource), so there is no second, unvalidated input path.
- Sub-types (`LoudnessTarget`, `DeliveryFormat`, `ShiftBounds`, ...) are `#[non_exhaustive]` with public read-only
  fields, so other crates cannot construct them. Inside `aitm-engine`, `#[non_exhaustive]` does not prevent
  construction; by convention only the `profiles` loader module constructs profile types, and code review
  enforces this.
- `ProfileSet` derives only `Debug, Clone, PartialEq` (no `Serialize`, no `Deserialize`); it is built only by the
  loader. The MCP resource serializes individual `PlatformProfile` values.
- Engine target-selection entry points (M6) take `&PlatformProfile` / `&MaterialProfile` (or `&ProfileSet` plus
  ids), never loose sub-structs or raw numbers, so every target the engine uses has passed validation.
- Top-level profile types and sub-types derive `Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema`.
  Serialization uses exactly the field names and nesting of the file format, including `schema_version`, `kind`,
  `id` and `revision` (sourced values serialize as `{ "value": ..., "source": ... }`), so the MCP resource shows
  provenance to the agent.

```rust
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub enum ProfileKind { Platform, Material }                       // "platform" | "material"

/// Validated, immutable set of profiles. Iteration order is ascending by id.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileSet { /* private */ }

impl ProfileSet {
    /// Loads `<root>/platforms/*.toml` and `<root>/materials/*.toml`. Missing subdirectory = empty kind.
    pub fn load_dir(root: &std::path::Path) -> Result<ProfileSet, ProfileError>;
    /// Same validation from in-memory files, e.g. profiles embedded at compile time.
    /// `rel_path` must be "platforms/<stem>.toml" or "materials/<stem>.toml" ('/' separators).
    pub fn from_files<'a, I>(files: I) -> Result<ProfileSet, ProfileError>
    where I: IntoIterator<Item = (&'a str /* rel_path */, &'a [u8] /* contents */)>;

    pub fn platform(&self, id: &str) -> Option<&PlatformProfile>;
    pub fn material(&self, id: &str) -> Option<&MaterialProfile>;
    pub fn platforms(&self) -> impl Iterator<Item = &PlatformProfile>;
    pub fn materials(&self) -> impl Iterator<Item = &MaterialProfile>;
}

/// Parse and validate a single file; `file_stem` is checked against `id`. Errors carry
/// `path = "platforms/<file_stem>.toml"` / `"materials/<file_stem>.toml"`. Size and UTF-8 rules apply.
pub fn parse_platform_profile(file_stem: &str, bytes: &[u8]) -> Result<PlatformProfile, ProfileError>;
pub fn parse_material_profile(file_stem: &str, bytes: &[u8]) -> Result<MaterialProfile, ProfileError>;

#[non_exhaustive] pub struct Sourced<T> { pub value: T, pub source: String }

pub enum SourceKind { OfficialDocumentation, Standard }          // "official_documentation" | "standard"
#[non_exhaustive] pub struct Source {
    pub id: String, pub kind: SourceKind, pub url: String,
    pub accessed: String /* YYYY-MM-DD */, pub title: String, pub excerpt: Option<String>,
}

/// Private fields; built only by the loader; Serialize + JsonSchema, not Deserialize.
/// Serialized form includes `"kind": "platform"`.
pub struct PlatformProfile { /* private */ }
impl PlatformProfile {
    pub fn schema_version(&self) -> u32;
    pub fn id(&self) -> &str;
    pub fn revision(&self) -> u32;
    pub fn name(&self) -> &str;
    pub fn description(&self) -> Option<&str>;
    pub fn sources(&self) -> &[Source];
    pub fn loudness(&self) -> &LoudnessTarget;
    pub fn loudness_shift(&self) -> &LoudnessShiftBounds;
    pub fn true_peak(&self) -> &TruePeakCeiling;
    pub fn delivery_formats(&self) -> &[DeliveryFormat];
}
#[non_exhaustive] pub struct LoudnessTarget { pub integrated_lufs: Sourced<f64> }
#[non_exhaustive] pub struct LoudnessShiftBounds { pub min_lu: f64, pub max_lu: f64, pub rationale: String, pub adr: Option<String> }
#[non_exhaustive] pub struct TruePeakCeiling { pub ceiling_dbtp: Sourced<f64> }

pub enum Codec { Wav, Flac, Aiff, Mp3, Aac, OggVorbis }           // snake_case in files
impl Codec { pub fn is_lossy(self) -> bool; }
pub enum SampleFormat { Int16, Int24, Float32 }                   // "int16" | "int24" | "float32"
#[non_exhaustive] pub struct DeliveryFormat {
    pub codec: Sourced<Codec>, pub sample_rate_hz: Sourced<u32>,
    pub sample_format: Option<Sourced<SampleFormat>>, pub bitrate_kbps: Option<Sourced<u32>>,
    pub channels: Sourced<u16>,
}

/// Private fields; built only by the loader; Serialize + JsonSchema, not Deserialize.
/// Serialized form includes `"kind": "material"`.
pub struct MaterialProfile { /* private */ }
impl MaterialProfile {
    pub fn schema_version(&self) -> u32;
    pub fn id(&self) -> &str;
    pub fn revision(&self) -> u32;
    pub fn name(&self) -> &str;
    pub fn description(&self) -> Option<&str>;
    pub fn tags(&self) -> &[String];
    pub fn origin(&self) -> &Origin;
    pub fn tonal(&self) -> &TonalTarget;
    pub fn dynamics(&self) -> &DynamicTarget;
    pub fn shift_bounds(&self) -> &ShiftBoundsSet;
}
#[non_exhaustive] pub enum Origin {                               // internally tagged by "kind"
    Corpus { corpus_manifest: String, track_count: u32, computed: String, tool: String },
    Provisional { rationale: String },
}
#[non_exhaustive] pub struct TonalTarget {
    pub analysis_method: String, pub freq_hz: Vec<f64>,
    pub p10_db: Vec<f64>, pub median_db: Vec<f64>, pub p90_db: Vec<f64>,
}
#[non_exhaustive] pub struct Range { pub min: f64, pub max: f64 }
#[non_exhaustive] pub struct DynamicTarget { pub plr_db: Range, pub lra_lu: Range }
#[non_exhaustive] pub struct BandShiftBounds { pub name: String, pub low_hz: f64, pub high_hz: f64, pub min_db: f64, pub max_db: f64 }
#[non_exhaustive] pub struct ShiftBounds {
    pub tilt_db_per_octave: Range, pub tilt_pivot_hz: f64, pub bands: Vec<BandShiftBounds>,
    pub plr_offset_db: Range, pub rationale: String, pub adr: Option<String>,
}
#[non_exhaustive] pub struct ShiftBoundsSet { pub intent: ShiftBounds, pub reference: ShiftBounds, pub combined: ShiftBounds }

/// Owned, serializable (for reports); not Deserialize. See "Error conventions".
#[derive(Debug, Clone, PartialEq, serde::Serialize, thiserror::Error)]
#[non_exhaustive]
pub enum ProfileError {
    RootNotFound { root: String },
    Io { path: String, message: String },
    UnexpectedEntry { path: String },
    DuplicatePath { path: String },
    TooManyProfiles { kind: ProfileKind, max: usize },
    FileTooLarge { path: String, size_bytes: u64, max_bytes: u64 },
    NotUtf8 { path: String },
    LineEnding { path: String, line: usize /* 1-based line containing the first CR */ },
    Syntax { path: String, line: usize, column: usize, message: String },
    SchemaVersionNotFirst { path: String },
    UnsupportedSchemaVersion { path: String, found: i64, supported: Vec<u32> },
    KindMismatch { path: String, expected: ProfileKind, found: ProfileKind },
    IdMismatch { path: String, id: String, file_stem: String },
    MissingField { path: String, field: String },
    UnknownField { path: String, field: String },
    WrongType { path: String, field: String, expected: String },
    NotAllowed { path: String, field: String, value: String, allowed: Vec<String> },
    NonFinite { path: String, field: String },
    OutOfRange { path: String, field: String, value: f64, min: f64, max: f64 },
    IntegerOutOfRange { path: String, field: String, value: String /* decimal */, min: i64, max: i64 },
    InvalidString { path: String, field: String, reason: String },
    InvalidId { path: String, field: String, value: String },
    InvalidDate { path: String, field: String, value: String },
    InvalidUrl { path: String, field: String, value: String },
    UnknownSource { path: String, field: String, source: String },
    DuplicateSource { path: String, source: String },
    UnusedSource { path: String, source: String },
    Duplicate { path: String, field: String, value: String },
    TooMany { path: String, field: String, count: usize, max: usize },
    InvalidBounds { path: String, field: String, reason: String },
    InvalidCurve { path: String, field: String, reason: String },
    FormatRule { path: String, field: String, reason: String },
    InvalidOrigin { path: String, field: String, reason: String },
}
```

`ProfileError` implements `std::error::Error` and `Display` (message names path, field and reason). The loader
never panics on any input. (`thiserror` is named per AGENTS.md "typed errors"; its approval follows M0-04 as above.)

### MCP resource (proposal for M4)

The server exposes the loaded `ProfileSet` read-only: a list resource of platform profiles and one resource per
platform profile, content = JSON serialization of `PlatformProfile`.
Exact URIs are part of the M4 MCP schema (open question 5). There is no MCP tool, CLI flag or config that edits a
profile at runtime or overrides the true-peak ceiling.

## Reference

No DSP. Independent definitions used by this format:

- ITU-R BS.1770 (current revision), integrated loudness (LUFS/LKFS) and true peak (dBTP):
  https://www.itu.int/rec/R-REC-BS.1770
- EBU Tech 3342, loudness range (LU): https://tech.ebu.ch/publications/tech3342
- TOML 1.0.0 specification (pending ADR in M6): https://toml.io/en/v1.0.0
- RFC 3339 `full-date` (date format): https://www.rfc-editor.org/rfc/rfc3339
- RFC 3986 (URI syntax): https://www.rfc-editor.org/rfc/rfc3986

Real platform values are not part of this spec; they are added in M6 from current official documentation.

## Acceptance criteria

- [ ] Profiles are data files under `profiles/platforms/` and `profiles/materials/`; no target value is a code
      constant (engine code reads them through validated `PlatformProfile` / `MaterialProfile` only).
- [ ] Every platform value is a sourced value referencing a source with `https` URL and access date; a value
      without a resolvable source fails to load.
- [ ] Material profiles contain only bounded numeric curves (bounded, see Proposed defaults) and scalars; no audio
      can be stored.
- [ ] Intent, reference and combined shift bounds are expressible and validated (`min <= 0 <= max`);
      true-peak ceiling is not shiftable; no field can disable post-codec re-check or "do no harm".
- [ ] No unvalidated profile can be constructed or deserialized outside the loader.
- [ ] Platform profiles serialize to JSON with provenance for the MCP resource; `schemars` schema generated.
- [ ] Loading is deterministic: same files give equal `ProfileSet` and the same first error on every OS.
- [ ] All examples in this spec use placeholder values.

## Tests

For test-author (M6 loader task), written against the Public API only; expected variants per the rule table:

- Valid fixtures: the two examples above parse. Parse TOML -> serialize to JSON -> the JSON validates against the
  `schemars` schema of the type and contains `schema_version`, `kind` (`"platform"` / `"material"`), `id`,
  `revision`, source ids and sourced values as `{ "value", "source" }`. (No JSON input path exists to test.)
- `schema_version` = 0, 2, missing, string -> `UnsupportedSchemaVersion` / `MissingField` / `WrongType`;
  `schema_version = 1` as the second key (after `kind`) -> `SchemaVersionNotFirst`.
- `kind = "preset"` -> `NotAllowed`; `kind = "material"` in `platforms/` -> `KindMismatch`; a file with both
  `kind = "preset"` and an unknown key -> `NotAllowed` (phase 3 before structure).
- CRLF line endings, or a single CR inside a comment -> `LineEnding`.
- Unknown key at top level, in `[loudness]`, in a `[[delivery_formats]]` entry -> `UnknownField` with field path.
  Specifically `skip_do_no_harm = true` or a `true_peak_shift` table is rejected.
- Unknown enum strings for each enum field -> `NotAllowed`.
- Sourced value without `source` -> `MissingField`; unknown id -> `UnknownSource`; duplicate -> `DuplicateSource`;
  unused -> `UnusedSource`.
- Source `url` with `http://`, relative URL, empty host -> `InvalidUrl`; `accessed` = `2025-02-29`,
  `2025-13-01`, `25-01-01` -> `InvalidDate`; `2024-02-29` accepted.
- Every numeric field: exact bounds accepted, next float outside rejected (`OutOfRange`); `nan`, `inf`, `-inf` ->
  `NonFinite`; float in integer field -> `WrongType`; `revision = 0`, `-1`, `4294967296` and
  `9223372036854775807` -> `IntegerOutOfRange`.
- Strings: empty `name`, over-length per PD-10 (limit accepted, limit + 1 rejected) -> `InvalidString`;
  `adr = "0007"` -> `InvalidString`.
- Delivery formats: lossy without bitrate, PCM with bitrate, lossy with `sample_format`, `flac` + `float32` ->
  `FormatRule`; sample rate 22050 and `channels = 1` -> `NotAllowed`; sample rate `-1` or `4294967296` and
  `channels = 65536` -> `IntegerOutOfRange`; two entries equal in values but with
  different sources -> `Duplicate`; empty list -> `MissingField`; a profile with only PCM formats is valid.
- Shift bounds with `min > max`, `min > 0`, `max < 0`; band `low_hz >= high_hz`; differing `tilt_pivot_hz`; band in
  `intent` missing from `combined`; same band name with different edges -> `InvalidBounds`; duplicate band name ->
  `Duplicate`; 17 bands -> `TooMany`.
- Curves: lengths differ; `freq_hz` not strictly increasing; 7 and 257 points; `p10 > median` at one point ->
  `InvalidCurve`; 8 and 256 points accepted.
- Tags: duplicate -> `Duplicate`; 33 tags -> `TooMany`; uppercase tag -> `InvalidId`.
- Origin: `corpus` without `track_count` -> `MissingField`; `provisional` with `track_count` (a known key, not
  `UnknownField`) -> `InvalidOrigin`; `corpus_manifest` with `..`, absolute, or outside `eval/` -> `InvalidOrigin`.
- `id` != file stem -> `IdMismatch`; invalid id characters -> `InvalidId`.
- `load_dir`: missing root -> `RootNotFound`; subdirectory or non-`.toml` file inside `platforms/` ->
  `UnexpectedEntry`; `<root>/platforms` is a regular file -> `UnexpectedEntry` with path `platforms`;
  257 profiles in `platforms/` plus a non-`.toml` file there -> `UnexpectedEntry` (per-path pass before
  `TooManyProfiles`); file of 262145 bytes -> `FileTooLarge` (262144 accepted); invalid UTF-8 -> `NotUtf8`;
  257 profiles of one kind -> `TooManyProfiles`; one bad file among good ones -> error, no partial set;
  bad files `materials/b.toml` and `platforms/a.toml` -> error of `materials/b.toml`; error `path` uses `/` on Windows.
- `from_files`: duplicate rel_path -> `DuplicatePath`; rel_path `platforms\\x.toml` or `other/x.toml` ->
  `UnexpectedEntry`; oversize content -> `FileTooLarge`; same content as `load_dir` gives an equal set.
- First-error ordering within a file (missing-field errors of a table come after all present-field errors of that
  table, nested tables included; the top-level table is the whole file):
  - top-level `name` missing and an unknown key on the last line of the file -> `UnknownField`;
  - top-level `name` missing and `[loudness]` with a string where a number is required (`integrated_lufs = { value = "x", ... }`) ->
    `WrongType` (nested present-field error before the top-level missing field);
  - `[true_peak]` without `ceiling_dbtp`, followed later in the file by `[[delivery_formats]]` with an unknown
    key -> `MissingField` (`true_peak.ceiling_dbtp`): the `true_peak` table is completed (its missing fields
    reported) before the later sibling table is visited.
  - `Syntax` line/column are 1-based.
- Fuzz/proptest: arbitrary bytes into `parse_*` never panic.

## Out of scope

- Real platform values and the actual profile files (M6, engine, from current official documentation).
- Loader implementation, syntax ADR and parser crate choice (M6, before the loader task).
- Tonal curves computed from a reference corpus and the tool that computes them (later; eval/analysis).
- Spectrum analysis method definition (`analysis_method`), mapping of intent words to shift amounts,
  objective function weights, dynamics/transient loss budgets (engine specs, M5-M7).
- Release quality targets such as the ±0.5 LU tolerance and the golden-set lossy check format
  (docs/spec/release-targets.md); they are not profile data.

## Open questions

1. **Syntax (ADR deferred to M6).** Proposal: TOML 1.0 (comments, human-editable, readable diffs for provenance).
   Cost: a new dependency (a TOML parser) needing dependency-auditor review. Alternative: JSON, likely already a
   transitive dependency via rmcp/serde_json, no comments. Decision recorded in the task file Notes: the ADR is
   written in M6 before the loader task.
2. **Profile distribution.** PLAN says platform targets are "updated separately from code". Embed `profiles/`
   into the binary at build time (`from_files`), read a directory at runtime (`load_dir`), or both with an
   override directory? An override changes results for the same input + intent; if allowed, reports must record
   the profile set's origin. Also: should reports include a content hash of each profile (algorithm = new decision)?
3. **Values without a platform loudness target.** Some deliveries (for example CD) may have no documented
   loudness target. Is `loudness.integrated_lufs` then a project decision (`rationale` instead of `source`), or
   is `loudness` optional and the target derived from the material? The current draft requires a sourced value.
4. **CI checks outside the loader.** Who flags platform sources whose `accessed` date is old, and after how many
   days? Should CI check that each `adr` exists in `docs/adr/`, and reject reserved example domains
   (`example.com`, `.invalid`, `.test`) in `profiles/` so placeholders never ship? Proposal: one scheduled/PR CI
   check, keeping the loader clock-free and repository-free.
5. **MCP resource URIs and whether material profiles are also exposed** (PLAN section 6 names only "targets per
   platform" and "presets"). To be fixed in the M4 MCP schema.
6. **Channels and codecs.** v1 fixes `channels = 2` (PD-11) and the PLAN section 13 codec list; Opus or other
   codecs a platform documents are not representable without a schema bump. Acceptable?
7. **Material profile selection.** How intent (genre, character) selects a material profile (`tags` matching,
   explicit id, analysis-based classification) is engine behavior in M6; is a `tags` field (PD-12) enough for now?
8. **Platform normalization semantics.** Some platforms document a normalization reference level rather than a
   delivery recommendation, or several user-selectable levels. Does v1 need a field describing what the number
   means, or multiple loudness targets per platform?
9. **Proposed defaults.** Confirm or change PD-1 ... PD-16.
10. **RT-1 target with loudness shift.** docs/spec/release-targets.md RT-1 measures "within ±0.5 LU of the
    profile's loudness target". When intent shifts loudness within `loudness_shift`, is RT-1 measured against the
    profile value or the shifted target? Draft proposal: against the effective target (profile value plus the
    bounded shift), which the engine records in the report next to the profile value and the applied shift.
