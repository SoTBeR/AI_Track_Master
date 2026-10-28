# Spec: probe MCP server over stdio (`aitm mcp serve`, `probe_echo`, `probe_elicit`)

- Status: accepted (maintainer, 2026-10-28)
- Layer: platform
- Task: docs/tasks/M0/09-mcp-probe-server.md
- Plan: docs/PLAN.md section 2 (principle 9, "Модель процессов"), section 3 ("Тихих сбоев нет"),
  section 4 (row "MCP-сервер"), section 6 ("Режимы автономности", "CLI"), section 9 M0,
  section 12 (rows "Изменения спецификации MCP", "Официальный MCP SDK для Rust..."), section 14 item 3
- Dependencies: ADR-0006 (MCP server stack: rmcp =3.5.1, tokio, serde, schemars, serde_json). ADR-0005 (clap), ADR-0003 (dependency policy),
  ADR-0001 (names).

## What

A minimal MCP server in a new library crate `crates/server` (package `aitm-server`), built on rmcp with
tokio and the stdio transport, started by `aitm mcp serve` from the single `aitm` binary. It exposes
exactly two **experimental, non-public** tools:

- `probe_echo`: returns the given message as a structured result.
- `probe_elicit`: asks the client for a one-field confirmation via MCP elicitation (form mode) and
  reports a definite outcome: `accepted`, `declined`, `cancelled`, `unsupported`, `timeout` or `error`.

