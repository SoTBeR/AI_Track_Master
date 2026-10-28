//! Black-box tests of the probe MCP server started by `aitm mcp serve` (task M0-09).
//!
//! Contract: `docs/spec/mcp-probe.md` ("Public API", "Tests"); the numbers in the section comments
//! below are the numbered tests of that spec.
//!
//! The tests play an MCP client by hand: they spawn `env!("CARGO_BIN_EXE_aitm") mcp serve` with
//! piped stdin/stdout/stderr and speak newline-delimited JSON-RPC 2.0 with std and `serde_json`
//! only (spec "Tests": no rmcp client, so the tests share no SDK with the implementation).
//!
//! Harness rules (spec "Harness requirements"):
//! - a reader thread turns stdout into a channel of messages; every await is bounded by
//!   `recv_timeout(AWAIT)`; every process must exit within `AWAIT` after EOF or it is killed;
//! - every stdout line (one trailing `\r` tolerated) must be a JSON object with `"jsonrpc": "2.0"`;
//! - any server request other than `elicitation/create` and any notification other than
//!   `notifications/cancelled` fails the test;
//! - panic guard on every process: exit code never 101, stderr never contains `panicked`;
//! - unless a test says otherwise, after EOF: exit code 0 and empty stderr.
//!
//! Nothing depends on path separators, line terminators of the host OS, colors or timing beyond
//! the bounds in the spec.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

/// Path of the built `aitm` binary, provided by Cargo for integration tests.
const AITM_BIN: &str = env!("CARGO_BIN_EXE_aitm");

/// Upper bound for every awaited message and for every process exit (spec "Acceptance criteria").
const AWAIT: Duration = Duration::from_secs(10);

/// Maximum incoming line size in bytes, excluding the terminator (spec "Errors").
const MAX_MESSAGE_BYTES: usize = 1_048_576;

/// Protocol version the server answers for anything it does not support (spec "Protocol era").
const LATEST: &str = "2025-11-25";

/// Text every invalid-arguments tool execution error starts with (spec "Errors").
const INVALID_ARGUMENTS_PREFIX: &str = "invalid arguments:";

/// Capabilities of a client that declares form-mode elicitation.
fn form_capable() -> Value {
    json!({"elicitation": {"form": {}}})
}

/// Capabilities of a client without elicitation.
fn no_capabilities() -> Value {
    json!({})
}

// =============================================================================================
// Harness
// =============================================================================================

/// One item produced by the stdout reader thread.
enum Line {
    /// A line that parsed as a JSON-RPC 2.0 object.
    Message(Value),
    /// A line that is not a JSON-RPC 2.0 object (or a final chunk without `\n`).
    Invalid(String),
    /// stdout reached EOF.
    Eof,
}

/// Summary of a finished server process.
struct Finished {
    code: Option<i32>,
    /// Messages received after the last awaited one, plus buffered unmatched responses.
    leftovers: Vec<Value>,
    stderr: String,
}

/// A running `aitm mcp serve` process with a JSON-RPC client harness around it.
struct Server {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    lines: Receiver<Line>,
    stderr_thread: Option<JoinHandle<Vec<u8>>>,
    /// `elicitation/create` requests received but not yet consumed by the test.
    server_requests: VecDeque<Value>,
    /// Responses received while waiting for a different id.
    unmatched: Vec<Value>,
    /// `notifications/cancelled` received from the server.
    cancelled: Vec<Value>,
    /// Next client request id (1 is reserved for `initialize`).
    next_id: i64,
    stdout_closed: bool,
}

fn truncate_for_message(s: &str) -> String {
    const LIMIT: usize = 400;
    if s.len() <= LIMIT {
        s.to_owned()
    } else {
        let mut end = LIMIT;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}... ({} bytes)", &s[..end], s.len())
    }
}

/// Parses one stdout line according to the harness rules.
fn parse_line(raw: &[u8]) -> Line {
    let Ok(text) = std::str::from_utf8(raw) else {
        return Line::Invalid(format!("non-UTF-8 stdout line: {raw:?}"));
    };
    let text = text.strip_suffix('\r').unwrap_or(text);
    match serde_json::from_str::<Value>(text) {
        Ok(value) if value.is_object() && value.get("jsonrpc") == Some(&Value::from("2.0")) => {
            Line::Message(value)
        }
        _ => Line::Invalid(truncate_for_message(text)),
    }
}

fn spawn_stdout_reader(stdout: impl Read + Send + 'static) -> Receiver<Line> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut buf = Vec::new();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) => {
                    let _ = tx.send(Line::Eof);
                    return;
                }
                Ok(_) => {
                    let line = match buf.strip_suffix(b"\n") {
                        Some(content) => parse_line(content),
                        None => Line::Invalid(format!(
                            "stdout ended with an unterminated line: {}",
                            truncate_for_message(&String::from_utf8_lossy(&buf))
                        )),
                    };
                    if tx.send(line).is_err() {
                        return;
                    }
                }
                Err(err) => {
                    let _ = tx.send(Line::Invalid(format!("stdout read error: {err}")));
                    let _ = tx.send(Line::Eof);
                    return;
                }
            }
        }
    });
    rx
}

fn spawn_stderr_reader(mut stderr: impl Read + Send + 'static) -> JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        buf
    })
}

/// Kind of an incoming JSON-RPC message.
enum Kind {
    Request,
    Notification,
    Response,
}

fn kind_of(msg: &Value) -> Kind {
    let has_method = msg.get("method").is_some();
    let has_id = msg.get("id").is_some();
    match (has_method, has_id) {
        (true, true) => Kind::Request,
        (true, false) => Kind::Notification,
        (false, _) => {
            let has_result = msg.get("result").is_some();
            let has_error = msg.get("error").is_some();
            assert!(
                has_result ^ has_error,
                "response must have exactly one of result/error: {msg}"
            );
            // Spec "Errors": a `tools/call` with non-object `arguments` is answered by rmcp with
            // `-32600` without the request id, so only error responses may lack an id.
            assert!(has_id || has_error, "result response without id: {msg}");
            Kind::Response
        }
    }
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}-{nanos}", std::process::id())
}

impl Server {
    /// Starts `aitm mcp serve` with piped stdin.
    fn start() -> Self {
        Self::start_with(Stdio::piped(), None)
    }

