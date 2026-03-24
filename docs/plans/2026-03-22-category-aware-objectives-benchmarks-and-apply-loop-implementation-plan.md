# Category-Aware Objectives, Benchmarks, And Apply Loop Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make operator objectives and benchmark judgment category-aware, then close the first autonomous config loop with bounded canary apply, watch-window comparison, and rollback over protected evidence.

**Architecture:** Extend the existing objective, benchmark, reconcile, and agent contracts instead of adding a second controller system. Persist category posture in `operator_objectives_v1`, add category-aware eligibility and comparison in `benchmark_results_v1`, and implement a new bounded apply state machine that reuses current decision-ledger, config validation, and shared-host agent paths.

**Tech Stack:** Rust admin and observability modules, existing hot-read and decision-ledger contracts, Makefile verification, shared-host live proof workflow.

---

## Guardrails

1. Close the config loop before reopening the later LLM diagnosis/config harness.
2. Keep one bounded config family per apply cycle.
3. Require protected evidence, category coverage, and category-aware benchmark judgment before any apply.
4. Make rollback explicit and durable; do not hide degraded evidence or watch-window failure behind no-op language.

## Task 0: Focused Verification Prep

**Files:**
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add a truthful focused `make` target for category-aware objective contracts, for example `test-operator-objectives-category-contract`.
2. Add a truthful focused `make` target for category-aware benchmark and eligibility logic, for example `test-benchmark-category-eligibility`.
3. Add a truthful focused `make` target for canary apply and rollback semantics, for example `test-oversight-apply`.
4. Add a truthful focused live verification target for the first bounded apply loop if a narrow remote proof path is missing.

**Acceptance criteria:**
1. objectives, benchmarks, and apply-loop behavior all have narrow proof paths.

## Task 1: `OPS-OBJECTIVES-3`

**Files:**
- Modify: `src/observability/operator_snapshot_objectives.rs`
- Modify: `src/observability/operator_objectives_store.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `src/admin/operator_objectives_api.rs`
- Modify: `src/observability/hot_read_contract.rs`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`

**Work:**
1. Extend `operator_objectives_v1` with per-category posture rows on the bounded scale:
   - `allowed`,
   - `tolerated`,
   - `cost_reduced`,
   - `restricted`,
   - `blocked`.
2. Persist objective revisioning with category posture as first-class state rather than overloading `non_human_posture`.
3. Expose category posture in snapshot payloads so later benchmark and tuning work use the same operator truth.
4. Preserve rollout guardrails and keep auto-apply disabled by default until `OVR-APPLY-1` is proven.

**Acceptance criteria:**
1. The controller has a truthful per-category utility function.
2. Operators can distinguish beneficial non-human categories from hostile or expensive ones in backend truth, not just later UI prose.

**Verification:**
1. `make test-operator-objectives-category-contract`
2. `make test-operator-snapshot-foundation`
3. `git diff --check`

## Task 2: `OPS-BENCH-3`

**Files:**
- Modify: `src/observability/benchmark_results.rs`
- Create: `src/observability/benchmark_non_human_categories.rs`
- Modify: `src/observability/benchmark_results_comparison.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `src/admin/benchmark_api.rs`
- Modify: `src/admin/oversight_reconcile.rs`
- Modify: `docs/api.md`

**Work:**
1. Add category-aware benchmark rollups keyed to operator posture and protected coverage.
2. Add benchmark blockers for:
   - incomplete category coverage,
   - stale classification,
   - advisory-only evidence,
   - synthetic-only evidence.
3. Add comparison semantics suitable for canary apply and rollback:
   - baseline,
   - candidate,
   - watch-window result,
   - degraded-evidence result.
4. Keep category-aware improvement or regression explicit in the benchmark payload so reconcile and apply logic can make bounded decisions.

**Acceptance criteria:**
1. `benchmark_results_v1` can say whether a proposed or applied change is safe to judge.
2. Category-aware progress or regression is explicit rather than collapsed into global “botness.”

**Verification:**
1. `make test-benchmark-category-eligibility`
2. `make test-benchmark-results-contract`
3. `make test-oversight-reconcile`
4. `git diff --check`

## Task 3: `OVR-APPLY-1`

Execution source of truth:

- [`2026-03-22-ovr-apply-1-canary-apply-and-rollback-implementation-plan.md`](2026-03-22-ovr-apply-1-canary-apply-and-rollback-implementation-plan.md)

**Files:**
- Create: `src/admin/oversight_apply.rs`
- Modify: `src/admin/oversight_api.rs`
- Modify: `src/admin/oversight_agent.rs`
- Modify: `src/admin/oversight_decision_ledger.rs`
- Modify: `src/admin/oversight_patch_policy.rs`
- Modify: `src/admin/oversight_reconcile.rs`
- Modify: `src/admin/mod.rs`
- Modify: `src/config/mod.rs`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`
- Modify: `docs/deployment.md`

**Work:**
1. Implement the first bounded apply state machine with explicit stages:
   - eligible,
   - canary_applied,
   - watch_window_open,
   - improved,
   - regressed,
   - rollback_applied,
   - refused.
2. Reuse current config validation and decision-ledger contracts.
3. Apply at most one bounded config family per cycle.
4. Require category-aware objective truth, protected evidence, and complete-enough category coverage before apply.
5. Compare watch-window outcome against the protected baseline and rollback on:
   - regression,
   - missing evidence,
   - degraded classification,
   - lost protected eligibility.
6. Keep the first auto-apply path shared-host-only and tightly scoped to config mutation, not code change.

**Acceptance criteria:**
1. Shuma can move from recommend-only to bounded canary apply and rollback.
2. Decision lineage clearly records recommendation, apply, watch result, and rollback if any.
3. Failure or evidence degradation fails closed.

**Verification:**
1. `make test-oversight-apply`
2. `make test-oversight-agent`
3. `make test-live-feedback-loop-remote`
4. `git diff --check`

## Later Work Kept Blocked

### `OVR-AGENT-2`

Keep the later LLM diagnosis/config harness blocked until this plan is complete and live-proved.

When it reopens, it should consume the later reference-stance and run-to-homeostasis methodology in [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md) rather than broadening immediately into stance-agnostic or one-shot automation.

### `OVR-CODE-1`

Keep the later benchmark-driven code-evolution loop blocked until `OVR-AGENT-2` exists and the closed config loop has already demonstrated stable utility.

When it reopens, code-evolution acceptance should continue to treat the strict development reference stance as a regression anchor even when optimizing more permissive target stances.

## Exit Criteria

This plan is complete when:

1. operator objectives are category-aware,
2. benchmark results can authorize or block auto-apply on protected category evidence,
3. the first bounded canary apply and rollback loop exists,
4. the loop is live-proved on shared-host infrastructure,
5. and only then the later LLM diagnosis and code-evolution loops are reconsidered.
