Date: 2026-03-27
Status: Active review driver

Related context:

- [`2026-03-27-rsi-game-arch-1f-restriction-tuning-purity-post-implementation-review.md`](2026-03-27-rsi-game-arch-1f-restriction-tuning-purity-post-implementation-review.md)
- [`2026-03-27-game-loop-restriction-tuning-purity-live-gap-review.md`](2026-03-27-game-loop-restriction-tuning-purity-live-gap-review.md)
- [`2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`2026-03-22-sim-protected-1-protected-tuning-evidence-post-implementation-review.md`](2026-03-22-sim-protected-1-protected-tuning-evidence-post-implementation-review.md)
- [`../plans/2026-03-27-game-loop-live-protected-scrapling-evidence-plan.md`](../plans/2026-03-27-game-loop-live-protected-scrapling-evidence-plan.md)
- [`../../src/observability/replay_promotion.rs`](../../src/observability/replay_promotion.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)

# Objective

Capture the next live blocker now that restriction tuning purity is fixed: strong Scrapling runtime board evidence is still not treated as protected tuning evidence, so the controller remains unable to make a bounded move.

# Live evidence

Fresh local `/admin/benchmark-results` now shows:

1. `overall_status = outside_budget`,
2. `problem_class = scrapling_exploit_progress_gap`,
3. `evidence_quality.status = high_confidence`,
4. `evidence_quality.attribution_status = surface_native_shared_path`,
5. `evidence_quality.recent_window_support_status = reproduced_recently`,
6. but `tuning_eligibility.status = blocked`,
7. and the only remaining blockers are:
   1. `protected_lineage_missing`,
   2. `protected_tuning_evidence_not_ready`.

That means the loop is now blocked by one specific controller safety model.

# What the code shows

## 1. Protected tuning evidence still only becomes eligible through replay-promotion lineage

[`../../src/observability/replay_promotion.rs`](../../src/observability/replay_promotion.rs) currently grants `tuning_eligible=true` only when:

1. promoted protected lineage exists,
2. replay-promotion thresholds have passed,
3. and owner review is not pending.

That is the entire protected-evidence model today.

## 2. Scrapling runtime evidence is not yet a first-class protected basis

Even after `RSI-GAME-ARCH-1F`, the live board can show:

1. localized breach loci,
2. shared-path runtime attribution,
3. sufficient samples,
4. and reproduced recent-window support,

but the protected-evidence summary still stays `advisory_only` unless replay-promotion lineage has been materialized separately.

## 3. This is now the narrow blocker preventing a real bounded RSI config move

The live controller has already moved past:

1. recognition-side contamination,
2. simulator-persona leakage,
3. and vague exploit evidence quality.

So the next question is no longer “is the board strong enough?”

It is:

1. when should strong live Scrapling runtime evidence itself count as protected?
2. and how do we do that without weakening the existing safety rules for:
   1. `synthetic_traffic`,
   2. raw frontier or LLM discoveries,
   3. and any simulator metadata leakage into restriction tuning?

# Main finding

The current protected-evidence model is too narrow for the newer Scrapling-first Game Loop.

It correctly excludes:

1. synthetic harness traffic,
2. and advisory discovery-only lineage,

but it is now also excluding the exact runtime-native Scrapling evidence path that the stricter board-state loop is supposed to act on.

# Required direction

The next repair should:

1. preserve synthetic exclusion and advisory frontier/LLM exclusion,
2. preserve the rule that simulator-known labels do not enter runtime or tuning,
3. add an explicit protected basis for strong live Scrapling runtime evidence,
4. and let the controller become tuning-eligible when that runtime-native protected basis is satisfied.

# Consequence for sequencing

`RSI-GAME-ARCH-1G` is now the next live Game Loop blocker and should land before:

1. legacy-surface retirement in `RSI-GAME-ARCH-1E`,
2. and before broader combined-attacker mainline claims.
