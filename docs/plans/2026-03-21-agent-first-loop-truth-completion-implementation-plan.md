# Agent-First Loop Truth Completion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the missing benchmark, objective, verified-identity-summary, decision-evidence, and replay-promotion truths that the first shared-host agent loop needs before reconcile and agent execution can be trusted.

**Architecture:** Extend the existing machine-first snapshot and benchmark contracts rather than inventing a second evidence model. Materialize persisted objectives, baseline and candidate benchmark comparison, typed replay-promotion state, and causal decision evidence in the backend control plane so later reconcile and agent work can consume one truthful contract chain.

**Tech Stack:** Rust observability/admin/config modules, existing hot-read projection pipeline, Python adversarial tooling integration points, Makefile verification, repo-native docs and TODO workflow.

---

## Guardrails

1. Do not add auto-apply behavior in this phase.
2. Prefer extending `operator_snapshot_v1` and `benchmark_results_v1` over adding parallel status documents.
3. Preserve the repo rule that telemetry is the map: replay-promotion state must derive from observed emergent findings and reviewed governance, not speculative inventories.
4. Keep exactness and basis metadata honest as these contracts mature.

## Task 0: Focused Verification Prep

**Files:**
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add a truthful focused `make` target for benchmark-comparison completion, for example `test-benchmark-comparison-contract`.
2. Add a truthful focused `make` target for operator objectives and decision-evidence materialization, for example `test-operator-objectives-contract`.
3. Add a truthful focused `make` target for replay-promotion backend contract work, for example `test-replay-promotion-contract`.
4. Keep each target narrow and contract-shaped.

**Acceptance criteria:**
1. The next semantic tranches can prove their exact contract without broad unrelated suites.

## Task 1: `OPS-BENCH-2`

**Files:**
- Modify: `src/observability/benchmark_results.rs`
- Modify: `src/observability/hot_read_contract.rs`
- Modify: `src/observability/hot_read_projection.rs`
- Create: `src/observability/benchmark_history.rs`
- Create: `src/observability/benchmark_comparison.rs`
- Create: `src/observability/benchmark_adversary_effectiveness.rs`
- Create: `src/observability/benchmark_beneficial_non_human.rs`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`

**Work:**
1. Add persisted prior-window or explicit baseline references for local benchmark comparison.
2. Materialize `improvement_status` and current-vs-baseline delta semantics.
3. Add representative adversary scenario-family outputs derived from current adversary-sim evidence.
4. Add beneficial non-human posture metrics that can evaluate verified-identity-aware local stance.
5. Add candidate-vs-current comparison semantics that later tuning or code-evolution loops can reuse.
6. Keep stale, unavailable, and unsupported states explicit rather than collapsing them.

**Acceptance criteria:**
1. `benchmark_results_v1` can answer whether the latest posture improved or regressed against a real baseline.
2. Representative adversary and beneficial non-human families stop reporting `not_yet_supported`.
3. The benchmark contract remains bounded and machine-first.

**Verification:**
1. `make test-benchmark-results-contract`
2. `make test-benchmark-comparison-contract`
3. `git diff --check`

## Task 2: `OPS-SNAPSHOT-2`

**Files:**
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `src/observability/hot_read_contract.rs`
- Modify: `src/observability/hot_read_projection.rs`
- Modify: `src/admin/api.rs`
- Modify: `src/admin/mod.rs`
- Create: `src/observability/operator_objectives_store.rs`
- Create: `src/observability/decision_ledger.rs`
- Create: `src/admin/operator_objectives_api.rs`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`

**Work:**
1. Replace backend-default objectives with persisted site-owned `operator_objectives_v1`.
2. Add objective revision or reference metadata to `operator_snapshot_v1`.
3. Replace placeholder verified-identity summary content with typed observed and policy-relevant summary fields.
4. Extend `recent_changes` into a causal decision or watch-evidence summary that can point to later decision-ledger records.
5. Add durable evidence references needed for later reconcile reasoning and rollback explanation.
6. Keep the snapshot bounded and exactness-tagged.

**Acceptance criteria:**
1. `operator_snapshot_v1` no longer depends on backend-default objectives as the main truth.
2. verified-identity stops being a placeholder-only section.
3. later reconcile and agent work can reconstruct why a proposed change exists and what evidence it relied on.

**Verification:**
1. `make test-operator-snapshot-foundation`
2. `make test-operator-objectives-contract`
3. `make test-telemetry-hot-read-contract`
4. `make test-telemetry-hot-read-projection`
5. `git diff --check`

## Task 3: `ADV-PROMO-1`

**Files:**
- Create: `src/observability/replay_promotion.rs`
- Create: `src/admin/replay_promotion_api.rs`
- Modify: `src/observability/hot_read_projection.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `src/observability/benchmark_results.rs`
- Modify: `src/admin/mod.rs`
- Modify: `scripts/tests/adversarial_promote_candidates.py`
- Modify: `scripts/tests/adversarial_simulation_runner.py`
- Modify: `docs/adversarial-operator-guide.md`

**Work:**
1. Define a typed replay-candidate and promotion-lineage backend contract.
2. Add a bounded backend-readable materialization path from the current Python governance artifacts into Shuma state.
3. Surface promoted, review-pending, and rejected replay-candidate lineage in machine-first contracts rather than only in sidecar JSON artifacts.
4. Keep deterministic-corpus mutation review-gated and explicit.
5. Ensure replay-promotion state is derived from observed findings and reviewed reduction, not from speculative generated cases.

**Acceptance criteria:**
1. promotion lineage is visible to snapshot or benchmark readers,
2. deterministic corpus mutation remains tightly governed,
3. later reconcile and agent work can reason about replay-promotion status without parsing sidecar artifacts directly.

**Verification:**
1. `make test-replay-promotion-contract`
2. `make test-adversarial-promote-candidates`
3. `make test-adversarial-python-unit`
4. `git diff --check`

## Exit Criteria

This plan is complete when:

1. benchmark comparison is truthful,
2. snapshot objectives and decision evidence are persisted and typed,
3. verified-identity summary is no longer a placeholder,
4. replay-promotion lineage is a first-class backend contract,
5. and reconcile or agent work can consume one coherent truth chain.
