# Branch protection for `main` (manual GitHub setting)

Settings -> Branches -> Add rule for `main`: require a pull request, require status checks to pass
before merging, require branches to be up to date. Required checks (names as shown by GitHub):

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

Also: disallow force pushes and deletions on `main`. Actions workflow permissions: read-only by default.
Update this list when jobs in `.github/workflows/ci.yml` are added, renamed or removed.

Notes:

- CI runs only after the repository is pushed to GitHub; none of these checks exist before that.
- The `python` job expects `reference/requirements.txt` once `reference/` exists (task M0-15).
- The workflow also triggers on `merge_group`, so the required checks report under a merge queue.
