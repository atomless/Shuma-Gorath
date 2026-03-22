# OVR-APPLY-1 State-Machine Readiness Review

Date: 2026-03-22
Status: Proposed

Related context:

- [`../plans/2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md`](../plans/2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md)
- [`../plans/2026-03-22-ovr-apply-1-canary-apply-and-rollback-implementation-plan.md`](../plans/2026-03-22-ovr-apply-1-canary-apply-and-rollback-implementation-plan.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/admin/oversight_decision_ledger.rs`](../../src/admin/oversight_decision_ledger.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/config/mod.rs`](../../src/config/mod.rs)

# Goal

Make `OVR-APPLY-1` execution-ready as the first truly closed shared-host config loop: bounded canary apply, protected-evidence watch window, explicit improvement judgment, and exact rollback to the pre-canary config.

# Findings

## 1. The current loop stops one step short of closed control

`oversight_api::execute_reconcile_cycle` already loads snapshot truth, runs bounded reconcile, validates the proposed patch, and records durable decision lineage. What is still missing is the state that lives across cycles:

1. pre-canary baseline capture,
2. one active canary per site,
3. watch-window re-entry,
4. explicit judgment against the captured baseline,
5. exact rollback when evidence degrades or the candidate regresses.

## 2. The dominant extension path is already in the codebase

The cleanest implementation is to extend the existing seams rather than introduce a second controller:

1. keep `/admin/oversight/reconcile` as the recommend-only manual surface,
2. keep the shared-host periodic and post-sim triggers in `oversight_agent`,
3. add one persisted active-canary document owned by a new `oversight_apply` module,
4. extend the existing oversight decision ledger with apply-stage lineage,
5. reuse the existing operator decision and recent-change ledgers for config mutations and rollback visibility.

## 3. Auto-apply must be gated by operator objectives, not only by reconcile pressure

The first bounded apply loop must require all of the following before it mutates config:

1. `operator_objectives_v1.rollout_guardrails.automated_apply_status == canary_only`,
2. reconcile outcome `recommend_patch`,
3. config validation status `valid`,
4. benchmark tuning eligibility `eligible`,
5. no existing active canary state,
6. one bounded patch family only.

That keeps the first loop explicit, opt-in, and site-owned.

## 4. Baseline truth should reuse benchmark comparable snapshots

The repo already has the right comparison substrate in `benchmark_comparison.rs`. The canary state should capture the pre-apply `BenchmarkComparableSnapshot` and later judge the watch window by applying candidate comparison against that stored baseline rather than relying on whatever the current prior-window happens to be.

## 5. Rollback must restore the exact pre-canary config

The first loop should not attempt inverse patches. It should store the full pre-canary persisted config and restore that exact config if:

1. candidate comparison regresses,
2. comparison is unavailable after the watch window,
3. tuning eligibility is lost,
4. non-human classification is no longer ready,
5. category coverage is no longer tuning-grade,
6. the watch-window snapshot is stale or contradictory.

That is the safest first closed loop and avoids clever rollback logic before live proof.

## 6. The manual surface should stay recommend-only

The operator-facing `/admin/oversight/reconcile` route should not silently mutate config. It should expose apply eligibility truthfully, but actual automatic mutation should remain on the shared-host agent path and the internal supervisor path only.

# Decision

Implement `OVR-APPLY-1` as one bounded shared-host-only state machine with:

1. recommend-only manual preview,
2. one persisted active canary,
3. exact pre-canary config snapshot,
4. candidate-vs-baseline watch-window judgment,
5. exact rollback on any loss of trustworthy evidence or any non-improving outcome,
6. durable lineage in both the oversight decision ledger and the operator recent-change path.

# Exit Criteria

This readiness review is complete when:

1. `OVR-APPLY-1` has a dedicated implementation plan with exact files and verification,
2. the later `OVR-AGENT-2` and `OVR-CODE-1` items remain blocked,
3. and the first code tranche can start without reopening the apply-state semantics.
