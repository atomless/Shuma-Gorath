# Agent-First Loop Structural Decomposition Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Decompose the hotspot admin, observability, config, and adversarial-runner modules in behavior-preserving slices so later benchmark, snapshot, reconcile, and agent work stop landing in control-plane monoliths.

**Architecture:** Keep the current public contracts and behavior stable while extracting narrow modules around real existing seams. Leave `src/admin/api.rs`, `src/observability/operator_snapshot.rs`, `src/observability/benchmark_results.rs`, `src/config/controller_action_surface.rs`, `src/admin/adversary_sim.rs`, and `scripts/tests/adversarial_simulation_runner.py` as thin orchestrators over focused helpers instead of replacing their semantics during the refactor.

**Tech Stack:** Rust admin/observability/config modules, Python adversarial tooling, Makefile verification, repo-native docs and TODO workflow.

---

## Guardrails

1. Each structural tranche is behavior-preserving.
2. One hotspot file should be the primary change target per tranche wherever practical.
3. Add or tighten focused `make` targets before the refactor if the current verification path is too broad or ambiguously named.
4. Do not mix contract expansion from `OPS-BENCH-2`, `OPS-SNAPSHOT-2`, `ADV-PROMO-1`, `OVR-RECON-1`, or `OVR-AGENT-1` into these slices.

## Task 0: Focused Verification Prep

**Files:**
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add a truthful focused `make` target for the admin-route-shell refactor, for example `test-admin-api-routing-contract`.
2. Add a truthful focused `make` target for controller-action-surface extraction, for example `test-controller-action-surface`.
3. Add a truthful focused `make` target for adversary-sim domain extraction, for example `test-adversary-sim-domain-contract`.
4. Add a truthful focused `make` target for adversarial-runner architecture refactoring, for example `test-adversarial-runner-architecture`.
5. Prefer composing existing focused tests rather than inventing new broad suites.

**Acceptance criteria:**
1. Target names match the exact contract they verify.
2. Structural tranches can rerun only the tests that prove the preserved boundary.

## Task 1: `ARCH-API-1`

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `src/admin/mod.rs`
- Create: `src/admin/recent_changes_ledger.rs`
- Create: `src/admin/operator_snapshot_api.rs`
- Create: `src/admin/benchmark_api.rs`
- Create: `src/admin/monitoring_api.rs`
- Create: `src/admin/config_api.rs`
- Create: `src/admin/adversary_sim_api.rs`
- Create: `src/admin/diagnostics_api.rs`
- Test: extracted tests moved or duplicated from `src/admin/api.rs`

**Work:**
1. Extract the bounded recent-change ledger helpers out of `src/admin/api.rs` first so later snapshot and decision-ledger work has a clean home.
2. Extract endpoint families behind dedicated modules while keeping auth, rate-limit, request parsing helpers, and top-level routing ownership in `src/admin/api.rs`.
3. Keep response payloads, HTTP status codes, and route paths unchanged.
4. Keep route-family tests co-located with the extracted modules when that reduces `api.rs` growth without hiding contract coverage.

**Acceptance criteria:**
1. `src/admin/api.rs` trends toward a router shell plus shared wrappers.
2. `recent_changes` ledger behavior is unchanged.
3. The admin API contract remains behavior-identical under the focused `make` gate.

**Verification:**
1. `make test-admin-api-routing-contract`
2. `make test-runtime-preflight-unit`
3. `git diff --check`

## Task 2: `ARCH-OBS-1`

**Files:**
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `src/observability/benchmark_results.rs`
- Modify: `src/observability/mod.rs`
- Modify: `src/observability/hot_read_projection.rs`
- Modify: `src/config/controller_action_surface.rs`
- Modify: `src/config/mod.rs`
- Create: `src/observability/operator_snapshot_objectives.rs`
- Create: `src/observability/operator_snapshot_live_traffic.rs`
- Create: `src/observability/operator_snapshot_runtime_posture.rs`
- Create: `src/observability/operator_snapshot_recent_changes.rs`
- Create: `src/observability/operator_snapshot_verified_identity.rs`
- Create: `src/observability/benchmark_results_comparison.rs`
- Create: `src/observability/benchmark_results_families.rs`
- Create: `src/config/controller_action_catalog.rs`
- Create: `src/config/controller_action_guardrails.rs`

