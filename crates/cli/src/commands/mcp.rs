//! `aitm mcp` commands.

use crate::args::McpCommand;
use crate::error::CliError;

/// Dispatches an `aitm mcp` subcommand.
pub fn run(command: McpCommand) -> Result<(), CliError> {
    match command {
        McpCommand::Serve => serve(),
    }
}

/// Starts the MCP server over stdio. Placeholder until task M0-09.
pub fn serve() -> Result<(), CliError> {
    Err(CliError::NotAvailable {
        command: "mcp serve",
        task: "M0-09",
    })
}
