# Monitoring Multi-Loop Benchmark Progress Review

Date: 2026-03-24
Status: Proposed planning driver

Related context:

- [`2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md`](2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md)
- [`../plans/2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md`](../plans/2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md)
- [`../plans/2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md`](../plans/2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md)
- [`../../src/observability/benchmark_comparison.rs`](../../src/observability/benchmark_comparison.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)

# Purpose

Decide whether the Monitoring overhaul should show only the latest loop outcome or a bounded picture of progress over multiple loops against Shuma's benchmark families.

# Findings

## 1. Showing only the last loop is not enough

If Monitoring only shows the latest completed loop or watch window, the operator cannot tell:

1. whether the current result is meaningful improvement or just noise,
2. whether the loop is converging toward a stable plateau,
3. whether recommendations and canary actions are helping repeatedly,
4. or whether rollback and refusal outcomes are part of a recurring pattern.

That would make Monitoring reactive, not accountability-grade.

## 2. The backend already supports bounded multi-loop progress semantics

Shuma already has the main machine-first ingredients:

1. prior-window comparison and `improvement_status` in [`../../src/observability/benchmark_comparison.rs`](../../src/observability/benchmark_comparison.rs),
2. bounded controller and decision history in [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs),
3. and the Monitoring plan already expects current-vs-prior movement and controller history.

So this is not a request for speculative new semantics.
It is a request to project already-real loop-comparison and history truth more explicitly.

## 3. The right analogy to `autoresearch` is progress against benchmark families, not one scalar

Karpathy-style progress display is the right intuition, but Shuma should not collapse everything into one global score.

Shuma has multiple guarded benchmark families:

1. suspicious-origin cost,
2. likely-human friction,
3. verified or tolerated non-human harm,
4. and later category-aware posture alignment.

So the Monitoring analogue of `autoresearch` progress should be:

1. bounded recent loop progress,
2. against multiple benchmark families,
3. plus controller action history,
4. rather than one score or a single latest result.

## 4. Monitoring should show bounded recent loop progress, not become an archive browser

The right Monitoring contract is not full historical analysis.

It is:

1. current status now,
2. recent movement across the last meaningful handful of completed loops,
3. and enough action history to explain whether the loop is learning, flat, or regressing.

Deep historical or raw-event archaeology should stay in Diagnostics or later dedicated history surfaces.

Conclusion:

1. Monitoring should show progress over multiple recent loops,
2. but in a bounded accountability-oriented window such as the last 5-10 completed loops, not an open-ended archive.

# Decisions

1. `MON-OVERHAUL-1` should not stop at "latest loop outcome".
2. Monitoring should show both:
   - current loop status,
   - and bounded progress over recent loops.
3. The progress view should be benchmark-family-oriented, not a single aggregate score.
4. Controller action history should sit alongside benchmark progress so operators can see recommendation/apply/retain/rollback patterns.
5. The first Monitoring contract should remain bounded and operator-readable; raw long-tail history stays out of scope.

# Result

The Monitoring target is now clearer:

1. top-level current status,
2. recent benchmark progress over multiple completed loops,
3. recent controller action history,
4. then the explanatory narrative and category/trust surfaces.

That makes Monitoring closer to an accountability dashboard for a self-improving loop rather than just a snapshot of the latest run.
