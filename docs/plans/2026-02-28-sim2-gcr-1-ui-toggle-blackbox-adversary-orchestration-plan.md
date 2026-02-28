# SIM2-GCR-1 Plan: UI Toggle Black-Box Adversary Orchestration

Date: 2026-02-28  
Status: Proposed

Reference research:

- [`docs/research/2026-02-28-sim2-gcr-1-ui-toggle-blackbox-adversary-orchestration-research.md`](../research/2026-02-28-sim2-gcr-1-ui-toggle-blackbox-adversary-orchestration-research.md)

## Objective

Implement a dev-only adversary toggle architecture that is:

1. retry-safe and idempotent,
2. explicit about desired vs actual lifecycle state,
3. lease-safe under multi-instance conditions,
4. auditable by operation and actor,
5. compatible with strict black-box attacker boundaries.

## Non-goals

1. Introducing external workflow/orchestration dependencies for this phase.
2. Expanding adversary availability to runtime-prod.
3. Replacing deterministic release gating with stochastic frontier outcomes.

## Architecture Decisions

1. `POST /admin/adversary-sim/control` becomes command submission with idempotency support and `operation_id` response.
2. Reconciliation moves to explicit controller loop behavior; read/status paths should not hide major lifecycle side effects.
3. Controller transitions require a lease/fencing token to prevent split-brain lifecycle mutation.
4. Transition logs include `operation_id`, actor identity, requested state, terminal outcome, and reason.
5. Stop semantics remain two-stage: graceful stop deadline then forced kill.

## Delivery Phases

### Phase 1: Command Contract and Idempotency

1. Extend control payload handling to support command metadata (`reason`) and idempotency key handling.
2. Persist command envelope and return `operation_id` for all accepted transitions.
3. Ensure duplicate command retries return stable operation linkage and do not duplicate start transitions.

Acceptance criteria:

1. Repeated identical toggle submissions with same idempotency key do not create duplicate starts/stops.
2. Every accepted command has a durable `operation_id`.
3. API responses include both `requested_state` and operation correlation fields.

### Phase 2: Desired vs Actual State Separation

1. Introduce explicit desired-state record and actual-state record for adversary lifecycle.
2. Move lifecycle mutation authority to controller reconciliation path.
3. Keep status endpoint read-oriented with explicit reporting of divergence (`desired != actual`) when present.

Acceptance criteria:

1. `desired_state` and `actual_state` are independently observable.
2. No hidden lifecycle mutation occurs outside controller transitions.
3. State divergence is explicit and diagnosable.

### Phase 3: Lease/Fencing Controller Safety

1. Add persisted lease ownership with expiry and monotonic fencing counter.
2. Require valid lease ownership for state transitions to `running`/`stopping`/`off`.
3. Add forced lease-expiry recovery behavior for stuck owners.

Acceptance criteria:

1. Concurrent controller contenders cannot both transition lifecycle state.
2. Expired/stale owner transitions are rejected deterministically.
3. Recovery from stale lease is bounded and test-covered.

### Phase 4: Auditability, Stop Guarantees, and Diagnostics

1. Extend transition/event logs with `operation_id`, actor label, reason, and terminal outcome.
2. Add explicit stop-path diagnostics (`manual_off`, `auto_window_expired`, `forced_kill_timeout`).
3. Ensure UI messaging can reference operation/status outcomes without implying procedural attack choreography.

Acceptance criteria:

1. Operators can trace each toggle action from command submission to terminal lifecycle state.
2. Forced-stop paths are observable and test-covered.
3. Dashboard reflects lifecycle truth without progress-sequence assumptions.

### Phase 5: Verification and Gate Wiring

1. Add tests for command idempotency and duplicate toggle race scenarios.
2. Add tests for desired/actual reconciliation behavior and lease ownership enforcement.
3. Add tests for operation-correlation lineage in event/audit output.
4. Wire checks into existing Makefile verification flow.

Acceptance criteria:

1. Lifecycle race and idempotency regressions fail deterministically with explicit diagnostics.
2. Makefile-driven adversarial and integration gates remain green.
3. New behavior remains dev-only and fail-closed in runtime-prod.

## Verification Strategy

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test-adversarial-fast` (with `make dev` running)
4. `make test` (umbrella)

## Rollback Plan

1. Keep previous control handler behavior behind a temporary guarded fallback during migration.
2. If reconciliation regressions appear, disable new command mode and revert to previous state model while preserving event logs for diagnosis.
3. Remove temporary fallback once race/idempotency suite is stable.

## Definition of Done

1. Toggle control is idempotent and operation-correlated.
2. Desired/actual lifecycle semantics are explicit and controller-owned.
3. Lease/fencing prevents split-brain lifecycle mutation.
4. Audit trail is sufficient for operator diagnosis and incident review.
5. Makefile verification remains the canonical passing gate.

