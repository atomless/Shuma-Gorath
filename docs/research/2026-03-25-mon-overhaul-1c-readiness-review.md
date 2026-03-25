Date: 2026-03-25
Status: Proposed

Related context:

- [`2026-03-24-mon-overhaul-1b-monitoring-accountability-post-implementation-review.md`](2026-03-24-mon-overhaul-1b-monitoring-accountability-post-implementation-review.md)
- [`2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md`](2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md)
- [`../plans/2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md`](../plans/2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md)
- [`../plans/2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-plan.md`](../plans/2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# MON-OVERHAUL-1C Readiness Review

## Question

Now that `Traffic` owns traffic visibility and `DIAG-CLEANUP-1` is verified as satisfied, what is the smallest truthful next `Game Loop` slice?

## Conclusion

The next slice should be `MON-OVERHAUL-1C`, and it should remain narrowly focused on:

1. numeric budget usage for the true objective budgets,
2. category target achievement for taxonomy posture outcomes,
3. and trust or actionability projection for evidence readiness and guardrails.

## Why this is now the right next step

The current `Game Loop` tab already projects the top-line accountability contracts, but it still compresses the most important next-level operator questions into generic lists:

1. the true numeric budgets are present in `benchmark_results_v1`, but still read mostly as text rows,
2. category posture is benchmarked as target-vs-current ratios, but is not yet surfaced as a first-class operator view,
3. and trust is present as broad blocker text without enough explicit readiness and guardrail structure.

That leaves the tab accurate but underpowered.

## Existing seams make this a focused tranche

The repo already has the necessary inputs:

1. numeric budget families in [`../../src/observability/benchmark_results_families.rs`](../../src/observability/benchmark_results_families.rs),
2. category posture metrics in [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs),
3. verified-identity and coverage summaries in [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs),
4. operator posture targets in [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs),
5. and the existing dashboard accountability surface in [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte).

So this tranche can stay UI-facing and machine-contract-driven without inventing any new backend payload.

## Decision

Make `MON-OVERHAUL-1C` the next active tranche.

Keep it narrow:

1. no traffic-surface rework,
2. no diagnostics redesign,
3. no objective-model change,
4. just a better Game Loop projection for budgets, category target achievement, and trust/actionability.
