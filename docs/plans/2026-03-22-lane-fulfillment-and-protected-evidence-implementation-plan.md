# Lane Fulfillment And Protected Evidence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the bounded LLM adversary fulfillment layer, prove category-to-lane coverage across Scrapling and LLM modes, and codify which adversary evidence becomes tuning-eligible protected evidence.

**Architecture:** Keep adversary generation split by lane role. Scrapling remains the crawler and scraper workhorse. Add a bounded containerized LLM lane behind a pluggable backend contract for the categories Scrapling cannot credibly fulfill. Materialize fulfillment and coverage receipts against the canonical taxonomy, then mark only protected, deterministic, category-backed evidence as eligible for later autonomous tuning.

**Tech Stack:** Rust adversary-sim control plane, Python adversarial runner modules, existing replay-promotion contract, frontier-backed LLM APIs as the initial reference backend, optional later local-model backend parity, Makefile verification.

---

## Guardrails

1. Do not let raw LLM findings count as tuning-grade evidence.
2. Do not let lane-local labels become the tuning truth; the canonical taxonomy stays authoritative.
3. Keep the first LLM tranche category-fulfillment-scoped, not a general freeform adversary runtime.
4. Preserve the current capability-safe boundary: the LLM lane must stay out of the request path and behind bounded tooling.

## Task 0: Focused Verification Prep

**Files:**
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add a truthful focused `make` target for bounded LLM fulfillment-contract work, for example `test-adversarial-llm-fit`.
2. Add a truthful focused `make` target for category-coverage receipts, for example `test-adversarial-coverage-receipts`.
3. Add a truthful focused `make` target for protected-evidence rules, for example `test-protected-tuning-evidence`.

**Acceptance criteria:**
1. Lane-fulfillment, coverage, and protected-evidence work each have a narrow proof path.

## Task 1: `SIM-LLM-FIT-1`

**Files:**
- Create: `src/admin/adversary_sim_llm_lane.rs`
- Modify: `src/admin/adversary_sim.rs`
- Modify: `src/admin/adversary_sim_lane_runtime.rs`
- Modify: `src/admin/adversary_sim_worker_plan.rs`
- Modify: `src/admin/mod.rs`
- Create: `scripts/tests/adversarial_runner/llm_fulfillment.py`
- Modify: `scripts/tests/adversarial_runner/contracts.py`
- Modify: `scripts/tests/adversarial_runner/execution.py`
- Modify: `scripts/tests/adversarial/frontier_action_contract.v1.json`
- Modify: `scripts/tests/adversarial/container_runtime_profile.v1.json`
- Modify: `docs/adversarial-operator-guide.md`

**Work:**
1. Define the bounded LLM fulfillment contract with:
   - browser mode,
   - request mode,
   - explicit tool or capability envelope,
   - pluggable backend identifiers,
   - frontier-backed initial reference execution.
2. Add runtime planning hooks so the adversary-sim control plane can request bounded LLM fulfillment work without turning the full LLM actor on yet.
3. Make backend choice explicit and typed:
   - `frontier_reference`,
   - `local_candidate`,
   - and degraded or unavailable states.
4. Keep the first local-model path optional and non-authoritative until parity is proven.

**Acceptance criteria:**
1. Shuma has a concrete bounded LLM fulfillment actor contract rather than a vague future lane.
2. Browser-vs-request mode is explicit and testable.
3. Frontier is the initial reference backend for the high-capability categories.

**Verification:**
1. `make test-adversarial-llm-fit`
2. `make test-adversarial-runner-architecture`
3. `make test-adversary-sim-runtime-surface`
4. `git diff --check`

## Task 2: `SIM-FULFILL-1`

