//! The experimental probe tools `probe_echo` and `probe_elicit`.
//!
//! Both tools are not part of the public MCP schema and may be removed without notice. They touch
//! no files, network or processes.

use std::borrow::Cow;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::Poll;
use std::time::Duration;

use rmcp::model::{
    BooleanSchema, CallToolRequestParams, CallToolResponse, CallToolResult, ClientResult,
    ContentBlock, ElicitRequestParams, ElicitResult, ElicitationAction, ElicitationSchema,
    Implementation, JsonObject, ListToolsResult, PaginatedRequestParams, PrimitiveSchemaDefinition,
    ProtocolVersion, Request, ServerCapabilities, ServerConfig, ServerRequest, Tool,
    ToolAnnotations,
};
use rmcp::serde::Serialize;
use rmcp::serde_json::{self, Value, json};
use rmcp::service::{Peer, PeerRequestOptions, RequestContext, ServiceError};
use rmcp::{ErrorData, RoleServer, ServerHandler};

use crate::guard::bounded;
use crate::{
    ELICIT_TIMEOUT_DEFAULT_MS, ELICIT_TIMEOUT_MAX_MS, ELICIT_TIMEOUT_MIN_MS, MAX_CALLS_IN_FLIGHT,
};

/// Tool name of the echo probe.
const ECHO_NAME: &str = "probe_echo";
/// Tool name of the elicitation probe.
const ELICIT_NAME: &str = "probe_elicit";
/// Longest `probe_echo` message, in Unicode scalar values.
const ECHO_MAX_CHARS: usize = 4096;
/// Prefix of every tool execution error caused by bad arguments.
const INVALID_ARGUMENTS: &str = "invalid arguments:";

const ECHO_DESCRIPTION: &str = "Experimental probe tool, not part of the public AI Track Master API; may be removed without notice. Returns the given message unchanged as a structured result. Reads and writes no files.";
const ELICIT_DESCRIPTION: &str = "Experimental probe tool, not part of the public AI Track Master API; may be removed without notice. Asks the user to confirm a test request through MCP elicitation and reports whether the client supports elicitation and what the user answered. Changes nothing; reads and writes no files.";
const ELICIT_MESSAGE: &str = "AI Track Master probe: please confirm that elicitation works. This is an experimental test request; nothing will be changed.";
const INSTRUCTIONS: &str = "AI Track Master experimental probe server. Its tools (probe_*) are experimental and may be removed without notice.";

/// The probe MCP server handler.
#[derive(Clone)]
pub(crate) struct ProbeServer {
    tools: Arc<[Tool]>,
    in_flight: Arc<AtomicUsize>,
}

/// One slot of the in-flight cap; released on drop, so also when a call is cancelled by the
/// client (the call future is dropped), times out or ends because the input ended.
struct CallSlot(Arc<AtomicUsize>);

impl CallSlot {
    /// Takes a slot without waiting; `None` when [`MAX_CALLS_IN_FLIGHT`] calls are running.
    fn try_acquire(counter: &Arc<AtomicUsize>) -> Option<Self> {
        counter
            .try_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < MAX_CALLS_IN_FLIGHT).then_some(n + 1)
            })
            .ok()
            .map(|_| Self(Arc::clone(counter)))
    }
}

impl Drop for CallSlot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

