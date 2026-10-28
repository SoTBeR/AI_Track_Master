//! MCP server of AI Track Master (task M0-09).
//!
//! In M0 this crate holds an **experimental probe server**: a minimal MCP server on `rmcp` over stdio
//! with two tools, `probe_echo` and `probe_elicit`. The probe tools are not part of the public MCP
//! schema and may be removed without notice. The server has no access to files, network or processes.
//!
//! The contract is `docs/spec/mcp-probe.md`; the dependency choices are ADR-0006.
//!
//! # Behavior that `rmcp` does not provide by itself
//!
//! * the supported protocol versions are narrowed to [`SUPPORTED_PROTOCOL_VERSIONS`];
//! * end of input before `initialize` is a clean shutdown, not an error;
//! * one incoming line is limited to [`MAX_MESSAGE_BYTES`];
//! * a stdin read error is a failure, never an end of input.
#![forbid(unsafe_code)]

mod guard;
mod probe;

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use std::future::Future;
use std::task::Poll;

use rmcp::RoleServer;
use rmcp::service::{QuitReason, RunningService, ServerInitializeError};

use crate::guard::{Failure, GuardedStdin, RecordingWriter};

/// Maximum size of one incoming JSON-RPC line on stdin, in bytes, excluding the line terminator.
pub const MAX_MESSAGE_BYTES: usize = 1_048_576;

/// Protocol versions the probe server accepts, oldest first.
pub const SUPPORTED_PROTOCOL_VERSIONS: [&str; 2] = ["2025-06-18", "2025-11-25"];

/// Maximum number of `tools/call` requests in flight (pending elicitations included).
pub const MAX_CALLS_IN_FLIGHT: usize = 32;

/// Default of `probe_elicit.timeout_ms`.
pub const ELICIT_TIMEOUT_DEFAULT_MS: u64 = 120_000;
/// Lower bound of `probe_elicit.timeout_ms`.
pub const ELICIT_TIMEOUT_MIN_MS: u64 = 100;
/// Upper bound of `probe_elicit.timeout_ms`.
pub const ELICIT_TIMEOUT_MAX_MS: u64 = 600_000;

/// How long the runtime may take to shut down after the service ended. Tokio's stdin uses a
/// blocking read that may never return while stdin is open, so the shutdown must not wait for it.
const RUNTIME_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);

/// How long in-flight calls may still answer after stdin has ended.
pub const INPUT_END_GRACE: Duration = Duration::from_millis(1000);

/// Fatal error of the stdio server.
#[non_exhaustive]
#[derive(Debug)]
pub enum ServeError {
    /// The async runtime could not be created.
    Runtime(std::io::Error),
    /// An incoming line exceeded [`MAX_MESSAGE_BYTES`].
    MessageTooLarge {
        /// The limit that was exceeded, in bytes.
        limit_bytes: usize,
    },
    /// Reading stdin or writing stdout failed (not end of input).
    Transport(String),
}

impl fmt::Display for ServeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime(err) => write!(f, "cannot start async runtime: {err}"),
            Self::MessageTooLarge { limit_bytes } => {
                write!(f, "incoming message exceeds {limit_bytes} bytes")
            }
            Self::Transport(reason) => write!(f, "transport error: {reason}"),
        }
    }
}

impl std::error::Error for ServeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(err) => Some(err),
            _ => None,
        }
    }
}

/// Runs the MCP server on stdin/stdout until stdin reaches end of input (`Ok(())`) or a fatal
/// error occurs. Builds and owns its own tokio runtime; the caller is synchronous.
///
/// Nothing but JSON-RPC messages is written to stdout, and nothing is written to stderr.
///
/// # Errors
///
/// [`ServeError::Runtime`] if the runtime cannot be created, [`ServeError::MessageTooLarge`] if a
/// line on stdin exceeds [`MAX_MESSAGE_BYTES`], [`ServeError::Transport`] on a stdin read or stdout
/// write error.
pub fn serve_stdio() -> Result<(), ServeError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(ServeError::Runtime)?;
    let result = runtime.block_on(run());
    runtime.shutdown_timeout(RUNTIME_SHUTDOWN_TIMEOUT);
    result
}

/// Serves one session and maps the outcome to the exit semantics of the spec.
async fn run() -> Result<(), ServeError> {
    let failure = Arc::new(Failure::default());
    let (stdin, stdout) = rmcp::transport::io::stdio();
    let transport = (
        GuardedStdin::new(stdin, Arc::clone(&failure)),
        RecordingWriter::new(stdout, Arc::clone(&failure)),
    );
    match rmcp::serve_server(probe::ProbeServer::new(), transport).await {
        // End of input before `initialize` completed: rmcp reports it as an error.
        Err(ServerInitializeError::ConnectionClosed(_)) => failure.take().map_or(Ok(()), Err),
        // Fixed texts only: no client data (message bodies, methods, ids) reaches stderr.
        Err(err) => Err(failure.take().unwrap_or_else(|| {
            ServeError::Transport(match err {
                ServerInitializeError::ExpectedInitializeRequest(_) => {
                    "client sent a notification or response before initialize".to_owned()
                }
                _ => "initialization failed".to_owned(),
            })
        })),
        // The service ran; a recorded failure wins over the way rmcp saw the connection end.
        Ok(service) => {
            let quit = serve_until_input_ends(service, &failure).await;
            if let Some(err) = failure.take() {
                return Err(err);
            }
            quit.map_or(Ok(()), |quit| {
                quit.map(|_| ())
                    .map_err(|err| ServeError::Transport(format!("service task failed: {err}")))
            })
        }
    }
}

/// Waits for the service to finish. After stdin has ended, in-flight calls get
/// [`INPUT_END_GRACE`] to answer; rmcp alone would wait up to 5 s for a call such as a pending
/// elicitation, but the client is gone. Returns `None` when the grace period ran out.
async fn serve_until_input_ends(
    service: RunningService<RoleServer, probe::ProbeServer>,
    failure: &Failure,
) -> Option<Result<QuitReason, tokio::task::JoinError>> {
    let mut waiting = Box::pin(service.waiting());
    let mut grace = Box::pin(async {
        failure.input_ended().await;
        tokio::time::sleep(INPUT_END_GRACE).await;
    });
    std::future::poll_fn(|cx| {
        if let Poll::Ready(quit) = waiting.as_mut().poll(cx) {
            return Poll::Ready(Some(quit));
        }
        if grace.as_mut().poll(cx).is_ready() {
            return Poll::Ready(None);
        }
        Poll::Pending
    })
    .await
}
