# Benchmark Results Snapshot Projection Post-Implementation Review

Date: 2026-03-20
Status: Completed

Related context:

- [`../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`../plans/2026-03-20-benchmark-suite-v1-design.md`](../plans/2026-03-20-benchmark-suite-v1-design.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`2026-03-20-benchmark-results-contract-post-implementation-review.md`](./2026-03-20-benchmark-results-contract-post-implementation-review.md)
- [`2026-03-20-benchmark-escalation-boundary-post-implementation-review.md`](./2026-03-20-benchmark-escalation-boundary-post-implementation-review.md)

## Review Goal

Review `OPS-BENCH-1-4` against the machine-first snapshot and benchmark plans, confirm that `benchmark_results_v1` is now projected directly into `operator_snapshot_v1` without introducing a second semantic model, and check whether any shortfall remains before the Monitoring-overhaul design discussion.

## What Was Intended

This slice was meant to do three things:

1. project `benchmark_results_v1` directly into `operator_snapshot_v1`,
2. make the standalone `/admin/benchmark-results` read path reuse that same current-instance contract,
3. and close the last benchmark-projection prerequisite before the Monitoring-overhaul discussion.

## What Landed

1. `operator_snapshot_v1` now carries a nested `benchmark_results` section in `src/observability/operator_snapshot.rs`.
2. The benchmark builder was refactored to a section-based helper in `src/observability/benchmark_results.rs`, so the snapshot projection and the benchmark-result contract both reuse the same backend logic.
3. `/admin/benchmark-results` now returns the already-materialized nested benchmark payload from `operator_snapshot_v1` in `src/admin/api.rs`, rather than rebuilding a parallel current-instance summary on the read path.
4. Operator-snapshot component contracts now include the benchmark section in `src/observability/hot_read_contract.rs`.
5. Focused tests now prove:
   - the nested benchmark payload exists in the snapshot contract,
   - the hot-read projection preserves it,
   - the operator-snapshot endpoint exposes it,
   - and the benchmark-results endpoint still returns the same bounded contract.

## Architectural Assessment

### 1. Machine-first layering

This landed in the right direction.

The benchmark contract is now inside the machine-first snapshot instead of floating beside it as an only-partially-related read surface. That keeps future Monitoring, controllers, and later benchmark-driven project evolution anchored to one bounded backend truth.

### 2. No second semantic model

This was the most important architectural requirement, and the delivered shape meets it.

The cleanest outcome here was not merely "same fields in two places." It was "one materialized current-instance benchmark contract, nested in the snapshot, with the standalone endpoint reading that same contract." That is now the state of the codebase.

### 3. Cost and hot-read discipline

The slice stayed aligned with Shuma's telemetry-efficiency work.

No raw event tails were added. No heavier read-time reconstruction was introduced. The projection is derived from already-bounded snapshot sections and served from the existing hot-read materialization flow.

### 4. Monitoring readiness

This slice clears the intended prerequisite.

Monitoring still does not render the new machine-first contract, but that is now a product/design sequencing choice rather than a missing backend prerequisite. The clean-slate Monitoring discussion can now happen against a truthful backend destination.

## Shortfalls Found During Review

One implementation shortfall briefly appeared during the slice:

1. the old snapshot-wrapper benchmark builder became unused after the endpoint switched to the nested snapshot payload, which introduced a new warning.

That shortfall was corrected before closeout by removing the dead wrapper and standardizing on the section-based helper.

No remaining architectural blocker was found in this review.

## Result

Treat `OPS-BENCH-1-4` as complete.

The repo is now in the intended state for the next conversation:

1. `operator_snapshot_v1` is the machine-first Monitoring destination,
2. `benchmark_results_v1` is nested inside that snapshot and preserved as its own contract,
3. `/admin/benchmark-results` reuses the same materialized contract,
4. Monitoring overhaul is now discussion-ready from the backend side,
5. and the remaining active benchmark work is later enrichment (`OPS-BENCH-1-5`), not another prerequisite for the Monitoring design discussion.
