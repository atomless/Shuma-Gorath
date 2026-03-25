# TEST-HYGIENE-6C Make Selector Contract Plan

Date: 2026-03-25

Related research:

- [`../research/2026-03-25-test-hygiene-6c-make-selector-contract-review.md`](../research/2026-03-25-test-hygiene-6c-make-selector-contract-review.md)
- [`2026-03-23-testing-surface-rationalization-plan.md`](2026-03-23-testing-surface-rationalization-plan.md)

## Objective

Complete `TEST-HYGIENE-6C` by separating retained Makefile selector microtests from feature-behavior targets and surfacing them as explicit contract lanes.

## Tasks

1. Add a focused contract test for the new split.
   - Prove the new explicit contract targets exist.
   - Prove the touched feature targets no longer call the selector microtests directly.

2. Rewire Makefile targets.
   - Add explicit contract lanes for:
     - adversary-sim make-target wiring,
     - verified-identity make-target wiring,
     - host-impact make-target wiring.
   - Remove the selector microtest invocations from the feature-behavior targets that currently hide them.

3. Refresh documentation and backlog truth.
   - Update [`docs/testing.md`](../testing.md) so the new contract lanes are discoverable.
   - Update README indexes for the new review and plan docs.
   - Close `TEST-HYGIENE-6C` in the TODO trail if the targeted selector-microtest drift is fully removed.

4. Write the post-implementation review and completion record.

## Verification

- `make test-adversary-sim-make-target-contract`
- `make test-verified-identity-make-target-contract`
- `make test-host-impact-make-target-contract`
- `make test-adversary-sim-scrapling-category-fit`
- `make test-verified-identity-guardrails`
- `make test-oversight-host-impact`
- `git diff --check`
