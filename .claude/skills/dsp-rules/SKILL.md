---
name: dsp-rules
description: 'Rules for DSP and analysis code in this project and the method for verifying it (64-bit float, oversampling, phase modes, denormals, NaN/Inf, block-size independence, null tests, scipy comparison, aliasing).'
user-invocable: false
---

# DSP rules

Source: `docs/PLAN.md` sections 7 and 8 (level 1). Full checklist: `${CLAUDE_SKILL_DIR}/checklist.md`.

## Real-time safety in `process`

- No heap allocation, locks, I/O or other syscalls. Preallocate in `prepare`/constructor.
- Enforced in tests with an allocation trap (e.g. `assert_no_alloc`).

## Numerics

- All processing in `f64`. Bit depth is reduced only at export, with dither, exactly once.
- Guard against denormals, NaN and Inf on input and output (flush/sanitize, never propagate).
- Results do not depend on block size: processing N samples in one block or in any split gives identical output (within stated tolerance, ideally bit-identical).
- Support 44.1-192 kHz; coefficients are recomputed per sample rate.
- Parameter changes are smoothed; no clicks under automation.
- Report latency and tail correctly for compensation.

## Nonlinear blocks

- Saturation, clipper, limiter and compressor detectors run with 4-8x oversampling on high-quality (steep, low-ripple) filters.
- Aliasing must stay below the threshold defined in the spec; measure it, do not assume it.
- True-peak limiting uses oversampled peak detection consistent with ITU-R BS.1770.

## Filters and EQ

- EQs support minimum-phase and linear-phase modes. Linear phase at low frequencies accounts for pre-ringing (document the trade-off and limit it).
- Frequency response matches scipy/reference within spec tolerance across sample rates, including near Nyquist (use bilinear prewarping or matched designs as specified).

## Unsafe

- DSP crates declare `#![forbid(unsafe_code)]`. Unsafe only in an isolated module with justification and Miri tests.

## Verification method

1. Frequency response vs scipy (`scipy.signal.freqz`/`sosfreqz`) at several sample rates.
2. Null test: neutral parameters give the input unchanged (bit-exact, or within the residual the spec states).
3. Linearity where expected (superposition, scaling).
4. Stability at extreme parameters; no NaN/Inf.
5. Block-size independence (1, 7, 64, 512, 4096, whole buffer).
6. Aliasing of nonlinear blocks: high sine sweep, measure inharmonic energy below threshold.
7. Comparison with the independent Python reference in `reference/` on shared fixtures.
