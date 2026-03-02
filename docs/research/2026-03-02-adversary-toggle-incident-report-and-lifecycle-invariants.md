# Adversary Toggle Incident Report and Lifecycle Invariants

Date: 2026-03-02
Status: active runtime lifecycle contract for adversary simulation toggle and supervisor behavior

Related:
- [`2026-03-02-sim-runtime-architecture-overview-and-gap-report.md`](./2026-03-02-sim-runtime-architecture-overview-and-gap-report.md)
- [`2026-03-02-sim-prime-directive-shared-corpus-and-out-of-process-heartbeat.md`](./2026-03-02-sim-prime-directive-shared-corpus-and-out-of-process-heartbeat.md)
- [`../plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md`](../plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md)

## Incident summary

Observed operator-facing failures:

1. Toggle no-op: UI toggle action accepted by UI but no sustained runtime generation.
2. On/off bounce: toggle briefly showed enabled, then disabled, then enabled again.
3. Restart stale-enabled illusion: after server restart, UI still appeared enabled while no generation heartbeat existed.
4. Control/status disagreement: control endpoint and status endpoint reflected different effective states.
5. Missing lifecycle diagnostics: triage required deep code inspection instead of status payload evidence.

## Root causes (consolidated)

1. Multiple state authorities (`config` + `control state`) with reconciliation lag and asymmetric writes.
2. Request-lifecycle-coupled heartbeat semantics made “enabled” ambiguous when request cadence changed.
3. Insufficiently explicit lifecycle diagnostics for lease/owner/beat recency.
4. Missing fast regression gate for exact toggle lifecycle failure modes.

## Non-negotiable lifecycle invariants

1. Toggle `on` means runtime desired state is `enabled=true` and control phase converges to `running` or fails explicitly.
2. Toggle `off` means control phase converges to `off`, with zero active run/lane counts.
3. Auto window expiry must force `off` and clear runtime enabled override.
4. Server restart must never preserve a stale running state for a prior process instance.
5. UI-visible enabled state must be derived from reconciled status, not optimistic local toggles.
6. Historical telemetry remains visible after stop/off; only generation stops.
7. Off-state inertness: no heartbeat loop, no generator work, no emitted simulation traffic.

## Runtime diagnostics contract

Status payload must expose structured lifecycle diagnostics:

1. `lifecycle_diagnostics.control`
   - `desired_enabled`
   - `actual_phase`
   - `controller_reconciliation_required`
   - `runtime_instance_id`
   - `owner_instance_id`
   - `last_transition_reason`
   - `last_terminal_failure_reason`
   - `last_control_operation_id`
   - `lease_expires_at`
2. `lifecycle_diagnostics.supervisor`
   - `heartbeat_expected`
   - `generated_tick_count`
   - `generated_request_count`
   - `last_successful_beat_at`
   - `seconds_since_last_successful_beat`
   - `last_generation_error`

## Regression gate and required checks

Use the fast lifecycle gate before merging any open SIM tranche:

```bash
make test-adversary-sim-lifecycle
```

This gate validates:

1. control start/stop/status round-trip,
2. stale enabled/read-path reconciliation to off,
3. restart ownership reconciliation (`process_restart`),
4. tick + diagnostics contract availability,
5. deterministic corpus parity across runtime and CI lanes.

## Triage sequence

When toggle behavior appears wrong:

1. read `/admin/adversary-sim/status` and inspect `lifecycle_diagnostics` first,
2. compare `desired_enabled` vs `actual_phase` and reconciliation flag,
3. inspect supervisor beat recency and generation error,
4. if owner mismatch is present, require status reconciliation before further control actions,
5. only then inspect logs/code.
