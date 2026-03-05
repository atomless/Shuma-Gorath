# Gateway-First Tranche Conformance Review

Date: 2026-03-05
Scope: `DEP-GW-1-*` implementation conformance against `docs/plans/2026-03-05-gateway-first-existing-site-deployment-plan.md` and `todos/todo.md` acceptance criteria.

## Summary

Gateway-first production posture is now implemented with one policy core and one allow transport adapter (`src/runtime/upstream_proxy.rs`).

Allow-path responses no longer terminate locally; they forward upstream with fail-closed transport semantics, canonicalized request/response boundaries, explicit loop guardrails, and profile-specific deployment validation.

## Conformance Matrix

1. Contract and guardrails (`DEP-GW-1-1-*`): complete.
- Evidence: `src/config/mod.rs`, `src/config/tests.rs`, `scripts/deploy/validate_gateway_contract.py`, `scripts/tests/test_validate_gateway_contract.py`, `scripts/deploy/validate_gateway_route_collisions.py`, `scripts/tests/test_validate_gateway_route_collisions.py`, `docs/adr/0011-gateway-only-upstream-contract.md`.

2. Runtime transport refactor (`DEP-GW-1-2-*`): complete.
- Evidence: `src/runtime/upstream_proxy.rs`, `src/runtime/upstream_canonicalization.rs`, `src/runtime/request_flow.rs`, `src/runtime/effect_intents/intent_types.rs`, `src/runtime/effect_intents/plan_builder.rs`, `src/runtime/effect_intents/response_renderer.rs`, `src/observability/metrics.rs`.

3. Integration/security/ops (`DEP-GW-1-3-*`): complete.
- Evidence: `tests/routing_order_integration.rs`, `scripts/tests/gateway_failure_harness.py`, `scripts/tests/test_gateway_failure_harness.py`, `Makefile` gateway profile/smoke targets, `.github/workflows/ci.yml`, `.github/workflows/release-gate.yml`, `docs/deployment.md`, gateway deploy skills.

4. Product positioning/cleanup (`DEP-GW-1-4-*`): complete.
- Evidence: `docs/deployment.md` gateway-only protocol and runbook posture, updated deployment skills, this review, and cleanup review doc.

## Acceptance Criteria Check

1. Existing-site gateway deployment documented as the only production path: pass.
2. No local allow-response path remains for public allow outcomes: pass.
3. Policy behavior preserved while allow transport moved upstream: pass.
4. Spin outbound capability and gateway config misalignment fails closed: pass.
5. Invalid config/upstream failure/direct-origin posture has no permissive bypass path: pass.
6. Gateway verification enforced through Make targets: pass.
7. Shared-server + edge profile coverage exists with explicit tests/gates: pass.
8. Loop prevention enforced at startup + runtime: pass.
9. Upstream 4xx/5xx treated as pass-through outcomes: pass.
10. Reserved-route collision preflight enforced: pass.
11. Gateway v1 support boundaries explicit and tested: pass.
12. Redirect confinement and cookie rewrite behavior tested and documented: pass.
13. Upstream TLS posture is strict-only with transport failure classification: pass.
14. Forwarded/provenance headers are proxy-regenerated from trusted context only: pass.
15. Origin-auth credentials are proxy-owned and client override is blocked: pass.

## Verification Evidence

Targeted gateway gates passed:
- `make test-gateway-harness`
- `make test-gateway-profile-shared-server`
- `make test-gateway-profile-edge`
- `make smoke-gateway-mode`

Final full-suite gates were run at tranche closure:
- `make test`
- `make build`

## Deviations and Shortfalls

No release-blocking shortfalls were found for `DEP-GW-1` acceptance criteria.

## Security/Operational/Resource Notes

1. Security: fail-closed gateway contract and canonicalization reduce origin-bypass and provenance-spoof risk.
2. Operations: deploy now requires explicit preflight artifacts/attestations (`deploy-env-validate`).
3. Resource: bounded request/response body limits and explicit outbound-pressure guidance are now part of the contract.
