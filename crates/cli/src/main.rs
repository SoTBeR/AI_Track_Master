//! The `aitm` binary: CLI and MCP server entry point (spec: `docs/spec/cli-entrypoint.md`).

#![forbid(unsafe_code)]

mod args;
mod commands;
mod error;

use std::io::Write;
use std::process::ExitCode;

use args::Command;
use error::CliError;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => report(&err),
    }
}

/// Parses the arguments and dispatches the command.
fn run() -> Result<ExitCode, CliError> {
    let cli = match args::parse() {
        Ok(cli) => cli,
        Err(err) => {
            // Help and version go to stdout with exit 0, usage errors to stderr with exit 2.
            let code = u8::try_from(err.exit_code()).unwrap_or(2);
            return match err.print() {
                Ok(()) => Ok(ExitCode::from(code)),
                Err(io) => Err(CliError::Output(io)),
            };
        }
    };
    match cli.command {
        None => {
            let help = args::top_level_help();
            let mut out = std::io::stdout().lock();
            out.write_all(help.as_bytes())
                .and_then(|()| out.flush())
                .map(|()| ExitCode::SUCCESS)
                .map_err(CliError::Output)
        }
        Some(Command::Mcp(mcp)) => commands::mcp::run(mcp.command).map(|()| ExitCode::SUCCESS),
    }
}

/// Prints a runtime error on stderr (without panicking) and returns its exit code.
fn report(err: &CliError) -> ExitCode {
    // A failed write to stderr cannot be reported anywhere else; the exit code still says "failed".
    let _ = writeln!(std::io::stderr().lock(), "error: {err}");
    ExitCode::from(err.exit_code())
}
