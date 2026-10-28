# Spec: CLI entry point (`aitm`, `--version`, `--help`, `mcp serve`)

- Status: accepted
- Layer: platform
- Task: docs/tasks/M0/08-cli-entrypoint.md
- Plan: docs/PLAN.md section 3 ("Тихих сбоев нет"), section 4 (row "CLI"), section 6 "CLI",
  section 9 M0

## What

The command-line entry point of the single `aitm` binary (ADR-0001), built on clap
(ADR-0005). In M0 it provides exactly: top-level `--version`/`-V`, top-level
`--help`/`-h`, and the subcommand `mcp serve`. Until task M0-09 plugs the probe MCP server in,
`mcp serve` fails with a clear "not available yet" error and a non-zero exit code.

## Why

PLAN section 4 fixes clap as the CLI library; PLAN section 6 "CLI" names `mcp serve` as the
service command that starts the MCP server from the same binary; PLAN section 9 M0 requires one
binary whose probe MCP server answers a real client. This task gives M0-09 a parsed command to
attach the server to. PLAN section 3 "Тихих сбоев нет" requires every failure to carry a reason:
the CLI reports every error on stderr with a non-zero exit code and never panics.

This is the first public CLI surface. Any later change to the command tree, output formats or
exit codes defined here requires a versioned migration and an ADR (AGENTS.md "Prohibitions").
Adding new subcommands in later milestones (PLAN section 6 "CLI") is an extension, not a change,
but still needs its own spec.

## Public API

The public API is the observable behavior of the built binary: arguments, stdout, stderr and
exit code. Tests are black-box integration tests in `crates/cli/tests/` that run
`env!("CARGO_BIN_EXE_aitm")` via `std::process::Command`. No library API is exported.

### Command tree (complete for M0)

```
aitm [--version | -V] [--help | -h]
aitm mcp [--help | -h]
aitm mcp serve [--help | -h]
```

Nothing else. In particular:

- There is no `help` subcommand (`aitm help`, `aitm mcp help` are usage errors); clap's
  automatic help subcommand is disabled.
- `--version`/`-V` exists only at the top level; it is not propagated to subcommands.
- There are no global options (no output-format, verbosity, log-level, project or config flags).
- `mcp serve` takes no arguments or options other than `--help`/`-h`.

### Exit codes

| Code | Meaning | Stream |
|---|---|---|
| 0 | Success: version, help, or no-arguments help | stdout |
| 1 | Runtime error: the command was parsed correctly but failed (in M0: "not available yet") | stderr |
| 2 | Usage error: unknown subcommand or flag, missing subcommand, unexpected argument, invalid UTF-8 argument (clap convention) | stderr |

No other exit code is produced on any input. In particular exit code 101 (Rust panic) never
occurs.

### Exact outputs

| Invocation | Exit | stdout | stderr |
|---|---|---|---|
| `aitm --version`, `aitm -V` | 0 | exactly `aitm <CARGO_PKG_VERSION>` followed by one line terminator | empty |
| `aitm --help`, `aitm -h` | 0 | help text (see below) | empty |
| `aitm` (no arguments) | 0 | the same text as `aitm --help` | empty |
| `aitm mcp --help`, `aitm mcp -h` | 0 | help text for `mcp`; contains `serve` | empty |
| `aitm mcp serve --help`, `aitm mcp serve -h` | 0 | help text for `mcp serve` | empty |
| `aitm mcp serve` (until M0-09) | 1 | empty | one line: `error: mcp serve: not available yet (planned in task M0-09)` plus line terminator |
| `aitm mcp` (no subcommand) | 2 | empty | non-empty usage error; contains `serve` |
| any other usage error (rows in "Usage errors" below) | 2 | empty | non-empty; begins with `error:` |

For `aitm mcp` without a subcommand the implementation may either emit clap's missing-subcommand
error or print the `mcp` help to stderr (clap `arg_required_else_help`); only exit 2, empty
stdout and `serve` in stderr are contractual.

`<CARGO_PKG_VERSION>` is the package version of `aitm` (the workspace version); the test reads it
with `env!("CARGO_PKG_VERSION")`. The line terminator is `\n`; tests compare after removing one
trailing `\n` or `\r\n` so that the check is OS-independent.

