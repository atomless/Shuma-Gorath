# TEST-HYGIENE-6B Live Wrapper And Integration Contract Plan

Date: 2026-03-24

Related research:

- [`../research/2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-review.md`](../research/2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-review.md)
- [`2026-03-23-testing-surface-rationalization-plan.md`](2026-03-23-testing-surface-rationalization-plan.md)

## Objective

Complete the current `TEST-HYGIENE-6B` slice by separating retained wrapper and shell-cleanup contract proof from feature-behavior proof in the live feedback-loop and integration lanes.

## Tasks

1. Split local live feedback-loop verifier coverage.
   - Keep behavior tests in [`scripts/tests/test_live_feedback_loop_remote.py`](../../scripts/tests/test_live_feedback_loop_remote.py).
   - Move wrapper/service-tree contract checks into a dedicated contract file.
   - Add a dedicated `make` target for that contract lane.

2. Make the integration cleanup shell-check lane explicitly contractual.
   - Rename the `make` target from `test-integration-script-unit` to a cleanup-contract name.
   - Update the umbrella integration targets to call the renamed contract lane.

3. Refresh documentation and backlog references.
   - Update [`docs/testing.md`](../testing.md).
   - Update README indexes for the new review/plan docs.
   - Close `TEST-HYGIENE-6B` in the TODO trail if the remaining open work is now only `TEST-HYGIENE-6C`.

4. Write the post-implementation review and completion record.

## Verification

- `make test-live-feedback-loop-remote-unit`
- `make test-live-feedback-loop-remote-contracts`
- `make test-integration-cleanup-contract`
- `git diff --check`