**Work:**
1. Keep `operator_snapshot.rs` as the top-level payload builder while extracting section builders into focused modules.
2. Keep `benchmark_results.rs` as the top-level bounded results builder while extracting family-evaluation and comparison helpers.
3. Extract controller-action catalog and guardrail helpers so later reconcile work does not add more logic to `controller_action_surface.rs`.
4. Preserve payload shape and exactness metadata during the structural tranche.

**Acceptance criteria:**
1. Public payload shapes stay unchanged in this tranche.
2. `operator_snapshot.rs`, `benchmark_results.rs`, and `controller_action_surface.rs` stop being the only homes for every related concern.
3. The existing machine-first contracts remain the single source of truth.

**Verification:**
1. `make test-operator-snapshot-foundation`
2. `make test-benchmark-results-contract`
3. `make test-telemetry-hot-read-contract`
4. `make test-telemetry-hot-read-projection`
5. `make test-controller-action-surface`
6. `git diff --check`

## Task 3: `ARCH-SIM-1`

**Files:**
- Modify: `src/admin/adversary_sim.rs`
- Modify: `src/admin/mod.rs`
- Create: `src/admin/adversary_sim_state.rs`
- Create: `src/admin/adversary_sim_lane_runtime.rs`
- Create: `src/admin/adversary_sim_diagnostics.rs`
- Create: `src/admin/adversary_sim_worker_plan.rs`
- Create: `src/admin/adversary_sim_corpus.rs`

**Work:**
1. Extract desired-state, lease-adjacent, and lifecycle state helpers into `adversary_sim_state.rs`.
2. Extract lane-specific runtime planning and worker-result shaping into `adversary_sim_lane_runtime.rs` and `adversary_sim_worker_plan.rs`.
3. Extract diagnostics and operator-facing state shaping into `adversary_sim_diagnostics.rs`.
4. Extract deterministic-corpus and replay-shaping helpers into `adversary_sim_corpus.rs` where they are currently mixed into orchestration code.
5. Keep the current placeholder `bot_red_team` behavior explicit.

**Acceptance criteria:**
1. lifecycle and desired-state behavior remain unchanged,
2. deterministic and Scrapling lane behavior remain unchanged,
3. `adversary_sim.rs` becomes orchestration composition rather than a mixed implementation file.

**Verification:**
1. `make test-adversary-sim-domain-contract`
2. `make test-adversary-sim-lifecycle`
3. `make test-adversary-sim-runtime-surface`
4. `git diff --check`

## Task 4: `ADV-RUN-ARCH-1`

**Files:**
- Modify: `scripts/tests/adversarial_simulation_runner.py`
- Create: `scripts/tests/adversarial_runner/__init__.py`
- Create: `scripts/tests/adversarial_runner/contracts.py`
- Create: `scripts/tests/adversarial_runner/execution.py`
- Create: `scripts/tests/adversarial_runner/evidence.py`
- Create: `scripts/tests/adversarial_runner/discovery_scoring.py`
- Create: `scripts/tests/adversarial_runner/governance.py`
- Create: `scripts/tests/adversarial_runner/reporting.py`
- Create: `scripts/tests/adversarial_runner/runtime_state.py`
- Modify: relevant Python unit tests under `scripts/tests/`

**Work:**
1. Keep `adversarial_simulation_runner.py` as the CLI and orchestration entrypoint.
2. Move contract loading and schema helpers to `contracts.py`.
3. Move scenario execution and profile coordination to `execution.py`.
4. Move evidence shaping and report-row materialization to `evidence.py`.
5. Move discovery scoring and frontier candidate shaping to `discovery_scoring.py`.
6. Move governance checks and artifact safety helpers to `governance.py`.
7. Move report rendering and output helpers to `reporting.py`.

**Acceptance criteria:**
1. runner artifact shapes remain unchanged,
2. promotion and frontier lineage semantics remain unchanged,
3. the orchestrator stops owning all concerns directly.

**Verification:**
1. `make test-adversarial-runner-architecture`
2. `make test-adversarial-python-unit`
3. `make test-adversarial-lane-contract`
4. `git diff --check`

## Exit Criteria

This plan is complete when:

1. all four hotspot refactors have focused implementation homes,
2. later loop-truth and oversight work can land without expanding the original monoliths by default,
3. and every structural tranche is proved by truthful focused `make` verification rather than only by broad umbrella runs.