Top-level help text must contain, as substrings: `aitm`, `mcp`, `--version`, `--help`, and
`usage` matched case-insensitively. Its exact wording is not part of the contract.

Stable assertion substrings for `aitm mcp serve` (until M0-09): stderr starts with `error:` and
contains `not available yet`. The full line in the table is the required text; tests should assert
on these two substrings so that wording tweaks inside the parentheses do not break them.

### Usage errors (exit 2, stderr, empty stdout)

- unknown subcommand: `aitm frobnicate`
- unknown flag: `aitm --frobnicate`, `aitm mcp --frobnicate`, `aitm mcp serve --frobnicate`
- unknown nested subcommand: `aitm mcp frobnicate`
- missing subcommand: `aitm mcp`
- unexpected positional argument: `aitm mcp serve extra`
- disabled help subcommand: `aitm help`
- version flag on a subcommand: `aitm mcp --version`
- (Unix only) a non-UTF-8 argument, e.g. bytes `0xFF 0xFE` as the subcommand

Commands not yet implemented but named in PLAN section 6 (`session`, `analyze`, `apply`, `plan`,
`render`, `export`, `mcp install`, `doctor`) are unknown subcommands in M0 and therefore usage
errors with exit code 2. They are not "not available yet" placeholders.

## Behavior

- **Stdin.** No M0 command reads stdin. The binary behaves identically whether stdin is closed,
  `/dev/null`/`NUL`, an open pipe that never delivers data, or a terminal. `aitm mcp serve` must
  return promptly (well under 10 s) with its error even when stdin is an open pipe; it must not
  block waiting for input. (From M0-09 on, `mcp serve` will speak MCP over stdio; that is M0-09's
  spec.)
- **Panics.** No input path panics. `unwrap()`/`expect()` are not used on anything derived from
  arguments or the environment. Failure to write to stdout or stderr (e.g. a closed pipe) is not a
  panic: the process exits with code 1 if it can, without printing a panic message
  (`println!`/`eprintln!` panic on write errors and must not be used for this output; write via
  `std::io::Write` and handle the `Result`).
- **Error reporting.** Every non-zero exit has a human-readable reason on stderr. Runtime errors
  are printed as one line: `error: <Display of the error>`. Usage errors are printed by clap in
  its own format (which also begins with `error:`).
- **Language.** All output is English.
- **Color.** Output is never colored: clap's `color` feature is off (ADR-0005), so help and errors
  are plain text on every stream. Tests still set `NO_COLOR=1` and remove `CLICOLOR_FORCE` from the
  child environment for robustness and must not depend on ANSI escape codes. No styling is part of
  the contract.
- **Determinism.** Output does not depend on the current directory, locale, time, or environment
  variables, terminal width or the name the binary was invoked under (argv[0]); clap's `wrap_help`
  is off (ADR-0005). Tests must not assert on line wrapping.
- **Side effects.** No M0 command creates, modifies or deletes any file, opens network
  connections, or spawns processes.

### Error type (implementation requirement, not tested directly)

The binary defines a typed error as a plain Rust enum implementing `std::fmt::Display` and
`std::error::Error` (no `thiserror`, no `anyhow`; the only new dependency in this task is clap
per ADR-0005). Required shape for M0:

```rust
/// Runtime error of an `aitm` command (exit code 1).
pub enum CliError {
    /// The command is part of the CLI surface but its implementation has not landed yet.
    /// Display: "<command>: not available yet (planned in task <task>)".
    NotAvailable { command: &'static str, task: &'static str },
    /// Writing to stdout or stderr failed.
    Output(std::io::Error),
}

impl CliError {
    /// Process exit code for this error. All runtime errors map to 1 in M0.
    pub fn exit_code(&self) -> u8;
}
```

Usage errors stay as clap's error type: clap renders them, and `main` exits with clap's exit code
(2) without calling `process::exit`; they are not wrapped in `CliError`. Variant names may be extended by later tasks.

### Code layout (implementation requirement)

- Package `aitm` in `crates/cli`, binary target `aitm` (unchanged from M0-06). Binary-only; no
  library target in this task.
