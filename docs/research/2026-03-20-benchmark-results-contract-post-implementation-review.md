# Benchmark Results Contract Post-Implementation Review

Date: 2026-03-20
Status: Complete

Related context:

- [`../plans/2026-03-20-benchmark-suite-v1-design.md`](../plans/2026-03-20-benchmark-suite-v1-design.md)
- [`../plans/2026-03-20-benchmark-suite-v1-implementation-plan.md`](../plans/2026-03-20-benchmark-suite-v1-implementation-plan.md)
- [`../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`2026-03-20-benchmark-suite-contract-post-implementation-review.md`](./2026-03-20-benchmark-suite-contract-post-implementation-review.md)

## Review Scope

Review the first `benchmark_results_v1` implementation slice against:

1. the benchmark-suite design,
2. the machine-first operator-loop direction,
3. the project requirement that Monitoring later project backend benchmark semantics instead of inventing a human-only success model,
4. and the repository requirement that new controller-facing read paths stay bounded and truthful.

## Delivered In This Slice

1. Added a shared backend-owned `benchmark_results_v1` contract in `src/observability/benchmark_results.rs`.
2. Exposed that contract through a read-only `/admin/benchmark-results` endpoint.
3. Built the first bounded result payload directly from the already-materialized `operator_snapshot_v1` hot-read document rather than from raw event tails.
4. Returned explicit machine-facing truth for:
   - subject kind,
   - watch window,
   - baseline-reference availability,
   - per-family statuses,
   - metric current/target/delta fields where already supported,
   - exactness, basis, and capability-gate metadata,
   - and an intentionally explicit placeholder escalation hint.
5. Added focused proof that the read path is `GET`-only and returns `503 benchmark_results_snapshot_missing` without materializing `operator_snapshot_v1` on read.

## Comparison Against Intent

### What matches the plan well

1. The result contract is machine-first, bounded, and schema-versioned.
2. It reuses `operator_snapshot_v1` rather than bypassing the machine-first layering.
3. It keeps unsupported benchmark families explicit rather than synthesizing false precision.
4. It stays summary-oriented and does not widen telemetry tails or create new write-on-read behavior.

### What remains intentionally deferred

1. Baseline-versus-current comparison history is not yet materialized, so `baseline_reference.status` and `improvement_status` remain `not_available`.
2. The escalation hint remains intentionally placeholder until the explicit config-versus-code decision boundary lands in the next benchmark tranche.
3. Benchmark results are currently served through the dedicated admin read path rather than through a separate hot-read document; that remains acceptable for this slice because the payload is bounded and derived from the already-materialized operator snapshot.
4. Monitoring projection work has still not started from this contract yet.

## Architecture Review

The slice is on the right bearing.

Why:

1. The result contract is already machine-consumable enough to anchor later Monitoring and controller work without forcing those layers to re-derive benchmark semantics.
2. Building from the existing operator snapshot preserves the repo's cost discipline, because benchmark reads do not reconstruct meaning from raw telemetry.
3. The unsupported or partial families are surfaced explicitly with capability gates, which preserves telemetry truthfulness during the staged rollout.

## Shortfalls Found

No new architectural blocker was found in this slice.

The remaining benchmark work is the expected next one:

1. materialize the explicit benchmark-driven escalation boundary, and
2. later project `benchmark_results_v1` into `operator_snapshot_v1` and Monitoring without creating a second semantic model.

## Conclusion

This slice meets the intended purpose:

1. `benchmark_results_v1` now exists as a real bounded backend contract,
2. the active backlog can stop describing result-envelope materialization as unfinished work,
3. and the next optimal step is the explicit escalation-boundary tranche rather than Monitoring UI work.
