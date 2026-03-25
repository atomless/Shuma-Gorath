# TEST-HYGIENE-6B Post-Implementation Review

Date: 2026-03-24
Status: Closed

Related context:

- [`2026-03-23-testing-surface-audit.md`](2026-03-23-testing-surface-audit.md)
- [`../plans/2026-03-23-testing-surface-rationalization-plan.md`](../plans/2026-03-23-testing-surface-rationalization-plan.md)
- [`../plans/2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-plan.md`](../plans/2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-plan.md)

## Scope Reviewed

This closeout reviewed the non-dashboard `TEST-HYGIENE-6B` slice:

1. separate live feedback-loop feature proof from retained wrapper and remote wiring contract proof,
2. make the integration cleanup shell-shape lane explicit in the `make` surface,
3. and keep the retained shell checks as contract proof rather than deleting them.

## What Landed

1. [`scripts/tests/test_live_feedback_loop_remote.py`](../../scripts/tests/test_live_feedback_loop_remote.py) is now split into:
   - `LiveFeedbackLoopRemoteBehaviorTests`
   - `LiveFeedbackLoopRemoteContractTests`
2. [`Makefile`](../../Makefile) now exposes a dedicated local contract lane:
   - `make test-live-feedback-loop-remote-contracts`
3. `make test-live-feedback-loop-remote-unit` now means local verifier behavior proof only, instead of quietly mixing in remote wrapper/process-tree contract checks.
4. The retained integration cleanup source-shape lane now has an explicit contract name:
   - `make test-integration-cleanup-contract`
5. [`make test`](../../Makefile) and [`make test-integration`](../../Makefile) now call the renamed integration cleanup contract lane before the real Spin HTTP integration run.
6. [`docs/testing.md`](../testing.md) now documents the narrowed live-feedback-loop behavior target, the new remote wiring contract target, and the renamed integration cleanup contract lane.

## Review Result

This slice achieved the intended cleanup:

1. the remaining live feedback-loop local wrapper checks no longer hide inside behavior proof,
2. the integration cleanup shell-shape guard is now named as what it is,
3. and the repo keeps the useful contract guards without pretending they are end-to-end feature behavior.

## Residual Follow-On

1. `TEST-HYGIENE-6C`
   - feature-specific Makefile selector microtests still need their own explicit contract or wiring lanes.
2. [`scripts/tests/test_integration_cleanup.py`](../../scripts/tests/test_integration_cleanup.py) still uses source-text shell proof.
   - That is acceptable for now because the lane is now explicitly contractual.
   - A later executable harness would be an improvement, but it is no longer mislabeled.

## Verification

- `make test-live-feedback-loop-remote-unit`
- `make test-live-feedback-loop-remote-contracts`
- `make test-integration-cleanup-contract`
- `git diff --check`
