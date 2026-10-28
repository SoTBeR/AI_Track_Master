//! Black-box tests of the `aitm` command-line entry point (task M0-08).
//!
//! Contract: `docs/spec/cli-entrypoint.md` ("Public API", "Behavior", "Tests"), except its
//! `mcp serve` row, "Stdin" paragraph and tests 6-7, which `docs/spec/mcp-probe.md` supersedes
//! (covered in `tests/mcp_probe.rs`).
//! Every test runs the built binary via `env!("CARGO_BIN_EXE_aitm")` with `NO_COLOR=1` and
//! `CLICOLOR_FORCE` removed. Assertions use only the substrings the spec pins; nothing depends on
//! ANSI codes, help wording, line wrapping or path separators.
//!
//! Std only (ADR-0005: clap is a dependency of the binary, not of the tests).

use std::ffi::OsStr;
use std::process::{Command, Output, Stdio};

/// Path of the built `aitm` binary, provided by Cargo for integration tests.
const AITM_BIN: &str = env!("CARGO_BIN_EXE_aitm");

/// Required stderr prefix of every error (spec "Error reporting").
const ERROR_PREFIX: &str = "error:";

/// Builds a command for the binary with a deterministic, color-free environment.
fn aitm_command<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(AITM_BIN);
    cmd.args(args)
        .env("NO_COLOR", "1")
        .env_remove("CLICOLOR_FORCE");
    cmd
}

/// Runs the binary with stdin closed (`/dev/null`/`NUL`) and applies the panic guard (test 11).
fn run<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = aitm_command(args);
    let output = cmd
        .stdin(Stdio::null())
        .output()
        .expect("failed to spawn the aitm binary");
    assert_no_panic(&format!("{cmd:?}"), &output);
    output
}

/// Spec test 11: exit code is never 101 and stderr never contains `panicked`.
fn assert_no_panic(what: &str, output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(
        output.status.code(),
        Some(101),
        "{what}: exit code 101 (panic)\nstderr: {stderr}"
    );
    assert!(
        !stderr.contains("panicked"),
        "{what}: stderr mentions a panic\nstderr: {stderr}"
    );
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is not valid UTF-8")
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is not valid UTF-8")
}

/// Removes exactly one trailing `\n` or `\r\n` (spec "Exact outputs": OS-independent compare).
fn strip_one_line_terminator(s: &str) -> Option<&str> {
    s.strip_suffix("\r\n").or_else(|| s.strip_suffix('\n'))
}

/// Diagnostic text for assertion messages.
fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "aitm {:?}: exit {:?}\nstdout: {}\nstderr: {}",
        args,
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    )
}

/// Asserts a successful invocation: exit 0, empty stderr. Returns stdout.
fn assert_success(args: &[&str]) -> String {
    let output = run(args);
    let info = describe(args, &output);
    assert_eq!(output.status.code(), Some(0), "expected exit 0\n{info}");
    assert!(output.stderr.is_empty(), "expected empty stderr\n{info}");
    stdout_of(&output)
}

/// Asserts a usage error: exit 2, empty stdout, stderr starts with `error:` (spec test 9).
fn assert_usage_error(args: &[&str]) {
    let output = run(args);
    let info = describe(args, &output);
    assert_eq!(output.status.code(), Some(2), "expected exit 2\n{info}");
    assert!(output.stdout.is_empty(), "expected empty stdout\n{info}");
    let stderr = stderr_of(&output);
    assert!(
        stderr.starts_with(ERROR_PREFIX),
        "expected stderr to start with {ERROR_PREFIX:?}\n{info}"
    );
}

// ---------------------------------------------------------------------------------------------
// Test 1: --version / -V
// ---------------------------------------------------------------------------------------------

fn check_version(flag: &str) {
    let stdout = assert_success(&[flag]);
    let line = strip_one_line_terminator(&stdout).unwrap_or_else(|| {
        panic!("aitm {flag}: stdout must end with a line terminator: {stdout:?}")
    });
    assert_eq!(
        line,
        format!("aitm {}", env!("CARGO_PKG_VERSION")),
        "aitm {flag}: unexpected version line"
    );
}

#[test]
fn version_long_flag_prints_exact_version() {
    check_version("--version");
}

#[test]
fn version_short_flag_prints_exact_version() {
    check_version("-V");
}

// ---------------------------------------------------------------------------------------------
// Test 2: --help / -h
// ---------------------------------------------------------------------------------------------

fn check_top_level_help(flag: &str) -> String {
    let stdout = assert_success(&[flag]);
    for needle in ["aitm", "mcp", "--version", "--help"] {
        assert!(
            stdout.contains(needle),
            "aitm {flag}: help lacks {needle:?}\nstdout: {stdout}"
        );
    }
    assert!(
        stdout.to_lowercase().contains("usage"),
        "aitm {flag}: help lacks \"usage\" (case-insensitive)\nstdout: {stdout}"
    );
    stdout
}

#[test]
fn help_long_flag_prints_top_level_help() {
    check_top_level_help("--help");
}

#[test]
fn help_short_flag_prints_top_level_help() {
    check_top_level_help("-h");
}

// ---------------------------------------------------------------------------------------------
// Test 3: no arguments
// ---------------------------------------------------------------------------------------------

#[test]
fn no_arguments_prints_help_and_exits_zero() {
    let stdout = assert_success(&[] as &[&str]);
    assert!(!stdout.is_empty(), "aitm without arguments: empty stdout");
    assert!(
        stdout.contains("mcp"),
        "aitm without arguments: stdout lacks \"mcp\"\nstdout: {stdout}"
    );
}

