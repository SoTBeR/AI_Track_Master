//! Command-line argument definitions (clap derive).

use clap::{Args, CommandFactory, Parser, Subcommand};

/// AI Track Master: autonomous mastering and mixing (CLI and MCP server).
#[derive(Debug, Parser)]
#[command(
    name = "aitm",
    bin_name = "aitm",
    version,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// The subcommand; without one, help is printed.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Top-level subcommands (complete for M0).
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the MCP server.
    Mcp(McpArgs),
}

/// Arguments of `aitm mcp`.
#[derive(Debug, Args)]
pub struct McpArgs {
    /// The MCP subcommand.
    #[command(subcommand)]
    pub command: McpCommand,
}

/// Subcommands of `aitm mcp`.
#[derive(Debug, Subcommand)]
pub enum McpCommand {
    /// Start the MCP server over stdio.
    Serve,
}

/// Parses the process arguments without exiting.
pub fn parse() -> Result<Cli, clap::Error> {
    Cli::try_parse()
}

/// Top-level help text, identical to the output of `aitm --help`.
pub fn top_level_help() -> String {
    Cli::command().render_help().to_string()
}