    fn start_with(stdin: Stdio, current_dir: Option<&Path>) -> Self {
        let mut cmd = Command::new(AITM_BIN);
        cmd.args(["mcp", "serve"])
            .env("NO_COLOR", "1")
            .stdin(stdin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = current_dir {
            cmd.current_dir(dir);
        }
        let mut child = cmd.spawn().expect("failed to spawn `aitm mcp serve`");
        let stdin = child.stdin.take();
        let lines = spawn_stdout_reader(child.stdout.take().expect("stdout is piped"));
        let stderr_thread = Some(spawn_stderr_reader(
            child.stderr.take().expect("stderr is piped"),
        ));
        Self {
            child: Some(child),
            stdin,
            lines,
            stderr_thread,
            server_requests: VecDeque::new(),
            unmatched: Vec::new(),
            cancelled: Vec::new(),
            next_id: 2,
            stdout_closed: false,
        }
    }

    /// Writes raw bytes to the server's stdin. Small writes only (they fit in the pipe buffer);
    /// large writes go through [`Server::send_large_line`].
    fn send_raw(&mut self, bytes: &[u8]) {
        let stdin = self.stdin.as_mut().expect("stdin is open");
        if let Err(err) = stdin.write_all(bytes).and_then(|()| stdin.flush()) {
            let diagnosis = self.diagnose();
            panic!("write to server stdin failed: {err}; {diagnosis}");
        }
    }

    /// Describes how the server ended after an unexpected failure: waits (bounded) for the exit,
    /// then reports the exit code and stderr. Used only on the failure path.
    fn diagnose(&mut self) -> String {
        drop(self.stdin.take());
        let code = match self.child.as_mut() {
            None => "already reaped".to_owned(),
            Some(child) => {
                let start = Instant::now();
                loop {
                    match child.try_wait() {
                        Ok(Some(status)) => break format!("{:?}", status.code()),
                        Ok(None) if start.elapsed() < AWAIT => {
                            thread::sleep(Duration::from_millis(10));
                        }
                        _ => {
                            let _ = child.kill();
                            let _ = child.wait();
                            break "still running after the wait; killed".to_owned();
                        }
                    }
                }
            }
        };
        self.child = None;
        let stderr = self
            .stderr_thread
            .take()
            .and_then(|t| t.join().ok())
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
        format!("server exit code: {code}; stderr: {stderr:?}")
    }

    /// Sends one JSON-RPC message terminated by `\n`.
    fn send(&mut self, msg: &Value) {
        let mut line = serde_json::to_vec(msg).expect("serialize message");
        line.push(b'\n');
        self.send_raw(&line);
    }

    /// Writes a large line from a helper thread so a server that stops reading cannot block the
    /// test forever; the write must complete within `AWAIT`.
    fn send_large_line(&mut self, line: Vec<u8>) {
        let mut stdin = self.stdin.take().expect("stdin is open");
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = stdin.write_all(&line).and_then(|()| stdin.flush());
            let _ = tx.send((result, stdin));
        });
        match rx.recv_timeout(AWAIT) {
            Ok((Ok(()), stdin)) => self.stdin = Some(stdin),
            Ok((Err(err), _)) => panic!("writing a large line to the server failed: {err}"),
            Err(_) => panic!("writing a large line did not finish within {AWAIT:?}"),
        }
    }

    fn fresh_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Receives the next message; fails on timeout, EOF or an invalid line.
    fn next_message(&mut self, waiting_for: &str) -> Value {
        assert!(
            !self.stdout_closed,
            "stdout already closed while waiting for {waiting_for}"
        );
        match self.lines.recv_timeout(AWAIT) {
            Ok(Line::Message(msg)) => msg,
            Ok(Line::Invalid(text)) => {
                panic!("stdout carried a non-JSON-RPC line while waiting for {waiting_for}: {text}")
            }
            Ok(Line::Eof) => {
                self.stdout_closed = true;
                let diagnosis = self.diagnose();
                panic!("server closed stdout while waiting for {waiting_for}; {diagnosis}")
            }
            Err(RecvTimeoutError::Timeout) => {
                panic!("no message within {AWAIT:?} while waiting for {waiting_for}")
            }
            Err(RecvTimeoutError::Disconnected) => {
                panic!("stdout reader stopped while waiting for {waiting_for}")
            }
        }
    }

    /// Routes a server request or notification; returns the message back if it is a response.
    fn route(&mut self, msg: Value) -> Option<Value> {
        match kind_of(&msg) {
            Kind::Request => {
                let method = msg["method"].as_str().unwrap_or_default();
                assert_eq!(
                    method, "elicitation/create",
                    "unexpected server request: {msg}"
                );
                self.server_requests.push_back(msg);
                None
            }
            Kind::Notification => {
                let method = msg["method"].as_str().unwrap_or_default();
                assert_eq!(
                    method, "notifications/cancelled",
                    "unexpected server notification: {msg}"
                );
                self.cancelled.push(msg);
                None
            }
            Kind::Response => Some(msg),
        }
    }

    /// Waits for the response with the given id, collecting everything else.
    fn wait_response(&mut self, id: &Value) -> Value {
        if let Some(pos) = self.unmatched.iter().position(|m| &m["id"] == id) {
            return self.unmatched.remove(pos);
        }
        let what = format!("the response to id {id}");
        loop {
            let msg = self.next_message(&what);
            if let Some(resp) = self.route(msg) {
                if &resp["id"] == id {
                    return resp;
                }
                self.unmatched.push(resp);
            }
        }
    }

    /// Waits for the next `elicitation/create` request from the server.
    fn wait_elicitation(&mut self) -> Value {
        loop {
            if let Some(req) = self.server_requests.pop_front() {
                return req;
            }
            let msg = self.next_message("an elicitation/create request");
            if let Some(resp) = self.route(msg) {
                self.unmatched.push(resp);
            }
        }
    }

    /// Sends a request and returns its id.
    fn send_request(&mut self, method: &str, params: Option<Value>) -> Value {
        let id = Value::from(self.fresh_id());
        let mut msg = json!({"jsonrpc": "2.0", "id": id, "method": method});
        if let Some(params) = params {
            msg["params"] = params;
        }
        self.send(&msg);
        id
    }

    /// Sends a request and waits for its response.
    fn request(&mut self, method: &str, params: Option<Value>) -> Value {
        let id = self.send_request(method, params);
        self.wait_response(&id)
    }

    /// Sends the standard initialize request (id 1) and returns the full response.
    fn initialize_raw(&mut self, protocol: &str, capabilities: Value) -> Value {
        let id = Value::from(1);
        self.send(&initialize_request(&id, protocol, capabilities));
        self.wait_response(&id)
    }

    /// Initializes the session, sends `notifications/initialized`, returns `result`.
    fn initialize(&mut self, protocol: &str, capabilities: Value) -> Value {
        let resp = self.initialize_raw(protocol, capabilities);
        let result = result_of(&resp).clone();
        self.send_initialized();
        result
    }

    fn send_initialized(&mut self) {
        self.send(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    }

    /// `tools/call` with the given arguments (`None` omits the `arguments` key).
    fn call_tool(&mut self, name: &str, arguments: Option<Value>) -> Value {
        let mut params = json!({"name": name});
        if let Some(arguments) = arguments {
            params["arguments"] = arguments;
        }
        self.request("tools/call", Some(params))
    }

    /// Answers a server request with a result.
    fn answer(&mut self, request: &Value, result: Value) {
        let id = request["id"].clone();
        self.send(&json!({"jsonrpc": "2.0", "id": id, "result": result}));
    }

    /// Calls `probe_elicit`, waits for its `elicitation/create`, returns (call id, request).
    fn start_elicit(&mut self, arguments: Value) -> (Value, Value) {
        let call_id = self.send_request(
            "tools/call",
            Some(json!({"name": "probe_elicit", "arguments": arguments})),
        );
        let req = self.wait_elicitation();
        assert!(
            !self.unmatched.iter().any(|m| m["id"] == call_id),
            "probe_elicit returned before the client answered: {:?}",
            self.unmatched
        );
        (call_id, req)
    }

    /// Asserts that no `elicitation/create` has been received so far.
    fn assert_no_elicitation_sent(&self) {
        assert!(
            self.server_requests.is_empty(),
            "server sent elicitation/create to a client without form-mode support: {:?}",
            self.server_requests
        );
    }

    /// Waits up to `AWAIT` for the process to exit; kills it and fails otherwise.
    fn wait_exit(&mut self, context: &str) -> ExitStatus {
        let child = self.child.as_mut().expect("child present");
        let start = Instant::now();
        loop {
            match child.try_wait().expect("try_wait failed") {
                Some(status) => {
                    self.child = None;
                    return status;
                }
                None if start.elapsed() >= AWAIT => {
                    let _ = child.kill();
                    let _ = child.wait();
                    self.child = None;
                    panic!("{context}: server did not exit within {AWAIT:?}; killed");
                }
                None => thread::sleep(Duration::from_millis(10)),
            }
        }
    }

    /// Closes stdin (if still held), waits for exit, drains stdout and stderr.
    fn finish(mut self, context: &str) -> Finished {
        drop(self.stdin.take());
        let status = self.wait_exit(context);
        let mut leftovers = std::mem::take(&mut self.unmatched);
        leftovers.extend(self.server_requests.drain(..));
        while !self.stdout_closed {
            match self.lines.recv_timeout(AWAIT) {
                Ok(Line::Message(msg)) => leftovers.push(msg),
                Ok(Line::Invalid(text)) => {
                    panic!("{context}: stdout carried a non-JSON-RPC line: {text}")
                }
                Ok(Line::Eof) | Err(RecvTimeoutError::Disconnected) => self.stdout_closed = true,
                Err(RecvTimeoutError::Timeout) => {
                    panic!("{context}: stdout not closed within {AWAIT:?} after exit")
                }
            }
        }
        let stderr = self
            .stderr_thread
            .take()
            .expect("stderr reader present")
            .join()
            .expect("stderr reader thread");
        let stderr = String::from_utf8_lossy(&stderr).into_owned();
        let finished = Finished {
            code: status.code(),
            leftovers,
            stderr,
        };
        assert_no_panic(context, &finished);
        finished
    }

    /// Closes the session and applies the default checks: exit 0, empty stderr, and nothing on
    /// stdout after the awaited messages except optional `notifications/cancelled`.
    fn close_clean(self, context: &str) {
        let finished = self.finish(context);
        assert_clean_exit(context, &finished);
        for msg in &finished.leftovers {
            assert!(
                msg.get("id").is_none() && msg["method"] == "notifications/cancelled",
                "{context}: unexpected message on stdout: {msg}"
            );
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        drop(self.stdin.take());
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Panic guard: exit code never 101, stderr never contains `panicked`.
fn assert_no_panic(context: &str, finished: &Finished) {
    assert_ne!(
        finished.code,
        Some(101),
        "{context}: exit code 101 (panic)\nstderr: {}",
        finished.stderr
    );
    assert!(
        !finished.stderr.contains("panicked"),
        "{context}: stderr mentions a panic\nstderr: {}",
        finished.stderr
    );
}

fn assert_clean_exit(context: &str, finished: &Finished) {
    assert_eq!(
        finished.code,
        Some(0),
        "{context}: expected exit 0 after EOF\nstderr: {}",
        finished.stderr
    );
    assert!(
        finished.stderr.is_empty(),
        "{context}: expected empty stderr\nstderr: {}",
        finished.stderr
    );
}

fn initialize_request(id: &Value, protocol: &str, capabilities: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "initialize",
        "params": {
            "protocolVersion": protocol,
            "capabilities": capabilities,
            "clientInfo": {"name": "aitm-test-client", "version": "0.0.0"}
        }
    })
}

/// Returns `result` of a successful response.
fn result_of(resp: &Value) -> &Value {
    assert!(
        resp.get("error").is_none(),
        "expected a result, got: {resp}"
    );
    resp.get("result")
        .unwrap_or_else(|| panic!("response has no result: {resp}"))
}

/// Returns the error code of an error response.
fn error_code(resp: &Value) -> i64 {
    assert!(
        resp.get("result").is_none(),
        "expected a JSON-RPC error, got: {resp}"
    );
    resp["error"]["code"]
        .as_i64()
        .unwrap_or_else(|| panic!("error without integer code: {resp}"))
}

/// Checks a successful tool result and returns its `structuredContent` (spec: `isError` absent or
/// false; `content` is exactly one text item holding the JSON of `structuredContent`).
fn structured_ok(resp: &Value) -> Map<String, Value> {
    let result = result_of(resp);
    let is_error = result.get("isError");
    assert!(
        is_error.is_none() || is_error == Some(&Value::Bool(false)),
        "expected a successful tool result: {resp}"
    );
    let structured = result
        .get("structuredContent")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("structuredContent missing or not an object: {resp}"))
        .clone();
    let content = result["content"]
        .as_array()
        .unwrap_or_else(|| panic!("content is not an array: {resp}"));
    assert_eq!(
        content.len(),
        1,
        "content must have exactly one item: {resp}"
    );
    assert_eq!(
        content[0]["type"], "text",
        "content item is not text: {resp}"
    );
    let text = content[0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("text item without text: {resp}"));
    let parsed: Value = serde_json::from_str(text)
        .unwrap_or_else(|e| panic!("content text is not JSON ({e}): {resp}"));
    assert_eq!(
        parsed,
        Value::Object(structured.clone()),
        "content text differs from structuredContent: {resp}"
    );
    structured
}

