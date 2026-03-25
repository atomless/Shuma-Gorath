Date: 2026-03-24
Status: Implemented

Related plan:

- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)

# Summary

`RSI-GAME-1B` is now landed as a machine-first shortfall-attribution and move-selection bridge rather than a coarse pressure heuristic.

Shuma no longer has to jump straight from "outside budget" to broad static patch-family priority. The benchmark layer now names the shortfall class explicitly, the reconcile layer preserves that explanation, and patch shaping now consumes narrower problem classes rather than the older two-bucket pressure collapse.

# What Landed

## 1. Explicit benchmark shortfall attribution

[`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs) and [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs) now expose richer `escalation_hint` truth:

1. `problem_class`
2. `guidance_status`
3. `tractability`
4. `expected_direction`
5. `trigger_metric_ids`
6. bounded `family_guidance`

That makes the bridge from missed targets to legal move families reviewable instead of implicit.

## 2. Risk-aware family guidance over the legal move ring

[`../../src/config/controller_action_surface.rs`](../../src/config/controller_action_surface.rs) now exposes bounded family risk profiles so benchmark guidance can say which candidate families are lower-friction or higher-risk rather than only listing family ids.

The intent is still conservative:

1. passive signal families first where possible,
2. broader human-visible gates later,
3. and code-evolution-only classification when the miss does not truthfully map to a bounded config move.

## 3. Reconcile now preserves the benchmark semantics

[`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs) now carries `problem_class`, `guidance_status`, and `tractability` through the recommend-only result.

It only upgrades those fields to:

1. `exact_bounded_move`
2. `exact_bounded_config_move`

when a real bounded proposal is actually shaped.

That keeps the operator- and later agent-facing story honest.

## 4. Patch policy now uses explicit problem classes

[`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs) now accepts explicit `OversightProblemClass` values:

1. `LikelyHumanFrictionOverspend`
2. `SuspiciousOriginReachOverspend`
3. `SuspiciousOriginLatencyOverspend`

This lets latency-specific suspicious-origin misses prefer lower-friction signal families first instead of being silently flattened into the broader suspicious-cost bucket.

# Verification

- `make test-oversight-move-selection-policy`
- `make test-oversight-reconcile`
- `make test-rsi-game-contract`
- `git diff --check`

# Follow-on

The next judge-side tranche should remain `RSI-SCORE-1`.

That slice can now define the canonical judge scorecard over an already explicit move-selection bridge rather than trying to do score semantics and shortfall attribution in one change.
