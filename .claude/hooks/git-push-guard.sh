#!/usr/bin/env bash
# PreToolUse hook for Bash: Claude may push only the dev branch to origin.
# - exactly `git push [-u|--set-upstream] origin dev`   -> allow
# - a `git push` invocation naming main, --all, --mirror -> deny (the maintainer updates main)
# - any other push (bare, compound, other branches)      -> ask the maintainer
# Only real invocations count: `git push` at the start of the command or after ; & | ( or a newline.
cmd=$(jq -r '.tool_input.command // empty' 2>/dev/null)
pushes=$(printf '%s\n' "$cmd" | grep -oE '(^|[;&|(])[[:space:]]*git[[:space:]]+push([^;&|)]*)' || true)
[ -n "$pushes" ] || exit 0

decide() {
  jq -n --arg d "$1" --arg r "$2" \
    '{hookSpecificOutput: {hookEventName: "PreToolUse", permissionDecision: $d, permissionDecisionReason: $r}}'
  exit 0
}

if printf '%s\n' "$pushes" | grep -Eq '(^|[[:space:]:/+])main([[:space:]]|$)|--all|--mirror'; then
  decide deny "pushing main is the maintainer's job: ask them to push or merge"
fi
trimmed=$(printf '%s' "$cmd" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')
if printf '%s' "$trimmed" | grep -Eq '^git push( (-u|--set-upstream))? origin dev$'; then
  decide allow "push of dev to origin is allowed"
fi
decide ask "only 'git push origin dev' is pre-approved; confirm this push"
