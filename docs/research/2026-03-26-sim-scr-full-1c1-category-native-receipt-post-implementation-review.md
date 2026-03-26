# SIM-SCR-FULL-1C1 Category-Native Receipt Post-Implementation Review

Date: 2026-03-26
Status: Closed

Related context:

- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/operator_snapshot_non_human.rs`](../../src/observability/operator_snapshot_non_human.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../../docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Scope Reviewed

This closeout reviewed `SIM-SCR-FULL-1C1`: replace projected recent-run category receipts with category-native adversary-sim evidence or explicit degraded status so category posture metrics no longer copy one aggregate run envelope into every observed category.

# What Landed

1. `summarize_non_human_classification()` no longer invents exact per-category request outcomes from aggregate adversary-sim scope when `request_outcomes.by_non_human_category` is absent.
2. Recent-run-only Scrapling category receipts now remain visible as:
   - `assignment_status=classified`
   - `exactness=derived`
   - `basis=projected_recent_sim_run`
   - `degradation_status=degraded`
   - zeroed request counts rather than fabricated forwarded or short-circuited totals
3. Operator snapshot non-human readiness now treats those receipts as partial or degraded rather than ready or current evidence.
4. Benchmark results now fail closed from those degraded receipts:
   - category posture metrics remain `insufficient_evidence`
   - tuning eligibility stays blocked by `non_human_classification_not_ready`
   - and the category coverage surface can still say the categories were seen, but only as stale coverage rather than current receipt-backed truth.

# Review Result

The tranche now matches the intended contract:

1. recent Scrapling run summaries still preserve attacker-lane category presence,
2. but Shuma no longer turns that presence into made-up category-native request math,
3. and controller-grade benchmark logic no longer reads those projected envelopes as ready evidence.

This is the right first repair because it removes a false-confidence path before the controller or Game Loop are asked to reason more deeply about attacker progress.

# Shortfalls Found

One downstream benchmark test and one snapshot test were still asserting the old projected-`ready` behavior.

Those tests were corrected in the same tranche so the repo now freezes the new fail-closed contract explicitly.

The next open work remains:

1. `SIM-SCR-FULL-1C2` to make Scrapling defense-surface truth controller-grade,
2. and `SIM-SCR-FULL-1C3` to audit whether the current operator-facing Scrapling picture is truthful or misleading.

# Verification

- `make test-traffic-classification-contract`
- `make test-benchmark-results-contract`
- `make test-adversary-sim-scrapling-category-fit`
