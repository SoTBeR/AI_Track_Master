//! Smoke test for the single `aitm` binary (task M0-06).
//!
//! Covers acceptance criterion "`cargo build --workspace` produces one binary with the name
//! from M0-01" (ADR-0001: binary name `aitm`). The binary name itself is checked at compile
//! time: `env!("CARGO_BIN_EXE_aitm")` only exists when the package under test defines a binary
//! target named exactly `aitm`, so this file fails to compile otherwise. No separate runtime
//! test of the name is needed.
//!
//! Std only: the task forbids third-party dependencies.

use std::process::{Command, Stdio};

/// Path of the built `aitm` binary, provided by Cargo for integration tests.
const AITM_BIN: &str = env!("CARGO_BIN_EXE_aitm");

#[test]
fn binary_runs_with_no_arguments_and_exits_zero() {
    let output = Command::new(AITM_BIN)
        .stdin(Stdio::null())
        .output()
        .expect("failed to spawn the aitm binary");

    assert!(
        output.status.success(),
        "aitm with no arguments exited with {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
