//! `aitm mcp` commands.

use crate::args::McpCommand;
use crate::error::CliError;

/// Dispatches an `aitm mcp` subcommand.
pub fn run(command: McpCommand) -> Result<(), CliError> {
    match command {
        McpCommand::Serve => serve(),
    }
}

/// Starts the MCP server over stdio and blocks until stdin reaches end of input.
pub fn serve() -> Result<(), CliError> {
    aitm_server::serve_stdio().map_err(CliError::Server)
}
