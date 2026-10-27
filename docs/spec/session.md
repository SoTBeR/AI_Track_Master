# Spec: Session model (tracks, buses, master bus, versions)

- Status: draft
- Layer: platform (`crates/core`, package `aitm-core`, see ADR-0001)
- Task: docs/tasks/M0/05-session-spec.md
- Plan: docs/PLAN.md section 2 (principles 1, 4, 5, 6, 7, 9), section 3, section 4 (row "Хранение"),
  section 6 (tools group "Сессия", "Ограничения безопасности"), section 9 (M1, M3), section 13, section 14 item 2

## What

The session is the single data model that every layer above `core` works on. A session is a
directed acyclic routing graph of **tracks**, **buses** and exactly one **master bus**; each track
plays one imported source file; each track, bus and the master bus carries an ordered plugin chain.
Every change to a session is committed as a new immutable **version**; any earlier version can be
restored. Sessions live inside a **project folder** that holds one SQLite database plus files.

A plain stereo file is not a special case: it is a session with one track routed to the master bus.

## Why

- PLAN section 2 principle 1: the multitrack model exists from day one so mixing and mastering share
  one pipeline ("Обычный стерео-трек — это сессия из одной дорожки").
- PLAN section 2 principle 7: sources are never modified; every change creates a session version
  with rollback. PLAN section 9 M3: rollback returns the previous state bit-for-bit.
- PLAN section 2 principle 5: MCP server, CLI and a future GUI are thin shells over one core API, so
  this API is the only way to read or change a session (MCP tools `create_session`, `import_audio`,
  `list_tracks`, `get_session_state`, `list_versions`, `restore_version`, `route_to_bus`,
  `apply_plugin`, `set_param` map onto it in M4; CLI `session` and `apply` commands in M3).
- PLAN section 4 row "Хранение": SQLite (rusqlite) and files, holding session versions, reports and
  the audit log.
- PLAN section 2 principle 9 and section 6 "Ограничения безопасности": writes only inside the project
  folder, in any mode.

## Public API

**This section is the M1/M3 contract draft.** Names and signatures are binding for test-author once
this spec is accepted; until then they may change. Items marked *(M2)* depend on the plugin-sdk spec
(parameter schemas) and are not testable before M2. Items marked *(M3)* are the persistence and
versioning part delivered in M3; in M1 the same types exist in memory only (see "Milestone split").

All items are in crate `aitm_core`, module `session` unless stated otherwise.

