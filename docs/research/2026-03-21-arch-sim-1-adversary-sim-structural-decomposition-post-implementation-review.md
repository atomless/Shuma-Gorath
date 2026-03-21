# ARCH-SIM-1 Post-Implementation Review

Date: 2026-03-21
Plan reference: `docs/plans/2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md`
Task: `ARCH-SIM-1`

## Scope Delivered

`ARCH-SIM-1` completed the planned structural decomposition of `src/admin/adversary_sim.rs` into focused modules while preserving the existing runtime and admin control contracts:

- `src/admin/adversary_sim_state.rs`
  - desired-state, lifecycle, ownership, and reconciliation helpers
- `src/admin/adversary_sim_corpus.rs`
  - deterministic corpus, runtime profile, and corpus metadata helpers
- `src/admin/adversary_sim_diagnostics.rs`
  - diagnostics payload builders plus lane diagnostics state/payload types
- `src/admin/adversary_sim_worker_plan.rs`
  - Scrapling worker plan/result contracts and shared runtime summary structs
- `src/admin/adversary_sim_lane_runtime.rs`
  - lane request-path generation, heartbeat/tick execution, worker-result application, and Scrapling worker planning
- `src/admin/adversary_sim.rs`
  - thin public orchestration surface, shared constants, public re-exports, and tranche-local regression tests

## Plan Conformance Review

### 1. Desired-state and lifecycle helpers

Delivered as planned in `src/admin/adversary_sim_state.rs`. `adversary_sim.rs` no longer owns the lifecycle state machine directly.

### 2. Lane runtime planning and worker-result shaping

Delivered as planned in `src/admin/adversary_sim_lane_runtime.rs` and `src/admin/adversary_sim_worker_plan.rs`. The runtime beat path, deterministic request generation, Scrapling worker dispatch plan creation, and worker-result application are no longer mixed into the top-level module.

### 3. Diagnostics and operator-facing shaping

Delivered as planned in `src/admin/adversary_sim_diagnostics.rs`. The diagnostics payload builders and lane diagnostics state now live together instead of being split between operator projections and the orchestration shell.

### 4. Deterministic corpus helpers

Delivered as planned in `src/admin/adversary_sim_corpus.rs`. The deterministic corpus, metadata payloads, and runtime profile helpers are no longer co-located with lifecycle and execution code.

### 5. Explicit placeholder `bot_red_team` behavior

Still explicit and unchanged. The lane runtime continues to record `bot_red_team_unimplemented` truthfully rather than implying mature behavior that does not exist yet.

## Verification Evidence

The behavior-preserving tranche was verified with the focused adversary-sim gates required by the plan:

- `make test-adversary-sim-domain-contract`
- `make test-adversary-sim-lifecycle`
- `make test-adversary-sim-runtime-surface`
- `git diff --check`

Runtime-surface proof remained green against the active local Spin dev server:

- `Spin server is ready at http://127.0.0.1:3000/health`
- `[runtime-surface-gate] PASS observed={"ban": true, "challenge": true, "fingerprint_or_cdp": true, "geo": true, "js_required": true, "maze_or_tarpit": true, "pow": true, "rate": true} live_summary={"challenge_failures": 0, "geo_violations": 0, "pow_attempts": 0, "rate_violations": 0}`

## Architectural Result

The hotspot file is no longer the only home for corpus, state machine, diagnostics, worker-plan contracts, and lane execution. After the tranche:

- `src/admin/adversary_sim.rs` is reduced to the public contract shell plus focused tests.
- the execution seam for later `OVR-*` and benchmarking work now exists in dedicated modules rather than inviting more logic back into the monolith.
- later agent-first loop work can now extend the adversary-sim control plane without re-concentrating unrelated concerns.

## Shortfall Check

No tranche-local shortfall was found that required an immediate reopen before proceeding to `ADV-RUN-ARCH-1`.
