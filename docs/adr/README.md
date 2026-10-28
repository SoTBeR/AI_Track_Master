# Architecture Decision Records

One file per decision: `NNNN-kebab-title.md` (four digits, starting at `0001`),
created from `docs/templates/adr.md` via `/adr <title>`. ADRs are never deleted;
a changed decision gets a new ADR that supersedes the old one.

Required for: every architectural change, every new dependency, every change to
public MCP/CLI schemas, every change to `unsafe` policy.

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-working-name.md) | Working name, package names and binary name | accepted |
| [0002](0002-rust-language.md) | Rust as the language of core, engine, MCP server and CLI | accepted |
| [0003](0003-dependency-license-policy.md) | Allowed dependencies, licenses and cargo-deny policy | accepted |
| [0004](0004-ci-tools-and-actions.md) | Approved CI tools and GitHub Actions | accepted |
| [0005](0005-clap-cli.md) | clap for the aitm command-line interface | accepted |
| [0006](0006-mcp-server-stack.md) | MCP server stack for the probe server (rmcp, tokio, serde, schemars) | accepted |