/// Checks an invalid-arguments tool execution error (spec "Errors").
fn assert_invalid_arguments(context: &str, resp: &Value) {
    assert_tool_execution_error(context, resp, INVALID_ARGUMENTS_PREFIX);
}

/// Checks a tool execution error: `isError: true`, no `structuredContent`, one text item starting
/// with `prefix` (spec "Errors").
fn assert_tool_execution_error(context: &str, resp: &Value, prefix: &str) {
    let result = result_of(resp);
    assert_eq!(
        result.get("isError"),
        Some(&Value::Bool(true)),
        "{context}: expected isError true: {resp}"
    );
    assert!(
        result.get("structuredContent").is_none(),
        "{context}: error result must not have structuredContent: {resp}"
    );
    let content = result["content"]
        .as_array()
        .unwrap_or_else(|| panic!("{context}: content is not an array: {resp}"));
    assert_eq!(
        content.len(),
        1,
        "{context}: expected one content item: {resp}"
    );
    assert_eq!(
        content[0]["type"], "text",
        "{context}: not a text item: {resp}"
    );
    let text = content[0]["text"].as_str().unwrap_or_default();
    assert!(
        text.starts_with(prefix),
        "{context}: text must start with {prefix:?}: {resp}"
    );
}

/// Checks a completed `probe_elicit` result and returns its `structuredContent`.
fn elicit_outcome(resp: &Value, expected_outcome: &str) -> Map<String, Value> {
    let structured = structured_ok(resp);
    assert_eq!(
        structured.get("outcome").and_then(Value::as_str),
        Some(expected_outcome),
        "unexpected probe_elicit outcome: {resp}"
    );
    for key in structured.keys() {
        assert!(
            ["outcome", "content", "reason"].contains(&key.as_str()),
            "unexpected key {key:?} in probe_elicit result: {resp}"
        );
    }
    for key in ["content", "reason"] {
        assert_ne!(
            structured.get(key),
            Some(&Value::Null),
            "optional field {key:?} must be omitted, not null: {resp}"
        );
    }
    structured
}

