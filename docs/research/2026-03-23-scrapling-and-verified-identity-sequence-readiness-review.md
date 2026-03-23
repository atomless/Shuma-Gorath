# Scrapling And Verified-Identity Sequence Readiness Review

Date: 2026-03-23
Status: Proposed

Related context:

- [`2026-03-23-scrapling-non-human-category-capability-review.md`](2026-03-23-scrapling-non-human-category-capability-review.md)
- [`../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)
- [`2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md`](2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md)
- [`../plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](../plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)
- [`../../Makefile`](../../Makefile)
- [`../../docs/testing.md`](../../docs/testing.md)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
- [`../../src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs)
- [`../../src/observability/operator_snapshot_verified_identity.rs`](../../src/observability/operator_snapshot_verified_identity.rs)
- [`../../src/observability/benchmark_beneficial_non_human.rs`](../../src/observability/benchmark_beneficial_non_human.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)

# Goal

Make the next pre-Monitoring sequence implementation-ready before code lands:

1. `SIM-SCR-FIT-1`
2. `SIM-SCR-FIT-2`
3. `SIM-SCR-COVER-2`
4. `VID-TAX-1`
5. `VID-TAX-2`
6. `VID-BOT-1`
7. `VID-GUARD-1`

# Findings

## 1. Scrapling already has the right seam, but it lacks the narrow proof target its plan assumes

The current Scrapling ownership seam is already centralized in [`src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs), and the worker-beat payload already carries `category_targets` in [`src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs). That means `SIM-SCR-FIT-1` can stay small.

But the Makefile does not yet expose the focused `SIM-SCR-FIT-1` gate the plan calls for. The existing worker target proves runtime beat/result exchange and real worker execution, which is stronger and broader than the ownership-contract slice.

Readiness implication:

1. add a dedicated `test-adversary-sim-scrapling-category-fit` target,
2. keep it scoped to lane-fulfillment ownership and the worker-plan contract,
3. and keep the heavier worker runtime proof in `test-adversary-sim-scrapling-worker`.

## 2. Verified-identity direction is sound, but its execution ownership is still too implicit

The current verified-identity calibration plan is architecturally correct, but it still leaves too much ambiguity about where each tranche should land.

The existing code already suggests the right ownership split:

1. taxonomy crosswalk belongs first in [`src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs),
2. alignment receipts and summarized operator truth belong in [`src/observability/operator_snapshot_verified_identity.rs`](../../src/observability/operator_snapshot_verified_identity.rs) and the non-human observability path,
3. verified-identity conflict metrics belong in [`src/observability/benchmark_beneficial_non_human.rs`](../../src/observability/benchmark_beneficial_non_human.rs) and [`src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs),
4. fail-closed tuning guardrails belong in [`src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs) and the reconcile admin adapter.

Readiness implication:

1. freeze that module ownership in the plan now,
2. add a narrow verification target for the current calibration seams,
3. and avoid starting `VID-*` work with only a broad mix of policy, telemetry, and reconcile suites.

## 3. The truthful next verified-identity target is readiness, not a future-feature claim

The repo already has good focused verified-identity targets for contracts, config, provider, native verification, proxy trust, policy, telemetry, and annotations. What is missing is the bridge target for the upcoming calibration track.

That target should not imply that alignment receipts, conflict metrics, or guardrails already exist. The honest scope is:

1. current taxonomy-crosswalk seam,
2. current verified-identity snapshot seam,
3. current beneficial benchmark seam,
4. and the current reconcile fail-closed entry point.

Readiness implication:

1. add `test-verified-identity-calibration-readiness`,
2. keep it limited to the current seam tests,
3. and let later `VID-*` tranches deepen behavior without renaming the baseline target.

# Decision

Before `SIM-SCR-FIT-1`, land one short readiness tranche that:

1. adds truthful focused Makefile targets for the Scrapling category-fit and verified-identity calibration-readiness seams,
2. tightens the two implementation plans with explicit file ownership and focused verification expectations,
3. and updates the testing guide so contributors can discover the intended proof path without scanning the Makefile directly.

# Exit Criteria

This readiness review is satisfied when:

1. `SIM-SCR-FIT-1` has a narrow ownership-contract proof path,
2. the `VID-*` plan names exact ownership boundaries for crosswalk, snapshot/receipt, benchmark, and reconcile work,
3. and the Makefile/testing docs expose those proof paths truthfully.
