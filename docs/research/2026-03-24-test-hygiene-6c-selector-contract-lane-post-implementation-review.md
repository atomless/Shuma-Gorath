# TEST-HYGIENE-6C Selector Contract Lane Post-Implementation Review

Date: 2026-03-24
Status: Closed

Related context:

- [`2026-03-23-testing-surface-audit.md`](2026-03-23-testing-surface-audit.md)
- [`../plans/2026-03-23-testing-surface-rationalization-plan.md`](../plans/2026-03-23-testing-surface-rationalization-plan.md)
- [`../../Makefile`](../../Makefile)
- [`../../scripts/tests/test_adversary_sim_make_targets.py`](../../scripts/tests/test_adversary_sim_make_targets.py)
- [`../../scripts/tests/test_host_impact_make_targets.py`](../../scripts/tests/test_host_impact_make_targets.py)
- [`../../scripts/tests/test_verified_identity_make_targets.py`](../../scripts/tests/test_verified_identity_make_targets.py)
- [`../testing.md`](../testing.md)

## Scope Reviewed

This tranche closed `TEST-HYGIENE-6C` by reclassifying feature-specific Makefile selector microtests into explicit contract lanes instead of leaving them hidden inside feature-behavior targets.

## What Landed

1. Added explicit Makefile selector and wiring lanes for the three remaining selector-microtest families:
   - `make test-adversary-sim-target-contracts`
   - `make test-verified-identity-target-contracts`
   - `make test-host-impact-target-contracts`
2. Removed selector-only Python tests from the behavior-oriented targets they had been piggybacking on:
   - adversary-sim lifecycle and Scrapling category-fit
   - verified-identity calibration, alignment, conflict-metric, and guardrail gates
   - host-impact telemetry, benchmark, and reconcile gates
3. Tightened the selector test files so they now prove:
   - the explicit contract lane owns the selector microtests,
   - and the feature gates no longer quietly include those file-shape checks.
4. Updated [`../testing.md`](../testing.md) so the new target-contract lanes are discoverable and the feature-gate descriptions no longer imply hidden selector proof.
5. During representative verification, fixed one stale expectation in:
   - [`../../src/observability/operator_snapshot_verified_identity.rs`](../../src/observability/operator_snapshot_verified_identity.rs)
   where a disabled-site test was still assuming verified identity defaulted off after the repo intentionally flipped it on by default.

## Review Result

This is the correct closure for `TEST-HYGIENE-6C`:

1. the selector-microtests remain useful,
2. they are now named and owned as Makefile contract proof rather than behavior proof,
3. and the affected feature gates still pass on their real runtime or benchmark behavior after the extraction.

## Verification

- `make test-adversary-sim-target-contracts`
- `make test-verified-identity-target-contracts`
- `make test-host-impact-target-contracts`
- `make test-adversary-sim-domain-contract`
- `make test-verified-identity-calibration-readiness`
- `make test-host-impact-telemetry`
- `git diff --check`
