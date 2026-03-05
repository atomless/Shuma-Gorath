# Gateway-First Post-Tranche Cleanup Review

Date: 2026-03-05
Scope: post-implementation cleanup and knock-on architecture review for `DEP-GW-1`.

## What Was Reviewed

1. Runtime transport boundaries (`request_flow`, effect intents, `upstream_proxy`).
2. Config/deploy guardrails and profile contracts.
3. Test/CI gate coverage and make-target naming truthfulness.
4. Operator docs and deployment skills consistency.

## Findings

1. Transport boundary clarity improved.
- Allow outcomes now converge through one adapter path.
- Enforcement-local routes remain explicit and unchanged.

2. Guardrail coverage is explicit and centralized.
- Runtime env validation and deploy preflight align on gateway contract.
- Reserved-route collision preflight is integrated into deploy validation.

3. CI gating now reflects deployment reality.
- Shared-server, edge, and smoke gateway gates are all explicit and wired in CI/release-gate workflows.

4. Operator guidance is now gateway-first.
- Deployment docs and both deploy skills include gateway-only posture, cutover, and rollback expectations.

## Residual Cleanup Opportunities (Non-Blocking)

1. Add a dedicated wasm32-only integration harness for richer upstream TLS/certificate failure permutations under real outbound transport.
2. Add an optional operator-facing origin-bypass active probe script (when environment permits) to complement origin-lock attestation.
3. Continue collapsing any remaining stale wording in legacy docs as those docs are touched by adjacent tranches.

## Follow-On Architecture Work

1. Treat wasm32 TLS-failure harness as the next trust-path hardening increment.
2. Extend deploy smoke tooling with optional origin-bypass probe contract for environments where direct-origin probing is safe/allowed.
3. Keep gateway support matrix versioned and update tests/docs together for any protocol-surface changes.