fn assert_non_empty_reason(structured: &Map<String, Value>) -> String {
    let reason = structured
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("reason missing or not a string: {structured:?}"));
    assert!(!reason.is_empty(), "reason is empty: {structured:?}");
    reason.to_owned()
}

/// Checks the `elicitation/create` request shape (spec test 13).
fn check_elicitation_request(req: &Value) {
    let params = &req["params"];
    let message = params["message"].as_str().unwrap_or_default();
    assert!(!message.is_empty(), "elicitation message is empty: {req}");
    let mode = params.get("mode");
    assert!(
        mode.is_none() || mode == Some(&Value::from("form")),
        "elicitation mode must be absent or \"form\": {req}"
    );
    let schema = &params["requestedSchema"];
    assert_eq!(schema["type"], "object", "requestedSchema.type: {req}");
    assert_eq!(
        schema["properties"]["confirm"]["type"], "boolean",
        "requestedSchema confirm type: {req}"
    );
    let required = schema["required"].as_array().cloned().unwrap_or_default();
    assert!(
        required.contains(&Value::from("confirm")),
        "requestedSchema.required must contain confirm: {req}"
    );
}

/// A `tools/list` request line padded through `params._meta.pad` to exactly `len` bytes.
fn padded_tools_list_line(id: i64, len: usize) -> Vec<u8> {
    let make = |pad: &str| {
        serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/list",
            "params": {"_meta": {"pad": pad}}
        }))
        .expect("serialize padded request")
    };
    let base = make("").len();
    assert!(base < len, "padding target too small");
    let line = make(&"a".repeat(len - base));
    assert_eq!(line.len(), len, "padded line has the wrong length");
    line
}

// =============================================================================================
// Shared checks (used by several numbered tests and by test 23)
// =============================================================================================

/// Test 3 assertions on an initialize result.
fn check_initialize_result(result: &Value, expected_version: &str) {
    assert_eq!(
        result["protocolVersion"], expected_version,
        "protocolVersion: {result}"
    );
    assert_eq!(
        result["serverInfo"]["name"], "aitm",
        "serverInfo.name: {result}"
    );
    assert_eq!(
        result["serverInfo"]["version"],
        env!("CARGO_PKG_VERSION"),
        "serverInfo.version: {result}"
    );
    let caps = result["capabilities"]
        .as_object()
        .unwrap_or_else(|| panic!("capabilities is not an object: {result}"));
    let tools = caps
        .get("tools")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("capabilities.tools is not an object: {result}"));
    let list_changed = tools.get("listChanged");
    assert!(
        list_changed.is_none() || list_changed == Some(&Value::Bool(false)),
        "capabilities.tools.listChanged must be absent or false: {result}"
    );
    for feature in [
        "resources",
        "prompts",
        "logging",
        "completions",
        "tasks",
        "experimental",
        "extensions",
    ] {
        assert!(
            !caps.contains_key(feature),
            "capabilities must not announce {feature:?}: {result}"
        );
    }
}

/// Test 3: full handshake plus ping.
fn check_handshake(server: &mut Server) {
    let result = server.initialize(LATEST, no_capabilities());
    check_initialize_result(&result, LATEST);
    let pong = server.request("ping", None);
    assert_eq!(result_of(&pong), &json!({}), "ping result must be {{}}");
}

fn tools_list(server: &mut Server) -> Value {
    let resp = server.request("tools/list", None);
    result_of(&resp).clone()
}

fn object_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

