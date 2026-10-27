---
name: mastering-domain
description: 'Domain knowledge for the autonomous mastering engine (plan section 3): engine stages, chain order, objective function, "do no harm" check, targets as data, typical problems of AI-generated tracks and home mixes.'
user-invocable: false
---

# Mastering domain

Source: `docs/PLAN.md` section 3 (and 2, 8). The plan is authoritative; this is a summary.

## Principle

The agent cannot hear. It states intent (platform, character, genre, reference), calls `auto_master`, reads the report.
The deterministic engine computes and verifies every parameter. Same input + intent = same result in any client or from the CLI.

## Engine stages

1. **Intake**: format, sample rate, bit depth, duration, channels, silence, DC offset. Invalid input rejected with a clear reason.
2. **Whole-track analysis (first pass)**: integrated/short-term/momentary loudness, LRA, true peak; smoothed perceptually weighted spectrum;
   per-band dynamics (crest factor, PLR); per-band stereo (correlation, width, mono compatibility); transients; sections by energy/density.
3. **Diagnosis** with severity and confidence: clipping, over-limited input, resonances, harshness and sibilance, rumble/infrasonics,
   phase/mono problems, codec HF cutoff and "ringing" (typical for AI generations), noise.
4. **Targets**: delivery targets (loudness, true peak, formats) from the platform profile; tonal and dynamic targets adapted to the material;
   intent and reference shift targets only within defined bounds.
5. **Planning**, fixed order: restoration → corrective EQ → dynamic EQ and resonance suppression → compression (multiband if needed)
   → stereo and low-end mono → saturation or clipper → limiter → resampling → dither. Blocks without a reason are not inserted.
   Initial parameters come from rules on (analysis − targets), section-aware so loud parts are not over-compressed.
6. **Optimization loop**: render → measure → objective → adjust within bounds; limited by iterations and time.
   Objective penalizes loudness and tonal-balance deviation, true-peak overs, transient and dynamics loss beyond budget,
   added distortion, worse mono compatibility.
7. **"Do no harm"**: compare result with input and with the baseline (loudness normalization + limiter) at matched loudness.
   Fail → weaken processing and repeat; still failing → deliver the baseline with a warning.
8. **Delivery**: export all requested formats; lossy formats are encoded, decoded and re-checked (true peak, artifacts).
   Report in JSON and human-readable form: problems, actions and reasons, before/after metrics, warnings.

No silent failures; a partial result is never presented as finished.

## Agent variations

"Warmer", "louder", "gentler on dynamics" map to bounded target shifts and pass the same checks.
Low-level tools exist, but export always goes through "do no harm".

## Targets as data

- Platform targets and material profiles live in versioned files under `profiles/`, not in code.
- Tonal targets are statistics of smoothed spectra of a reference corpus per material type; only aggregated curves are committed, never audio.
- **Warning:** take platform loudness/true-peak values from the platform's current official documentation (cite URL and access date).
  Never use values from model memory; platforms change them.

## Typical input problems

- **AI generations**: codec-like HF cutoff (often 16-20 kHz shelf/brickwall), HF "ringing"/metallic artifacts, smeared transients,
  phasey or unstable stereo, already-loud and limited output, low-level noise.
- **Home mixes**: low-end buildup and rumble, resonances from untreated rooms, harsh 2-5 kHz, sibilance, inconsistent stereo/mono-incompatible bass,
  clipping or over-limiting, DC offset, too-low or too-hot levels.
- Do not chase loudness without headroom; prefer an honest report with warnings.
