# Architecture Decision Records (ADR)

Use ADRs to capture significant architectural or cross-cutting decisions.

## When an ADR is required

Create an ADR when a change:

- Introduces or changes module boundaries.
- Adds/removes provider interfaces or pluggability behavior.
- Changes security posture or trust boundaries.
- Changes deployment/operational model in a non-trivial way.
- Adds a new high-cost defense mechanism with trade-offs.
- Breaks or intentionally alters established behavior.

## Naming and location

- Location: `docs/adr/`
- File format: `NNNN-short-title.md` (example: `0001-rate-limiter-backend.md`)
- Start from template: [`docs/adr/0000-template.md`](0000-template.md)

## Status lifecycle

- `Proposed`
- `Accepted`
- `Superseded`
- `Deprecated`

If superseded, link the replacing ADR.

## Writing rules

- Keep it concise.
- Focus on context, decision, and consequences.
- Include alternatives considered.
- Include security, operational, and resource implications.
- Link to relevant PRs/issues.

## Current ADRs

1. [`0001-profile-gated-state-plane.md`](0001-profile-gated-state-plane.md)
2. [`0002-dashboard-sveltekit-cutover.md`](0002-dashboard-sveltekit-cutover.md)
3. [`0003-dashboard-runtime-policy.md`](0003-dashboard-runtime-policy.md)
4. [`0004-tarpit-v2-progression-contract.md`](0004-tarpit-v2-progression-contract.md)
5. [`0005-adversarial-lane-coexistence-policy.md`](0005-adversarial-lane-coexistence-policy.md)
6. [`0006-functional-core-policy-orchestration.md`](0006-functional-core-policy-orchestration.md)
7. [`0007-adversary-sim-toggle-command-controller.md`](0007-adversary-sim-toggle-command-controller.md)
8. [`0008-realtime-monitoring-cursor-sse-hybrid.md`](0008-realtime-monitoring-cursor-sse-hybrid.md)
9. [`0009-telemetry-lifecycle-retention-cost-security.md`](0009-telemetry-lifecycle-retention-cost-security.md)
10. [`0010-adversary-sim-autonomous-heartbeat.md`](0010-adversary-sim-autonomous-heartbeat.md)
11. [`0011-gateway-only-upstream-contract.md`](0011-gateway-only-upstream-contract.md)