impl ProbeServer {
    /// Creates the handler with its fixed tool list.
    pub(crate) fn new() -> Self {
        Self {
            tools: Arc::from(vec![echo_tool(), elicit_tool()]),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl ServerHandler for ProbeServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("aitm", env!("CARGO_PKG_VERSION"))
                    .with_title("AI Track Master"),
            )
            .with_protocol_version(ProtocolVersion::V_2025_11_25)
            .with_instructions(INSTRUCTIONS)
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        // Must equal `SUPPORTED_PROTOCOL_VERSIONS` (checked by a unit test).
        Cow::Owned(vec![
            ProtocolVersion::V_2025_06_18,
            ProtocolVersion::V_2025_11_25,
        ])
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.tools.to_vec()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let Some(_slot) = CallSlot::try_acquire(&self.in_flight) else {
            return Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "too many calls in flight: at most {MAX_CALLS_IN_FLIGHT} tools/call requests may run at once; retry later."
            ))])
            .into());
        };
        // The client may cancel the call (`notifications/cancelled`); rmcp then only cancels
        // `context.ct` and drops whatever the handler returns, so the work is raced against it.
        // Dropping the work frees the slot and abandons a pending elicitation.
        let work = async {
            match request.name.as_ref() {
                ECHO_NAME => call_echo(request.arguments.as_ref()),
                ELICIT_NAME => call_elicit(request.arguments.as_ref(), &context.peer).await,
                other => Err(ErrorData::invalid_params(
                    format!("unknown tool: {}", bounded(other)),
                    None,
                )),
            }
        };
        let Some(result) = until_cancelled(work, context.ct.cancelled()).await else {
            // rmcp drops the response of a cancelled request; nothing reaches the client.
            return Err(ErrorData::invalid_request("request cancelled", None));
        };
        let result = result?;
        Ok(result.into())
    }
}

/// Runs `work` until it completes or `cancelled` does; `None` means cancelled (`work` is dropped).
async fn until_cancelled<T>(
    work: impl Future<Output = T>,
    cancelled: impl Future<Output = ()>,
) -> Option<T> {
    let mut work = Box::pin(work);
    let mut cancelled = Box::pin(cancelled);
    std::future::poll_fn(|cx| {
        if cancelled.as_mut().poll(cx).is_ready() {
            return Poll::Ready(None);
        }
        work.as_mut().poll(cx).map(Some)
    })
    .await
}

/// Builds a `Tool` from JSON schema literals.
fn tool(
    name: &'static str,
    title: &str,
    description: &'static str,
    input_schema: Value,
    output_schema: Value,
    idempotent: bool,
) -> Tool {
    let mut tool = Tool::default();
    tool.name = Cow::Borrowed(name);
    tool.title = Some(title.to_owned());
    tool.description = Some(Cow::Borrowed(description));
    tool.input_schema = Arc::new(into_object(input_schema));
    tool.output_schema = Some(Arc::new(into_object(output_schema)));
    tool.annotations = Some(
        ToolAnnotations::new()
            .read_only(true)
            .destructive(false)
            .idempotent(idempotent)
            .open_world(false),
    );
    tool
}

/// Unwraps a JSON object literal; the schemas in this file are always objects.
fn into_object(value: Value) -> JsonObject {
    match value {
        Value::Object(map) => map,
        _ => JsonObject::new(),
    }
}

fn echo_tool() -> Tool {
    tool(
        ECHO_NAME,
        "Probe: echo",
        ECHO_DESCRIPTION,
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "maxLength": ECHO_MAX_CHARS,
                    "description": "Text to echo (at most 4096 Unicode scalar values)."
                }
            },
            "required": ["message"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "message": { "type": "string" },
                "length_chars": { "type": "integer", "minimum": 0 }
            },
            "required": ["message", "length_chars"]
        }),
        true,
    )
}

fn elicit_tool() -> Tool {
    tool(
        ELICIT_NAME,
        "Probe: elicitation",
        ELICIT_DESCRIPTION,
        json!({
            "type": "object",
            "properties": {
                "timeout_ms": {
                    "type": "integer",
                    "minimum": ELICIT_TIMEOUT_MIN_MS,
                    "maximum": ELICIT_TIMEOUT_MAX_MS,
                    "default": ELICIT_TIMEOUT_DEFAULT_MS,
                    "description": "How long to wait for the client's elicitation answer, in milliseconds."
                }
            },
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "outcome": {
                    "type": "string",
                    "enum": ["accepted", "declined", "cancelled", "unsupported", "timeout", "error"]
                },
                "content": {
                    "type": "object",
                    "properties": { "confirm": { "type": "boolean" } },
                    "required": ["confirm"]
                },
                "reason": { "type": "string" }
            },
            "required": ["outcome"]
        }),
        false,
    )
}

/// A tool execution error caused by bad arguments (`isError: true`, not a JSON-RPC error).
fn invalid_arguments(detail: &str) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(format!(
        "{INVALID_ARGUMENTS} {detail}"
    ))])
}

