# DSP review checklist

Use item by item for every DSP/analysis change. Mark each OK / FAIL (evidence) / N/A.

## Real-time safety
- [ ] No allocation in `process` (Vec growth, Box, String, format!, collect, clone of heap types).
- [ ] No locks, channels, I/O, logging, time queries in `process`.
- [ ] Allocation-trap test exists and covers `process`.

## Numerics
- [ ] All internal math in `f64`; no accidental `f32` intermediates.
- [ ] Denormal handling on state variables (feedback paths, envelopes, IIR state).
- [ ] NaN/Inf input does not poison state; output is finite; test exists.
- [ ] Division, `ln`, `log10`, `sqrt`, `powf` guarded against zero/negative.
- [ ] dB/linear conversions consistent (20·log10 for amplitude, 10·log10 for power).
- [ ] Block-size independence test with uneven splits.
- [ ] Sample rates 44.1, 48, 88.2, 96, 176.4, 192 kHz tested.
- [ ] Parameter smoothing present; zipper/click test for automation.
- [ ] Latency and tail values reported and verified by impulse test.

## Filters
- [ ] Coefficient formulas match the cited source (e.g. RBJ cookbook, BS.1770 K-weighting); check signs and normalization (a0).
- [ ] Frequency prewarping / Nyquist behavior as specified.
- [ ] Response vs scipy within tolerance at several rates and extreme settings (very low/high f, high Q, large gain).
- [ ] Stability: poles inside unit circle across parameter ranges.
- [ ] Linear-phase mode: latency correct, pre-ringing limited and documented; min-phase mode has no extra latency.

## Nonlinear / oversampling
- [ ] Oversampling factor 4-8x applied to every nonlinear stage, including detectors.
- [ ] Up/down filters: stopband attenuation and passband ripple meet spec.
- [ ] Aliasing measured with high-frequency sine/sweep and below threshold.
- [ ] DC introduced by asymmetric nonlinearity handled.

## Loudness / true peak
- [ ] K-weighting, gating (absolute -70 LUFS, relative -10 LU), block sizes and overlap per BS.1770.
- [ ] EBU Tech 3341/3342 signals pass within tolerance.
- [ ] True peak uses oversampled interpolation (≥4x at 48 kHz) per BS.1770 Annex 2.
- [ ] Limiter output never exceeds true-peak ceiling on the test corpus.

## Tests and references
- [ ] Tolerances come from the spec or standard, not from observed output.
- [ ] Python reference is independent (no translated Rust logic).
- [ ] Null test with neutral parameters.
- [ ] Extreme-parameter stability test.

## Determinism
- [ ] No dependence on thread scheduling, HashMap iteration order or uninitialized state.
- [ ] Cross-platform differences only within stated tolerances (no reliance on fused multiply-add unless specified).