```rust
// ---------- Identifiers (deterministic sequential numbers) ----------
// The display forms "s<N>", "t<N>", "b<N>", "p<N>", "src<N>", "v<N>" are a draft proposal (Q15).

/// Session identifier, unique within a project. Display form: "s<N>", N >= 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SessionId(pub u32);

/// Track identifier, unique within a session, never reused after removal. Display: "t<N>".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TrackId(pub u32);

/// Bus identifier, unique within a session, never reused after removal. Display: "b<N>".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BusId(pub u32);

/// Plugin slot identifier, unique within a session, never reused. Display: "p<N>".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SlotId(pub u32);

/// Source identifier, unique within a session. Display: "src<N>".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceId(pub u32);

/// Version identifier: the version number within its session, starting at 1, strictly
/// increasing by 1 per commit. Display: "v<N>".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VersionId(pub u64);

// ---------- Graph nodes ----------

/// A node that owns a plugin chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NodeId { Track(TrackId), Bus(BusId), Master }

/// Where a track or bus sends its output. The master bus has no output target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputTarget { Bus(BusId), Master }

// ---------- Session state (value type, immutable once committed) ----------

/// Complete state of a session at one version.
/// `PartialEq` is hand-written as `self.to_canonical_bytes() == other.to_canonical_bytes()`
/// (not derived: derived f64 equality treats -0.0 == 0.0, canonical bytes do not).
/// No public `Deserialize` (it contains `SourceRef`); the only decoder is `from_canonical_bytes`.
#[derive(Clone, Debug, Serialize)]
pub struct SessionState {
    pub name: String,
    /// Processing sample rate. `None` only while the session has no tracks.
    pub sample_rate_hz: Option<u32>,
    pub sources: BTreeMap<SourceId, SourceRef>,
    pub tracks: BTreeMap<TrackId, Track>,
    pub buses: BTreeMap<BusId, Bus>,
    pub master: MasterBus,
    /// Next values for the per-kind id counters (ids are never reused).
    pub next_ids: IdCounters,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdCounters { pub track: u32, pub bus: u32, pub slot: u32, pub source: u32 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub source: SourceId,
    pub output: OutputTarget,
    pub chain: Vec<PluginSlot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bus {
    pub name: String,
    pub output: OutputTarget,
    pub chain: Vec<PluginSlot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MasterBus { pub chain: Vec<PluginSlot> }

/// An imported audio file. The file itself is never written (PLAN section 2 principle 7).
///
/// Opaque: all fields are private and there is no public constructor, no `Default` and no
/// public `Deserialize`. The only ways to obtain a `SourceRef` are `probe_source` (a fresh,
/// validated intake of a real file) and reading one out of a state restored by
/// `SessionState::from_canonical_bytes` / `Project::state` (a previously probed source).
/// So unprobed or forged source metadata can never reach a stored project (`Project::commit`
/// rejects `AddTrack`; only `import_audio` adds tracks).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceRef { /* private fields, read through the accessors below */ }

impl SourceRef {
    /// Path exactly as given by the caller (for reports).
    pub fn given_path(&self) -> &str;
    /// Absolute canonical path resolved at import time.
    pub fn canonical_path(&self) -> &str;
    /// Digest of the file bytes at import time (algorithm: see Open questions Q9).
    pub fn content_digest(&self) -> ContentDigest;
    /// Container/codec detected from content, not extension.
    pub fn format(&self) -> AudioFormat;
    pub fn sample_rate_hz(&self) -> u32;
    /// 1 or 2 in this draft.
    pub fn channels(&self) -> u16;
    pub fn frames(&self) -> u64;
    /// Source bit depth, `None` for formats without a fixed one.
    pub fn bit_depth(&self) -> Option<u16>;
}

/// One plugin instance in a chain. *(M2)* for any non-empty chain.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginSlot {
    pub slot: SlotId,
    /// Built-in plugin identifier and version, as defined by the plugin-sdk spec.
    pub plugin_id: String,
    pub plugin_version: String,
    /// Plugin parameter-schema version, for state migrations (PLAN section 7).
    pub schema_version: u32,
    /// Every parameter of the schema, keyed by parameter name. Always complete:
    /// defaults are materialized at insertion, so state never depends on implicit defaults.
    pub params: BTreeMap<String, ParamValue>,
}

/// Parameter value. Floats are finite (NaN and Inf are rejected on input).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ParamValue { Float(f64), Int(i64), Bool(bool), Choice(String) }

impl PartialEq for SessionState { /* canonical-bytes equality, see above */ }

impl SessionState {
    /// Canonical, platform-independent byte serialization of the full state.
    /// Deterministic: maps in key order, fixed field order, f64 serialized so that it
    /// round-trips bit-exactly (including -0.0). Basis for equality and `StateDigest`.
    /// The exact encoding is an implementation choice recorded in an ADR (Q9); tests rely only
    /// on determinism, round-trip and cross-platform digest equality, not on fixed bytes.
    pub fn to_canonical_bytes(&self) -> Vec<u8>;
    /// Inverse of `to_canonical_bytes`. Rejects with `CorruptState` any input that does not
    /// decode, contains a non-finite `ParamValue::Float` (NaN, +Inf, -Inf), or violates an
    /// invariant from Behavior "Entities and invariants".
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<SessionState, SessionError>;

    // ----- M1 in-memory API (no Project, no SQLite) -----

    /// Empty session: no sources, tracks or buses, empty master chain, `sample_rate_hz = None`,
    /// all id counters at 1. Fails with `InvalidName` on an invalid name.
    pub fn new_empty(name: &str) -> Result<SessionState, SessionError>;
    /// Apply `edits` in order to a copy of `self` and return the new state, all-or-nothing:
    /// on any error `self` is untouched and no partial state is returned.
    /// `limits` is injected by the caller (never a constant inside `core`).
    pub fn apply(&self, edits: &[Edit], limits: &SessionLimits)
                 -> Result<SessionState, SessionError>;

    pub fn digest(&self) -> StateDigest;
    /// Topological render order of all nodes, ending with `NodeId::Master`.
    /// Ties broken by `NodeId` ordering (tracks by id, then buses by id), so it is deterministic.
    pub fn render_order(&self) -> Vec<NodeId>;
}

/// Intake of one audio file for import (PLAN section 3 stage 1): opens the file read-only,
/// detects the format by content, computes `content_digest`, and returns its metadata.
/// Rejects properties of the file itself: not audio / malformed (`Intake`), rate outside the
/// supported range (`UnsupportedSampleRate`), unsupported channel count
/// (`UnsupportedChannelCount`). Knows nothing about sessions and never writes anything.
/// The only constructor of `SourceRef`. Used by `Project::import_audio` and directly by M1 tests.
pub fn probe_source(path: &Path) -> Result<SourceRef, SessionError>;

// M1 one-track construction, no storage:
//   let src = probe_source(path)?;
//   let s = SessionState::new_empty("song")?
//       .apply(&[Edit::AddTrack { name: None, source: src }], &limits)?;

// ---------- Edits: the only way to change a session ----------

/// A single atomic change. A list of edits passed to `commit` is applied all-or-nothing
/// and produces exactly one new version.
/// No public `Deserialize` (`AddTrack` holds an opaque `SourceRef`); MCP and CLI build edits
/// from their own versioned schemas (M3/M4), never by deserializing `Edit` directly.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Edit {
    RenameSession { name: String },
    /// Adds `source` as a new source entry (new SourceId = next_ids.source) and one new track
    /// (new TrackId = next_ids.track) playing it, routed to `OutputTarget::Master`.
    /// `source` can only come from `probe_source` (or a stored state), see `SourceRef`.
    /// `name = None` derives the name from the source file stem (Q16).
    /// Rejects inconsistency with the session: `SampleRateMismatch`, `TrackLimitExceeded`,
    /// `InvalidName`. (File properties were already validated by `probe_source`.)
    /// Accepted by `SessionState::apply`; rejected by `Project::commit` with `EditNotAllowed`:
    /// in a project, tracks are added only through `Project::import_audio`.
    AddTrack { name: Option<String>, source: SourceRef },
    RenameTrack { track: TrackId, name: String },
    RemoveTrack { track: TrackId },
    AddBus { name: String },                       // new BusId = next_ids.bus
    RenameBus { bus: BusId, name: String },
    /// Fails with `BusInUse` while any track or bus outputs to it.
    RemoveBus { bus: BusId },
    /// `route_to_bus`. Fails with `RoutingCycle` if the graph would become cyclic.
    SetOutput { node: NodeId, to: OutputTarget },  // node must be Track or Bus
    /// *(M2)* `apply_plugin`. `index` in 0..=chain.len(); params start at schema defaults,
    /// then `params` overrides are applied and validated against the schema ranges.
    InsertPlugin { node: NodeId, index: usize, plugin_id: String,
                   params: BTreeMap<String, ParamValue> },   // new SlotId = next_ids.slot
    /// *(M2)*
    RemovePlugin { slot: SlotId },
    /// *(M2)* `index` is the target position after removal from the old position.
    MovePlugin { slot: SlotId, index: usize },
    /// *(M2)* `set_param`. Validated against the plugin's parameter schema.
    SetParam { slot: SlotId, param: String, value: ParamValue },
}

/// Who/what caused a version. Metadata only: not part of `SessionState`, not in the digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeNote {
    /// Operation name, e.g. "import_audio", "set_param", "auto_master", "restore_version".
    pub operation: String,
    /// Short human-readable description for reports and `list_versions`.
    pub summary: String,
}

// ---------- Project folder and storage (M3; M1 has an in-memory equivalent) ----------

/// The project folder: the only place this crate ever writes.
pub struct ProjectRoot { /* canonical absolute path */ }

impl ProjectRoot {
    /// Resolve a project-relative path for writing. Rejects absolute paths, `..` components,
    /// and any path whose resolved location (following symlinks of existing components)
    /// is outside the root. Never creates anything.
    pub fn resolve_write(&self, relative: &Path) -> Result<PathBuf, PathError>;
    pub fn path(&self) -> &Path;
}

pub struct Project { /* ProjectRoot + SQLite connection */ }

impl Project {
    /// Create a new project in an empty or non-existent directory. `limits` is stored in the
    /// `Project` and passed to `SessionState::apply` by every mutating method below.
    pub fn create(root: &Path, limits: SessionLimits) -> Result<Project, StorageError>;
    /// Open an existing project. Fails on unknown or newer project format versions.
    pub fn open(root: &Path, limits: SessionLimits) -> Result<Project, StorageError>;
    pub fn root(&self) -> &ProjectRoot;
    pub fn limits(&self) -> &SessionLimits;

    /// `create_session`. Creates version v1: empty session (no tracks, no buses,
    /// empty master chain, `sample_rate_hz = None`).
    pub fn create_session(&mut self, name: &str) -> Result<(SessionId, VersionId), SessionError>;
    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>, SessionError>;

    /// `import_audio`. The only way to add a track to a stored session: `probe_source(path)`,
    /// then `SessionState::apply(&[Edit::AddTrack { .. }], self.limits())` on the head state,
    /// committed as one version: one source, one track routed to `OutputTarget::Master`.
    /// Draft proposal (Q18): rejects sources at managed paths of the project folder
    /// (`SourceInsideProject`, see "Write-path protection").
    /// Never writes to, moves or deletes the source file.
    pub fn import_audio(&mut self, session: SessionId, expected_head: VersionId,
                        path: &Path, track_name: Option<&str>)
                        -> Result<ImportOutcome, SessionError>;

    /// Apply `edits` atomically on top of `expected_head` (via `SessionState::apply` with
    /// `self.limits()`) and commit exactly one new version.
    /// On any error nothing is committed and the head is unchanged.
    /// Rejects any `Edit::AddTrack` in `edits` with `EditNotAllowed` (use `import_audio`).
    /// `expected_head` (also on `import_audio`, `restore_version`) is a draft proposal (Q14).
    pub fn commit(&mut self, session: SessionId, expected_head: VersionId,
                  edits: &[Edit], note: ChangeNote) -> Result<VersionId, SessionError>;

    /// Current head version.
    pub fn head(&self, session: SessionId) -> Result<VersionId, SessionError>;
    /// `get_session_state`. State at `version`, or at head if `None`.
    pub fn state(&self, session: SessionId, version: Option<VersionId>)
                 -> Result<SessionState, SessionError>;
    /// `list_tracks`: tracks of the head state in `TrackId` order.
    pub fn list_tracks(&self, session: SessionId) -> Result<Vec<(TrackId, Track)>, SessionError>;
    /// `list_versions`: all versions, ascending.
    pub fn list_versions(&self, session: SessionId) -> Result<Vec<VersionInfo>, SessionError>;
    /// `restore_version`. Commits a NEW version whose state is byte-identical to `target`'s
    /// state. History is never truncated or rewritten.
    /// Draft (Q19): fails with `TrackLimitExceeded` if the target's track count exceeds the
    /// current `self.limits()` (limits may have been lowered since the target was committed).
    pub fn restore_version(&mut self, session: SessionId, expected_head: VersionId,
                           target: VersionId) -> Result<VersionId, SessionError>;
}

pub struct ImportOutcome {
    pub track: TrackId,
    pub source: SourceId,
    pub version: VersionId,
    pub source_info: SourceRef,
}

pub struct SessionSummary { pub id: SessionId, pub name: String, pub head: VersionId,
                            pub track_count: usize, pub bus_count: usize }

pub struct VersionInfo {
    pub id: VersionId,
    /// Version this one was committed on top of; `None` only for v1.
    pub parent: Option<VersionId>,
    /// For versions created by `restore_version`: the restored target.
    pub restored_from: Option<VersionId>,
    pub note: ChangeNote,
    /// Wall-clock commit time, UTC, RFC 3339. Metadata only.
    pub created_at: String,
    pub digest: StateDigest,
}

/// Opaque fixed-size digest of `to_canonical_bytes()`. Equal states have equal digests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateDigest(/* bytes */);

/// Opaque digest of a source file's bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentDigest(/* bytes */);

/// Session-wide limits. Always injected by the caller (CLI, server, tests); `core` holds no
/// default value. The production value comes from the performance targets in
/// docs/spec/release-targets.md (task M0-13; maintainer decision 2026-10-27: 64 tracks),
/// loaded from configuration by the calling layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionLimits { pub max_tracks: usize }

// ---------- Errors (typed; never panic on user input) ----------

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("session {0:?} not found")]            SessionNotFound(SessionId),
    #[error("version {0:?} not found")]            VersionNotFound(VersionId),
    #[error("track {0:?} not found")]              TrackNotFound(TrackId),
    #[error("bus {0:?} not found")]                BusNotFound(BusId),
    #[error("source {0:?} not found")]             SourceNotFound(SourceId),
    /// Draft proposal (Q18): the source is a managed path of the project folder
    /// (`renders/`, `exports/`, the database or its side files).
    #[error("source is a managed path of the project folder")] SourceInsideProject,
    #[error("edit not allowed here: {reason}")]    EditNotAllowed { reason: String },
    #[error("plugin slot {0:?} not found")]        SlotNotFound(SlotId),
    #[error("stale head: expected {expected:?}, actual {actual:?}")]
                                                   StaleHead { expected: VersionId, actual: VersionId },
    #[error("routing would create a cycle")]       RoutingCycle { node: NodeId, to: OutputTarget },
    #[error("master bus has no output")]           InvalidRoutingNode(NodeId),
    #[error("bus {0:?} still has inputs")]         BusInUse(BusId),
    #[error("track limit {limit} reached")]        TrackLimitExceeded { limit: usize },
    #[error("invalid name: {reason}")]             InvalidName { reason: String },
    #[error("sample rate {source_hz} Hz differs from session rate {session_hz} Hz")]
                                                   SampleRateMismatch { session_hz: u32, source_hz: u32 },
    #[error("unsupported sample rate {0} Hz")]     UnsupportedSampleRate(u32),
    #[error("unsupported channel count {0}")]      UnsupportedChannelCount(u16),
    #[error("intake rejected the file: {0}")]      Intake(IntakeError),      // defined by audio I/O specs
    // Field is `source_id`, not `source`: thiserror treats a field named `source` as the
    // underlying error source.
    #[error("source changed since import")]        SourceChanged { source_id: SourceId },
    #[error("source not readable: {reason}")]      SourceUnavailable { source_id: SourceId, reason: String },
    #[error("unknown plugin {0}")]                 UnknownPlugin(String),                      // (M2)
    #[error("invalid parameter {param}: {reason}")] InvalidParam { slot: Option<SlotId>, param: String,
                                                                   reason: String },           // (M2)
    #[error("plugin index {index} out of range")]  IndexOutOfRange { index: usize },
    #[error("corrupt session state: {0}")]         CorruptState(String),
    #[error(transparent)]                          Storage(#[from] StorageError),
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("path outside project folder")]        Path(#[from] PathError),
    #[error("not a project folder")]               NotAProject,
    #[error("directory not empty")]                NotEmpty,
    #[error("unsupported project format version {found} (supported up to {supported})")]
                                                   UnsupportedFormatVersion { found: u32, supported: u32 },
    #[error("database error: {0}")]                Database(String),
    #[error("i/o error: {0}")]                     Io(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("absolute path not allowed")]          Absolute,
    #[error("parent component not allowed")]       ParentComponent,
    #[error("resolves outside the project folder")] EscapesRoot,
    #[error("target is a symlink")]                Symlink,
    #[error("target is a known source file")]      IsSource,
}
```

