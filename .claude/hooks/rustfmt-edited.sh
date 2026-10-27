#!/usr/bin/env bash
# PostToolUse hook: format the single edited .rs file inside the project. Never blocks.
file=$(jq -r '.tool_input.file_path // empty' 2>/dev/null)
case "$file" in
  *.rs) ;;
  *) exit 0 ;;
esac
[ -f "$file" ] || exit 0
case "$(realpath "$file")" in
  "$(realpath "${CLAUDE_PROJECT_DIR:-.}")"/*) ;;
  *) exit 0 ;;
esac
command -v rustfmt >/dev/null 2>&1 || exit 0
# Use the repo's rustfmt.toml once M0 adds it; until then assume the 2024 edition.
if [ -f "${CLAUDE_PROJECT_DIR:-.}/rustfmt.toml" ]; then
  rustfmt "$file" >/dev/null 2>&1
else
  rustfmt --edition 2024 "$file" >/dev/null 2>&1
fi
exit 0
