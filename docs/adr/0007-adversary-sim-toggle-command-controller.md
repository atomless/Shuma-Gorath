# ADR 0007: Adversary Sim Toggle Command Controller and Trust-Boundary Enforcement

- Status: Accepted
- Date: 2026-02-28
- Owners: Shuma core maintainers
- Related:
  - `docs/research/2026-02-28-sim2-gcr-1-ui-toggle-blackbox-adversary-orchestration-research.md`
  - `docs/research/2026-02-28-sim2-gcr-3-ui-toggle-trust-boundary-controls-research.md`
  - `docs/research/2026-02-28-sim2-gcr-2-containerized-black-box-capability-orchestration-research.md`

## Context

Adversary-sim control was functional but mixed command submission, lifecycle reconciliation, and trust-boundary policy in ways that allowed race/replay ambiguity and read-path mutation side effects. We need deterministic command semantics, strict trust controls, and auditable lifecycle transitions while preserving dev ergonomics.

## Decision

Adopt command-controller architecture for adversary-sim lifecycle:

1. `POST /admin/adversary-sim/control` is command submission only, returns `operation_id`, and requires idempotency key.
2. Lifecycle uses explicit `desired_state` vs `actual_state`; reconciliation is controller-owned.
3. Controller transitions require lease/fencing ownership to prevent split-brain mutation.
4. Trust-boundary policy for control submissions is explicit and fail-closed:
   - admin auth + session freshness,
   - CSRF token,
   - strict origin/referer,
   - fetch-metadata policy,
   - replay-safe idempotency contract,
   - submission throttling and structured audit events.
5. Containerized frontier workers execute under least-authority runtime profile with signed capability envelopes and bounded one-way command channels.

## Alternatives Considered

1. Keep state-flip + poll reconciliation model.
2. Synchronous request-path lifecycle execution.
3. External workflow platform for orchestration.

## Consequences

### Positive

- Retry-safe toggle semantics and clearer operator diagnostics.
- Stronger trust-boundary guarantees with explicit failure taxonomy.
- Split-brain lifecycle mutation risk reduced through lease/fencing.
- Black-box worker control surface remains capability-bounded.

### Negative / Trade-offs

- Higher implementation complexity (command store, reconciler, lease state).
- More moving parts in control-plane testing.

## Security Impact

- Reduces CSRF/replay/race abuse risk on high-risk control endpoint.
- Enforces least-authority worker boundaries and denies implicit privileged surfaces.
- Improves auditability for incident reconstruction.

## Human Friction Impact

- Minimal additional operator friction (toggle flow preserved).
- Potentially more explicit error states when trust checks fail.

## Adversary Cost Placement

- Increases attacker effort to abuse control endpoints (layered authenticity/replay/throttle checks).
- Preserves attacker black-box constraints in frontier lane.

## Operational Impact

- Deploy: add command/reconciliation state handling and controller cadence.
- Config: add/validate idempotency TTL, lease expiry, control throttling knobs as needed.
- Monitoring/alerts: track operation outcomes, lease contention, trust-boundary rejects.
- Rollback: temporarily disable controller mode and revert to legacy control path while preserving audit signals.

## Resource Impact

- Bandwidth: negligible incremental control payload.
- CPU: modest overhead for idempotency + lease checks.
- Memory: low overhead for command/lease records.
- Energy/efficiency notes: small cost increase for significantly improved control correctness.

## Verification

- Tests:
  - toggle idempotency/replay/lease contention regressions,
  - trust-boundary negative-path tests,
  - capability-envelope rejection tests.
- Benchmarks (if relevant): N/A for control-plane throughput.
- Docs updated: yes (research, plan, TODO mappings).

## Follow-ups

- Implement `SIM2-GC-2-*`, `SIM2-GC-8-*`, and `SIM2-GC-11-*` items aligned to this ADR.
- Capture post-implementation drift checks in CI guardrails.
