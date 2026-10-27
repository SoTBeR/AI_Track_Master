---
name: new-plugin
description: 'Scaffold a built-in plugin (crate, manifest, parameter schema, process/analyze stubs, Python reference stub, test stubs) from templates.'
argument-hint: <Name> <analysis|mixing|mastering>
disable-model-invocation: true
---

# New plugin: $0 ($1)

Plugin contract: `docs/PLAN.md` section 7. Templates: `${CLAUDE_SKILL_DIR}/templates/`.

1. Validate: `$0` is UpperCamelCase; `$1` is one of `analysis`, `mixing`, `mastering`. Derive `snake` (e.g. `ParametricEq` → `parametric_eq`)
   and `kebab` names. Stop if `crates/plugins/<snake>/` exists.
2. If `crates/plugin-sdk/` does not exist yet, tell the maintainer the stubs will not compile until the SDK lands (M2) and ask whether to continue.
3. Copy templates, replacing `{{Name}}`, `{{snake}}`, `{{kebab}}`, `{{category}}`:

   | Template | Destination |
   |---|---|
   | `Cargo.toml.tmpl` | `crates/plugins/<snake>/Cargo.toml` |
   | `manifest.toml.tmpl` | `crates/plugins/<snake>/manifest.toml` |
   | `lib.rs.tmpl` | `crates/plugins/<snake>/src/lib.rs` |
   | `params.rs.tmpl` | `crates/plugins/<snake>/src/params.rs` |
   | `tests.rs.tmpl` | `crates/plugins/<snake>/tests/<snake>.rs` |
   | `reference.py.tmpl` | `reference/plugins/<snake>.py` |
   | `test_reference.py.tmpl` | `reference/tests/test_<snake>.py` |

4. Add the crate to the workspace `members` if the root `Cargo.toml` lists members explicitly.
5. Do not fill in DSP: the stubs are placeholders. Create a task file `docs/tasks/<current M#>/NN-plugin-<kebab>.md` (layer `plugins`)
   from `docs/templates/task.md` so the plugin goes through `/task` (spec → reference → tests → implementation).
6. Report the created files and the task path.

The manifest format and SDK trait in these templates follow the plan's contract; once the plugin-sdk ADR/spec exists (M2),
the SDK is authoritative and the templates must be updated to match.
