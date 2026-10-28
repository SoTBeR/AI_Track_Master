//! Typed runtime errors of `aitm` commands.

use std::fmt;

/// Runtime error of an `aitm` command (exit code 1), (usage errors stay clap's own type).
#[derive(Debug)]
pub enum CliError {
    /// The command is part of the CLI surface but its implementation has not landed yet.
    NotAvailable {
        /// Command path, e.g. `mcp serve`.
        command: &'static str,
        /// Task that will implement it, e.g. `M0-09`.
        task: &'static str,
    },
    /// Writing to stdout or stderr failed.
    Output(std::io::Error),
}

impl CliError {
    /// Process exit code for this error. All runtime errors map to 1 in M0.
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::NotAvailable { .. } | Self::Output(_) => 1,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAvailable { command, task } => {
                write!(f, "{command}: not available yet (planned in task {task})")
            }
            Self::Output(err) => write!(f, "cannot write output: {err}"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Output(err) => Some(err),
            _ => None,
        }
    }
}