/// Structured result with the serialized JSON as the single text item.
fn structured<T: Serialize>(value: &T) -> Result<CallToolResult, ErrorData> {
    let value = serde_json::to_value(value).map_err(|err| {
        ErrorData::internal_error(format!("cannot serialize result: {err}"), None)
    })?;
    Ok(CallToolResult::structured(value))
}

/// Checks that `arguments` is an object with no key outside `allowed`.
fn check_object<'a>(
    arguments: Option<&'a JsonObject>,
    allowed: &[&str],
) -> Result<Option<&'a JsonObject>, CallToolResult> {
    if let Some(object) = arguments
        && let Some(key) = object.keys().find(|key| !allowed.contains(&key.as_str()))
    {
        return Err(invalid_arguments(&format!(
            "unknown field `{}`.",
            bounded(&key.escape_debug().to_string())
        )));
    }
    Ok(arguments)
}

#[derive(Serialize)]
struct EchoOutput<'a> {
    message: &'a str,
    length_chars: usize,
}

/// `probe_echo`: validates the arguments and echoes the message.
fn call_echo(arguments: Option<&JsonObject>) -> Result<CallToolResult, ErrorData> {
    let arguments = match check_object(arguments, &["message"]) {
        Ok(arguments) => arguments,
        Err(result) => return Ok(result),
    };
    let message = match arguments.and_then(|object| object.get("message")) {
        None => return Ok(invalid_arguments("missing required field `message`.")),
        Some(Value::String(message)) => message,
        Some(_) => return Ok(invalid_arguments("field `message` must be a string.")),
    };
    let length_chars = message.chars().count();
    if length_chars > ECHO_MAX_CHARS {
        return Ok(invalid_arguments(&format!(
            "field `message` is {length_chars} characters long; the limit is {ECHO_MAX_CHARS}."
        )));
    }
    structured(&EchoOutput {
        message,
        length_chars,
    })
}

#[derive(Serialize)]
struct ConfirmContent {
    confirm: bool,
}

#[derive(Serialize)]
struct ElicitOutput {
    outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<ConfirmContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

impl ElicitOutput {
    fn bare(outcome: &'static str) -> Self {
        Self {
            outcome,
            content: None,
            reason: None,
        }
    }

