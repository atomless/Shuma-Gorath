# Puzzle Challenge Carry-Forward Plan

Date: 2026-02-25
Status: Active (carry-forward)
Supersedes: Historical baseline in [`docs/plans/2026-02-13-puzzle-challenge-excellence-plan.md`](2026-02-13-puzzle-challenge-excellence-plan.md)

## Scope

This plan captures remaining puzzle challenge work after the 2026-02-13 plan.

Delivered baseline already includes:
- Signed puzzle seed/token flow with expiry.
- Replay and ordering protections through shared operation-envelope checks.
- Attempt-window controls.
- Not-a-Bot to Puzzle routing integration.

## Remaining Work

1. PZ-1: Add variant-family abstraction and versioning.
   - Move beyond single transform-family assumptions to explicit variant family contracts.

2. PZ-2: Add variant rotation controls and metadata logging.
   - Support controlled family rollout and operator-visible version metadata.

3. PZ-7: Add accessibility-equivalent puzzle modality.
   - Provide equivalent-strength accessible path with matching signature/expiry/replay guarantees.

4. PZ-8: Add per-variant solve-rate and latency metrics.
   - Track solve quality by variant family/version to guide rollout decisions.

5. PZ-9: Add adversarial solver regression coverage.
   - Add deterministic regression scenarios for scripted/solver-assisted attack attempts.

6. PZ-10: Publish operator tuning and rollback guidance.
   - Include variant rollout gates, fallback thresholds, and reversal criteria.

## Definition of Done

- Puzzle variants are first-class, versioned, and rotatable.
- Accessibility path has verification parity with standard puzzle flow.
- Per-variant observability and adversarial regression coverage are in canonical test runs.
