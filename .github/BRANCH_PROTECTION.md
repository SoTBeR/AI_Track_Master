# Branch protection (manual GitHub settings)

Branches: `main` holds reviewed work and is updated only by the maintainer through a pull request
from `dev`; `dev` is the integration branch that task branches are merged into (AGENTS.md "Git").
Protection is configured as repository rulesets: Settings -> Rules -> Rulesets -> New branch ruleset.

## Ruleset `main`

- Enforcement status: Active. Bypass list: empty.
- Target: the default branch (`main`).
- Restrict deletions: on.
- Block force pushes: on.
- Require linear history: off (`dev` -> `main` uses merge commits).
- Require signed commits: off (the project uses DCO sign-off, see CONTRIBUTING.md).
- Require a pull request before merging: on.
  - Required approvals: 0 (single maintainer; GitHub does not allow approving your own PR).
  - Require conversation resolution before merging: on.
  - Allowed merge methods: Merge only. Squash or rebase would make `dev` and `main` diverge.
- Require status checks to pass: on, "Require branches to be up to date before merging" on,
  source GitHub Actions. Required checks (names as shown by GitHub):
  - `fmt`
  - `clippy`
  - `deny`
  - `audit`
  - `dev-only-guard`
  - `test (linux-x86_64)`
  - `test (windows-x86_64)`
  - `test (macos-arm64)`
  - `test (macos-x86_64)`
  - `python` (reports success when `reference/` does not exist yet)

## Ruleset `dev`

- Enforcement status: Active. Target: branch name pattern `dev`.
- Restrict deletions: on.
- Block force pushes: on.
- Nothing else: a required pull request or required status checks would block direct pushes to `dev`.
  CI still runs on every push to `dev`.

## Other repository settings

- Settings -> General -> Pull Requests: allow merge commits only.
- Settings -> Actions -> General: workflow permissions read-only; fork pull request workflows require
  approval for all external contributors.

Update the required checks when jobs in `.github/workflows/ci.yml` are added, renamed or removed.

Notes:

- The `python` job expects `reference/requirements.txt` once `reference/` exists (task M0-15).
- The workflow also triggers on `merge_group`, so the required checks report under a merge queue.