    fn with_reason(outcome: &'static str, reason: String) -> Self {
        Self {
            outcome,
            content: None,
            reason: Some(reason),
        }
    }
}

/// Reads `timeout_ms`; defaults when absent.
fn elicit_timeout_ms(arguments: Option<&JsonObject>) -> Result<u64, CallToolResult> {
    let Some(value) = arguments.and_then(|object| object.get("timeout_ms")) else {
        return Ok(ELICIT_TIMEOUT_DEFAULT_MS);
    };
    match value.as_u64() {
        Some(ms) if (ELICIT_TIMEOUT_MIN_MS..=ELICIT_TIMEOUT_MAX_MS).contains(&ms) => Ok(ms),
        _ => Err(invalid_arguments(&format!(
            "field `timeout_ms` must be an integer from {ELICIT_TIMEOUT_MIN_MS} to {ELICIT_TIMEOUT_MAX_MS}."
        ))),
    }
}

/// `probe_elicit`: asks the client for confirmation and reports a definite outcome.
async fn call_elicit(
    arguments: Option<&JsonObject>,
    peer: &Peer<RoleServer>,
) -> Result<CallToolResult, ErrorData> {
    let arguments = match check_object(arguments, &["timeout_ms"]) {
        Ok(arguments) => arguments,
        Err(result) => return Ok(result),
    };
    let timeout_ms = match elicit_timeout_ms(arguments) {
        Ok(ms) => ms,
        Err(result) => return Ok(result),
    };
    if !supports_form_elicitation(peer) {
        return structured(&ElicitOutput::with_reason(
            "unsupported",
            "client did not declare form-mode elicitation support at initialize".to_owned(),
        ));
    }
    let output = request_confirmation(peer, timeout_ms).await;
    structured(&output)
}

/// Form-mode elicitation is supported when the client declared `elicitation` as an empty object or
/// with a `form` key (MCP 2025-11-25 "Elicitation", Capabilities). `url` alone is not enough.
fn supports_form_elicitation(peer: &Peer<RoleServer>) -> bool {
    peer.peer_info().is_some_and(|info| {
        info.capabilities
            .elicitation
            .as_ref()
            .is_some_and(|elicitation| elicitation.form.is_some() || elicitation.url.is_none())
    })
}

/// Sends `elicitation/create` and maps the answer to an outcome.
async fn request_confirmation(peer: &Peer<RoleServer>, timeout_ms: u64) -> ElicitOutput {
    let schema = ElicitationSchema::builder()
        .required_property(
            "confirm",
            PrimitiveSchemaDefinition::Boolean(
                BooleanSchema::new()
                    .title("Confirm")
                    .description("Set to true to confirm.")
                    .with_default(false),
            ),
        )
        .build();
    let schema = match schema {
        Ok(schema) => schema,
        Err(err) => {
            return ElicitOutput::with_reason(
                "error",
                format!("cannot build request schema: {err}"),
            );
        }
    };
    let params = ElicitRequestParams::FormElicitationParams {
        meta: None,
        message: ELICIT_MESSAGE.to_owned(),
        requested_schema: schema,
    };
    let options = PeerRequestOptions::with_timeout(Duration::from_millis(timeout_ms));
    let request = ServerRequest::ElicitRequest(Request::new(params));
    let handle = match peer.send_request_with_option(request, options).await {
        Ok(handle) => handle,
        Err(err) => return failure_output(&err, timeout_ms),
    };
    match handle.await_response().await {
        Ok(ClientResult::ElicitResult(result)) => answer_output(result),
        Ok(_) => ElicitOutput::with_reason(
            "error",
            "client answered with a result that is not an elicitation result".to_owned(),
        ),
        Err(err) => failure_output(&err, timeout_ms),
    }
}

/// Maps a failed elicitation request to an outcome.
fn failure_output(err: &ServiceError, timeout_ms: u64) -> ElicitOutput {
    match err {
        ServiceError::Timeout { .. } => ElicitOutput::with_reason(
            "timeout",
            format!("no elicitation answer within {timeout_ms} ms"),
        ),
        ServiceError::McpError(error) => ElicitOutput::with_reason(
            "error",
            format!(
                "client answered with JSON-RPC error {}: {}",
                error.code.0,
                bounded(&error.message)
            ),
        ),
        other => ElicitOutput::with_reason(
            "error",
            format!(
                "elicitation request failed: {}",
                bounded(&other.to_string())
            ),
        ),
    }
}

/// Maps the client's `ElicitResult` to an outcome.
fn answer_output(result: ElicitResult) -> ElicitOutput {
    match result.action {
        ElicitationAction::Decline => ElicitOutput::bare("declined"),
        ElicitationAction::Cancel => ElicitOutput::bare("cancelled"),
        ElicitationAction::Accept => {
            let confirm = result
                .content
                .as_ref()
                .and_then(|content| content.get("confirm"))
                .and_then(Value::as_bool);
            match confirm {
                Some(confirm) => ElicitOutput {
                    outcome: "accepted",
                    content: Some(ConfirmContent { confirm }),
                    reason: None,
                },
                None => ElicitOutput::with_reason(
                    "error",
                    "accepted answer has no boolean `confirm` in its content".to_owned(),
                ),
            }
        }
        _ => {
            ElicitOutput::with_reason("error", "client answered with an unknown action".to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SUPPORTED_PROTOCOL_VERSIONS;

    #[test]
    fn supported_versions_match_the_constant() {
        let versions = ProbeServer::new().supported_protocol_versions();
        let names: Vec<&str> = versions.iter().map(ProtocolVersion::as_str).collect();
        assert_eq!(names, SUPPORTED_PROTOCOL_VERSIONS);
    }

    #[test]
    fn tool_names_start_with_probe_and_are_sorted() {
        let server = ProbeServer::new();
        let names: Vec<&str> = server.tools.iter().map(|tool| tool.name.as_ref()).collect();
        assert_eq!(names, [ECHO_NAME, ELICIT_NAME]);
    }
}