The server has no access to files, network or processes. Its only purpose is to validate rmcp, stdio
and elicitation before M4 builds the real tool set (PLAN section 12: "Пробный сервер и проверка
elicitation в M0").

### Status of the probe tools (contractual)

`probe_echo` and `probe_elicit` are experimental and **not part of the public MCP schema** of
AI Track Master. They may change or be removed at any time without notice, without a versioned
schema migration and without an ADR (maintainer decision in the task Notes; AGENTS.md
"Prohibitions" covers public schemas only). Every probe tool name starts with `probe_`, and every
probe tool description states "experimental" and "may be removed without notice". Any future tool
whose name starts with `probe_` has the same status.

## Why

PLAN section 4 fixes rmcp (official Rust SDK) with tokio and stdio as the primary transport, and
says "elicitation проверяется в M0". PLAN section 6 "Режимы автономности" depends on elicitation for
`commit_plan` confirmation and needs a definite answer when a client does not support it. PLAN
section 9 M0 acceptance: one binary whose probe MCP server answers a tool call from a real client
(real clients are task M0-10; this task provides the server and an automated stdio test). PLAN
section 3: no silent failures, so every probe outcome is explicit and nothing hangs. PLAN section 12
row "Изменения спецификации MCP": pinned SDK version.

## Public API

The contract is the observable behavior of `aitm mcp serve`: the JSON-RPC messages on stdin/stdout,
stderr and the exit code. Tests are black-box (see "Tests"). The library API of `aitm-server` below is
an implementation requirement, not tested directly.

### Protocol era and versions

The M0-09 server is a **legacy-era** MCP server in the terminology of MCP 2026-07-28 ("Versioning and
Compatibility"): it requires the `initialize` handshake. It does not implement the modern
(2026-07-28) stateless mode, `server/discover`, per-request `_meta` negotiation, or Multi Round-Trip
Requests. Dual-era clients fall back to `initialize` when `server/discover` returns a non-modern error
(MCP 2026-07-28, stdio backward compatibility), so they work with this server. Clients that speak only
2026-07-28 cannot connect (accepted; see "Resolved questions" 1).

Supported protocol versions (exactly): `2025-06-18`, `2025-11-25`. Both define elicitation (form
mode), `structuredContent` and `outputSchema`.

Negotiation in the `initialize` result (`result.protocolVersion`):

| `params.protocolVersion` requested by the client | `result.protocolVersion` |
|---|---|
| `2025-06-18` | `2025-06-18` |
| `2025-11-25` | `2025-11-25` |
| anything else (e.g. `2024-11-05`, `2025-03-26`, `2026-07-28`, `1999-01-01`) | `2025-11-25` |

This follows the legacy lifecycle rule: if the server does not support the requested version it
answers with the latest version it supports, and the client decides whether to continue. The server
never puts `2026-07-28` (or any version outside the table) in any message.

### Initialize result

- `result.serverInfo.name` == `"aitm"`.
- `result.serverInfo.version` == the workspace package version (`CARGO_PKG_VERSION`; tests compare with
  `env!("CARGO_PKG_VERSION")` of the `aitm` package, which shares `version.workspace = true`).
- `result.serverInfo.title` MAY be `"AI Track Master"`; not tested.
- `result.capabilities` contains `tools` (an object; `listChanged` absent or `false`) and nothing else
  that announces a feature: no `resources`, `prompts`, `logging`, `completions`, `tasks`,
  `experimental`, `extensions`.
- `result.instructions` MAY be present; if present it contains the word `experimental`. Not tested.

### Messages before `initialize`

- The server writes nothing to stdout until it receives a request.
- `server/discover` received before `initialize` (a dual-era client's era probe) gets a JSON-RPC error
  response with the request's `id` and code `-32602` (Invalid params). rmcp 3.5.1 produces this code
  above the handler; the maintainer accepted it (ADR-0006 Decision 10). The code is outside
  `-32099..=-32020` (the range reserved for modern MCP errors), so the client classifies the server as
  legacy. The process stays alive and a following `initialize` succeeds. No transport wrapper
  intercepts this request.
- Any other request before `initialize` (except `ping`) gets a JSON-RPC error response, or is ignored;
  in both cases the process keeps running and a following `initialize` succeeds. It never exits,
  hangs or panics because of it. (Only `server/discover` is tested.)
- A **notification or response** received before `initialize` (other than EOF): rmcp 3.5.1 ends the
  session. The server then exits with code 1 and one `error:` line on stderr (reviews, 2026-10-28). That
  line is a short fixed text of at most 200 characters (including `error: mcp serve: `) and never
  contains client data (no message body, method name or id). Not tested.

### Tools (`tools/list`)

`tools/list` returns exactly these two tools, in this order (sorted by name, deterministic), and no
`nextCursor` (absent or `null`). The result is byte-for-byte identical across runs.

#### `probe_echo`

- `name`: `"probe_echo"`
- `title` (optional, not tested): `"Probe: echo"`
- `description` (exact text): `"Experimental probe tool, not part of the public AI Track Master API; may be removed without notice. Returns the given message unchanged as a structured result. Reads and writes no files."`
- `inputSchema` (contractual parts; generator-specific keys such as `$schema` or `title` may be added):

```json
{
  "type": "object",
  "properties": {
    "message": { "type": "string", "maxLength": 4096, "description": "Text to echo (at most 4096 Unicode scalar values)." }
  },
  "required": ["message"],
  "additionalProperties": false
}
```

- `outputSchema` (contractual parts):

```json
{
  "type": "object",
  "properties": {
    "message": { "type": "string" },
    "length_chars": { "type": "integer", "minimum": 0 }
  },
  "required": ["message", "length_chars"]
}
```

- `annotations`: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`,
  `openWorldHint: false`.

Successful call result:

- `isError` absent or `false`.
- `structuredContent` == `{"message": <message unchanged>, "length_chars": <number of Unicode scalar values in message>}`.
- `content` has exactly one item, `{"type": "text", "text": <JSON serialization of structuredContent>}`
  (MCP 2025-06-18 "Structured Content": a tool returning structured content SHOULD also return the
  serialized JSON in a text block). Tests parse `text` as JSON and compare with `structuredContent`.

#### `probe_elicit`

- `name`: `"probe_elicit"`
- `title` (optional, not tested): `"Probe: elicitation"`
- `description` (exact text): `"Experimental probe tool, not part of the public AI Track Master API; may be removed without notice. Asks the user to confirm a test request through MCP elicitation and reports whether the client supports elicitation and what the user answered. Changes nothing; reads and writes no files."`
- `inputSchema` (contractual parts):

```json
{
  "type": "object",
  "properties": {
    "timeout_ms": {
      "type": "integer", "minimum": 100, "maximum": 600000, "default": 120000,
      "description": "How long to wait for the client's elicitation answer, in milliseconds."
    }
  },
  "additionalProperties": false
}
```

  `arguments` may be omitted or `{}`; then `timeout_ms` = 120000.

- `outputSchema` (contractual parts):

```json
{
  "type": "object",
  "properties": {
    "outcome": { "type": "string", "enum": ["accepted", "declined", "cancelled", "unsupported", "timeout", "error"] },
    "content": {
      "type": "object",
      "properties": { "confirm": { "type": "boolean" } },
      "required": ["confirm"]
    },
    "reason": { "type": "string" }
  },
  "required": ["outcome"]
}
```

- `annotations`: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: false`,
  `openWorldHint: false`.

Every completed call returns `isError` absent or `false`, `structuredContent` as below, and `content`
with exactly one text item holding the JSON serialization of `structuredContent`. Optional fields are
omitted, never `null`.

| `outcome` | When | `content` | `reason` |
|---|---|---|---|
| `unsupported` | The client did not declare form-mode elicitation at `initialize` (see "Capability detection"). No `elicitation/create` is sent; the result is returned immediately. | absent | present, non-empty |
| `accepted` | The client answered `action: "accept"` with `content.confirm` a boolean. | `{"confirm": <bool>}` (only this key; other keys from the client are dropped) | absent |
| `declined` | The client answered `action: "decline"`. | absent | absent |
| `cancelled` | The client answered `action: "cancel"`. | absent | absent |
| `timeout` | No answer within `timeout_ms` after sending `elicitation/create`. | absent | present, non-empty, contains the timeout in ms |
| `error` | The client answered with a JSON-RPC error, or with `accept` whose `content` is missing or does not match the schema (e.g. `confirm` missing or not a boolean), or with a result that cannot be parsed as an `ElicitResult`. | absent | present, non-empty; for a JSON-RPC error it contains the error code |

`accepted` with `confirm: false` is still `accepted`: the probe reports the answer, it does not judge it.

Reason texts are not contractual (recommended: `unsupported`: "client did not declare form-mode
elicitation support at initialize"; `timeout`: "no elicitation answer within <N> ms").

##### Capability detection

Form-mode elicitation is supported if and only if the client's `initialize` `params.capabilities`
contains `elicitation` as an object that is not url-only. Unknown keys inside it are ignored (rmcp
drops them when parsing). So `{}` (equal to `{"form": {}}` by MCP 2025-11-25 backward compatibility),
`{"form": {}}`, `{"form": {}, "url": {}}` and `{"foo": {}}` are form-capable; `{"url": {}}` (with or
without unknown keys) is unsupported, and so is a missing `elicitation`. The server never sends `elicitation/create` to a client that does not
support form mode (MCP 2025-11-25 "Elicitation", Capabilities: "Servers MUST NOT send elicitation
requests with modes that are not supported by the client").

##### Elicitation request sent to the client

A JSON-RPC request from server to client, `method: "elicitation/create"`, with a fresh `id` (number
or string; the test echoes it as-is), and `params`:

```json
{
  "mode": "form",
  "message": "AI Track Master probe: please confirm that elicitation works. This is an experimental test request; nothing will be changed.",
  "requestedSchema": {
    "type": "object",
    "properties": {
      "confirm": { "type": "boolean", "title": "Confirm", "description": "Set to true to confirm.", "default": false }
    },
    "required": ["confirm"]
  }
}
```

`mode` MAY be omitted (it is absent from MCP 2025-06-18; MCP 2025-11-25 treats a missing `mode` as
`form`). Tests accept `mode` absent or equal to `"form"`. `message` is exactly the text above
(deterministic); tests assert only that it is non-empty.

##### Timeout and late answers

- The wait starts when the server sends `elicitation/create` and ends after `timeout_ms`.
- On timeout the server returns outcome `timeout` and SHOULD send
  `notifications/cancelled` with `params.requestId` = the elicitation request id. Tests tolerate but do
  not require this notification.
- An answer that arrives after the timeout is ignored (no response, no error, no crash); the server
  keeps serving.

##### Concurrency

While a `probe_elicit` call waits for the client, the server keeps answering other requests (for
example `tools/list`, `probe_echo`, `ping`). Several `probe_elicit` calls may be pending at once;
each has its own elicitation request id.

**In-flight cap (maintainer, 2026-10-28):** at most 32 `tools/call` requests are in flight at once;
a `probe_elicit` waiting for the client counts as in flight. A `tools/call` arriving while 32 are in
flight is answered immediately with a tool execution error: `isError: true`, no `structuredContent`,
`content` = one text item starting with `too many calls in flight:`. It is not queued. The server keeps
serving; other methods (`tools/list`, `ping`) are not capped, and a slot is freed as soon as an
in-flight call completes (including by elicitation answer or timeout) or is cancelled by the client.
**Client cancellation:** on `notifications/cancelled` for an in-flight `tools/call`, the server stops
that call at once (rmcp 3.5.1 cancellation is cooperative: the handler races its work against the
request's cancellation token), frees its slot, abandons a pending elicitation, and sends no response
for the cancelled request (MCP: the receiver SHOULD NOT respond to a cancelled request).

### Errors

| Situation | Response |
|---|---|
| Unknown tool name in `tools/call` (e.g. `probe_nope`, `auto_master`) | JSON-RPC error, code `-32602` (MCP 2025-11-25 "Tools", Error Handling: unknown tools are protocol errors) |
| Unknown method (e.g. `frobnicate/now`) | JSON-RPC error, code `-32601` |
| `tools/call` for a known tool with invalid `arguments` that is an object, absent or `null` (missing required field, wrong type, value out of range, string too long, unknown field; for `probe_echo` also `arguments` absent or `null`) | Tool execution error: a normal result with `isError: true`, no `structuredContent`, `content` = one text item starting with `invalid arguments:` and naming the offending field (MCP 2025-11-25 changelog, SEP-1303: input validation errors are tool execution errors so the model can self-correct). |
| `tools/call` whose `arguments` is present but neither an object nor `null` (e.g. `"x"`, `42`, `[]`) | Protocol error, known rmcp 3.5.1 limitation (maintainer, 2026-10-28): rmcp rejects the message while parsing and answers with a JSON-RPC error (observed: `-32600` without the request `id`, or `-32601` with it), never a result. The server keeps serving. No transport rewrite of client messages is done to change this; revisit on an rmcp upgrade or in M4. |
| `tools/call` beyond the in-flight cap | Tool execution error starting with `too many calls in flight:` (see "Concurrency"). |
| Line on stdin that is not valid JSON | The server SHOULD answer with a JSON-RPC error code `-32700` and `id: null`; it MAY ignore the line. Either way it keeps serving. |
| Incoming line longer than 1 048 576 bytes, excluding the line terminator (`\n`, or `\r\n` for CRLF clients: a `\r` directly before `\n` is not counted) | Fatal: the server stops, prints one `error:` line on stderr (see "Process lifecycle") and exits with code 1. It never buffers more than this limit plus a fixed small overhead for one line. |

No error path panics. Error messages are English.

Known limitation (rmcp 3.5.1, recorded 2026-10-28, not tested): a request whose `id` is neither a
string nor an integer within the i64/u64 range (e.g. `1.5`, `18446744073709551616`) gets no response.
The server keeps serving. Revisit on an rmcp upgrade.

### Process lifecycle

- **stdout** carries only JSON-RPC 2.0 messages, one per line, UTF-8, each terminated by `\n`, with no
  embedded newlines (MCP stdio transport). Nothing else is ever written to stdout: no banner, no logs.
- **stderr**: in M0-09 the server installs no logging subscriber. stderr stays empty in every session
  that ends by EOF on stdin; it carries only the final `error:` line on a runtime failure. (MCP allows
  stderr logs; structured logging is deferred, see "Out of scope".) Nothing secret is ever printed;
  the server has no secrets.
- **Shutdown by EOF**: when stdin reaches EOF (the client closed it), the server stops and the process
  exits with code 0 within 5 s, at any point: before `initialize`, after it, and while `probe_elicit`
  is waiting for the client. Pending calls are dropped without a response (the client has gone);
  in-flight handlers get at most a 1 s grace period before they are dropped.
  With stdin as `/dev/null`/`NUL` the process exits 0 at once with empty stdout and stderr.
- **Runtime failure** (cannot build the async runtime, transport read or write error other than EOF,
  incoming message over the limit): exit code 1, stderr = one line `error: mcp serve: <reason>` plus a
  line terminator (`error: <Display of CliError>`, `docs/spec/cli-entrypoint.md`). For the size limit
  the reason contains `exceeds 1048576 bytes`. Exit must happen within 5 s even if stdin is still open
  (the implementation must not wait on a blocked stdin read at shutdown; see implementation requirements).
- **Signals**: no signal handlers are installed. SIGTERM/SIGINT (Ctrl-C) and Windows console control
  events terminate the process with the OS default behavior. The server holds no files or state, so
  nothing is lost. Not tested.
- **Exit codes**: 0 (EOF, clean shutdown) and 1 (runtime failure), consistent with the CLI table in
  `docs/spec/cli-entrypoint.md`. Never 101. Usage errors of `aitm mcp serve ...` (exit 2) are unchanged.

### Security and resource limits

- The probe performs **no file I/O** (no reads, no writes, no directory listing), **no network**, and
  spawns **no processes**. Enforced three ways: (a) `crates/server` source contains no use of
  `std::fs`, `std::net`, `std::process`, `tokio::fs`, `tokio::net`, `tokio::process`, and does not
  enable rmcp features `transport-child-process`, any `transport-streamable-http-*`, `reqwest*`,
  `client` or `auth*` (security-reviewer checks); (b) the effective tokio features are those of
  ADR-0006 (rt, sync, time, macros, io-util, io-std) and exclude `fs`, `net`, `process`, `signal`; (c) a black-box test runs a session in an
  empty current directory and checks that it stays empty.
- Bounded memory per request: one incoming line at most 1 048 576 bytes (terminator excluded);
  `probe_echo` message at most 4096 Unicode scalar values; the accepted elicitation `content` is reduced
  to `{"confirm": bool}`.
- Bounded concurrency: at most 32 `tools/call` in flight, pending elicitations included (see
  "Concurrency"); calls beyond the cap are rejected immediately, not queued.
- Error lines on stderr never echo client data (see "Messages before `initialize`").
- The server reads stdin and writes stdout/stderr only. It reads no environment variables and does not
  depend on the current directory.
- Request timeouts for tools other than `probe_elicit`, per-tool limits and the audit log are M4.

### Library API of `aitm-server` (implementation requirement)

```rust
//! crates/server/src/lib.rs, package `aitm-server`

/// Maximum size of one incoming JSON-RPC line on stdin, in bytes, excluding the line terminator.
pub const MAX_MESSAGE_BYTES: usize = 1_048_576;

/// Maximum number of `tools/call` requests in flight (pending elicitations included).
pub const MAX_CALLS_IN_FLIGHT: usize = 32;

/// Grace period for in-flight handlers after end of input, before they are dropped.
pub const INPUT_END_GRACE: std::time::Duration = std::time::Duration::from_secs(1);

/// Protocol versions the probe server accepts, oldest first.
pub const SUPPORTED_PROTOCOL_VERSIONS: [&str; 2] = ["2025-06-18", "2025-11-25"];

/// Default and bounds of `probe_elicit.timeout_ms`.
pub const ELICIT_TIMEOUT_DEFAULT_MS: u64 = 120_000;
pub const ELICIT_TIMEOUT_MIN_MS: u64 = 100;
pub const ELICIT_TIMEOUT_MAX_MS: u64 = 600_000;

/// Runs the MCP server on stdin/stdout until stdin reaches EOF (`Ok(())`) or a fatal error occurs.
/// Builds and owns its own tokio runtime; the caller is synchronous.
pub fn serve_stdio() -> Result<(), ServeError>;

/// Fatal error of the stdio server.
#[non_exhaustive]
#[derive(Debug)]
pub enum ServeError {
    /// The async runtime could not be created. Display: "cannot start async runtime: {0}".
    Runtime(std::io::Error),
    /// An incoming line exceeded `MAX_MESSAGE_BYTES`. Display: "incoming message exceeds {limit_bytes} bytes".
    MessageTooLarge { limit_bytes: usize },
    /// Reading stdin or writing stdout failed (not EOF). Display: "transport error: {0}".
    Transport(String),
}
// impl std::fmt::Display and std::error::Error (plain enum; ADR-0006 adds no thiserror/anyhow).
```

Implementation requirements:

- `crates/server/Cargo.toml`: `name = "aitm-server"`, workspace version/edition/license/rust-version/
  repository, `[lints] workspace = true` (so `unsafe_code = "forbid"` applies). Add `crates/server` to
  the workspace `members`.
- Dependencies exactly as in ADR-0006: rmcp with an **exact pin**
  `rmcp = { version = "=3.5.1", default-features = false, features = ["server", "macros", "transport-io"] }`
  (PLAN section 12: pinned SDK; no `elicitation` feature), tokio 1.53.3 with `default-features = false`
  and features `rt`, `time`, `sync` (ADR-0006 amendment), serde 1.0.229 (`derive`), schemars 1.2.3; the six `Cargo.lock` pins of ADR-0006
  Decision 9. No other crate, no logging subscriber.
- Runtime: `tokio::runtime::Builder::new_current_thread()` with the time driver enabled (`enable_time()`,
  or `enable_all()`, which enables only the drivers compiled in). No I/O driver is needed: tokio's
  stdin/stdout use the blocking pool. After the service ends, the runtime is shut down with
  `Runtime::shutdown_timeout` (at most 1 s), because tokio's stdin uses a blocking read that may never
  return while stdin is open; exit must never hang on it.

Behavior that rmcp 3.5.1 does not provide by default and the implementation must add (dependency-auditor
findings, ADR-0006; no new crate for any of them):

- **Protocol versions.** Override rmcp's default supported protocol versions to exactly
  `SUPPORTED_PROTOCOL_VERSIONS` (`2025-06-18`, `2025-11-25`) and select the `initialize` result version
  per the negotiation table (ADR-0006 Decision 5). Without the override rmcp serves 2026-07-28
  per-request `_meta` requests statelessly.
- **EOF before `initialize`.** rmcp reports a closed connection during initialization as an error; the
  implementation maps that "connection closed" init error to `Ok(())`, so the process exits 0.
- **1 MiB line limit.** rmcp's stdio path is unbounded. Wrap stdin in a counting `AsyncRead` that tracks
  bytes since the last `\n` and, when a line exceeds `MAX_MESSAGE_BYTES`, records the condition and
  fails the read, so an oversized line is never fully buffered. The terminator is not counted: neither
  the `\n` nor a `\r` immediately before it (CRLF clients). `serve_stdio` then returns
  `ServeError::MessageTooLarge`. The wrapper only counts and fails; it never rewrites client bytes.
- **Read errors are not EOF.** A stdin read error (other than the size limit) is recorded in the same
  wrapper and turned into `ServeError::Transport` (exit 1), even if rmcp itself treats it like a closed
  connection. Only a real EOF yields `Ok(())`.
- **Grace after end of input.** rmcp 3.5.1 waits up to 5 s for in-flight handlers after the input ends.
  The server caps this wait at `INPUT_END_GRACE` (1 s) so that "exit within 5 s" holds; in-flight
  calls still running then are dropped without a response (as in "Shutdown by EOF").
- **Early notification or response.** When rmcp ends the session because a notification or response
  arrived before `initialize`, `serve_stdio` returns an error whose Display is a fixed short text (the
  whole stderr line at most 200 characters) without any client data.
- **In-flight cap.** Enforce `MAX_CALLS_IN_FLIGHT` for `tools/call` (a counter or semaphore acquired
  without waiting; a call that cannot acquire a slot gets the `too many calls in flight:` result at
  once). rmcp 3.5.1 has no such limit.
- **No message rewriting.** Client messages are passed to rmcp unchanged; rmcp limitations on
  non-object `arguments` and non-standard ids are accepted as recorded in "Errors".
- **Elicitation capability.** Detect it from the client's `initialize` params as in "Capability
  detection": an `elicitation` object is form-capable unless it is url-only; unknown keys are ignored.
- **Elicitation request.** Send `elicitation/create` with `Peer::send_request(ServerRequest::ElicitRequest(..))`
  and read `ClientResult::ElicitResult`, with the timeout applied through the request options
  (`PeerRequestOptions`); do not use the `elicitation`-feature-gated helpers (`create_elicitation`,
  `elicit_url`), which are not enabled (ADR-0006 Decision 4).
- **Argument validation.** Invalid arguments of a known tool produce `isError: true` results as in
  "Errors". If rmcp 3.5.1's typed parameter extraction answers schema-invalid arguments with JSON-RPC
  `-32602`, the implementation validates the raw `arguments` itself (bounds, types, unknown fields,
  missing fields) to produce the tool execution error. This is a requirement, not an option.
- **`server/discover` before `initialize`.** Left to rmcp (answered with `-32602`, ADR-0006
  Decision 10); no interception.
- **Fallback (task Notes):** if the pinned rmcp cannot send `elicitation/create` and receive its result
  on stdio, or cannot meet another requirement of this spec, the implementer stops and reports to the
  maintainer. No workaround is improvised (no hand-written MCP layer); the fallback of PLAN section 12
  (thin Python layer) is a maintainer decision recorded in M0-10.

### CLI wiring (same platform layer, allowed in this task)

`crates/cli` and `crates/server` are both platform layer, so this one task may change both. The CLI
change is limited to:

- `crates/cli/Cargo.toml`: `aitm-server = { path = "../server", version = "0.1.0" }`
  (versioned path dependency, ADR-0003 wildcard rule, ADR-0006 Decision 8); `[dev-dependencies]`
  `serde_json = "1.0.154"` for the tests (ADR-0006 Decision 7).
- `commands::mcp::serve()` body becomes a single call: `aitm_server::serve_stdio().map_err(CliError::Server)`.
- `CliError` gains `Server(aitm_server::ServeError)` with Display `"mcp serve: {0}"` and exit code 1;
  `NotAvailable` is removed (no longer used; it is an internal type, not CLI surface).
- `aitm mcp serve --help` text may describe the server (e.g. "Start the MCP server on stdio"); wording
  is not contractual. No new arguments or options.
- Nothing else in the CLI changes (command tree, other outputs, exit codes).

### Changes to the M0-08 contract

This spec supersedes these parts of `docs/spec/cli-entrypoint.md`:

- Row "`aitm mcp serve` (until M0-09) | 1 | empty | error: ... not available yet": replaced by "Process
  lifecycle" above (with stdin `/dev/null`: exit 0, empty stdout and stderr).
- "Behavior", "Stdin": `aitm mcp serve` now reads stdin and blocks until EOF; other commands are
  unchanged.
- Tests 6 and 7 of that spec are replaced by tests 1 and 2 below (test-author edits
  `crates/cli/tests/cli.rs` accordingly); all other M0-08 tests stay unchanged.

## Behavior

Covered by "Public API". Summary of invariants:

- Determinism: same requests produce the same responses (same tool list, schemas, texts, outcomes);
  only JSON-RPC ids chosen by the server for its own requests may differ.
- The server only sends to the client: responses, `elicitation/create` requests (only from
  `probe_elicit`, only to form-capable clients), and optionally `notifications/cancelled` after an
  elicitation timeout. No `roots/list`, `sampling/createMessage`, `ping`, logging or list-changed
  notifications.
- `ping` after `initialize` returns an empty result `{}`.

## Reference

- MCP specification 2025-11-25: Lifecycle (version negotiation), Transports (stdio), Tools (structured
  content, output schema, error handling), Client/Elicitation (capabilities, form mode, actions),
  changelog item SEP-1303. https://modelcontextprotocol.io/specification/2025-11-25 (accessed 2026-10-28).
- MCP specification 2025-06-18 (elicitation introduced, `structuredContent`, `outputSchema`).
  https://modelcontextprotocol.io/specification/2025-06-18 (accessed 2026-10-28).
- MCP specification 2026-07-28: changelog (stateless protocol, `server/discover`, MRTR, error code
  ranges) and "Versioning and Compatibility" (legacy/dual-era behavior, stdio era probe).
  https://modelcontextprotocol.io/specification/2026-07-28/changelog,
  https://modelcontextprotocol.io/specification/2026-07-28/basic/lifecycle (accessed 2026-10-28).
- JSON-RPC 2.0 specification (error codes -32700, -32601, -32602). https://www.jsonrpc.org/specification
- rmcp: https://crates.io/crates/rmcp, https://github.com/modelcontextprotocol/rust-sdk
  (`examples/servers/src/elicitation_stdio.rs`; `rmcp::model::ProtocolVersion::LATEST_WITH_INITIALIZE`
  = 2025-11-25 in 3.8.0), accessed 2026-10-28.
- `docs/spec/cli-entrypoint.md` (exit codes, error line format).

## Acceptance criteria

- [ ] `crates/server` (package `aitm-server`) exists; it uses rmcp with tokio and the stdio transport;
      rmcp is pinned with `=3.5.1` and all dependencies, features and lock pins match ADR-0006.
- [ ] `aitm mcp serve` runs the server from the single `aitm` binary; the CLI change is limited to
      "CLI wiring".
- [ ] All tests in "Tests" pass on Linux, Windows and macOS in CI; each awaited message arrives within
      10 s and each process exits within 10 s.
- [ ] `probe_elicit` reports `unsupported`, `accepted`, `declined`, `cancelled`, `timeout` and `error`
      as specified; no `probe_elicit` call hangs or ends without a response while stdin is open.
- [ ] At most 32 `tools/call` in flight; calls beyond the cap get `too many calls in flight:` at once
      (test 27).
- [ ] Tool names start with `probe_`; descriptions contain `experimental` and
      `may be removed without notice`; this spec states the non-public status.
- [ ] No file I/O, network or process use in `crates/server` (security-reviewer confirms; test 23).
- [ ] No exit code 101 and no `panicked` on stderr in any test.
- [ ] `CHANGELOG.md` "Unreleased" lists `aitm mcp serve` (stdio MCP server) and the experimental probe
      tools, marked as experimental and non-public.

## Tests

Black-box integration tests in `crates/cli/tests/mcp_probe.rs` (the `aitm` package, because
`env!("CARGO_BIN_EXE_aitm")` exists only there). They spawn `env!("CARGO_BIN_EXE_aitm")` with args
`mcp serve`, `stdin`/`stdout`/`stderr` piped, env `NO_COLOR=1`, and speak newline-delimited JSON-RPC
with **std + `serde_json` only** (dev-dependency).

Decision: tests do **not** use rmcp's client. Raw JSON-RPC keeps the tests independent of the
implementation's SDK (a shared SDK would hide shared bugs) and lets the test play clients that rmcp's
client would not imitate: no elicitation capability, URL-only elicitation, never answering, answering
late, answering with an error or invalid content, sending malformed or oversized lines.

Harness requirements (test-author designs the helper):

- A reader thread reads stdout line by line into a channel; each await uses `recv_timeout(10 s)`.
- Every stdout line (after removing one trailing `\r` for robustness) must parse as a JSON object with
  `"jsonrpc": "2.0"`; anything else fails the test.
- While waiting for a response, the helper collects server requests and notifications. Any server
  request other than `elicitation/create`, or any notification other than `notifications/cancelled`,
  fails the test.
- Standard initialize request (fields varied per test):
  `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"aitm-test-client","version":"0.0.0"}}}`,
  then the notification `{"jsonrpc":"2.0","method":"notifications/initialized"}`.
- Closing the session = dropping the child's stdin; then wait for exit (poll `try_wait` up to 10 s; on
  timeout kill the child and fail).
- Panic guard on every process: exit code never 101, stderr never contains `panicked`.
- Unless the test says otherwise, after EOF: exit code 0 and stderr empty.

Test list:

1. **stdin null**: `stdin(Stdio::null())`: exit 0 within 10 s, stdout empty, stderr empty.
   (Replaces `cli-entrypoint` test 6.)
2. **Silence before initialize, EOF exit**: stdin piped and held open 1 s without writing; no stdout
   bytes arrive; drop stdin; exit 0 within 10 s; stdout empty. (Replaces `cli-entrypoint` test 7.)
3. **Handshake 2025-11-25**: initialize → `result.protocolVersion == "2025-11-25"`,
   `serverInfo.name == "aitm"`, `serverInfo.version == env!("CARGO_PKG_VERSION")`,
   `capabilities.tools` is an object, `capabilities` has none of `resources`, `prompts`, `logging`,
   `completions`. After `notifications/initialized`, `ping` returns `result == {}`.
4. **Version negotiation** (fresh process each): requested `2025-06-18` → `2025-06-18`;
   `2024-11-05`, `2025-03-26`, `2026-07-28`, `1999-01-01` → `2025-11-25`.
5. **tools/list**: tool names exactly `["probe_echo", "probe_elicit"]` in this order; each `description`
   contains `experimental` (case-insensitive) and `may be removed without notice`; `inputSchema.type ==
   "object"` and `additionalProperties == false`; `probe_echo` input has `message` of type `string` with
   `maxLength` 4096 and `required` contains `message`; `probe_elicit` input has `timeout_ms` of type
   `integer`, `minimum` 100, `maximum` 600000; `outputSchema.type == "object"` with property keys
   `message`, `length_chars` (echo) and `outcome`, `content`, `reason` (elicit), `required` containing
   `outcome`; `annotations.readOnlyHint == true` and `annotations.openWorldHint == false` for both;
   `nextCursor` absent or null.
6. **Determinism**: the `tools/list` result from two separate processes is equal (`serde_json::Value`).
7. **probe_echo success**: messages `"héllo 🎚"` (expect `length_chars` 7), `""` (0) and a string of
   4096 × `"é"` (4096): `isError` absent or false; `structuredContent` == expected object;
   `content` has one text item whose `text` parses to the same JSON as `structuredContent`.
8. **probe_echo invalid arguments**.
   (a) Each of these gets a result with `isError == true`, no `structuredContent`, one text item
   starting with `invalid arguments:`: `{}` (missing), `{"message": 42}`, 4097 × `"a"`,
   `{"message": "x", "extra": 1}`, `arguments` key absent, `"arguments": null`.
   (b) Non-object `arguments` (`"arguments": "x"`, then in the same session `"arguments": 42`): the
   test does not require a response to that call. Any message the server sends in reaction to it is a
   JSON-RPC error (has `error`, no `result`). A following `tools/list` (new id) is answered with the
   correct two tools, and a following valid `probe_echo` call succeeds.
9. **Unknown tool**: `probe_nope` and `auto_master` → JSON-RPC error code `-32602`; a following
   `tools/list` is answered.
10. **Unknown method**: `frobnicate/now` → JSON-RPC error code `-32601`.
11. **Elicit, no capability**: capabilities `{}` (2025-11-25), `probe_elicit` with `{}` → `outcome ==
    "unsupported"`, non-empty `reason`, no `content`; no `elicitation/create` was sent; the response
    arrives within 10 s (far below the 120 s default, proving no wait). Repeat with protocol
    `2025-06-18`.
12. **Elicit, URL-only capability**: capabilities `{"elicitation": {"url": {}}}` → `unsupported`, no
    `elicitation/create` sent.
13. **Elicit, accept**: capabilities `{"elicitation": {"form": {}}}` (2025-11-25) and, in a second run,
    `{"elicitation": {}}` (2025-06-18). The test receives `elicitation/create`, checks `params.message`
    non-empty, `params.mode` absent or `"form"`, `params.requestedSchema.type == "object"`,
    `properties.confirm.type == "boolean"`, `required` contains `confirm`; it answers with the same `id`
    and `{"action": "accept", "content": {"confirm": true}}` → `outcome == "accepted"`,
    `content == {"confirm": true}`, no `reason`. Variant: `confirm: false` → `accepted` with
    `{"confirm": false}`. Variant: content `{"confirm": true, "x": 1}` → `content == {"confirm": true}`.
14. **Elicit, decline**: answer `{"action": "decline"}` → `declined`, no `content`, no `reason`.
15. **Elicit, cancel**: answer `{"action": "cancel"}` → `cancelled`, no `content`, no `reason`.
16. **Elicit, invalid accept**: answers `{"action": "accept", "content": {"confirm": "yes"}}`,
    `{"action": "accept"}` and `{"action": "maybe"}` (each in its own call) → `error` with non-empty
    `reason`.
17. **Elicit, client error**: answer with JSON-RPC error `{"code": -32603, "message": "boom"}` →
    `error`, `reason` contains `-32603`.
18. **Elicit, timeout and late answer**: `{"timeout_ms": 500}`; receive `elicitation/create`, do not
    answer; the result arrives within 10 s with `outcome == "timeout"` and non-empty `reason`; time
    from sending `tools/call` to the result is at least 500 ms. An optional `notifications/cancelled`
    is tolerated. Then send the late answer for that `id`; a following `tools/list` is answered and no
    other message appears.
19. **Elicit, argument bounds**: `timeout_ms` 99, 600001, -1, `"abc"`, and `{"extra": 1}` → tool
    execution error (`isError == true`, text starts with `invalid arguments:`). With a client without
    elicitation capability, `timeout_ms` 100 and 600000 are valid → `unsupported`.
20. **Concurrency**: form-capable client; call `probe_elicit`; after receiving `elicitation/create`
    (not yet answered) call `probe_echo` → its result arrives; then answer accept → `accepted`.
21. **EOF while elicitation pending**: after receiving `elicitation/create`, drop stdin → exit 0 within
    10 s.
22. **Malformed line**: send `{not json` + `\n`; if a response arrives for it, it is an error with
    code `-32700` and `id: null`; a following `tools/list` is answered; exit 0 on EOF.
23. **No file side effects**: create a fresh empty directory under `std::env::temp_dir()` (unique name
    from pid and time), run tests 3, 5, 7 and 11 in one session with that `current_dir`, close; the
    directory is still empty; remove it.
24. **Era probe before initialize**: first message
    `{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}`
    → error response with `id` 1 and code `-32602` (ADR-0006 Decision 10); the process is still
    running, and a following initialize (id 2) succeeds with `2025-11-25`.
25. **Size limit, boundary**: a `tools/list` request padded through `params._meta.pad` (a string) to
    exactly 1 048 576 bytes excluding `\n` is answered normally.
26. **Size limit, over**: a padded line of 1 048 577 bytes plus `\n`, stdin kept open afterwards
    (write errors from the test side are ignored: the server may exit while the test is writing) →
    exit code 1 within 10 s; stderr starts with `error:` and contains `exceeds 1048576 bytes`; every
    stdout line is valid JSON-RPC.
27. **In-flight cap**: form-capable client (2025-11-25). Send 32 `probe_elicit` calls with
    `{"timeout_ms": 600000}` and collect the 32 `elicitation/create` requests without answering. Then
    send a 33rd `tools/call` (`probe_echo`, `{"message": "x"}`): its result arrives within 10 s with
    `isError == true`, no `structuredContent`, one text item starting with `too many calls in flight:`.
    A following `tools/list` is answered correctly. Answer all 32 elicitations with
    `{"action": "decline"}`: each `probe_elicit` returns `declined`. A new `probe_echo` call then
    succeeds (`isError` absent or false).
28. **CRLF line terminator at the size limit**: the padded `tools/list` request of test 25 (exactly
    1 048 576 bytes) sent with `\r\n` instead of `\n` is answered normally.
29. **Cancellation frees the slot**: form-capable client (2025-11-25). Send 32 `probe_elicit` calls
    with `{"timeout_ms": 600000}` and collect the 32 `elicitation/create` requests without answering.
    Send `notifications/cancelled` for all 32 call ids. Then, within 10 s, 32 new `probe_elicit` calls
    each produce an `elicitation/create` request (none gets `too many calls in flight:`); no message
    with a `result` for any cancelled id is ever received. Answer the new 32 with `decline`; each
    returns `outcome == "declined"`.

No test asserts on reason wording beyond the listed substrings, on server-chosen request ids, or on
message timing other than the bounds above.

## Out of scope

- The real tool set, resources and prompts (PLAN section 6), job queue and async jobs, autonomy modes
  and `commit_plan` confirmation, write-path protection, audit log, per-tool limits and concurrency
  caps: M4.
- Modern-era MCP (2026-07-28): stateless requests, `server/discover` results, Multi Round-Trip
  Requests elicitation (see "Resolved questions" 1).
- Streamable HTTP transport (optional per PLAN section 4); URL-mode elicitation.
- Structured logging to stderr and a log-level option: deferred to M4.
- Signal handling beyond OS defaults.
- Testing in real MCP clients and recording which clients support elicitation: task M0-10 (maintainer).
- `aitm mcp install`, `doctor` and other CLI commands.

## Resolved questions

Resolved by the maintainer on 2026-10-28:

1. **Protocol era.** The probe is handshake-only (legacy era) with versions `2025-06-18` and
   `2025-11-25`. Any other requested version, including `2024-11-05` and `2025-03-26`, is answered with
   `2025-11-25`. Clients that speak only 2026-07-28 cannot connect; 2026-07-28 support (stateless
   requests, Multi Round-Trip Requests elicitation) is revisited after M0-10.
2. **`server/discover` before `initialize`** returns JSON-RPC error `-32602`, as rmcp 3.5.1 produces it
   (ADR-0006 Decision 10); `-32601` is not required and no transport wrapper is added. Test 24 asserts
   `-32602` and that a following `initialize` succeeds.
3. **Oversized input.** A line over 1 MiB is fatal: exit 1 with the `error:` line, as specified in
   "Process lifecycle".
4. **Elicitation timeout.** The default of 120 s is accepted.
5. **Argument errors** are `isError: true` tool execution errors per MCP 2025-11-25 (SEP-1303). If rmcp
   3.5.1 answers schema-invalid arguments with `-32602`, the implementation validates arguments itself
   (requirement in "Implementation requirements").
6. **M0-08 contract.** `docs/spec/cli-entrypoint.md` is superseded for `aitm mcp serve` (its row in
   "Exact outputs", the "Stdin" paragraph and tests 6-7), with a dated note in that spec; see
   "Changes to the M0-08 contract".
7. **No transport rewrite of client messages.** A `tools/call` whose `arguments` is present but
   neither an object nor `null` is a protocol error left to rmcp 3.5.1 (`-32600` without the `id`; that
   call gets no response of its own); the server keeps serving. Recorded as a known rmcp limitation,
   revisited on an rmcp upgrade or in M4 (test 8b).
8. **In-flight cap.** At most 32 `tools/call` in flight, pending elicitations included; a call beyond
   the cap gets an immediate `isError` result starting with `too many calls in flight:` (test 27).
