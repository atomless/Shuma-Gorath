# SSH Tarpit Carry-Forward Plan

Date: 2026-02-25
Status: Active (carry-forward, deferred scope)
Supersedes: Historical baseline in [`docs/plans/2026-02-13-ssh-tarpit-excellence-plan.md`](2026-02-13-ssh-tarpit-excellence-plan.md)

## Scope

The HTTP tarpit v2 path is implemented, but SSH tarpit remains unimplemented and must stay isolated from the HTTP runtime.

## Remaining Work

1. SSH-1: Define standalone SSH tarpit service boundary and interfaces.
2. SSH-2: Define event schema for Shuma ingestion.
3. SSH-3: Implement tiered interaction modes with safety caps.
4. SSH-4: Add protocol realism variation controls.
5. SSH-5: Add anti-fingerprinting regression checks.
6. SSH-6: Add strict sandbox/egress controls and audit trails.
7. SSH-7: Add retention/privacy guardrails for captured data.
8. SSH-8: Add Shuma-side correlation of SSH signals with HTTP abuse signals.
9. SSH-9: Add isolated deployment runbook.
10. SSH-10: Add incident rollback and emergency disable procedures.

## Guardrails

- Do not embed SSH protocol handling into core HTTP request paths.
- Keep SSH deception runtime isolated (separate network policy, credentials, and failure domain).
- Treat SSH ingestion into Shuma as an external signal boundary.

## Definition of Done

- SSH tarpit is deployable as an isolated component with clear safety controls.
- Ingestion contract into Shuma is explicit, tested, and operationally documented.
