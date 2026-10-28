//! Transport guards around stdin and stdout.
//!
//! rmcp's stdio path is unbounded and treats a read error like end of input. The wrappers here add
//! the bounded line length of the spec and record failures so that [`crate::serve_stdio`] can
//! report them. Client messages are passed through unchanged.

use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::Notify;

use crate::{MAX_MESSAGE_BYTES, ServeError};

/// First fatal failure seen by a transport wrapper.
#[derive(Default)]
pub(crate) struct Failure {
    slot: Mutex<Option<ServeError>>,
    input_ended: Notify,
}

impl Failure {
    /// Records `error` unless a failure is already recorded.
    pub(crate) fn record(&self, error: ServeError) {
        let mut slot = self
            .slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot.is_none() {
            *slot = Some(error);
        }
    }

    /// Signals that stdin delivered its last byte (end of input or failure).
    pub(crate) fn signal_input_ended(&self) {
        self.input_ended.notify_one();
    }

    /// Completes once stdin has ended.
    pub(crate) async fn input_ended(&self) {
        self.input_ended.notified().await;
    }

    /// Takes the recorded failure, if any.
    pub(crate) fn take(&self) -> Option<ServeError> {
        self.slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}

/// Counts the bytes of the current line and enforces [`MAX_MESSAGE_BYTES`]. The terminator is
/// not counted: neither the `\n` nor a `\r` directly before it (CRLF clients).
#[derive(Debug, Default)]
pub(crate) struct LineCounter {
    /// Bytes since the last `\n`.
    len: usize,
    /// The last byte seen was `\r`.
    last_cr: bool,
}

impl LineCounter {
    /// Scans `bytes`. Returns the index of the first byte that makes the current line too long.
    pub(crate) fn scan(&mut self, bytes: &[u8]) -> Result<(), usize> {
        for (index, &byte) in bytes.iter().enumerate() {
            if byte == b'\n' {
                self.len = 0;
                self.last_cr = false;
                continue;
            }
            self.len += 1;
            self.last_cr = byte == b'\r';
            // A trailing `\r` may still turn out to be half of a CRLF terminator.
            let allowed = MAX_MESSAGE_BYTES + usize::from(self.last_cr);
            if self.len > allowed {
                return Err(index);
            }
        }
        Ok(())
    }
}

/// Pass-through reader that enforces the line limit and records read errors as failures. It stores
/// no line data: memory use is constant. Client messages are never rewritten.
pub(crate) struct GuardedStdin<R> {
    inner: R,
    failure: Arc<Failure>,
    counter: LineCounter,
    /// Set after an oversized line; the next read fails.
    too_long: bool,
}

impl<R> GuardedStdin<R> {
    /// Wraps `inner`.
    pub(crate) fn new(inner: R, failure: Arc<Failure>) -> Self {
        Self {
            inner,
            failure,
            counter: LineCounter::default(),
            too_long: false,
        }
    }

    fn too_long_error() -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, "incoming line is too long")
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for GuardedStdin<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if this.too_long {
            return Poll::Ready(Err(Self::too_long_error()));
        }
        let start = buf.filled().len();
        match Pin::new(&mut this.inner).poll_read(cx, buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => {
                this.failure.record(ServeError::Transport(bounded(&format!(
                    "cannot read stdin: {err}"
                ))));
                this.failure.signal_input_ended();
                Poll::Ready(Err(err))
            }
            Poll::Ready(Ok(())) => {
                let filled = buf.filled();
                if filled.len() == start {
                    this.failure.signal_input_ended();
                    return Poll::Ready(Ok(()));
                }
                match this.counter.scan(&filled[start..]) {
                    Ok(()) => Poll::Ready(Ok(())),
                    Err(at) => {
                        this.failure.record(ServeError::MessageTooLarge {
                            limit_bytes: MAX_MESSAGE_BYTES,
                        });
                        this.failure.signal_input_ended();
                        this.too_long = true;
                        // Deliver the complete lines before the oversized one, then fail.
                        let new = &filled[start..];
                        let keep = new[..at]
                            .iter()
                            .rposition(|&b| b == b'\n')
                            .map_or(0, |p| p + 1);
                        if keep == 0 {
                            return Poll::Ready(Err(Self::too_long_error()));
                        }
                        buf.set_filled(start + keep);
                        Poll::Ready(Ok(()))
                    }
                }
            }
        }
    }
}

/// Longest text of a transport error kept for stderr, in characters.
const MAX_REASON_CHARS: usize = 160;

/// Truncates `text` to [`MAX_REASON_CHARS`] characters.
pub(crate) fn bounded(text: &str) -> String {
    text.chars().take(MAX_REASON_CHARS).collect()
}

/// Writer that records write errors as failures.
pub(crate) struct RecordingWriter<W> {
    inner: W,
    failure: Arc<Failure>,
}

