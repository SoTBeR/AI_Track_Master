---
name: quality-gates
description: 'Quality levels, release targets, golden-set rules and what blocks a merge (plan sections 8, 10, 11).'
user-invocable: false
---

# Quality gates

Source: `docs/PLAN.md` sections 8, 10 and 11. The plan is authoritative; this is a summary.

## Six verification levels (cheap → expensive)

1. **DSP unit tests**: frequency response vs scipy, null test, linearity, extreme-parameter stability, no NaN/Inf,
   block-size independence, aliasing below threshold.
2. **Measurements**: LUFS, LRA, true peak (also after lossy encoding), PLR/PSR, tonal balance, transient preservation,
   per-band correlation and width, mono compatibility, THD+N, IMD.
3. **Problem detectors**: corpus with injected defects and known labels; recall and precision per detector; thresholds fixed.
4. **Golden set**: manifests (synthetic, freely licensed, own material, AI generations, home mixes, problem input);
   per-track metric tolerances and a final score. **A score drop blocks the merge.**
5. **Agent eval runs**: target attainment, constraint violations, run-to-run stability, model independence
   (different MCP clients and CLI without agent must agree).
6. **Blind listening** (MUSHRA-like, hidden reference and anchor, loudness matched, statistics): calibrates objective weights and
   "do no harm" thresholds in M7.

## Release targets (fixed in M0, refined in M7)

- Integrated loudness within ±0.5 LU of target; true peak not above target in all formats, including lossy.
- No golden-set track worse than the baseline (loudness normalization + limiter) in blind evaluation.
- Statistically significant preference over the baseline; aim: not worse than external automatic mastering systems.

## Merge blockers

- Any failing build on Windows, macOS, Linux; failing tests; clippy warnings; cargo-deny or cargo-audit failures; ruff failures.
- Miri/sanitizer failures where they apply.
- Metric tests (LUFS, true peak, frequency response, golden-set score) regress.
- Any reviewer blocker; edits to tests/reference by an executor; tests fitted to the implementation.
- New dependency without ADR; public MCP/CLI schema change without migration; multi-layer change; oversized PR.
- Any path bypassing "do no harm".
- Merge into main is done only by the maintainer.

## Golden-set rules

- Tolerances and manifests change only through a dedicated task with a reason, never inside the task that needs them changed.
- Only freely licensed or own material; commit manifests and aggregates, not unlicensed audio.
- Compare branch vs main with the same tool versions; report deltas, not only absolute values.
