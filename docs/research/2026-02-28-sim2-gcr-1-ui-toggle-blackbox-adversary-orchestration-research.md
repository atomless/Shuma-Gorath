# SIM2-GCR-1 Research: UI Toggle -> Black-Box LLM Adversary Orchestration

Date: 2026-02-28  
Status: Recommended architecture selected

## Objective

Identify the best architecture for triggering/stopping a black-box LLM adversary from a dev-only admin UI toggle, with strong trust boundaries, deterministic control semantics, and clear operational visibility.

## Repository Baseline (Current State)

1. Dashboard toggle directly calls `POST /admin/adversary-sim/control` and then refreshes state (`dashboard/src/lib/domain/api-client.js`, `dashboard/src/routes/+page.svelte`).
2. Control handler updates config + lifecycle state and returns immediately (`src/admin/api.rs:7936`).
3. Lifecycle state machine exists (`off`, `running`, `stopping`) with guardrails and timeout-based stop reconciliation (`src/admin/adversary_sim.rs`).
4. Status endpoint performs reconcile-on-read behavior and can mutate state/config as part of status polling (`src/admin/api.rs:7870`).

Current model is functional, but it blends command submission and reconciliation in request paths and does not yet provide operation-level idempotency or controller lease semantics.

## Primary-Source Findings

1. Asynchronous operations should use explicit accepted/processing semantics and separate status observation from submission semantics.  
   Source: [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)
2. Idempotency keys are the preferred way to make retried state-changing requests safe against duplicate side effects.  
   Source: [IETF Idempotency-Key HTTP Header (Internet-Draft)](https://datatracker.ietf.org/doc/html/draft-ietf-httpapi-idempotency-key-header)
3. Reconciliation loops based on desired vs actual state are a proven controller pattern for reliable lifecycle convergence.  
   Source: [Kubernetes Controller Pattern](https://kubernetes.io/docs/concepts/architecture/controller/)
4. Admin state-change endpoints must enforce strong CSRF and session defenses.  
   Source: [OWASP CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
5. REST admin surfaces should strictly constrain methods and avoid ambiguous control behavior.  
   Source: [OWASP REST Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html)
6. Security-relevant control transitions should be auditable with actor, action, outcome, and correlation identifiers.  
   Source: [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)

## Architecture Options

### Option A: Synchronous Toggle Execution in Request Path

`POST /control` directly starts/stops orchestration work and blocks until execution setup is complete.

### Option B: Immediate State Flip + Poll-Driven Reconcile (Current Shape)

`POST /control` mutates config/state quickly; periodic status polling reconciles stop/off behavior.

### Option C: Command API + Reconciler Controller + Worker Lease (Recommended)

`POST /control` submits an idempotent command (`enable`/`disable`) and returns an operation identifier.  
A controller loop reconciles desired state to actual state using lease/fencing for single active orchestrator authority.  
UI reads status/operation resources for progress and terminal outcomes.

### Option D: External Workflow Engine for Dev Toggle Control

Use external orchestration backend (queue/workflow platform) for command handling and lifecycle state.

## Decision Matrix

| Option | Benefits | Risks | Resource Cost | Security Impact | Rollback Complexity |
|---|---|---|---|---|---|
| A. Synchronous request-path execution | Simple mental model, minimal new primitives | Timeout/race risk, duplicate submissions can duplicate starts, mixes admin request path with execution lifecycle | Low initial, high debugging cost | Weakest; large privileged request surface | Low |
| B. Current shape (state flip + poll reconcile) | Already present, fast responses | Reconcile side effects on read path, weak operation idempotency/correlation, cross-instance ownership ambiguity | Low-medium | Moderate; acceptable but fragile under retries/races | Low |
| C. Command + controller + lease (recommended) | Clear separation of submit/read/execute, retry-safe via idempotency key, deterministic ownership semantics, auditable lifecycle | Requires new operation store/controller loop and lease logic | Medium | Strong; least-authority and explicit control boundaries | Medium |
| D. External workflow engine | Mature orchestration features, robust retries/visibility | Overkill for dev-only control, dependency + ops complexity | High | Strong but operationally heavy | High |

## Recommendation

Adopt **Option C**: idempotent command submission plus explicit controller reconciliation with lease/fencing ownership.

This preserves dev-only simplicity while giving production-grade control semantics:

1. **Command submission is cheap and retry-safe**: `POST /admin/adversary-sim/control` writes command + desired state and returns `operation_id`.
2. **Reconciliation is centralized**: one controller loop mutates actual state; read endpoints do not perform heavy reconciliation side effects.
3. **Single active authority**: controller lease/fencing token prevents split-brain multi-instance starts.
4. **Explicit stop guarantees**: graceful stop deadline + deterministic forced-stop path remain first-class.
5. **Auditable lifecycle**: every command and transition includes `operation_id`, actor, reason, and outcome.

## Recommended Control Contract

1. `POST /admin/adversary-sim/control` accepts:
   1. `enabled: boolean`
   2. `reason: string` (optional but logged)
   3. idempotency key header
2. Response:
   1. `operation_id`
   2. `requested_state`
   3. `accepted_at`
3. `GET /admin/adversary-sim/status` returns actual state and current operation linkage without hidden state mutation surprises.
4. Optional `GET /admin/adversary-sim/operations/:id` provides operation outcome details.
5. Controller lease is persisted with owner + expiry + fencing counter; only lease-holder can transition running/stopping/off.

## Security and Ops Implications

1. Better replay/race resistance for repeated UI clicks and network retries.
2. Stronger trust boundary: UI action submission is separate from privileged execution transitions.
3. Better diagnostics for operators during stuck-run or forced-stop incidents.
4. Easier future extension to emergent-lane controls without widening admin request-path authority.

## Plan and TODO Impact

1. New plan doc: `docs/plans/2026-02-28-sim2-gcr-1-ui-toggle-blackbox-adversary-orchestration-plan.md`.
2. TODO refinements required:
   1. Add operation-idempotency and desired-vs-actual contract details under `SIM2-GC-2`.
   2. Add command-operation lineage requirements under `SIM2-GC-1` and `SIM2-GC-11`.
   3. Add lease/fencing and race/regression verification requirements under `SIM2-GC-2` and `SIM2-GC-11`.