/// Test 5 assertions on a `tools/list` result.
fn check_tools_list(result: &Value) {
    let tools = result["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("tools is not an array: {result}"));
    let names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(
        names,
        ["probe_echo", "probe_elicit"],
        "tool names: {result}"
    );
    let cursor = result.get("nextCursor");
    assert!(
        cursor.is_none() || cursor == Some(&Value::Null),
        "nextCursor must be absent or null: {result}"
    );

    for tool in tools {
        let name = tool["name"].as_str().unwrap_or_default();
        assert!(
            name.starts_with("probe_"),
            "tool name without probe_: {tool}"
        );
        let description = tool["description"].as_str().unwrap_or_default();
        assert!(
            description.to_lowercase().contains("experimental"),
            "{name}: description must say experimental: {tool}"
        );
        assert!(
            description.contains("may be removed without notice"),
            "{name}: description must say \"may be removed without notice\": {tool}"
        );
        let input = &tool["inputSchema"];
        assert_eq!(input["type"], "object", "{name}: inputSchema.type: {tool}");
        assert_eq!(
            input["additionalProperties"], false,
            "{name}: inputSchema.additionalProperties: {tool}"
        );
        assert_eq!(
            tool["outputSchema"]["type"], "object",
            "{name}: outputSchema.type: {tool}"
        );
        assert_eq!(
            tool["annotations"]["readOnlyHint"], true,
            "{name}: readOnlyHint: {tool}"
        );
        assert_eq!(
            tool["annotations"]["openWorldHint"], false,
            "{name}: openWorldHint: {tool}"
        );
    }

    let echo = &tools[0];
    let message = &echo["inputSchema"]["properties"]["message"];
    assert_eq!(message["type"], "string", "probe_echo message type: {echo}");
    assert_eq!(
        message["maxLength"], 4096,
        "probe_echo message maxLength: {echo}"
    );
    let required = echo["inputSchema"]["required"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        required.contains(&Value::from("message")),
        "probe_echo inputSchema.required must contain message: {echo}"
    );
    let mut echo_out = object_keys(&echo["outputSchema"]["properties"]);
    echo_out.sort();
    assert_eq!(
        echo_out,
        ["length_chars", "message"],
        "probe_echo outputSchema properties: {echo}"
    );

    let elicit = &tools[1];
    let timeout = &elicit["inputSchema"]["properties"]["timeout_ms"];
    assert_eq!(timeout["type"], "integer", "timeout_ms type: {elicit}");
    assert_eq!(timeout["minimum"], 100, "timeout_ms minimum: {elicit}");
    assert_eq!(timeout["maximum"], 600_000, "timeout_ms maximum: {elicit}");
    let mut elicit_out = object_keys(&elicit["outputSchema"]["properties"]);
    elicit_out.sort();
    assert_eq!(
        elicit_out,
        ["content", "outcome", "reason"],
        "probe_elicit outputSchema properties: {elicit}"
    );
    let required = elicit["outputSchema"]["required"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        required.contains(&Value::from("outcome")),
        "probe_elicit outputSchema.required must contain outcome: {elicit}"
    );
}

/// Test 7 assertions for one message.
fn check_echo(server: &mut Server, message: &str, expected_chars: u64) {
    let resp = server.call_tool("probe_echo", Some(json!({"message": message})));
    let structured = structured_ok(&resp);
    assert_eq!(
        Value::Object(structured),
        json!({"message": message, "length_chars": expected_chars}),
        "probe_echo structuredContent for a {expected_chars}-char message"
    );
}

/// Test 7: all three messages.
fn check_echo_success(server: &mut Server) {
    check_echo(server, "h\u{e9}llo \u{1F39A}", 7);
    check_echo(server, "", 0);
    check_echo(server, &"\u{e9}".repeat(4096), 4096);
}

/// Test 11 assertions in an initialized session without elicitation capability.
fn check_elicit_unsupported(server: &mut Server, arguments: Value) {
    let resp = server.call_tool("probe_elicit", Some(arguments));
    let structured = elicit_outcome(&resp, "unsupported");
    assert_non_empty_reason(&structured);
    assert!(
        !structured.contains_key("content"),
        "unsupported must not carry content: {resp}"
    );
    server.assert_no_elicitation_sent();
}

// =============================================================================================
// Test 1: stdin null
// =============================================================================================

#[test]
fn t01_stdin_null_exits_zero_silently() {
    let server = Server::start_with(Stdio::null(), None);
    let finished = server.finish("stdin null");
    assert_clean_exit("stdin null", &finished);
    assert!(
        finished.leftovers.is_empty(),
        "stdin null: stdout must be empty: {:?}",
        finished.leftovers
    );
}

// =============================================================================================
// Test 2: silence before initialize, EOF exit
// =============================================================================================

#[test]
fn t02_silent_before_initialize_and_exits_on_eof() {
    let server = Server::start();
    match server.lines.recv_timeout(Duration::from_secs(1)) {
        Err(RecvTimeoutError::Timeout) => {}
        Ok(Line::Message(msg)) => panic!("server wrote before any request: {msg}"),
        Ok(Line::Invalid(text)) => panic!("server wrote before any request: {text}"),
        Ok(Line::Eof) | Err(RecvTimeoutError::Disconnected) => {
            panic!("server closed stdout while stdin was still open")
        }
    }
    let finished = server.finish("silence then EOF");
    assert_clean_exit("silence then EOF", &finished);
    assert!(
        finished.leftovers.is_empty(),
        "stdout must stay empty: {:?}",
        finished.leftovers
    );
}

// =============================================================================================
// Test 3: handshake 2025-11-25
// =============================================================================================

#[test]
fn t03_handshake_latest_version_and_ping() {
    let mut server = Server::start();
    check_handshake(&mut server);
    server.close_clean("handshake");
}

// =============================================================================================
// Test 4: version negotiation
// =============================================================================================

fn check_negotiation(requested: &str, expected: &str) {
    let mut server = Server::start();
    let resp = server.initialize_raw(requested, no_capabilities());
    check_initialize_result(result_of(&resp), expected);
    let text = resp.to_string();
    assert!(
        !text.contains("2026-07-28"),
        "requested {requested}: the server must never put 2026-07-28 in a message: {text}"
    );
    server.send_initialized();
    let tools = server.request("tools/list", None);
    result_of(&tools);
    server.close_clean(&format!("negotiation {requested}"));
}

#[test]
fn t04_version_2025_06_18_is_kept() {
    check_negotiation("2025-06-18", "2025-06-18");
}

#[test]
fn t04_version_2025_11_25_is_kept() {
    check_negotiation("2025-11-25", "2025-11-25");
}

#[test]
fn t04_version_2024_11_05_gets_latest() {
    check_negotiation("2024-11-05", LATEST);
}

#[test]
fn t04_version_2025_03_26_gets_latest() {
    check_negotiation("2025-03-26", LATEST);
}

#[test]
fn t04_version_2026_07_28_gets_latest() {
    check_negotiation("2026-07-28", LATEST);
}

#[test]
fn t04_version_1999_01_01_gets_latest() {
    check_negotiation("1999-01-01", LATEST);
}

// =============================================================================================
// Test 5: tools/list
// =============================================================================================

#[test]
fn t05_tools_list_contract() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    let result = tools_list(&mut server);
    check_tools_list(&result);
    server.close_clean("tools/list");
}

// =============================================================================================
// Test 6: determinism
// =============================================================================================

#[test]
fn t06_tools_list_is_identical_across_processes() {
    let mut results = Vec::new();
    for _ in 0..2 {
        let mut server = Server::start();
        server.initialize(LATEST, no_capabilities());
        results.push(tools_list(&mut server));
        server.close_clean("determinism");
    }
    assert_eq!(results[0], results[1], "tools/list differs between runs");
}

// =============================================================================================
// Test 7: probe_echo success
// =============================================================================================

#[test]
fn t07_probe_echo_returns_message_and_length() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    check_echo_success(&mut server);
    server.close_clean("probe_echo success");
}

// =============================================================================================
// Test 8a: probe_echo invalid arguments (object, absent or null)
// =============================================================================================

#[test]
fn t08a_probe_echo_invalid_arguments_are_tool_errors() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    let cases: [(&str, Option<Value>); 5] = [
        ("missing message", Some(json!({}))),
        ("message is a number", Some(json!({"message": 42}))),
        ("4097 chars", Some(json!({"message": "a".repeat(4097)}))),
        ("unknown field", Some(json!({"message": "x", "extra": 1}))),
        ("arguments absent", None),
    ];
    for (context, arguments) in cases {
        let resp = server.call_tool("probe_echo", arguments);
        assert_invalid_arguments(context, &resp);
    }
    let resp = server.request(
        "tools/call",
        Some(json!({"name": "probe_echo", "arguments": null})),
    );
    assert_invalid_arguments("arguments is null", &resp);
    server.close_clean("probe_echo invalid arguments");
}

// =============================================================================================
// Test 8b: probe_echo with non-object arguments (known rmcp 3.5.1 limitation)
// =============================================================================================

/// Asserts that a message sent in reaction to a non-object `arguments` call is a JSON-RPC error
/// (spec "Errors": observed `-32600` without the id or `-32601` with it), never a result. The id may
/// be the call's id, `null` or absent; the error code is not pinned.
fn assert_reaction_is_error(msg: &Value) {
    assert!(
        msg.get("error").is_some() && msg.get("result").is_none(),
        "a reaction to non-object arguments must be a JSON-RPC error without result: {msg}"
    );
}

#[test]
fn t08b_probe_echo_non_object_arguments_do_not_break_the_session() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    // No response to these calls is required (spec "Errors"); their ids are never awaited.
    for arguments in [json!("x"), json!(42)] {
        server.send_request(
            "tools/call",
            Some(json!({"name": "probe_echo", "arguments": arguments})),
        );
    }
    let result = tools_list(&mut server);
    check_tools_list(&result);
    check_echo(&mut server, "still serving", 13);
    for msg in &server.unmatched {
        assert_reaction_is_error(msg);
    }
    let finished = server.finish("non-object arguments");
    assert_clean_exit("non-object arguments", &finished);
    for msg in &finished.leftovers {
        assert_reaction_is_error(msg);
    }
}

