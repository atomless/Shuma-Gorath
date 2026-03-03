# Adversary-Sim Production Availability Decision Criteria

Date: 2026-03-03
Status: Decision criteria established (`SIM-DEPLOY-1`)

## Objective

Define explicit approval criteria for promoting adversary simulation beyond `runtime-dev` while preserving operator safety, tenant isolation, and predictable resource usage.

## Decision Criteria

Production availability may only be approved when all criteria below are satisfied:

1. Explicit operator consent model is in place.
   - Runtime default remains disabled.
   - Enablement requires an explicit production opt-in switch.
2. Strict execution lifecycle guarantees are enforced.
   - Generation worker starts only when simulation is actively enabled.
   - Worker exits on toggle-off, run-window expiry, server stop, and sustained unreachability.
3. Resource envelopes are bounded and auditable.
   - Request-rate, CPU, memory, and runtime window caps are enforced.
   - Status payload exposes degraded/failure state taxonomy.
4. Control-plane and attacker-plane separation remains strict.
   - Toggle/control endpoints mutate lifecycle state only.
   - Traffic generation remains in attacker plane through normal public request pipeline.
5. Trust-boundary parity holds for simulation and external-equivalent traffic.
   - No simulation-specific forwarding trust bypasses.
   - Policy/telemetry decisions must match external-equivalent requests.
6. Kill switch and rollback are operator-grade.
   - One-step disable path exists.
   - Disable action reconciles state to off and stops generation promptly.
7. Multi-tenant/isolation posture is documented per deployment model.
   - Single-host, sidecar, and external supervisor deployments include explicit trust assumptions and blast-radius boundaries.
8. Monitoring and auditability are first-class.
   - Simulation events are distinguishable by metadata tags only.
   - Operational diagnostics identify control, heartbeat, and generation failure causes without synthetic success states.

## Current Decision (2026-03-03)

Production availability is **not approved yet**.

Rationale:

1. Deterministic runtime lane and out-of-process heartbeat decoupling are now stable.
2. Strategic production controls (explicit production opt-in contract, deployment-specific runbook hardening, and LLM runtime lane completion) are still in progress.
3. `SIM-DEPLOY-2` remains the implementation gate for any production enablement.

## Rollback Baseline

If production pilot work is attempted and instability is observed:

1. Force simulation OFF through control surface.
2. Disable supervisor launch path.
3. Verify `generation_active=false` and `phase=off` in status diagnostics.
4. Preserve telemetry history for audit; do not couple rollback to history deletion.
