# ADR 0011: Gateway-Only Upstream Contract and Guardrails

- Status: Proposed
- Date: 2026-03-05
- Owners: @shuma-maintainers
- Related: `DEP-GW-1`, `docs/plans/2026-03-05-gateway-first-existing-site-deployment-plan.md`

## Context

Shuma is moving to gateway-first production posture (`client -> shuma -> origin`) and must remove ambiguity between policy outcomes and delivery transport.

Spin/Fermyon constraints require explicit outbound host capabilities and do not support manual outbound `Host` header overrides. Without strict contract validation, gateway deployment can drift into unsafe states (misconfigured upstream, outbound wildcard posture, origin bypass, forwarding loops, or permissive fallback expectations).

## Decision

1. Production (`SHUMA_RUNTIME_ENV=runtime-prod`) uses a strict gateway env contract:
   - `SHUMA_GATEWAY_UPSTREAM_ORIGIN` is mandatory and must be canonical `scheme://host[:port]`.
   - `SHUMA_GATEWAY_DEPLOYMENT_PROFILE` is explicit (`shared-server` or `edge-fermyon`).
   - `edge-fermyon` profile requires HTTPS upstream and signed-header origin-auth mode.
2. Insecure upstream (`http://`) is only allowed for `shared-server` with explicit acknowledgement and constrained local/private IP-literal targets.
3. Gateway loop-prevention is fail-closed:
   - canonical upstream authority must not match configured public ingress authorities.
   - loop-hop budget is explicit and bounded.
4. Origin lock and reserved-route collision checks are explicit production attestations:
   - `SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true`
   - `SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true`
5. Upstream TLS posture is strict-only:
   - `SHUMA_GATEWAY_TLS_STRICT=true`
   - no production skip-verify path.
6. Deployment guardrails (`make deploy-env-validate`) include Spin outbound capability alignment:
   - upstream origin must be present in `component.bot-defence.allowed_outbound_hosts`,
   - wildcard outbound entries are rejected for production,
   - edge profile rejects variable-templated outbound host entries.

## Alternatives Considered

1. Keep dual production modes (native/front-door + gateway).
2. Enforce only docs/runbook guidance with no startup/deploy checks.
3. Allow permissive wildcard outbound posture for operational convenience.

## Consequences

### Positive

- One explicit production transport posture.
- Early failure for unsafe deployment combinations.
- Cleaner separation between policy decision and transport contract.
- Lower risk of origin bypass and loop recursion.

### Negative / Trade-offs

- More deploy-time env/config requirements.
- Additional operator setup for origin lock and collision preflight attestation.
- Existing production-like local flows must now set explicit gateway contract values.

## Security Impact

- Tightens trust-boundary posture by preventing permissive outbound drift.
- Prevents insecure public-routable HTTP upstream use.
- Requires explicit origin lock controls and signed-header auth in edge profile.

## Human Friction Impact

- Slightly higher operator setup burden at deployment time.
- No additional visitor-facing friction on request path.

## Adversary Cost Placement

- Preserves central policy enforcement while forcing all allowed traffic through bounded, observable gateway path.
- Reduces attacker leverage from direct-origin bypass opportunities.

## Operational Impact

- Deploy: `make deploy-env-validate` now verifies gateway + outbound alignment in production.
- Config: new `SHUMA_GATEWAY_*` env contract keys become mandatory for production posture.
- Monitoring/alerts: forward-failure taxonomy metrics are already established in GW-0 and are consumed by later runtime slices.
- Rollback: revert to prior release artifact/env bundle; restore previous upstream contract and outbound host allowlist.

## Resource Impact

- Bandwidth: unchanged in this contract tranche.
- CPU: negligible increase (startup/deploy validation only).
- Memory: negligible.
- Energy/efficiency notes: fail-fast validation avoids wasted runtime retries under invalid posture.

## Verification

- Tests: `src/config/tests.rs` gateway contract validation coverage and parser cases.
- Benchmarks (if relevant): not applicable for this contract-only tranche.
- Docs updated:
  - `docs/configuration.md`
  - `docs/deployment.md`
  - `Makefile` deploy guardrail description + check script integration.

## Follow-ups

1. Complete `DEP-GW-1-2` runtime forwarding adapter and remove local allow-response exits.
2. Add profile-specific integration and smoke targets (`DEP-GW-1-3`).
3. Enforce reserved-route collision preflight artifact generation in deployment workflows.
