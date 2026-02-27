# SIM2 Plan 1: Black-Box Lane Capability Enforcement

Date: 2026-02-27  
Status: Proposed

Reference research:

- [`docs/research/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement.md`](../research/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement.md)

## Objective

Guarantee that adversarial attacker-plane execution is black-box by construction:

1. attacker requests cannot access privileged headers/secrets,
2. control-plane operations remain explicit and isolated,
3. abuse scenarios (including stale-token) remain executable without white-box signing paths.

## Non-goals

1. Replacing deterministic runner with container-only lane.
2. Broad rewrite of all driver flows in one slice.

## Architecture Decisions

1. Create explicit runner capability separation:
   - `ControlPlaneClient` holds admin credentials and trusted forwarded secret.
   - `AttackerPlaneClient` can only emit public traffic headers.
2. Remove secret propagation from shared header helper paths.
3. Eliminate stale-token re-signing using challenge secret; use black-box stale generation path.
4. Add contract assertions that fail when attacker lane touches privileged material.

## Delivery Phases

### Phase 1: Plane Capability Contract

1. Define a machine-readable lane contract (`attacker` vs `control`) for allowed paths and headers.
2. Enforce contract in runner construction and runtime checks.

Acceptance criteria:

1. Attacker lane forbids all privileged headers including forwarded secret.
2. Contract validation fails fast before scenario execution.

### Phase 2: Runner Refactor for Capability-Typed Clients

1. Introduce dedicated request clients per plane.
2. Route all scenario drivers through attacker client for public traffic and control client only for admin setup/reset.

Acceptance criteria:

1. Attacker request helper has no access to API key or signing secrets.
2. Control-plane calls remain functional for baseline setup and cleanup.

### Phase 3: Stale-Token Scenario Black-Box Rewrite

1. Replace `make_expired_seed` re-signing flow with black-box stale generation technique.
2. Keep deterministic bounded runtime.

Acceptance criteria:

1. Stale-token abuse scenario passes without `SHUMA_CHALLENGE_SECRET`.
2. Scenario remains deterministic within declared runtime budgets.

### Phase 4: Verification and CI Guardrails

1. Add focused tests for lane contract enforcement and secret non-leak guarantees.
2. Add make-target-integrated lane contract check.

Acceptance criteria:

1. Contract regression causes deterministic CI failure with explicit diagnostics.
2. `make test-adversarial-fast` and `make test-adversarial-coverage` pass after refactor.

## Verification Strategy

1. `make test-adversarial-manifest`
2. `make test-adversarial-fast`
3. `make test-adversarial-coverage`
4. `make test` (with `make dev` running)

## Operational and Security Notes

1. Security posture improves by reducing ambient authority in adversary paths.
2. Operator behavior unchanged; only simulation internals and gate confidence improve.

## Definition of Done

1. Black-box lane no longer depends on privileged signing/admin secrets for attacker requests.
2. Lane contract is enforced in code and tests, not only in policy text.
3. All existing mandatory adversarial gates remain green.
