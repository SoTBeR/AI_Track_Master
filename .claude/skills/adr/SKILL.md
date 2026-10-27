---
name: adr
description: 'Create a new numbered ADR in docs/adr/ from the template (context, decision, alternatives, consequences).'
argument-hint: <title>
disable-model-invocation: true
---

# New ADR: $ARGUMENTS

1. Find the next number: highest `docs/adr/NNNN-*.md` + 1 (start at `0001`).
2. Delegate to the **architect** agent: write `docs/adr/NNNN-<kebab-title>.md` from `docs/templates/adr.md` for "$ARGUMENTS",
   status `proposed`, today's date, grounded in `docs/PLAN.md` (cite sections) and existing ADRs; add a row to the index in
   `docs/adr/README.md`. Return a 5-line summary and open questions.
3. Show the maintainer the summary. The maintainer changes the status to `accepted`.