impl<W> RecordingWriter<W> {
    /// Wraps `inner`.
    pub(crate) fn new(inner: W, failure: Arc<Failure>) -> Self {
        Self { inner, failure }
    }

    fn note<T>(&self, result: Poll<io::Result<T>>) -> Poll<io::Result<T>> {
        if let Poll::Ready(Err(err)) = &result {
            self.failure
                .record(ServeError::Transport(format!("cannot write stdout: {err}")));
        }
        result
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for RecordingWriter<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        data: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_write(cx, data);
        this.note(result)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_flush(cx);
        this.note(result)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_shutdown(cx);
        this.note(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use tokio::io::AsyncRead;

    /// Reader that returns scripted chunks, then end of input; an `Err` entry fails the read.
    struct Script(VecDeque<io::Result<Vec<u8>>>);

    impl AsyncRead for Script {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            match self.0.pop_front() {
                None => Poll::Ready(Ok(())),
                Some(Err(err)) => Poll::Ready(Err(err)),
                Some(Ok(chunk)) => {
                    buf.put_slice(&chunk);
                    Poll::Ready(Ok(()))
                }
            }
        }
    }

    /// Reads everything through the guard in 8 KiB reads. Returns the bytes, whether the read
    /// ended with an error, and the recorded failure.
    fn run(chunks: Vec<io::Result<Vec<u8>>>) -> (Vec<u8>, bool, Option<ServeError>) {
        let failure = Arc::new(Failure::default());
        let mut guard = GuardedStdin::new(Script(chunks.into()), Arc::clone(&failure));
        let mut out = Vec::new();
        let mut errored = false;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("test runtime");
        runtime.block_on(std::future::poll_fn(|cx| {
            loop {
                let mut storage = [0_u8; 8192];
                let mut buf = ReadBuf::new(&mut storage);
                match Pin::new(&mut guard).poll_read(cx, &mut buf) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(_)) => {
                        errored = true;
                        return Poll::Ready(());
                    }
                    Poll::Ready(Ok(())) if buf.filled().is_empty() => return Poll::Ready(()),
                    Poll::Ready(Ok(())) => out.extend_from_slice(buf.filled()),
                }
            }
        }));
        (out, errored, failure.take())
    }

    fn line(len: usize, terminator: &[u8]) -> Vec<u8> {
        let mut bytes = vec![b'a'; len];
        bytes.extend_from_slice(terminator);
        bytes
    }

    /// Splits `bytes` into chunks of `size` bytes.
    fn chunked(bytes: &[u8], size: usize) -> Vec<io::Result<Vec<u8>>> {
        bytes.chunks(size).map(|c| Ok(c.to_vec())).collect()
    }

    #[test]
    fn line_at_the_limit_passes_across_chunk_boundaries() {
        for terminator in [&b"\n"[..], &b"\r\n"[..]] {
            let bytes = line(MAX_MESSAGE_BYTES, terminator);
            for size in [8192, 8191, 1000] {
                let (out, errored, failure) = run(chunked(&bytes, size));
                assert!(!errored && failure.is_none(), "{terminator:?} {size}");
                assert_eq!(out, bytes);
            }
        }
    }

    #[test]
    fn line_over_the_limit_fails_and_keeps_earlier_lines() {
        for terminator in [&b"\n"[..], &b"\r\n"[..]] {
            let mut bytes = b"{}\n".to_vec();
            bytes.extend(line(MAX_MESSAGE_BYTES + 1, terminator));
            let (out, errored, failure) = run(chunked(&bytes, 8192));
            assert!(errored, "{terminator:?}");
            assert!(matches!(
                failure,
                Some(ServeError::MessageTooLarge {
                    limit_bytes: MAX_MESSAGE_BYTES
                })
            ));
            assert!(out.len() <= bytes.len() && out.starts_with(b"{}\n"));
            assert!(out.len() <= 3 + MAX_MESSAGE_BYTES + 1);
        }
    }

    #[test]
    fn oversized_line_without_terminator_fails() {
        let (_, errored, failure) = run(chunked(&line(MAX_MESSAGE_BYTES + 2, b""), 8192));
        assert!(errored);
        assert!(matches!(failure, Some(ServeError::MessageTooLarge { .. })));
    }

    #[test]
    fn eof_mid_line_is_a_clean_end_and_passes_the_bytes() {
        let (out, errored, failure) = run(vec![Ok(b"{\"a\":".to_vec())]);
        assert!(!errored && failure.is_none());
        assert_eq!(out, b"{\"a\":");
    }

    #[test]
    fn read_error_is_recorded_and_not_end_of_input() {
        let err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let (out, errored, failure) = run(vec![Ok(b"x\n".to_vec()), Err(err)]);
        assert!(errored);
        assert_eq!(out, b"x\n");
        assert!(matches!(failure, Some(ServeError::Transport(_))));
    }
}
