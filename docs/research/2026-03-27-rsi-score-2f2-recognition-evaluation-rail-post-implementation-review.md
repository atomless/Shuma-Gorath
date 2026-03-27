Date: 2026-03-27
Status: Implemented

Related context:

- [`2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`2026-03-27-rsi-game-arch-1a-restriction-vs-recognition-snapshot-split-post-implementation-review.md`](2026-03-27-rsi-game-arch-1a-restriction-vs-recognition-snapshot-split-post-implementation-review.md)
- [`../plans/2026-03-27-game-loop-category-posture-truth-repair-plan.md`](../plans/2026-03-27-game-loop-category-posture-truth-repair-plan.md)
- [`../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# RSI-SCORE-2F2 Recognition-Evaluation Rail Post-Implementation Review

## What landed

`RSI-SCORE-2F2` is now materially closed.

The recognition-evaluation rail already existed after `RSI-GAME-ARCH-1A`, but this slice tightened it so it now records the categorisation side quest honestly:

1. current Shuma-side collapse to `unknown_non_human` now counts as a real recognition outcome in the evaluator,
2. harness-only `projected_recent_sim_run` placeholders no longer masquerade as degraded category matches,
3. simulator-known category truth remains evaluator-only and does not leak into runtime or restriction scoring,
4. and the repo now states clearly which categories are exact today versus still not exact on the shared hostile path.

The implementation is centered in:

- [`../../src/observability/operator_snapshot_non_human.rs`](../../src/observability/operator_snapshot_non_human.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)

with fixture contract updates in:

- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_beneficial_non_human.rs`](../../src/observability/benchmark_beneficial_non_human.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)

## Why this mattered

Before this slice, the recognition evaluator still had two truth problems:

1. it could fail to report "Shuma only got as far as `unknown_non_human`" because the runtime truthfully marks that bucket as `insufficient_evidence`, not `classified`,
2. and it could wrongly treat recent-run category projection as if Shuma had weakly inferred the category.

Those faults would have made the categorisation side quest look more capable than it really is.

The new behavior is stricter:

1. if Shuma only reaches a coarse current hostile inference, the evaluator says so,
2. if only harness projection exists for a category, the evaluator leaves that row `not_materialized`,
3. and if exact shared-path inference does not exist yet, the gap stays explicit rather than being softened into a fake degraded match.

## Current shared-path inference truth

The repo now says one consistent thing:

1. `indexing_bot` can be exact today when Shuma truly observes declared crawler or verified-search evidence,
2. `unknown_non_human` is the current coarse shared-path fallback for suspicious automation,
3. `ai_scraper_bot`, `automated_browser`, and `http_agent` are still not exact today for undeclared hostile shared-path traffic,
4. and those gaps belong to the recognition side quest, not the restriction scorer.

No new exact hostile-category inference was introduced in this slice.
That was deliberate.
The work here was to stop the evaluator from overstating what Shuma already knows.

## Verification

Focused proof passed:

1. `make test-traffic-classification-contract`
2. `make test-benchmark-results-contract`
3. `make test-dashboard-game-loop-accountability`

## Remaining follow-on

The next active loop work remains restriction-first:

1. `RSI-GAME-ARCH-1B`
2. `RSI-SCORE-2F3`

Those slices now need to recenter the Game Loop around board progression, host cost, Shuma confidence, and the abuse backstop, while keeping this recognition rail clearly secondary and evaluator-only.
