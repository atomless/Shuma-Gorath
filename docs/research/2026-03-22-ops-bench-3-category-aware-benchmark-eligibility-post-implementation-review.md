# OPS-BENCH-3 Post-Implementation Review

Date: 2026-03-22

## Scope reviewed

- `OPS-BENCH-3`
- Plan reference: [`../plans/2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md`](../plans/2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md)

## What landed

1. `benchmark_results_v1` now exposes explicit category-aware tuning eligibility through [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs), including a bounded `tuning_eligibility` contract with `eligible` versus `blocked` status and explicit blockers for non-human classification, category coverage, and protected replay evidence.
2. The benchmark contract now includes a canonical `non_human_category_posture` family in [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs) and [`../../src/observability/benchmark_suite.rs`](../../src/observability/benchmark_suite.rs), with one bounded alignment metric per canonical category keyed to the persisted operator posture rows.
3. Current-instance benchmark materialization, admin benchmark reads, comparison helpers, and reconcile fixtures now all understand the stronger contract in [`../../src/observability/benchmark_comparison.rs`](../../src/observability/benchmark_comparison.rs), [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs), [`../../src/admin/api.rs`](../../src/admin/api.rs), and [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs).
4. The focused verification surface now has a truthful category-aware benchmark gate in [`../../Makefile`](../../Makefile) and [`../../docs/testing.md`](../../docs/testing.md).

## Acceptance check

### 1. `benchmark_results_v1` can say whether a proposed or applied change is safe to judge

Passed.

- The payload now carries `tuning_eligibility.status` plus explicit blockers instead of relying on inference from `escalation_hint` alone.
- Non-human classification readiness, mapped category coverage, and protected replay lineage remain fail-closed blockers before tuning can proceed.
- The same payload still preserves bounded prior-window and candidate comparison semantics for later apply work.

### 2. Category-aware progress or regression is explicit rather than collapsed into global “botness”

Passed.

- The new `non_human_category_posture` family scores per-category alignment against the persisted operator posture scale.
- Category metrics are keyed by canonical category id and remain explicit about `supported`, `partially_supported`, and `not_yet_supported` capability states.
- Benchmark suite and admin benchmark reads now expose that family as part of the machine-first benchmark truth.

## Verification run

1. `make test-benchmark-category-eligibility`
2. `make test-operator-snapshot-foundation`
3. `make test-oversight-reconcile`
4. `git diff --check`

## Architectural review

The tranche stayed within the intended boundary:

- it extended the existing benchmark suite and results contracts instead of creating a separate controller-only category-judgment surface,
- it reused the persisted operator objective rows as the posture source of truth rather than duplicating category policy into benchmark config,
- it kept reconcile conservative by exposing stronger benchmark truth without introducing apply-state mutation early,
- and it reused the existing bounded comparison helpers so later canary apply can build on the same current-versus-reference semantics rather than inventing a second comparison model.

That is the right shape for this stage because the benchmark layer now carries both the category-aware judgment the closed loop needs and the explicit eligibility blockers that prevent premature tuning when evidence is still incomplete or degraded.

## Shortfalls found

One real tranche-local issue surfaced during closeout review and was fixed immediately inside `OPS-BENCH-3`:

1. The first draft of the new category-aware benchmark family pushed `operator_snapshot_v1` beyond its bounded hot-read budget once the richer benchmark payload was nested into the snapshot.
   - Fix: keep the category-aware benchmark truth intact and raise the operator-snapshot hot-read budget modestly from `38 KiB` to `40 KiB` in [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs), then re-run the snapshot foundation gate.
   - Evidence: [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs), [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)

No remaining tranche-local shortfall is left open.

## Next step

Proceed to `OVR-APPLY-1`.