#[test]
fn no_arguments_prints_the_same_text_as_help() {
    // Spec "Exact outputs": `aitm` prints "the same text as `aitm --help`".
    let no_args = assert_success(&[] as &[&str]);
    let help = assert_success(&["--help"]);
    assert_eq!(no_args, help, "aitm and aitm --help differ");
}

// ---------------------------------------------------------------------------------------------
// Test 4: mcp --help / mcp -h
// ---------------------------------------------------------------------------------------------

fn check_mcp_help(flag: &str) {
    let stdout = assert_success(&["mcp", flag]);
    assert!(
        stdout.contains("serve"),
        "aitm mcp {flag}: help lacks \"serve\"\nstdout: {stdout}"
    );
}

#[test]
fn mcp_help_long_flag_lists_serve() {
    check_mcp_help("--help");
}

#[test]
fn mcp_help_short_flag_lists_serve() {
    check_mcp_help("-h");
}

// ---------------------------------------------------------------------------------------------
// Test 5: mcp serve --help / -h
// ---------------------------------------------------------------------------------------------

fn check_mcp_serve_help(flag: &str) {
    let stdout = assert_success(&["mcp", "serve", flag]);
    assert!(
        !stdout.trim().is_empty(),
        "aitm mcp serve {flag}: empty help"
    );
}

#[test]
fn mcp_serve_help_long_flag_prints_help() {
    check_mcp_serve_help("--help");
}

#[test]
fn mcp_serve_help_short_flag_prints_help() {
    check_mcp_serve_help("-h");
}

// ---------------------------------------------------------------------------------------------
// Tests 6 and 7: superseded
// ---------------------------------------------------------------------------------------------
//
// The M0-08 placeholder behavior of `aitm mcp serve` ("not available yet") is superseded by
// `docs/spec/mcp-probe.md` (section "Changes to the M0-08 contract"). Its tests 1 (stdin null) and
// 2 (stdin held open, then EOF) in `tests/mcp_probe.rs` replace tests 6 and 7 of this file.

// ---------------------------------------------------------------------------------------------
// Test 8: mcp without subcommand
// ---------------------------------------------------------------------------------------------

#[test]
fn mcp_without_subcommand_is_usage_error_mentioning_serve() {
    let args = ["mcp"];
    let output = run(args);
    let info = describe(&args, &output);
    assert_eq!(output.status.code(), Some(2), "expected exit 2\n{info}");
    assert!(output.stdout.is_empty(), "expected empty stdout\n{info}");
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("serve"),
        "expected \"serve\" in stderr\n{info}"
    );
}

// ---------------------------------------------------------------------------------------------
// Test 9: usage errors
// ---------------------------------------------------------------------------------------------

#[test]
fn unknown_subcommand_is_usage_error() {
    assert_usage_error(&["frobnicate"]);
}

#[test]
fn unknown_top_level_flag_is_usage_error() {
    assert_usage_error(&["--frobnicate"]);
}

#[test]
fn unknown_mcp_flag_is_usage_error() {
    assert_usage_error(&["mcp", "--frobnicate"]);
}

#[test]
fn unknown_mcp_serve_flag_is_usage_error() {
    assert_usage_error(&["mcp", "serve", "--frobnicate"]);
}

#[test]
fn unknown_nested_subcommand_is_usage_error() {
    assert_usage_error(&["mcp", "frobnicate"]);
}

#[test]
fn unexpected_positional_after_mcp_serve_is_usage_error() {
    assert_usage_error(&["mcp", "serve", "extra"]);
}

#[test]
fn help_subcommand_is_usage_error() {
    assert_usage_error(&["help"]);
}

#[test]
fn mcp_help_subcommand_is_usage_error() {
    // Spec "Command tree": `aitm mcp help` is a usage error.
    assert_usage_error(&["mcp", "help"]);
}

#[test]
fn version_flag_on_mcp_is_usage_error() {
    assert_usage_error(&["mcp", "--version"]);
}

#[test]
fn version_flag_on_mcp_serve_is_usage_error() {
    // Spec "Command tree": `--version` is not propagated to subcommands; `mcp serve` takes only
    // `--help`/`-h`.
    assert_usage_error(&["mcp", "serve", "--version"]);
}

#[test]
fn future_plan_commands_are_unknown_subcommands() {
    // Spec "Usage errors": commands from PLAN section 6 not yet implemented are unknown in M0.
    for args in [
        &["session"][..],
        &["analyze"],
        &["apply"],
        &["plan"],
        &["render"],
        &["export"],
        &["doctor"],
        &["mcp", "install"],
    ] {
        assert_usage_error(args);
    }
}

// ---------------------------------------------------------------------------------------------
// Test 10: non-UTF-8 argument (Unix only)
// ---------------------------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn non_utf8_argument_is_usage_error() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let arg = OsString::from_vec(vec![0xFF, 0xFE]);
    let mut cmd = aitm_command([&arg]);
    let output = cmd
        .stdin(Stdio::null())
        .output()
        .expect("failed to spawn the aitm binary");
    assert_no_panic("aitm <0xFF 0xFE>", &output);
    let info = format!(
        "aitm <0xFF 0xFE>: exit {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(2), "expected exit 2\n{info}");
    assert!(output.stdout.is_empty(), "expected empty stdout\n{info}");
    assert!(
        !output.stderr.is_empty(),
        "expected non-empty stderr\n{info}"
    );
}
