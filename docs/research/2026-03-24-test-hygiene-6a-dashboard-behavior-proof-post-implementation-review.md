# TEST-HYGIENE-6A Post-Implementation Review

Date: 2026-03-24
Status: Closed

Related context:

- [`2026-03-23-testing-surface-audit.md`](2026-03-23-testing-surface-audit.md)
- [`../plans/2026-03-23-testing-surface-rationalization-plan.md`](../plans/2026-03-23-testing-surface-rationalization-plan.md)
- [`../../e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js)
- [`../../Makefile`](../../Makefile)
- [`../testing.md`](../testing.md)

## Scope Reviewed

This closeout reviewed the first execution slice of `TEST-HYGIENE-6`:

1. replace the audit-cited dashboard Node source-regex tests with behavior-first proof,
2. keep explicit contract lanes intact where source shape is still the real contract,
3. and make the focused dashboard make targets more truthful about what they actually verify.

## What Landed

1. Replaced the old native-runtime source-regex ownership test with behavior proof that now drives:
   - runtime mount,
   - session restore,
   - heartbeat-owned connected state,
   - tab normalization,
   - config mutation invalidation,
   - and logout session clearing.
2. Replaced the old refresh-runtime source-regex ownership test with behavior proof that now drives:
   - cache removal,
   - freshness reset,
   - and runtime state cleanup through `clearAllCaches()`.
3. Replaced the old Monitoring accountability wiring regex test with behavior proof that now drives:
   - shared config refresh,
   - operator snapshot materialization,
   - benchmark result materialization,
   - oversight history materialization,
   - and oversight agent-status materialization.
4. Added a new focused make target, `make test-dashboard-runtime-unit-contracts`, for the native/refresh runtime behavior proofs.
5. Narrowed `make test-dashboard-verified-identity-pane` so it no longer quietly depends on an unrelated refresh-runtime archaeology test.

## Review Result

This slice achieved the intended first dashboard cleanup:

1. the three examples called out most directly by the audit no longer prove behavior by regexing runtime source files,
2. the focused make targets now map more honestly to the dashboard surfaces they claim to verify,
3. and the resulting proof is still small and fast enough to stay useful as a routine focused gate.

## Residual Follow-On

1. `TEST-HYGIENE-6B`
   - shell-wrapper and integration-cleanup archaeology still needs its own executable or explicitly reclassified proof path.
2. `TEST-HYGIENE-6C`
   - feature-specific Makefile selector microtests still need to be moved into explicit contract or wiring lanes.
3. Some dashboard source-contract tests still remain intentionally in explicit IA or ownership lanes.
   - That is acceptable where file shape is the contract.
   - The goal of this slice was to remove the low-signal behavior impostors first, not to erase all source-contract proof.

## Verification

- `make test-dashboard-runtime-unit-contracts`
- `make test-dashboard-game-loop-accountability`
- `make test-dashboard-verified-identity-pane`
- `git diff --check`
