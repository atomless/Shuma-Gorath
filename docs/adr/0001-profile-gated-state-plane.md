# ADR 0001: Profile-Gated State Plane (Single Project for Self-Hosted and Enterprise)

- Status: Accepted
- Date: 2026-02-13
- Owners: @jamestindall
- Related: `docs/plans/2026-02-13-provider-externalization-design.md`, `todos/todo.md` (P1 Distributed State)

## Context

Shuma-Gorath must serve two operational personas:

- `self_hosted_minimal`: often single-instance, low-ops overhead, no managed-edge dependency.
- `enterprise_akamai`: multi-instance/edge deployment where distributed-state correctness is required.

The key architecture risk is distributed state drift across instances:

- non-atomic per-instance rate limiting,
- unsynchronized ban/unban propagation,
- replay/TOCTOU gaps under concurrency.

We need to avoid burdening self-hosted users with enterprise state-sync complexity while still solving enterprise correctness.

## Decision

Keep one project and one shared policy engine, with profile-gated state backends.

- Shared policy plane for all personas:
  - botness scoring,
  - escalation routing,
  - defence mode composition.
- Profile-gated state plane:
  - `self_hosted_minimal`: internal/local state adapters as first-class baseline.
  - `enterprise_akamai`: distributed/external state adapters for multi-instance correctness.
- No persona-specific policy fork.
- Enterprise rollout guardrail:
  - multi-instance enterprise operation must not be treated as production-safe without distributed rate and ban-sync semantics.

## Alternatives Considered

1. Split into two projects (self-hosted and enterprise).
2. Keep one project but with a single always-distributed state model.
3. Keep one project with profile-gated state plane and shared policy plane.

## Consequences

### Positive

- Preserves self-hosted simplicity and low configuration burden.
- Avoids policy drift between product variants.
- Keeps enterprise correctness work focused on explicit state contracts.

### Negative / Trade-offs

- Requires strict contract design for state adapters.
- Needs clear rollout guardrails so enterprise users do not run unsafe multi-instance configs.
- Increases documentation and observability requirements across profiles.

## Security Impact

- Reduces multi-instance TOCTOU and consistency attack surface when enterprise profile uses distributed adapters.
- Maintains explicit risk boundary: local-only state remains acceptable for single-instance self-hosted posture.

## Human Friction Impact

- No added baseline friction for self-hosted operators.
- Enterprise setup has more validation requirements, but avoids latent abuse-control gaps.

## Adversary Cost Placement

- Maintains current self-hosted cost posture.
- Improves enterprise attacker-cost consistency by reducing cross-instance evasion opportunities.

## Operational Impact

- Deploy:
  - profile-specific deployment posture remains explicit (`self_hosted_minimal`, `enterprise_akamai`).
- Config:
  - distributed state settings become enterprise/hybrid controls, not baseline defaults.
- Monitoring/alerts:
  - monitor limiter fallback usage, ban-sync lag/drift, and state backend health in enterprise profile.
- Rollback:
  - enterprise fallback to internal remains available, with explicit advisory-only posture during fallback windows.

## Resource Impact

- Bandwidth:
  - negligible direct impact.
- CPU:
  - enterprise profile may add state-backend overhead; self-hosted path unchanged.
- Memory:
  - enterprise profile may add bounded cache/replay tracking.
- Energy/efficiency notes:
  - keeps self-hosted baseline efficient by default; enterprise spends only where correctness requires it.

## Verification

- Tests:
  - follow-up implementation will add adapter and integration tests for atomicity/sync semantics.
- Benchmarks (if relevant):
  - compare local vs distributed limiter overhead before authoritative rollout.
- Docs updated:
  - `docs/bot-defence.md`
  - `docs/configuration.md`
  - `docs/deployment.md`
  - `docs/plans/2026-02-13-provider-externalization-design.md`
  - `todos/todo.md`
  - `todos/security-review.md`

## Follow-ups

- Add explicit startup/deploy guardrails for unsafe multi-instance enterprise state posture.
- Implement atomic distributed `rate_limiter` and synced `ban_store` adapters.
- Add enterprise drift/fallback observability and rollback runbooks.