- `src/main.rs` stays thin: parse arguments, dispatch, map `Result` to `std::process::ExitCode`.
- Argument definitions (clap) live in their own module (e.g. `src/args.rs`), and dispatch of
  `mcp serve` lives in a function in its own module (e.g. `src/commands/mcp.rs` with
  `fn serve() -> Result<(), CliError>`), so M0-09 replaces only that function body.
- `#![forbid(unsafe_code)]` in the binary crate.

## Reference

- clap documentation and conventions: usage errors exit with code 2 (`clap::Error::exit_code`),
  help/version exit 0 on stdout.
- BSD `sysexits`/POSIX shell convention: 0 success, 1 general failure, 2 misuse.
- Rust std: exit code 101 on panic (used by tests as a "never panicked" check).
- NO_COLOR convention: https://no-color.org/

## Acceptance criteria

- [x] CLI is built on clap; ADR-0005 (clap) is accepted before merge.
- [ ] Every row of "Exact outputs" and "Usage errors" holds on Linux, Windows and macOS in CI.
- [x] No invocation exits with code 101 or prints `panicked` on stderr.
- [x] `aitm mcp serve` with stdin as an open pipe exits with code 1 within 10 s.
- [x] Existing test `crates/cli/tests/smoke.rs` (no arguments, exit 0) passes unchanged.
- [x] No dependencies added other than clap (and its transitive dependencies) per ADR-0005.
- [x] `CHANGELOG.md` "Unreleased" lists the new CLI surface.

## Tests

Black-box integration tests in `crates/cli/tests/` (e.g. `cli.rs`), std only, using
`Command::new(env!("CARGO_BIN_EXE_aitm"))` with `stdin(Stdio::null())` unless stated, env
`NO_COLOR=1`, `CLICOLOR_FORCE` removed.

1. `--version` and `-V`: exit 0, stdout trimmed == `format!("aitm {}", env!("CARGO_PKG_VERSION"))`,
   stderr empty.
2. `--help` and `-h`: exit 0, stdout contains `aitm`, `mcp`, `--version`, `--help`,
   case-insensitive `usage`; stderr empty.
3. No arguments: exit 0, stdout non-empty and contains `mcp`; stderr empty. (`smoke.rs` stays as is.)
4. `mcp --help`, `mcp -h`: exit 0, stdout contains `serve`; stderr empty.
5. `mcp serve --help`, `mcp serve -h`: exit 0, stdout non-empty; stderr empty.
6. `mcp serve`: exit 1, stdout empty, stderr starts with `error:` and contains `not available yet`.
7. `mcp serve` with `stdin(Stdio::piped())` kept open (handle not dropped): poll `try_wait` for up
   to 10 s; must exit with code 1; on timeout kill the child and fail.
8. `mcp` without subcommand: exit 2, stdout empty, stderr contains `serve`.
9. Each usage error from the "Usage errors" list except `aitm mcp` (test 8) and the non-UTF-8
   case (test 10): exit 2, stdout empty, stderr starts with `error:`. Include `session`, `doctor` and `mcp install` as unknown subcommands.
10. Non-UTF-8 argument (`#[cfg(unix)]`, `OsString::from_vec(vec![0xFF, 0xFE])`): exit 2, stderr
    non-empty.
11. Panic guard applied to every invocation above: exit code is never 101 and stderr never
    contains `panicked`.

No test asserts on ANSI codes, help wording beyond the listed substrings, or line wrapping.

## Out of scope

- All other commands from PLAN section 6 "CLI": `session`, `analyze`, `apply`, `plan`
  (`propose`, `approve`, `commit`), `render`, `export`, `mcp install`, `doctor` (M3 and later).
- The MCP server and stdio transport behind `mcp serve` (M0-09).
- Global options: PLAN section 6 does not require machine-readable (JSON) output, verbosity,
  logging or config flags in M0; they are deferred to the tasks that introduce commands needing
  them.
- Shell completions, man pages, localization.
- Broken-pipe behavior is specified above but not covered by a CI test (hard to make portable).

## Resolved questions

Resolved by the maintainer on 2026-10-28:

1. Runtime errors and "not available yet" share exit code 1 (no separate code).
2. No arguments: help on stdout, exit 0, as the long-term contract.
3. No `help` subcommand; `--help` only. `aitm help` is a usage error (exit 2).
