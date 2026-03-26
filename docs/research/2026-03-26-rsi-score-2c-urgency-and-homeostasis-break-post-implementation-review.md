# RSI-SCORE-2C Urgency And Homeostasis-Break Post-Implementation Review

Date: 2026-03-26
Status: Closed

Related context:

- [`../../src/observability/benchmark_urgency.rs`](../../src/observability/benchmark_urgency.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_comparison.rs`](../../src/observability/benchmark_comparison.rs)
- [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Scope Reviewed

This closeout reviewed `RSI-SCORE-2C`: add explicit urgency scoring and event-triggered homeostasis break so the loop can distinguish current exploit pressure from regression/burn-rate evidence and break homeostasis immediately when the latest completed cycle proves a meaningful exploit regression.

# What Landed

1. Shuma now materializes a first-class `urgency` summary on `benchmark_results_v1`.
2. That summary records:
   - short exploit-success window status,
   - long exploit-success window status,
   - short likely-human harm window status,
   - long likely-human harm window status,
   - explicit `homeostasis_break_status`,
   - and named `homeostasis_break_reasons`.
3. Completed episode rows now preserve:
   - `benchmark_urgency_status`,
   - `homeostasis_break_status`,
   - `homeostasis_break_reasons`,
   - and the explicit restart baseline that the loop re-entered after retain or rollback.
4. `BenchmarkHomeostasisSummary` now carries urgency, break reasons, and restart-baseline lineage instead of only a coarse improving/mixed/flat verdict.
5. Homeostasis now breaks immediately when the latest completed cycle records an urgent exploit regression, even if the older completed-cycle history would otherwise still look flat enough for the old classifier to call it homeostasis.
6. A focused make path, `make test-rsi-score-urgency-and-homeostasis`, now proves the new urgency and break contract end to end.

# Acceptance Review

`RSI-SCORE-2C` required explicit urgency and break reasons in benchmark and episode-comparison contracts, immediate homeostasis break on new exploit regression, preserved restart-baseline lineage, and focused proof.

Those criteria are now satisfied:

1. benchmark and episode-comparison contracts expose urgency and break reasons explicitly;
2. a latest-cycle exploit regression can move the archive summary to `broken` even when older cycles were flat;
3. archive/status surfaces preserve both why homeostasis broke and what baseline was re-entered;
4. and the repo now has the required proof surface through:
   - `make test-rsi-score-urgency-and-homeostasis`
   - `make test-rsi-game-mainline`
   - `make test-oversight-episode-archive`

The key behavioral correction is this:

homeostasis is no longer only a rolling-history label.

It now respects the latest urgent exploit regression as an immediate break condition and keeps the restart baseline machine-visible instead of leaving it implicit in patch history.

# Shortfalls Found

This slice still does not separate scored state, diagnosis, and chosen move lineage cleanly enough for smallest-effective repair selection.

The following planned work remains open:

1. `RSI-SCORE-2D` sharper judge/diagnoser/move-selector separation plus explicit config-ring exhaustion;
2. `RSI-SCORE-2E` Game Loop projection of the richer judge truth.

So this tranche makes urgency and break state explicit, but it does not yet rank bounded moves or emit code-evolution referral when the config ring is exhausted.

# Verification

- `make test-rsi-score-urgency-and-homeostasis`
- `make test-rsi-game-mainline`
- `make test-oversight-episode-archive`
- `git diff --check`
