# Taxonomy And Classification Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Land the canonical non-human taxonomy and the shared classification-confidence contract so Shuma can reason about simulated and observed non-human traffic through one stable machine-first and operator-facing basis.

**Architecture:** Add one runtime-owned taxonomy module as the single source of truth for category ids, labels, descriptions, posture compatibility, and unknown or degraded states. Thread that taxonomy through traffic classification, snapshot, benchmark, and reconcile surfaces so later representativeness, protected evidence, and category-aware tuning work all consume the same bounded contract.

**Tech Stack:** Rust runtime and observability modules, existing hot-read projection pipeline, admin read surfaces, Makefile verification, repo-native docs and TODO workflow.

---

## Guardrails

1. Do not create a second taxonomy in dashboard-only or plan-only form.
2. Keep the first taxonomy seeded and stable; classification quality improves first, taxonomy breadth does not churn by default.
3. Preserve the existing traffic-lane summaries while adding the richer category layer beside them.
4. Fail closed for later tuning if category confidence or evidence freshness is too weak.

## Task 0: Focused Verification Prep

**Files:**
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add a truthful focused `make` target for taxonomy contract work, for example `test-traffic-taxonomy-contract`.
2. Add a truthful focused `make` target for classification-confidence work, for example `test-traffic-classification-contract`.
3. Keep the first target about the canonical category catalog and the second about runtime or snapshot classification receipts.

**Acceptance criteria:**
1. `TRAFFIC-TAX-1` and `TRAFFIC-TAX-2` can be proved without broad unrelated suites.

## Task 1: `TRAFFIC-TAX-1`

**Files:**
- Create: `src/runtime/non_human_taxonomy.rs`
- Modify: `src/runtime/mod.rs`
- Modify: `src/runtime/traffic_classification.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `src/observability/hot_read_contract.rs`
- Modify: `src/observability/mod.rs`
- Modify: `src/admin/operator_snapshot_api.rs`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`

**Work:**
1. Define the canonical category catalog in `src/runtime/non_human_taxonomy.rs`, including:
   - stable machine ids,
   - stable human-facing labels,
   - plain-English descriptions,
   - category family semantics,
   - posture compatibility on the bounded scale `allowed`, `tolerated`, `cost_reduced`, `restricted`, `blocked`.
2. Define explicit non-category states for:
   - `unknown`,
   - `mixed`,
   - `insufficient_evidence`.
3. Add a bounded snapshot-visible taxonomy payload so `operator_snapshot_v1` exposes the exact category basis the backend is using.
4. Keep existing traffic lanes intact for continuity, but make it explicit that taxonomy is the later tuning and operator objective basis.
5. Keep the taxonomy read-only in this tranche; no operator editing surface yet.

**Acceptance criteria:**
1. Shuma has one canonical non-human category catalog.
2. The same machine ids and human-facing labels can later flow into objectives, monitoring, and tuning.
3. Snapshot readers can see the taxonomy basis without inventing their own mapping.

**Verification:**
1. `make test-traffic-taxonomy-contract`
2. `make test-operator-snapshot-foundation`
3. `git diff --check`

## Task 2: `TRAFFIC-TAX-2`

**Files:**
- Modify: `src/runtime/traffic_classification.rs`
- Create: `src/observability/non_human_classification.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `src/observability/operator_snapshot_live_traffic.rs`
- Modify: `src/observability/benchmark_results.rs`
- Modify: `src/admin/oversight_reconcile.rs`
- Modify: `src/observability/hot_read_contract.rs`
- Modify: `src/observability/mod.rs`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`

**Work:**
1. Extend runtime classification to produce bounded category assignments in addition to the existing lane summary.
2. Add shared classification receipts that carry:
   - category id,
   - assignment status,
   - confidence or exactness,
   - basis,
   - supporting evidence references,
   - stale or degraded status.
3. Materialize category-classification summary in `operator_snapshot_v1` for both live and adversary-sim traffic.
4. Thread category-confidence blockers into `benchmark_results_v1` and recommend-only reconcile so later tuning can fail closed when the category layer is weak.
5. Define the cumulative abuse-score relationship explicitly in code and docs:
   - fingerprinting and evidence inform categorization,
   - categorization informs cumulative abuse score,
   - abuse score informs later posture severity.
6. Keep the category model stable while allowing evidence weighting and classifier quality to improve later.

**Acceptance criteria:**
1. Both simulated and observed traffic can be classified against the same taxonomy.
2. Snapshot and benchmark readers can distinguish classified from best-guess or degraded assignments.
3. Later tuning logic has a machine-readable reason to stop when classification trust is insufficient.

**Verification:**
1. `make test-traffic-classification-contract`
2. `make test-benchmark-results-contract`
3. `make test-oversight-reconcile`
4. `git diff --check`

## Exit Criteria

This plan is complete when:

1. Shuma exposes one stable non-human taxonomy,
2. traffic classification produces category-aware receipts with bounded confidence,
3. snapshot and benchmark contracts consume that richer classification,
4. and later tuning gates can treat weak category evidence as a hard blocker rather than a silent fallback.