Dependencies implied by these sketches (serde, thiserror, rusqlite, a digest crate) each need a
dependency-auditor review and an accepted ADR before the M1/M3 task that introduces them.

The rendering entry point (offline 64-bit float render of a `SessionState`) is specified in the M1
render spec; this spec fixes only the graph semantics it must follow (Behavior, "Render semantics").

## Behavior

### Entities and invariants

Every committed `SessionState` satisfies all of the following; `commit` rejects any edit list whose
result would violate one, with the error named.

1. **Exactly one master bus.** It cannot be removed, renamed or routed; it has no output target
   (`InvalidRoutingNode` if `SetOutput { node: Master, .. }`). Master output is stereo (2 channels).
2. **Tracks.** Each track references exactly one source in `sources` and has exactly one output target.
   A track has no inputs.
3. **Buses.** Each bus has exactly one output target (another bus or the master). A bus may have zero
   inputs (it then renders silence plus its chain's output; see Open questions Q6).
4. **Acyclic routing.** Following output targets from any track or bus reaches the master bus in a
   finite number of steps (`RoutingCycle`). A bus cannot output to itself.
5. **References are valid.** Every `TrackId`, `OutputTarget::Bus`, `SourceId` and `SlotId` refers to
   an existing entity (`TrackNotFound`, `BusNotFound`, `SourceNotFound`, `SlotNotFound`). `RemoveBus`
   on a bus with inputs fails with `BusInUse`. Sources are not shared: each `AddTrack` creates its own
   source entry (importing the same file twice gives two `SourceId`s), and `RemoveTrack` removes the
   track's source entry with it, so `sources` always has exactly one entry per track.
6. **Ids** are never reused after removal: `commit` and `import_audio` only increase `next_ids`.
   `restore_version` is the one exception: the restored state carries the target's counters, so ids
   allocated after a restore may repeat ids that existed only in the versions after the target
   (see Open questions Q4).
7. **Names.** Session, track and bus names are non-empty after trimming whitespace and contain no
   control characters (`InvalidName`). Draft proposal (Q16): track names are unique among tracks and
   bus names among buses within a session; `AddTrack` with `name = None` uses the source file stem,
   made unique by appending ` (2)`, ` (3)`, ... in that order.
8. **Track limit.** The number of tracks never exceeds `limits.max_tracks` of the `SessionLimits`
   passed to `apply` (or stored in `Project`) (`TrackLimitExceeded`). The value is injected, never a
   constant in `core`; production callers load it from configuration matching
   `docs/spec/release-targets.md` (task M0-13: 64 tracks).
9. **Plugin slots** *(M2)*: `params` contains exactly the parameters of the plugin's schema, each
   within the schema's range and type; float values are finite (`InvalidParam`). Plugin ids must be
   built-in plugins (`UnknownPlugin`); no external plugin loading in v1 (PLAN section 7).

### Stereo file = one-track session

`SessionState::new_empty(..)` followed by `apply(&[Edit::AddTrack { .. }], ..)` with the
`probe_source` result of a stereo file (M1), or equivalently `create_session` followed by one
`import_audio` (M3), yields, by construction and with no special-case code: one source, one track with output `Master`, zero buses, empty track and master
chains, `sample_rate_hz = Some(source rate)`. Rendering this session (M1 render) returns the decoded
source samples unchanged, bit-for-bit, as 64-bit float (null test of the graph).

### Processing format and sample rate

- All rendering of a session is offline, in 64-bit float (`f64`) samples, at the session's
  `sample_rate_hz` (PLAN section 2 principle 4, section 13).
- `sample_rate_hz` is set by the first successful `AddTrack` to the source's rate and does not
  change while the session has tracks. In this draft it returns to `None` when the last track is
  removed, so an emptied session accepts any supported rate again (Q13). Supported rates: 44 100 Hz to 192 000 Hz inclusive (PLAN section 7
  "Поддержка частот дискретизации 44,1–192 кГц", section 13 "до 192 кГц"); other rates fail with
  `UnsupportedSampleRate`.
- Importing a source whose rate differs from the session rate fails with `SampleRateMismatch` in this
  draft (see Open questions Q2). Resampling to delivery rates is a chain/delivery concern of the engine
  (PLAN section 3 stage 5), not of the session.
- Sources with 1 or 2 channels are accepted; other counts fail with `UnsupportedChannelCount`
  (Open questions Q3).

### Render semantics (constraints on the M1 render spec)

- Nodes are processed in `render_order()`. A node's input is the sample-wise sum, in `f64`, of the
  outputs of all nodes routed to it, summed in ascending `NodeId` order (fixed order makes the
  floating-point result deterministic). No implicit gain, pan law or normalization is applied by the
  graph; level and pan are plugins (Gain, Pan).
- A track's input is its decoded source, starting at frame 0 of the session timeline.
- A mono source feeding a stereo node is duplicated to both channels unchanged in this draft
  (Open questions Q3).
- Plugin latency and tail reported by plugins are compensated so that all inputs of a node are
  time-aligned (PLAN section 7 "задержка (latency) и хвост (tail) для корректной компенсации"); the
  exact algorithm is in the render spec.
- Draft proposal (Q17): before reading a source for render or analysis, core verifies that the file's
  content digest equals `SourceRef::content_digest`; on mismatch it fails with `SourceChanged`, on read failure with
  `SourceUnavailable`. It never silently renders a different file (PLAN section 3, "Тихих сбоев нет").
- Same `SessionState` + same build = bit-identical render on the same platform; across platforms only
  within the tolerances fixed by the render spec (PLAN section 2 principle 6).

### Versions and rollback

- A **version** is an immutable, committed `SessionState` plus `VersionInfo` metadata. Versions are
  numbered v1, v2, ... per session with no gaps.
- **Every change creates a version** (PLAN section 2 principle 7). The only mutating entry points are
  `create_session` (creates v1), `import_audio`, `commit` and `restore_version`; each successful call
  creates exactly one version. There is no API that mutates a committed version or the head state in
  place. This applies equally to the CLI, the MCP server and the engine (for example, `auto_master`
  commits its chain through `commit`); no caller has a private write path.
- **Atomicity.** A call either commits one complete version and advances the head, or returns an error
  and leaves storage unchanged (single SQLite transaction). A crash at any point leaves the head at
  either the old or the new version, never a partial one (PLAN section 3: a partial result is never
  presented as finished).
- **Optimistic concurrency** (draft proposal, Q14; if accepted, `expected_head` also appears in the
  M4 MCP tool schemas). Mutating calls take `expected_head`. If the head has moved, the call
  fails with `StaleHead` and commits nothing; callers re-read and retry. This prevents a concurrent
  job from silently overwriting another's change.
- **Empty edit list.** The behavior of `commit` with `edits = []` (or with edits that leave the state
  unchanged) is unspecified until Open questions Q5 is resolved; tests must not depend on it.
- **Read-only operations** (state, list, analysis, render preview, reports) never create versions.
- **Rollback** (`restore_version`) creates a new version vN+1 with `parent = vN`,
  `restored_from = Some(target)`, and a state whose `to_canonical_bytes()` equals the target's byte for
  byte; therefore its `digest` equals the target's digest and its render is bit-identical to the
  target's render (same build, same platform) (PLAN section 9 M3 acceptance). Versions after the
  target remain in history and can themselves be restored later.
- Versions are never deleted in v1 (Open questions Q7).

### Storage: what is persisted where

All writes go inside the project folder (PLAN section 6). Source audio files are read from wherever the
caller points (PLAN section 6: "Читать исходное аудио можно из любых путей") and are opened read-only.

| Data | Where | Notes |
|---|---|---|
| Project format version | SQLite | Checked on `open`; newer versions rejected with `UnsupportedFormatVersion`; upgrades only via versioned migrations. |
| Sessions (id, name, head) | SQLite | |
| Versions: canonical state bytes, digest, parent, restored_from, note, created_at | SQLite | Full state per version (not diffs) in this draft; see Q8. |
| Source metadata (`SourceRef`) | SQLite, inside the version state | The audio itself stays at the source path; not copied in this draft (Q1). |
| Reports (analysis, engine, QC), linked to session + version | SQLite | Report content is specified by engine/analysis specs. |
| Audit log of tool calls | SQLite | Written by the server layer (M4); listed here only so the storage file accounts for it. |
| Rendered previews and exports (audio files) | Files under the project folder | Names and layout are defined by render/delivery specs; every path goes through `ProjectRoot::resolve_write`. |
| Analysis cache (M9) | Files or SQLite under the project folder | Out of scope here. |

Draft layout (names provisional until M3):

```
<project>/
  aitm-project.sqlite      database (sessions, versions, reports, audit log)
  renders/                 preview renders
  exports/                 delivered files
```

### Write-path protection

`ProjectRoot` is created from the canonicalized absolute project path. `resolve_write` rejects:

- absolute inputs (`Absolute`) and any `..` component (`ParentComponent`);
- any path whose deepest existing ancestor, after resolving symlinks (and Windows junctions), lies
  outside the root (`EscapesRoot`) (PLAN section 6: "защита от выхода за её пределы (включая
  символические ссылки)");
- a final component that is an existing symlink (`Symlink`);
- a target that is a known source, i.e. referenced by any stored version (not only the head) of any
  session in the project (`IsSource`), compared both by
  canonical path and by file identity (device + inode on Unix, volume serial + file index on
  Windows) so hardlinks to a source are caught (PLAN section 6: "Исходные файлы никогда не
  изменяются и не удаляются").

Further rules:

- **No in-place writes** (audio and all other non-database files). Every such file is written to a new
  temporary file in the same target directory (created with create-new semantics, never following
  symlinks), flushed, then atomically renamed over the target. An existing such file is never opened
  for writing. **Exemption:** the SQLite database file and its `-journal`/`-wal`/`-shm` side files are
  written in place by SQLite; they stay confined by `resolve_write` (their paths are resolved through
  it before the database is opened) and their consistency relies on SQLite's atomic commit.
- **TOCTOU.** Checking a path and then opening it is racy. Implementations must re-check at open
  time: no-follow open of the parent directory where the OS allows it, create-new for the temporary
  file, and a file-identity check of the rename target against known sources just before the rename.
  The remaining race window and per-OS details are reviewed by security-reviewer in M3/M4.
- **Sources inside the project** (draft proposal, Q18). `import_audio` rejects sources whose canonical
  path, or file identity, is a managed path of the project: anything under `renders/` or `exports/`,
  the database file or its side files (`SourceInsideProject`). Other files inside the project folder
  may be imported, keeping PLAN section 6 "read from any path" except for our own outputs; they are
  then protected by `IsSource` like any other source. This keeps files the application itself
  overwrites from ever becoming sources. Users who want to master a previous export copy it out of
  the managed folders first.
- **SQLite side files.** The database's journal/WAL files (`-journal`, `-wal`, `-shm`) live next to the
  database inside the project folder. SQLite temporary storage must not use the system temp
  directory: `temp_store = MEMORY`, or a temp directory inside the project folder.
- Every file write in `core` goes through `resolve_write`, including the SQLite file and its side
  files (which are exempt only from the temp-file + rename rule). The server layer adds
  its own checks in M4; this spec does not replace them.

### Relation to "do no harm"

The session API stores and renders state; it does not deliver. It exposes no export or delivery
function, so it offers no path around the engine's "do no harm" check: delivery (export to requested
formats, post-codec checks) is an engine function (PLAN section 3 stages 7-8) and is specified there.
Preview renders written under `renders/` are not deliverables: `core` never writes them under
`exports/`, and every report or tool response that references one must mark it as a non-deliverable
preview. Enforcement in responses and reports belongs to the engine (delivery/report) and server
specs; only the engine's delivery path, after "do no harm", writes under `exports/`.

### Milestone split

- **M1** (core graph, in memory): identifiers, `SessionState`, `Track`, `Bus`, `MasterBus`,
  `SourceRef`, `probe_source`, `SessionState::new_empty`, `SessionState::apply` with injected
  `SessionLimits`, invariant validation, `to_canonical_bytes`/`from_canonical_bytes`/`digest`,
  `render_order`, render semantics. No `Project`, no SQLite. Plugin-chain edits are not available
  until M2, so all chains are empty in M1.
- **M2**: plugin-slot edits and parameter validation against plugin-sdk schemas.
- **M3** (storage): `Project`, `ProjectRoot`, SQLite persistence, versions, `restore_version`, CLI
  `session` commands.

## Reference

Not a DSP module. Independent sources of truth:

- docs/PLAN.md sections 2, 3, 6, 9 (requirements cited inline above).
- IEEE 754-2019 binary64 for sample format and the bit-exact float round-trip requirement.
- SQLite atomic commit semantics (https://www.sqlite.org/atomiccommit.html) for the atomicity rule.
- Graph acyclicity and topological order: standard DAG definitions; test-author may check
  `render_order` against an independent Kahn's-algorithm implementation in the test.

## Acceptance criteria

- [ ] `SessionState::new_empty` + `apply([AddTrack])` with `probe_source` of a stereo WAV (M1), and
      `create_session` + `import_audio` (M3), each yield exactly one track routed to `Master`, zero
      buses, empty chains, and `sample_rate_hz` equal to the file's rate.
- [ ] Rendering that one-track session with empty chains is bit-identical to the decoded source (M1).
- [ ] Every successful mutating call creates exactly one new version; every failed call leaves
      `head` and `list_versions` unchanged.
- [ ] `restore_version(target)` produces a version whose `to_canonical_bytes()` and `digest` equal the
      target's, and whose render is bit-identical to the target's render (M3, PLAN section 9 M3).
- [ ] Source files are byte-identical and have unchanged modification time after any sequence of API
      calls; no file is created outside the project folder.
- [ ] All invariants in "Entities and invariants" are enforced with the named errors; no panic on any
      input.
- [ ] For a state built by the same edit sequence, `digest()` is identical on Windows, macOS and Linux.
- [ ] The track limit is enforced only from the injected `SessionLimits`; `core` contains no default
      value for it.
- [ ] No API writes to a source file (of any stored version) or to a path outside the project folder.
- [ ] No API writes in place over an existing audio or other non-database file (temp file + atomic
      rename); the SQLite database and its side files are exempt (see "Write-path protection").
- [ ] `SourceRef` is constructible only through `probe_source` (or restored from stored state), and
      `Project::commit` rejects `Edit::AddTrack`.

## Tests

For test-author (unit tests in `crates/core/tests/`, proptest where stated). M1 tests use only
`probe_source`, `SessionState::new_empty`, `SessionState::apply` and the canonical-form methods; M3
tests use `Project`.

- Construction (M1, `new_empty` + `apply`): empty state; one-track state from `probe_source` of a
  stereo file; mono file accepted. `probe_source` rejects file properties: 3+ channel file
  (`UnsupportedChannelCount`), 32 kHz and 384 kHz files (`UnsupportedSampleRate`), non-audio and
  malformed files (`Intake`). `AddTrack` rejects session inconsistency: a second source with a
  different rate fails with `SampleRateMismatch`; after removing the last track, `sample_rate_hz` is
  `None` and a different rate is accepted.
- `SourceRef` opacity (M1): `compile_fail` doctests show that a `SourceRef` cannot be built by struct
  literal, has no `Default`, and cannot be deserialized with serde; `Edit` has no `Deserialize`.
  `apply` with `AddTrack` carrying a `SourceRef` from `probe_source` succeeds.
- AddTrack via commit (M3): `Project::commit` with an `Edit::AddTrack` (source from `probe_source`)
  fails with `EditNotAllowed` and commits nothing; `import_audio` of the same file succeeds.
- Restore under lower limits (M3, draft Q19): commit a version with n tracks, reopen the project with
  `max_tracks = n - 1`, `restore_version` to it fails with `TrackLimitExceeded` and commits nothing.
- Routing (M1): track to bus to master; bus to itself and two-bus cycle rejected (`RoutingCycle`);
  `SetOutput` on `Master` rejected; `RemoveBus` with inputs rejected; dangling `BusId` rejected
  (`BusNotFound`); unknown `TrackId` rejected (`TrackNotFound`).
- Sources (M1): importing the same file twice gives two `SourceId`s; `RemoveTrack` removes its source
  entry; `from_canonical_bytes` of a state with a track pointing at a missing source fails with
  `CorruptState`.
- proptest (M1): random edit sequences applied with `apply` never return a state that violates an
  invariant; `render_order` is a valid topological order ending in `Master` (checked against an
  independent Kahn's-algorithm implementation in the test) and is identical across repeated calls.
- Atomicity (M1): `apply` with a multi-edit list whose last edit is invalid returns an error and the
  original state is unchanged. (M3) the same through `commit` leaves head, versions and stored state
  unchanged; `StaleHead` on wrong `expected_head` (if Q14 is accepted).
- Ids (M1): ids increase and are not reused after removal (display forms only if Q15 is accepted).
- Names (M1): empty, whitespace-only, control-character names rejected; duplicate names and
  default-name dedup only if Q16 is accepted.
- Track limit (M1): with `SessionLimits { max_tracks: n }` for small n chosen by the test,
  the (n+1)-th `AddTrack` fails with `TrackLimitExceeded { limit: n }`.
- Canonical form (M1): `from_canonical_bytes(to_canonical_bytes(s)) == s`; f64 parameter values
  including -0.0, subnormals and extremes round-trip bit-exactly (M2, once params exist); states that
  differ only in 0.0 vs -0.0 are unequal; `from_canonical_bytes` rejects malformed bytes and
  non-finite floats with `CorruptState`. Cross-OS: the test builds a fixed state from a fixed edit
  sequence in code and asserts its `digest()` equals the value computed by the same test on the other
  CI OSes (e.g. the CI job compares the printed digests); no golden-bytes fixture is generated from
  the implementation.
- Versions (M3): v1..vN contiguous; `restore_version` creates vN+1 with byte-equal state and equal
  digest; restoring v1 then the old head returns to the old head's bytes; history not truncated;
  state survives `Project` close and reopen.
- Rollback render (M3): render target version, mutate, restore, render again: bit-identical buffers.
- Source protection: after import, render and restore the source file bytes and mtime are unchanged;
  modifying the source on disk after import makes render fail with `SourceChanged`; deleting it yields
  `SourceUnavailable`.
- Write paths (M3): `resolve_write` rejects absolute paths, `..`, a final-component symlink
  (`Symlink`), symlinked parent directories pointing outside (`EscapesRoot`), a path equal to a known
  source and a hardlink to a known source (`IsSource`), including a source referenced only by an
  older version (removed from the head); accepts nested relative paths.
  Draft Q18: `import_audio` of a file under `renders/` or `exports/`, of the database or its side
  files, or of a hardlink to any of them fails with `SourceInsideProject`; a file elsewhere inside
  the project folder is accepted and then protected by `IsSource`. After any successful write of an
  audio or other non-database file, the previous file at the target (if any) is replaced whole,
  never partially (no in-place writes). No file appears outside the project folder
  during M3 tests, including SQLite `-journal`/`-wal`/`-shm` and temp files (check the system temp
  directory and the project's parent directory before/after).
- Storage: `open` on a non-project folder fails with `NotAProject`; a newer format version fails with
  `UnsupportedFormatVersion`; `create` on a non-empty directory fails with `NotEmpty`.
- Errors: no API panics on any input (proptest over edits and malformed canonical bytes).

## Out of scope

- MCP tool schemas and CLI syntax (M4, M3 specs); they map onto this API but are versioned separately.
- Render algorithm details: latency compensation, tails, block processing, cross-platform tolerances
  (M1 render spec).
- Audio decoding/encoding and intake checks in detail (audio I/O specs, e.g. docs/spec/audio-io-wav.md).
- Plugin trait, parameter schemas and state migrations (plugin-sdk spec, M2).
- Sends, sidechain routing, track time offsets, clips, automation (M8 or later; require a spec update).
- Stored plans for confirmation mode (`propose_plan`/`commit_plan`), job queue, audit-log content (M4).
- Engine reports content, export/delivery and the "do no harm" check (engine specs, M6).
- SQLite DDL beyond what this model requires; crate choices for hashing and serialization (ADRs).

## Open questions

- **Q1. Copy sources into the project?** The plan says sources are never modified and may be read from
  any path, but not whether import copies them into the project folder. This draft stores a reference
  plus content digest and fails with `SourceChanged`/`SourceUnavailable` if the file changes or
  disappears, which makes old versions unrenderable if the user moves files. Copying guarantees
  rollback renders but duplicates large files. Maintainer decision needed.
- **Q2. Mixed sample rates across stems.** Should `import_audio` resample a source to the session rate
  (rubato, M1) or reject it? Draft rejects. If resampling is chosen, which rate wins (first import,
  highest rate)?
- **Q3. Channel counts.** Are mono stems in scope for v1 (draft: yes, duplicated to L/R at unity), and
  what pan law applies? Are sources with more than 2 channels rejected (draft) or downmixed?
- **Q4. Id reuse after restore.** A restore returns the id counters of the target version, so a later
  `AddBus` may reuse an id that existed only in an abandoned branch of history. Acceptable (ids are
  unique per state), or should counters be session-global and monotonic across all versions? The
  latter breaks byte-identity of the restored state unless counters are moved out of `SessionState`.
- **Q5. Empty commits.** Should `commit` with no edits (or edits that produce an identical state) be
  rejected, be a no-op without a new version, or create a version? Affects "every change creates a
  version".
- **Q6. Buses with no inputs.** Allowed (draft) or rejected? A bus with no inputs but a generating
  chain is unlikely with the plan's plugin list.
- **Q7. Version retention.** Can versions or whole sessions ever be deleted (disk use with full-state
  versions and many optimization iterations)? Draft: never in v1. Does the engine's optimization loop
  commit intermediate iterations as versions (draft: no; it works on in-memory states and commits only
  the final result, or one version per agent-visible operation)?
- **Q8. Full state vs diffs per version.** Draft stores full canonical state per version (simple,
  bit-exact restore). Revisit if state size becomes a problem.
- **Q9. Digest algorithm and canonical encoding** for `StateDigest`, `ContentDigest` and
  `to_canonical_bytes` (each requires a dependency review and ADR).
- **Q10. Project vs session.** The plan has a "project folder" and `create_session` but does not say
  whether one project holds one or many sessions. Draft: many sessions per project folder.
- **Q11. Master channel count.** The master bus is fixed to stereo output in this draft; is mono or
  multichannel delivery ever needed in v1?
- **Q12. Name length and character limits** beyond "non-empty, no control characters" (relevant for
  MCP input validation in M4).
- **Q13. Session sample rates.** (a) Files below 44.1 kHz (e.g. 32 kHz or 22.05 kHz AI-generated
  files): reject (draft, per PLAN section 7 range) or resample up at import? (b) Accept any integer
  rate in 44 100..=192 000 Hz (draft) or only the standard rates 44.1, 48, 88.2, 96, 176.4 and
  192 kHz? (c) Should `sample_rate_hz` return to `None` when the last track is removed (draft) or stay
  fixed for the session's lifetime?
- **Q14. Optimistic concurrency via `expected_head`.** Draft proposal; the plan does not mention it.
  If accepted, it becomes a field of mutating MCP tools in M4 (`import_audio`, `apply_plugin`,
  `set_param`, `route_to_bus`, `restore_version`) and of the corresponding CLI commands.
- **Q15. Id display forms** `s1`, `t1`, `b1`, `p1`, `src1`, `v1`: draft proposal for reports and MCP/CLI
  output.
- **Q16. Name uniqueness and default-name dedup** (` (2)`, ` (3)`): draft proposal; the plan does not
  require unique names.
- **Q17. Source digest check on every render/analysis read.** Draft proposal for "no silent
  failures"; hashing large sources before every render costs time. Alternatives: check size + mtime
  and hash only on mismatch, or hash once per job.
- **Q18. Sources inside the project folder.** Options: (a) reject every path inside the project
  folder; (b) reject only managed paths (`renders/`, `exports/`, the database and its side files);
  (c) allow everything and rely on `IsSource` plus temp-file + rename. Draft: (b), which keeps
  PLAN section 6 "read from any path" except for the application's own outputs.
- **Q19. Restore under changed limits.** Draft: `restore_version` fails with `TrackLimitExceeded`
  if the target exceeds the current `SessionLimits`. Alternative: allow restore (the state was valid
  when committed) and enforce limits only on growth.