**Files:**
- Modify: `src/runtime/non_human_taxonomy.rs`
- Modify: `src/admin/adversary_sim_lane_runtime.rs`
- Modify: `src/admin/adversary_sim_worker_plan.rs`
- Create: `src/observability/non_human_lane_fulfillment.rs`
- Modify: `scripts/tests/adversarial/coverage_contract.v2.json`
- Modify: `scripts/tests/adversarial/scenario_intent_matrix.v1.json`
- Modify: `scripts/tests/adversarial_runner/evidence.py`
- Modify: `docs/adversarial-operator-guide.md`

**Work:**
1. Freeze the category-to-lane matrix in code and fixtures.
2. For each canonical category, record:
   - Scrapling mode if sufficient,
   - bounded LLM browser mode if required,
   - bounded LLM request mode if required,
   - unresolved gap if neither lane is yet credible.
3. Materialize a machine-readable fulfillment summary that later snapshot and benchmark work can reuse.
4. Keep unresolved gaps explicit instead of silently over-claiming representativeness.

**Acceptance criteria:**
1. Shuma can say which lane is intended to represent which category.
2. Gaps are explicit and machine-readable.

**Verification:**
1. `make test-adversarial-lane-contract`
2. `make test-adversarial-scenario-review`
3. `make test-adversarial-llm-fit`
4. `git diff --check`

## Task 3: `SIM-COVER-1`

**Files:**
- Create: `src/observability/non_human_coverage.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `src/observability/benchmark_results.rs`
- Modify: `src/observability/replay_promotion.rs`
- Modify: `src/observability/mod.rs`
- Modify: `scripts/tests/adversarial_runner/reporting.py`
- Modify: `scripts/tests/adversarial_runner/evidence.py`
- Modify: `scripts/tests/adversarial/coverage_contract.v2.json`
- Modify: `docs/api.md`

**Work:**
1. Materialize bounded coverage receipts by canonical category.
2. Record:
   - covered,
   - partial,
   - stale,
   - unavailable,
   - and uncovered states.
3. Distinguish raw lane execution from category-backed fulfilled coverage.
4. Surface coverage summary into snapshot and benchmark payloads so later objective and apply logic can use it directly.

**Acceptance criteria:**
1. The system can say which canonical categories are currently covered well enough for tuning.
2. Partial or stale category coverage is machine-readable and blocks later apply.

**Verification:**
1. `make test-adversarial-coverage-receipts`
2. `make test-benchmark-results-contract`
3. `make test-operator-snapshot-foundation`
4. `git diff --check`

## Task 4: `SIM-PROTECTED-1`

**Files:**
- Modify: `src/observability/replay_promotion.rs`
- Modify: `src/observability/non_human_coverage.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `src/observability/benchmark_results.rs`
- Modify: `src/admin/oversight_reconcile.rs`
- Modify: `src/admin/oversight_patch_policy.rs`
- Modify: `src/admin/replay_promotion_api.rs`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`

**Work:**
1. Add explicit protected-evidence eligibility metadata to adversary evidence summaries.
2. Make `synthetic_traffic` tuning-ineligible in machine-readable contract form.
3. Treat replay-promoted LLM lineage as the protected path for emergent discoveries, while raw frontier attempts remain advisory.
4. Thread protected-evidence and coverage blockers into benchmark and reconcile outputs.
5. Preserve the observed-telemetry rule: protection status must derive from materialized lineage, not from lane self-assertion.

**Acceptance criteria:**
1. Reconcile and later apply logic can distinguish contract-test evidence from tuning-grade evidence.
2. Synthetic ineligibility and replay-promotion promotion are explicit backend facts.

**Verification:**
1. `make test-protected-tuning-evidence`
2. `make test-replay-promotion-contract`
3. `make test-oversight-reconcile`
4. `git diff --check`

## Exit Criteria

This plan is complete when:

1. the bounded LLM fulfillment actor exists as a typed contract,
2. category-to-lane fulfillment is explicit,
3. category coverage receipts are visible to snapshot and benchmark readers,
4. and only protected category-backed evidence can later authorize autonomous tuning.
