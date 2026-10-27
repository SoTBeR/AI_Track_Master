# Spec: <module or feature>

- Status: draft | accepted
- Layer: <layer>
- Task: docs/tasks/<M#>/<NN-slug>.md
- Plan: docs/PLAN.md section N

## What

What the module does, in a few sentences.

## Why

The problem it solves and the plan requirement behind it.

## Public API

Exact signatures (Rust) or schema (MCP/CLI) that tests are written against.

```rust
// pub fn ...
```

## Behavior

Inputs, outputs, units, ranges, defaults, error cases. For DSP: sample rates,
block-size independence, latency and tail, oversampling, phase mode.

## Reference

Independent source of truth: standard (e.g. ITU-R BS.1770, EBU Tech 3341),
paper, scipy function, or Python reference in `reference/`.

## Acceptance criteria

- [ ] Measurable criterion with tolerance.

## Tests

Test list for test-author: unit, proptest, null, block-size, NaN/Inf, reference comparison.

## Out of scope

## Open questions
