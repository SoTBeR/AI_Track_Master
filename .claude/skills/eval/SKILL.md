---
name: eval
description: 'Run the golden set and eval scenarios with the eval-runner agent and compare the current branch with main.'
argument-hint: "[scenario or subset]"
disable-model-invocation: true
---

# Eval

Current branch: !`git branch --show-current`

1. Delegate to **eval-runner**: compare the current branch with `main`, subset "$ARGUMENTS" (empty = full golden set).
   Use the eval command from AGENTS.md; if the harness does not exist yet, report that and stop.
2. Show the maintainer (in Russian): verdict per quality-gates, the table of changed or failing rows, findings with evidence.
   Do not change tolerances or code.