// =============================================================================================
// Test 9: unknown tool
// =============================================================================================

#[test]
fn t09_unknown_tool_is_invalid_params() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    for name in ["probe_nope", "auto_master"] {
        let resp = server.call_tool(name, Some(json!({})));
        assert_eq!(error_code(&resp), -32602, "unknown tool {name}: {resp}");
    }
    let result = tools_list(&mut server);
    check_tools_list(&result);
    server.close_clean("unknown tool");
}

// =============================================================================================
// Test 10: unknown method
// =============================================================================================

#[test]
fn t10_unknown_method_is_method_not_found() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    let resp = server.request("frobnicate/now", Some(json!({})));
    assert_eq!(error_code(&resp), -32601, "unknown method: {resp}");
    server.close_clean("unknown method");
}

// =============================================================================================
// Test 11: elicit, no capability
// =============================================================================================

#[test]
fn t11_elicit_without_capability_is_unsupported_2025_11_25() {
    let mut server = Server::start();
    server.initialize("2025-11-25", no_capabilities());
    check_elicit_unsupported(&mut server, json!({}));
    server.close_clean("elicit unsupported 2025-11-25");
}

#[test]
fn t11_elicit_without_capability_is_unsupported_2025_06_18() {
    let mut server = Server::start();
    server.initialize("2025-06-18", no_capabilities());
    check_elicit_unsupported(&mut server, json!({}));
    server.close_clean("elicit unsupported 2025-06-18");
}

// =============================================================================================
// Test 12: elicit, URL-only capability
// =============================================================================================

#[test]
fn t12_elicit_with_url_only_capability_is_unsupported() {
    let mut server = Server::start();
    server.initialize(LATEST, json!({"elicitation": {"url": {}}}));
    check_elicit_unsupported(&mut server, json!({}));
    server.close_clean("elicit url-only");
}

// =============================================================================================
// Test 13: elicit, accept
// =============================================================================================

/// Runs one accept round trip and returns the `structuredContent`.
fn elicit_with_answer(server: &mut Server, answer: Value) -> (Value, Map<String, Value>) {
    let (call_id, req) = server.start_elicit(json!({}));
    check_elicitation_request(&req);
    server.answer(&req, answer);
    let resp = server.wait_response(&call_id);
    let structured = structured_ok(&resp);
    (resp, structured)
}

fn check_accept(server: &mut Server, content: Value, expected: Value) {
    let (resp, _) = elicit_with_answer(server, json!({"action": "accept", "content": content}));
    let structured = elicit_outcome(&resp, "accepted");
    assert_eq!(
        structured.get("content"),
        Some(&expected),
        "accepted content: {resp}"
    );
    assert!(
        !structured.contains_key("reason"),
        "accepted must not carry a reason: {resp}"
    );
}

fn check_accept_variants(protocol: &str, capabilities: Value) {
    let mut server = Server::start();
    server.initialize(protocol, capabilities);
    check_accept(
        &mut server,
        json!({"confirm": true}),
        json!({"confirm": true}),
    );
    check_accept(
        &mut server,
        json!({"confirm": false}),
        json!({"confirm": false}),
    );
    check_accept(
        &mut server,
        json!({"confirm": true, "x": 1}),
        json!({"confirm": true}),
    );
    server.close_clean(&format!("elicit accept {protocol}"));
}

#[test]
fn t13_elicit_accept_with_form_capability() {
    check_accept_variants("2025-11-25", form_capable());
}

#[test]
fn t13_elicit_accept_with_empty_elicitation_capability_2025_06_18() {
    check_accept_variants("2025-06-18", json!({"elicitation": {}}));
}

// =============================================================================================
// Tests 14, 15: decline, cancel
// =============================================================================================

fn check_bare_outcome(action: &str, expected: &str) {
    let mut server = Server::start();
    server.initialize(LATEST, form_capable());
    let (resp, _) = elicit_with_answer(&mut server, json!({"action": action}));
    let structured = elicit_outcome(&resp, expected);
    assert!(
        !structured.contains_key("content"),
        "{expected} must not carry content: {resp}"
    );
    assert!(
        !structured.contains_key("reason"),
        "{expected} must not carry a reason: {resp}"
    );
    server.close_clean(&format!("elicit {action}"));
}

#[test]
fn t14_elicit_decline() {
    check_bare_outcome("decline", "declined");
}

#[test]
fn t15_elicit_cancel() {
    check_bare_outcome("cancel", "cancelled");
}

// =============================================================================================
// Test 16: invalid accept
// =============================================================================================

#[test]
fn t16_elicit_invalid_answers_are_errors() {
    let mut server = Server::start();
    server.initialize(LATEST, form_capable());
    for answer in [
        json!({"action": "accept", "content": {"confirm": "yes"}}),
        json!({"action": "accept"}),
        json!({"action": "maybe"}),
    ] {
        let (resp, _) = elicit_with_answer(&mut server, answer.clone());
        let structured = elicit_outcome(&resp, "error");
        assert_non_empty_reason(&structured);
        assert!(
            !structured.contains_key("content"),
            "error must not carry content (answer {answer}): {resp}"
        );
    }
    server.close_clean("elicit invalid answers");
}

// =============================================================================================
// Test 17: client error
// =============================================================================================

#[test]
fn t17_elicit_client_error_is_error_with_code() {
    let mut server = Server::start();
    server.initialize(LATEST, form_capable());
    let (call_id, req) = server.start_elicit(json!({}));
    let id = req["id"].clone();
    server.send(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": -32603, "message": "boom"}
    }));
    let resp = server.wait_response(&call_id);
    let structured = elicit_outcome(&resp, "error");
    let reason = assert_non_empty_reason(&structured);
    assert!(
        reason.contains("-32603"),
        "reason must contain the error code: {resp}"
    );
    assert!(!structured.contains_key("content"), "no content: {resp}");
    server.close_clean("elicit client error");
}

// =============================================================================================
// Test 18: timeout and late answer
// =============================================================================================

#[test]
fn t18_elicit_timeout_then_late_answer_is_ignored() {
    let mut server = Server::start();
    server.initialize(LATEST, form_capable());
    let sent_at = Instant::now();
    let (call_id, req) = server.start_elicit(json!({"timeout_ms": 500}));
    let resp = server.wait_response(&call_id);
    let elapsed = sent_at.elapsed();
    let structured = elicit_outcome(&resp, "timeout");
    assert_non_empty_reason(&structured);
    assert!(!structured.contains_key("content"), "no content: {resp}");
    assert!(
        elapsed >= Duration::from_millis(500),
        "timeout result after {elapsed:?}, expected at least 500 ms"
    );

    // Late answer: ignored, the server keeps serving.
    server.answer(
        &req,
        json!({"action": "accept", "content": {"confirm": true}}),
    );
    let result = tools_list(&mut server);
    check_tools_list(&result);
    assert!(
        server.unmatched.is_empty(),
        "unexpected responses: {:?}",
        server.unmatched
    );
    assert!(
        server.server_requests.is_empty(),
        "unexpected server requests: {:?}",
        server.server_requests
    );
    for note in &server.cancelled {
        assert_eq!(
            note["params"]["requestId"], req["id"],
            "notifications/cancelled must name the elicitation request: {note}"
        );
    }
    server.close_clean("elicit timeout");
}

