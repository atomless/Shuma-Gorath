# Adversary-Sim Production Availability Decision Criteria

Date: 2026-03-03
Status: Superseded as a production-availability gate on 2026-03-07; retained as ongoing hardening criteria for `SIM-DEPLOY-2`

## Objective

Define explicit hardening criteria for operating adversary simulation in production while preserving operator safety, tenant isolation, and predictable resource usage.

## Decision Criteria

Production adversary-sim operation should continue to satisfy all criteria below:

1. Operator control model is explicit and predictable.
   - The control surface is available by default.
   - Traffic generation remains off until an operator enables it.
   - Deployments may still disable the surface entirely with `SHUMA_ADVERSARY_SIM_AVAILABLE=false`.
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

## Current Decision

As of 2026-03-07, adversary simulation is part of Shuma's production operating stance and must not be runtime-prod-disabled.

Implications:

1. Production availability is no longer gated behind a separate approval step.
2. `SIM-DEPLOY-2` remains active as the hardening tranche for production lane posture, resource envelopes, kill switch behavior, and no-impact verification.
3. This document now serves as the checklist for that hardening work instead of a blocker against production use.

## Rollback Baseline

If production pilot work is attempted and instability is observed:

1. Force simulation OFF through control surface.
2. Disable supervisor launch path.
3. Verify `generation_active=false` and `phase=off` in status diagnostics.
4. Preserve telemetry history for audit; do not couple rollback to history deletion.
