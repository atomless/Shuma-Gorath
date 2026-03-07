# SIM2-GCR-3 Plan: UI Toggle Trust-Boundary Controls

Date: 2026-02-28  
Status: Proposed

Reference research:

- [`docs/research/2026-02-28-sim2-gcr-3-ui-toggle-trust-boundary-controls-research.md`](../research/2026-02-28-sim2-gcr-3-ui-toggle-trust-boundary-controls-research.md)

## Objective

Implement a control-plane policy bundle for adversary toggle operations so admin-triggered start/stop is protected by explicit session, CSRF, replay, abuse-throttling, and audit boundaries.

## Non-goals

1. Defining lane/resource-envelope policy for production adversary simulation.
2. Introducing interactive MFA/step-up UX for every toggle in this phase.
3. Replacing command controller architecture selected in `SIM2-GCR-1`.

## Architecture Decisions

1. Treat `POST /admin/adversary-sim/control` as high-risk state mutation endpoint with endpoint-specific trust policy.
2. Enforce layered request authenticity (`auth session`, `csrf token`, `origin/referer`, `fetch-metadata`).
3. Enforce replay-safe semantics with mandatory `Idempotency-Key` bound to actor/session and canonical payload.
4. Enforce anti-abuse throttling envelope on control submissions.
5. Emit structured operation security-audit records for every accepted/rejected/throttled submission.

## Delivery Phases

### Phase 1: Session + CSRF + Origin/Fetch-Metadata Hardening

1. Add control-endpoint policy gate requiring:
   1. Valid admin session.
   2. CSRF token validation.
   3. Origin allowlist match (or strict referer fallback).
   4. Fetch Metadata policy for unsafe method requests.
2. Add configurable session freshness threshold for control operations.
3. Return explicit denial reason taxonomy for boundary failures.

Acceptance criteria:

1. Missing/invalid CSRF token fails closed.
2. Origin mismatch fails closed.
3. Cross-site unsafe requests blocked by fetch-metadata policy.
4. Session age/idle threshold breaches require reauth before control action.

### Phase 2: Replay/Idempotency Contract

1. Require `Idempotency-Key` for control submissions.
2. Canonicalize payload and bind key record to actor/session + payload hash.
3. Return stable `operation_id` for exact retries within TTL.
4. Reject same key with payload mismatch as explicit replay misuse.

Acceptance criteria:

1. Duplicate retries do not duplicate control transitions.
2. Mismatched replay attempts are rejected deterministically.
3. Idempotency records expire deterministically and safely.

### Phase 3: Abuse Throttling Envelope

1. Add per-session + per-IP rate controls specific to adversary control endpoint.
2. Add bounded queue/debounce logic to avoid toggle storms.
3. Emit explicit `throttled` outcomes tied to `operation_id` or attempted key hash.

Acceptance criteria:

1. Repeated rapid toggle attempts are bounded and observable.
2. Legitimate low-frequency control actions remain unaffected.
3. Throttle events surface in monitoring/audit channels.

### Phase 4: Structured Security Audit Schema

1. Define control-audit event schema fields:
   1. `operation_id`
   2. `actor_id`
   3. `session_id`
   4. `request_origin`
   5. `idempotency_key_hash`
   6. `decision`
   7. `reason`
   8. `timestamp`
2. Ensure both acceptance and rejection paths emit auditable events.
3. Add retention-sensitive handling for audit fields (hashed keys, no raw secrets).

Acceptance criteria:

1. Every control submission emits exactly one auditable decision event.
2. Event fields are sufficient for incident reconstruction.
3. Sensitive values are redacted/hashed per policy.

### Phase 5: Verification and Gate Wiring

1. Add unit/integration tests for:
   1. CSRF/origin/fetch-metadata failure paths.
   2. Session freshness reauth path.
   3. Idempotency replay exact-match vs mismatch.
   4. Control-endpoint throttling behavior.
2. Extend `SIM2-GC-11` matrix with trust-boundary regression rows.
3. Keep verification on Makefile path.

Acceptance criteria:

1. Trust-boundary regressions fail deterministically in CI.
2. Toggle control remains operable for valid requests.
3. No bypass path exists around control endpoint policy gate.

## Verification Strategy

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test-adversarial-fast` (with `make dev` running)
4. `make test`

## Rollback Plan

1. Keep new control-policy gate behind internal feature guard during rollout window.
2. If false-positive denials occur, temporarily relax only one boundary at a time (for example fetch-metadata), with explicit log annotation.
3. Preserve idempotency/audit schema even during temporary relaxations so incident visibility remains intact.

## Definition of Done

1. Control endpoint trust boundaries are explicit, enforced, and test-covered.
2. Replay and abuse risks are bounded with deterministic behavior.
3. Audit output is actionable for operators and security review.
4. Behavior remains aligned with the deployment's adversary availability posture and fails closed when the surface is disabled.
