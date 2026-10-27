---
name: mcp-design
description: 'Design rules for the MCP server and CLI (plan section 6): strict schemas, responses with metrics and explanations, async jobs, autonomy modes, write-path protection, audit log, no bypassing the engine.'
user-invocable: false
---

# MCP server and CLI design

Source: `docs/PLAN.md` section 6. The plan is authoritative; this is a summary.

## Tools

| Group | Tools |
|---|---|
| Session | create_session, import_audio, list_tracks, get_session_state, list_versions, restore_version |
| Analysis | analyze_track, analyze_mix, diagnose, compare (A/B, loudness matched), match_reference |
| Processing | list_plugins, describe_plugin, apply_plugin, set_param, route_to_bus |
| High level | auto_master, auto_mix, propose_plan, commit_plan |
| Render/export | render_preview, export (formats, platform targets, post-codec check), qc_report, get_report |
| Jobs | get_job, cancel_job |

Resources: presets, platform targets, analysis reports, session version history (targets are updatable data).
Prompts: one-call autonomous mastering, stem mixing, clipping repair.

## Rules

- Strictly typed schemas (serde + schemars). Plugin parameter schemas generate MCP tool schemas, validation and CLI help.
- Every response carries metrics and a short explanation, e.g. "peak −0.8 dBTP vs target −1.0; lower limiter output".
- MCP schemas are versioned separately from the package; changing a public schema needs a migration and an ADR.
- Long work is async: return a job id, send progress notifications, result fetched later; cancel frees resources.
- Render runs in a worker pool behind `catch_unwind`; one crash never takes down the server.
- The server is a thin shell over the core/engine API; the CLI mirrors every tool (`session`, `analyze`, `apply`,
  `plan propose|approve|commit`, `render`, `export`, `mcp serve`, `mcp install`, `doctor`).
- **No tool may bypass the engine or the "do no harm" check.** Export always runs it.

## Autonomy modes

- **Autonomous (default)**: tools execute immediately; `auto_master` runs the full cycle.
- **Confirmation**: mutating operations go through `propose_plan` (stored, applies nothing). `commit_plan` requests human
  confirmation via MCP elicitation; without elicitation the plan waits for `plan approve` in the CLI, which works only in an
  interactive terminal, reads from the controlling terminal (not args/stdin), has no auto-yes flag and requires a short plan code.
  `commit_plan` without approval is rejected. Approvals are bound to one plan and single use.
- This protects against mistakes and overreach, not against a hostile agent with shell access (that needs client-side isolation).

## Security (all modes)

- Writes only inside the project folder; block escapes including `..` and symlinks.
- Source audio may be read from any path given by the agent or user. `import_audio` accepts only audio (format detected by content)
  and returns metrics and metadata, never arbitrary file contents.
- Parameter ranges enforced from plugin schemas.
- Limits: renders per job, file size, execution time.
- Source files are never modified or deleted.
- Audit log of every tool call.
- ffmpeg runs as a separate process.
