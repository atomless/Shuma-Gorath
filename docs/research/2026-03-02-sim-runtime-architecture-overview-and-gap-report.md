# SIM Runtime Architecture Overview and Gap Report

Date: 2026-03-02  
Status: Active pre-implementation report (mandatory pre-read for open `SIM-*` execution)
Execution companion: [`2026-03-02-sim-prime-directive-shared-corpus-and-out-of-process-heartbeat.md`](./2026-03-02-sim-prime-directive-shared-corpus-and-out-of-process-heartbeat.md)

## Scope

Document the current adversary-simulation architecture as implemented, identify where runtime behavior diverges from intended design, and define guardrails for the next SIM tranche so work does not duplicate or conflict across parallel lanes.

## High-Level Architecture in Place

### 1. Dashboard runtime-toggle lane (operator-facing control path)

- UI toggle uses:
  - `POST /admin/adversary-sim/control` via [`dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js:790)
  - `GET /admin/adversary-sim/status` via [`dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js:658)
- Toggle interaction path in [`dashboard/src/routes/+page.svelte`](../../dashboard/src/routes/+page.svelte:523).
- Runtime state and lifecycle are implemented in [`src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs:50).
- Post-response runtime invocation exists in [`src/lib.rs`](../../src/lib.rs:1014) and [`src/lib.rs`](../../src/lib.rs:1063).

### 2. Deterministic scenario-runner lane (test harness path)

- Python runner implements seeded deterministic scenario execution and control-plane mutation contracts:
  - [`scripts/tests/adversarial_simulation_runner.py`](../../scripts/tests/adversarial_simulation_runner.py:1055)
  - deterministic execution helpers around [`scripts/tests/adversarial_simulation_runner.py`](../../scripts/tests/adversarial_simulation_runner.py:3209)
- Repeatability gate exists independently:
  - [`scripts/tests/adversarial_repeatability.py`](../../scripts/tests/adversarial_repeatability.py:27)
- Invoked by Make targets (not by dashboard toggle):
  - [`Makefile`](../../Makefile:501), [`Makefile`](../../Makefile:641)

### 3. Containerized black-box lane (test orchestration path)

- Host/container worker tooling exists under:
  - [`scripts/tests/adversarial_container_runner.py`](../../scripts/tests/adversarial_container_runner.py:2)
  - [`scripts/tests/adversarial_container/worker.py`](../../scripts/tests/adversarial_container/worker.py:306)
- Invoked via scheduled/manual Make target:
  - [`Makefile`](../../Makefile:690)

## Key Findings

### A. Runtime toggle is wired to Rust runtime lane only

The dashboard toggle controls runtime state in Rust and does not directly invoke the Python runner or container runner.

### B. Runtime generation still indicates request-loop trigger surface

Status payload currently reports `"trigger_surface": "runtime_request_loop"` in [`src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs:698), and runtime tick execution is still called from the request lifecycle path in [`src/lib.rs`](../../src/lib.rs:1063).  
This is a residual coupling risk relative to the intended autonomous supervisor model.

### C. Lane-status representation is broader than executed runtime lanes

Status currently maps both `deterministic` and `containerized` lane phases from the same control phase:
- [`src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs:400)
- [`src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs:401)

This can make runtime status appear as if containerized runtime execution is active when containerized execution is actually test-lane tooling.

### D. Parallel lanes are valid but currently under-explicitly separated

The repository has:
1. runtime operator lane (dashboard toggle),
2. deterministic verification lane (Python harness),
3. containerized verification lane (manual/scheduled test tool).

This is not inherently wrong, but ownership boundaries and runtime-vs-test semantics need to be explicit to avoid operator confusion and implementation drift.

## Gap vs Intended SIM Direction

Referenced intent (ADR/plan):
- [`docs/adr/0010-adversary-sim-autonomous-heartbeat.md`](../adr/0010-adversary-sim-autonomous-heartbeat.md)
- [`docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md`](../plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md)

Primary gap:
- Runtime-toggle path still has request-loop heartbeat characteristics and incomplete ownership clarity for lane semantics.

Secondary gap:
- Containerized/LLM lane exists as test/orchestration tooling, but is not yet a first-class runtime actor behind the operator toggle.

## Recommended Execution Guardrails for Open SIM Work

1. Keep one runtime cadence owner for toggle-driven generation (backend supervisor only).
2. Keep deterministic and containerized Python systems as verification harnesses unless explicitly promoted into runtime architecture via ADR-backed work.
3. Make runtime status diagnostics explicit about lane execution reality (do not imply runtime container lane when absent).
4. Preserve strict off-state inertness and ephemeral toggle semantics.
5. Preserve trust-boundary parity: simulated traffic must traverse the same enforcement path as external traffic.

## Decision Note for Upcoming SIM Tranche

Treat this document as the architecture baseline for open `SIM-*` work.  
Before implementing any open `SIM-*` item, confirm the change aligns with:
- runtime lane ownership,
- test-lane vs runtime-lane separation,
- and explicit control-plane/attacker-plane boundaries.