// =============================================================================================
// Test 19: argument bounds
// =============================================================================================

fn invalid_elicit_arguments() -> [(&'static str, Value); 5] {
    [
        ("timeout_ms 99", json!({"timeout_ms": 99})),
        ("timeout_ms 600001", json!({"timeout_ms": 600_001})),
        ("timeout_ms -1", json!({"timeout_ms": -1})),
        ("timeout_ms string", json!({"timeout_ms": "abc"})),
        ("unknown field", json!({"extra": 1})),
    ]
}

#[test]
fn t19_elicit_argument_bounds_with_form_capable_client() {
    let mut server = Server::start();
    server.initialize(LATEST, form_capable());
    for (context, arguments) in invalid_elicit_arguments() {
        let resp = server.call_tool("probe_elicit", Some(arguments));
        assert_invalid_arguments(context, &resp);
        assert!(
            server.server_requests.is_empty(),
            "{context}: invalid arguments must not start an elicitation"
        );
    }
    server.close_clean("elicit argument bounds (form client)");
}

#[test]
fn t19_elicit_argument_bounds_without_capability() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    for (context, arguments) in invalid_elicit_arguments() {
        let resp = server.call_tool("probe_elicit", Some(arguments));
        assert_invalid_arguments(context, &resp);
    }
    check_elicit_unsupported(&mut server, json!({"timeout_ms": 100}));
    check_elicit_unsupported(&mut server, json!({"timeout_ms": 600_000}));
    server.close_clean("elicit argument bounds (no capability)");
}

// =============================================================================================
// Test 20: concurrency
// =============================================================================================

#[test]
fn t20_server_answers_while_elicitation_is_pending() {
    let mut server = Server::start();
    server.initialize(LATEST, form_capable());
    let (call_id, req) = server.start_elicit(json!({}));
    check_elicitation_request(&req);

    let echo = server.call_tool("probe_echo", Some(json!({"message": "busy"})));
    let structured = structured_ok(&echo);
    assert_eq!(
        Value::Object(structured),
        json!({"message": "busy", "length_chars": 4})
    );

    server.answer(
        &req,
        json!({"action": "accept", "content": {"confirm": true}}),
    );
    let resp = server.wait_response(&call_id);
    let structured = elicit_outcome(&resp, "accepted");
    assert_eq!(structured.get("content"), Some(&json!({"confirm": true})));
    server.close_clean("concurrency");
}

// =============================================================================================
// Test 21: EOF while elicitation pending
// =============================================================================================

#[test]
fn t21_eof_while_elicitation_pending_exits_zero() {
    let mut server = Server::start();
    server.initialize(LATEST, form_capable());
    let (_call_id, _req) = server.start_elicit(json!({}));
    let finished = server.finish("EOF while elicitation pending");
    assert_clean_exit("EOF while elicitation pending", &finished);
}

// =============================================================================================
// Test 22: malformed line
// =============================================================================================

#[test]
fn t22_malformed_line_is_tolerated() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    server.send_raw(b"{not json\n");
    let result = tools_list(&mut server);
    check_tools_list(&result);
    let finished = server.finish("malformed line");
    assert_clean_exit("malformed line", &finished);
    for msg in &finished.leftovers {
        assert_eq!(
            msg["id"],
            Value::Null,
            "only a parse error with id null may answer a malformed line: {msg}"
        );
        assert_eq!(error_code(msg), -32700, "malformed line response: {msg}");
    }
}

// =============================================================================================
// Test 23: no file side effects
// =============================================================================================

/// Removes the directory when the test ends, even on failure.
struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn t23_session_leaves_current_directory_empty() {
    let dir = TempDir(std::env::temp_dir().join(format!("aitm-mcp-probe-{}", unique_suffix())));
    std::fs::create_dir(&dir.0).expect("create empty temp dir");

    let mut server = Server::start_with(Stdio::piped(), Some(&dir.0));
    check_handshake(&mut server); // test 3
    let result = tools_list(&mut server); // test 5
    check_tools_list(&result);
    check_echo_success(&mut server); // test 7
    check_elicit_unsupported(&mut server, json!({})); // test 11
    server.close_clean("no file side effects");

    let entries: Vec<_> = std::fs::read_dir(&dir.0)
        .expect("read temp dir")
        .map(|e| e.map(|e| e.file_name()))
        .collect::<Result<_, _>>()
        .expect("read temp dir entries");
    assert!(
        entries.is_empty(),
        "the probe server created files in its current directory: {entries:?}"
    );
}

// =============================================================================================
// Test 24: era probe before initialize
// =============================================================================================

#[test]
fn t24_server_discover_before_initialize_then_initialize() {
    let mut server = Server::start();
    server.send(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "server/discover",
        "params": {"_meta": {"io.modelcontextprotocol/protocolVersion": "2026-07-28"}}
    }));
    let resp = server.wait_response(&Value::from(1));
    assert_eq!(error_code(&resp), -32602, "server/discover: {resp}");
    assert!(
        server
            .child
            .as_mut()
            .expect("child present")
            .try_wait()
            .expect("try_wait")
            .is_none(),
        "the process must keep running after server/discover"
    );

    let id = Value::from(2);
    server.send(&initialize_request(&id, LATEST, no_capabilities()));
    let resp = server.wait_response(&id);
    check_initialize_result(result_of(&resp), LATEST);
    server.send_initialized();
    server.close_clean("server/discover then initialize");
}

// =============================================================================================
// Test 25: size limit, boundary
// =============================================================================================

#[test]
fn t25_line_of_exactly_one_mebibyte_is_accepted() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    let id = server.fresh_id();
    let mut line = padded_tools_list_line(id, MAX_MESSAGE_BYTES);
    line.push(b'\n');
    server.send_large_line(line);
    let resp = server.wait_response(&Value::from(id));
    check_tools_list(result_of(&resp));
    server.close_clean("1 MiB line");
}

// =============================================================================================
// Test 28: CRLF line terminator at the size limit
// =============================================================================================

#[test]
fn t28_crlf_line_of_exactly_one_mebibyte_is_accepted() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    let id = server.fresh_id();
    let mut line = padded_tools_list_line(id, MAX_MESSAGE_BYTES);
    line.extend_from_slice(b"\r\n");
    server.send_large_line(line);
    let resp = server.wait_response(&Value::from(id));
    check_tools_list(result_of(&resp));
    server.close_clean("1 MiB line with CRLF");
}

// =============================================================================================
// Test 26: size limit, over
// =============================================================================================

