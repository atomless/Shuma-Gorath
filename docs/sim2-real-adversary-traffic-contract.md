# SIM2 Real Adversary Traffic Contract

Date: 2026-02-28  
Status: Active (`SIM2-GC-1`)

## Purpose

Define what counts as a valid adversarial simulation run in Shuma.

A run is valid only when simulated attacker traffic exercises real request-path defenses and produces runtime-generated telemetry that can be observed through canonical monitoring surfaces.

## Required Invariants

1. Traffic source must be attacker-plane requests sent through public routes only.
2. Execution lane must remain black-box (`execution_lane=black_box`), with no privileged control-plane access in attacker requests.
3. Defense path must be the same runtime decision/enforcement pipeline used for non-simulation traffic.
4. Telemetry emission must be runtime-generated (event/monitoring writes emitted by request processing), not synthetic test-only injection.
5. Monitoring visibility must show run effects through canonical admin monitoring/event surfaces.
6. Run evidence must include request lineage, per-scenario runtime telemetry evidence, and control-plane lineage metadata.

## Prohibited Patterns

The following patterns must not be used to satisfy simulation gates:

1. Synthetic monitoring injection (writing fake monitoring rows to make coverage pass).
2. Out-of-band metrics writes that bypass runtime request handling.
3. Control-plane-only success signals where no attacker-plane runtime traffic evidence exists.

## Evidence Schema (`sim-run-evidence.v1`)

The adversarial report must include an `evidence` object with:

1. `run`
   - `request_id_lineage` (`sim_run_id`, `sim_profile`, `sim_lane`)
   - `scenario_ids`
   - `lane`
   - `defenses_touched`
   - `decision_outcomes`
   - `latency_ms` (`suite_runtime_ms`, `p95_ms`)
2. `scenario_execution` rows with required fields:
   - `scenario_id`
   - `runtime_request_count`
   - `monitoring_total_delta`
   - `coverage_delta_total`
   - `simulation_event_count_delta`
   - `has_runtime_telemetry_evidence`
3. `control_plane_lineage` with required fields:
   - `control_operation_id`
   - `requested_state`
   - `desired_state`
   - `actual_state`
   - `actor_session`

## Monitoring and IP-Ban Definition of Done

A simulation run must be considered done only when all rules below are true:

1. The run report is generated and `passed=true`.
2. Every passed scenario includes runtime telemetry evidence in `evidence.scenario_execution`.
3. Monitoring deltas and event visibility are present for exercised defense paths.
4. Control-plane lineage fields are present and reconstructable.
5. No prohibited pattern is used to satisfy coverage/gate outcomes.

## Implementation Hooks

1. Contract artifact: `scripts/tests/adversarial/real_traffic_contract.v1.json`.
2. Runner enforcement: `scripts/tests/adversarial_simulation_runner.py`.
3. Unit contract checks: `scripts/tests/test_adversarial_simulation_runner.py`.
4. Operator workflow: `docs/adversarial-operator-guide.md`.

