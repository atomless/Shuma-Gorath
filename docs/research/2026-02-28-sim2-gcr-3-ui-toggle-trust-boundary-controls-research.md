# SIM2-GCR-3 Research: UI Toggle Trust-Boundary Controls

Date: 2026-02-28  
Status: Recommended control set selected

## Objective

Define the trust-boundary controls required for a dev-admin UI toggle that starts/stops black-box adversary orchestration, with explicit defenses for session abuse, CSRF, replay, command flooding, and auditability.

## Repository Baseline (Current State)

1. Control path exists at `POST /admin/adversary-sim/control` and is reachable from dashboard toggle flow.
2. Existing admin auth + CSRF plumbing is present, but adversary-control-specific hardening requirements are not yet explicitly codified as acceptance-tested contract controls.
3. Control path lacks explicit, documented anti-replay semantics for repeated submissions beyond basic state handling.
4. Control actions are observable, but operation-level audit schema for trust-boundary events is not yet comprehensive.

## Primary-Source Findings

1. CSRF mitigation should be layered: anti-CSRF token plus origin validation; `SameSite` cookies are defense-in-depth, not replacement.
   Source: [OWASP CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
2. Missing `Origin`/`Referer` should default to block for sensitive state-changing requests (or staged monitor then block).
   Source: [OWASP CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
3. Session security must include ID regeneration on privilege transitions and reauthentication for high-risk actions.
   Source: [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
4. Workflow endpoints should enforce server-side state transitions and reject out-of-order requests with explicit errors.
   Source: [OWASP REST Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html)
5. Retry-safe non-idempotent commands should use unique idempotency keys bound to payload semantics.
   Source: [IETF Idempotency-Key HTTP Header (Internet-Draft)](https://datatracker.ietf.org/doc/html/draft-ietf-httpapi-idempotency-key-header)
6. Async command submission should use explicit accepted semantics with a status monitor resource.
   Source: [RFC 9110 (HTTP Semantics)](https://www.rfc-editor.org/rfc/rfc9110)
7. Security logging should capture authorization/session failures and monitor access to logs themselves.
   Source: [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)
8. High-risk endpoints require explicit resource-consumption controls (rate limits, timeouts, bounded processing).
   Source: [OWASP API Security Top 10 API4:2023](https://owasp.org/API-Security/editions/2023/en/0xa4-unrestricted-resource-consumption/)

## Architecture Options

### Option A: Keep Generic Admin Auth/CSRF Only

Rely on existing admin middleware without adversary-control-specific replay, abuse, and audit contracts.

### Option B: Endpoint-Specific Defense Bundle Without Typed Command Contract

Add stricter CSRF/origin/rate-limit controls on endpoint, but keep submission semantics and replay handling loosely coupled.

### Option C: Typed Command Contract + Trust-Boundary Policy Bundle (Recommended)

Combine strict endpoint controls (auth/session/CSRF/origin/fetch metadata), payload-bound idempotency and replay handling, per-endpoint abuse throttling, and structured operation audit schema.

### Option D: Out-of-band Approval UI/MFA Gate for Every Toggle

Require extra approval ceremony (step-up prompt or MFA-like second factor) for every run-state toggle even in runtime-dev.

## Decision Matrix

| Option | Benefits | Risks | Resource Cost | Security Impact | Rollback Complexity |
|---|---|---|---|---|---|
| A. Generic controls only | Minimal implementation effort | Leaves replay/race/abuse/audit gaps for sensitive control endpoint | Low | Weakest | Low |
| B. Endpoint hardening only | Better immediate posture than baseline | Still weak contract consistency; replay and status linkage drift risk | Low-medium | Moderate | Low |
| C. Typed contract + policy bundle (recommended) | Coherent trust boundary, replay-safe semantics, testable controls, auditable operations | Requires new typed control-policy modules and tests | Medium | Strong | Medium |
| D. Always-on step-up approval | Strong human-in-loop friction | High operator friction in dev, complexity not justified pre-launch dev-only toggle | Medium-high | Strong but usability-heavy | Medium |

## Recommendation

Adopt **Option C**. Treat adversary toggle as a high-risk control-plane operation with explicit policy-by-construction.

Required control set:

1. **Session/Auth boundary**
   1. Keep admin auth mandatory.
   2. Regenerate session on privilege elevation.
   3. Require short-session freshness for control submission (reauth gate after idle/age threshold).
2. **CSRF boundary**
   1. Require CSRF token validation.
   2. Require `Origin` match (or strict `Referer` fallback).
   3. Enforce `SameSite` cookie hardening and Fetch Metadata policy for unsafe methods.
3. **Replay boundary**
   1. Require `Idempotency-Key` on control command submissions.
   2. Bind key to canonicalized request payload and actor/session scope.
   3. Reject key reuse with payload mismatch; return same `operation_id` for exact retries within TTL.
4. **Abuse-throttling boundary**
   1. Add per-session and per-IP ceiling for control submissions.
   2. Add bounded queue/debounce for repeated toggle storms.
   3. Emit explicit throttled-state diagnostics.
5. **Auditability boundary**
   1. Log actor/session, command, idempotency key hash, decision (`accepted|rejected|throttled`), reason, and `operation_id`.
   2. Log CSRF/origin/session failure reasons as structured security events.
   3. Protect log access and include tamper-evidence expectations.

## Security and Ops Implications

1. Reduces risk of cross-site toggle abuse and control replay side effects.
2. Prevents high-frequency toggle storms from degrading sim-control availability.
3. Makes incident review and postmortem attribution practical at operation granularity.
4. Preserves dev velocity without introducing high-friction manual approval loops.

## Plan and TODO Impact

1. New plan doc: `docs/plans/2026-02-28-sim2-gcr-3-ui-toggle-trust-boundary-controls-plan.md`.
2. Add explicit trust-boundary implementation tasks under `SIM2-GC-2`:
   1. CSRF/origin/fetch-metadata/session-hardening controls.
   2. Payload-bound replay/idempotency policy and TTL semantics.
   3. Control-endpoint abuse throttling.
   4. Structured security audit schema for control operations.
3. Add verification tasks under `SIM2-GC-11` for negative-path trust-boundary regression tests.