#[test]
fn t26_line_over_one_mebibyte_is_fatal() {
    let mut server = Server::start();
    server.initialize(LATEST, no_capabilities());
    let id = server.fresh_id();
    let mut line = padded_tools_list_line(id, MAX_MESSAGE_BYTES + 1);
    line.push(b'\n');

    // The writer thread keeps stdin open after writing until the test releases it; write errors
    // are ignored because the server may exit while the test is still writing.
    let mut stdin = server.stdin.take().expect("stdin is open");
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let writer = thread::spawn(move || {
        let _ = stdin.write_all(&line).and_then(|()| stdin.flush());
        let _ = release_rx.recv();
        drop(stdin);
    });

    let finished = server.finish("line over 1 MiB");
    let _ = release_tx.send(());
    writer.join().expect("writer thread");

    assert_eq!(
        finished.code,
        Some(1),
        "expected exit 1 for an oversized line\nstderr: {}",
        finished.stderr
    );
    assert!(
        finished.stderr.starts_with("error:"),
        "stderr must start with \"error:\": {:?}",
        finished.stderr
    );
    assert!(
        finished.stderr.contains("exceeds 1048576 bytes"),
        "stderr must contain \"exceeds 1048576 bytes\": {:?}",
        finished.stderr
    );
    let line = finished
        .stderr
        .strip_suffix("\r\n")
        .or_else(|| finished.stderr.strip_suffix('\n'))
        .unwrap_or_else(|| {
            panic!(
                "stderr must end with a line terminator: {:?}",
                finished.stderr
            )
        });
    assert!(
        !line.contains('\n') && !line.contains('\r'),
        "stderr must be exactly one line: {:?}",
        finished.stderr
    );
    // Every stdout line was validated as JSON-RPC by the reader (`finish` fails otherwise).
}

// =============================================================================================
// Test 27: in-flight cap
// =============================================================================================

/// Maximum number of `tools/call` requests in flight (spec "Concurrency", in-flight cap).
const MAX_CALLS_IN_FLIGHT: usize = 32;

/// Text every over-cap tool execution error starts with (spec "Concurrency").
const TOO_MANY_PREFIX: &str = "too many calls in flight:";

#[test]
fn t27_in_flight_cap_rejects_the_33rd_call() {
    let mut server = Server::start();
    server.initialize("2025-11-25", form_capable());

    let mut call_ids = Vec::new();
    for _ in 0..MAX_CALLS_IN_FLIGHT {
        call_ids.push(server.send_request(
            "tools/call",
            Some(json!({"name": "probe_elicit", "arguments": {"timeout_ms": 600_000}})),
        ));
    }
    let mut requests = Vec::new();
    for _ in 0..MAX_CALLS_IN_FLIGHT {
        let req = server.wait_elicitation();
        check_elicitation_request(&req);
        requests.push(req);
    }
    let mut ids: Vec<String> = requests.iter().map(|r| r["id"].to_string()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(
        ids.len(),
        MAX_CALLS_IN_FLIGHT,
        "each pending elicitation must have its own request id"
    );
    for call_id in &call_ids {
        assert!(
            !server.unmatched.iter().any(|m| &m["id"] == call_id),
            "a probe_elicit call returned before its elicitation was answered: {:?}",
            server.unmatched
        );
    }

    let over = server.call_tool("probe_echo", Some(json!({"message": "x"})));
    assert_tool_execution_error("33rd call", &over, TOO_MANY_PREFIX);

    let result = tools_list(&mut server);
    check_tools_list(&result);

    for req in &requests {
        server.answer(req, json!({"action": "decline"}));
    }
    for call_id in &call_ids {
        let resp = server.wait_response(call_id);
        elicit_outcome(&resp, "declined");
    }

    check_echo(&mut server, "x", 1);
    server.close_clean("in-flight cap");
}

// =============================================================================================
// Test 29: cancellation frees the slot
// =============================================================================================

/// Sends 32 `probe_elicit` calls (timeout 600000 ms) and collects their 32 `elicitation/create`
/// requests. Fails at once if any of these calls gets a response (e.g. `too many calls in flight:`).
fn start_full_batch_of_elicitations(server: &mut Server) -> (Vec<Value>, Vec<Value>) {
    let mut call_ids = Vec::new();
    for _ in 0..MAX_CALLS_IN_FLIGHT {
        call_ids.push(server.send_request(
            "tools/call",
            Some(json!({"name": "probe_elicit", "arguments": {"timeout_ms": 600_000}})),
        ));
    }
    let mut requests: Vec<Value> = Vec::new();
    while requests.len() < MAX_CALLS_IN_FLIGHT {
        if let Some(resp) = server
            .unmatched
            .iter()
            .find(|m| call_ids.contains(&m["id"]))
        {
            panic!("a probe_elicit call was answered before its elicitation: {resp}");
        }
        if let Some(req) = server.server_requests.pop_front() {
            check_elicitation_request(&req);
            requests.push(req);
            continue;
        }
        let msg = server.next_message("an elicitation/create request of the batch");
        if let Some(resp) = server.route(msg) {
            server.unmatched.push(resp);
        }
    }
    (call_ids, requests)
}

/// Fails if any message with a `result` for one of the cancelled call ids was received.
fn assert_no_result_for(cancelled_ids: &[Value], messages: &[Value]) {
    for msg in messages {
        assert!(
            !(msg.get("result").is_some() && cancelled_ids.contains(&msg["id"])),
            "a cancelled tools/call received a result: {msg}"
        );
    }
}

#[test]
fn t29_cancellation_frees_the_slot() {
    let mut server = Server::start();
    server.initialize("2025-11-25", form_capable());

    let (cancelled_ids, _abandoned) = start_full_batch_of_elicitations(&mut server);
    for id in &cancelled_ids {
        server.send(&json!({
            "jsonrpc": "2.0",
            "method": "notifications/cancelled",
            "params": {"requestId": id, "reason": "test cancels the call"}
        }));
    }

    let (new_ids, requests) = start_full_batch_of_elicitations(&mut server);
    assert_no_result_for(&cancelled_ids, &server.unmatched);

    for req in &requests {
        server.answer(req, json!({"action": "decline"}));
    }
    for call_id in &new_ids {
        let resp = server.wait_response(call_id);
        elicit_outcome(&resp, "declined");
    }
    assert_no_result_for(&cancelled_ids, &server.unmatched);

    let context = "cancellation frees the slot";
    let finished = server.finish(context);
    assert_clean_exit(context, &finished);
    assert_no_result_for(&cancelled_ids, &finished.leftovers);
}

// =============================================================================================
// Capability detection (spec "Capability detection"; beyond the numbered list)
// =============================================================================================

#[test]
fn elicit_url_only_with_unknown_keys_is_unsupported() {
    let mut server = Server::start();
    server.initialize(LATEST, json!({"elicitation": {"url": {}, "foo": {}}}));
    check_elicit_unsupported(&mut server, json!({}));
    server.close_clean("elicit url-only with unknown keys");
}

fn check_form_capable(capabilities: Value) {
    let context = format!("capabilities {capabilities}");
    let mut server = Server::start();
    server.initialize(LATEST, capabilities);
    let (resp, _) = elicit_with_answer(&mut server, json!({"action": "decline"}));
    elicit_outcome(&resp, "declined");
    server.close_clean(&context);
}

#[test]
fn elicit_form_and_url_capability_is_form_capable() {
    check_form_capable(json!({"elicitation": {"form": {}, "url": {}}}));
}

#[test]
fn elicit_unknown_key_only_capability_is_form_capable() {
    check_form_capable(json!({"elicitation": {"foo": {}}}));
}
