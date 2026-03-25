# TEST-HYGIENE-6C Review

Date: 2026-03-25

## Goal

Finish `TEST-HYGIENE-6C` by moving retained Makefile selector microtests into explicit contract or wiring lanes, so feature-behavior targets stop quietly depending on source-shape proof.

## Current State

Three families still mix real feature proof with selector-only Makefile microtests:

1. adversary-sim lanes:
   - [`scripts/tests/test_adversary_sim_make_targets.py`](../../scripts/tests/test_adversary_sim_make_targets.py)
   - its checks are useful target-shape guards, but they currently run from feature targets like `test-adversary-sim-lifecycle`, `test-adversary-sim-scrapling-category-fit`, `test-adversary-sim-scrapling-malicious-request-native`, `test-adversary-sim-scrapling-coverage-receipts`, and `test-adversarial-llm-fit`.

2. verified-identity lanes:
   - [`scripts/tests/test_verified_identity_make_targets.py`](../../scripts/tests/test_verified_identity_make_targets.py)
   - currently hidden inside multiple feature targets even though it only verifies Makefile selector wiring.

3. host-impact lanes:
   - [`scripts/tests/test_host_impact_make_targets.py`](../../scripts/tests/test_host_impact_make_targets.py)
   - likewise hidden inside feature targets instead of exposed as a target-contract lane.

## Recommended Shape

1. Keep the selector microtests.
   - They still provide useful target-truthfulness protection.

2. Give each family an explicit `make` contract lane.
   - `test-adversary-sim-make-target-contract`
   - `test-verified-identity-make-target-contract`
   - `test-host-impact-make-target-contract`

3. Remove those microtests from the feature-behavior targets.
   - The behavior targets should prove runtime, telemetry, or benchmark semantics.
   - The selector microtests should prove only target wiring.

4. Document the new split clearly in the testing guide.

## Why This Is The Right Slice

- It improves target truthfulness without deleting useful seam guards.
- It keeps the change small and local to Makefile wiring, focused tests, and docs.
- It reduces day-to-day confusion about what a given feature target actually proves.

## Expected Verification

- focused make-target contract lanes for adversary-sim, verified identity, and host impact
- one representative feature target per touched family to prove the behavior lanes still work after the split
- `git diff --check`
