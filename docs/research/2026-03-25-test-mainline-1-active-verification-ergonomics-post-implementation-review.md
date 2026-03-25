# TEST-MAINLINE-1 Post-Implementation Review

Date: 2026-03-25

## Scope Reviewed

This closeout reviewed the active-mainline verification ergonomics tranche:

- add one truthful aggregate `make` path for the current attacker-faithful Scrapling -> first-working-game-loop mainline,
- surface that path clearly in the testing guide,
- and prove the new command itself without broadening the slice into unrelated testing cleanup.

Reference plan:

- [`../plans/2026-03-25-testing-suite-structure-and-mainline-friction-plan.md`](../plans/2026-03-25-testing-suite-structure-and-mainline-friction-plan.md)

## What Landed

1. [`../../Makefile`](../../Makefile)
   - added `make test-scrapling-game-loop-mainline`
   - wired it as a truthful aggregate over:
     - `test-adversary-sim-scrapling-owned-surface-contract`
     - `test-adversary-sim-scrapling-malicious-request-native`
     - `test-adversary-sim-scrapling-coverage-receipts`
     - `test-rsi-game-mainline`
   - added the target to `.PHONY`

2. [`../../scripts/tests/test_adversary_sim_make_targets.py`](../../scripts/tests/test_adversary_sim_make_targets.py)
   - added a focused target-contract test proving the new aggregate stays aligned to the current active mainline and does not quietly widen into live/shared-host or deeper adversarial oracle coverage

3. [`../../docs/testing.md`](../../docs/testing.md)
   - added the new command to the official quick-command list
   - added a short `Current Active Mainline` section near the top
   - clarified that the new bundle is local/pre-merge proof only and does not replace `make test`, deeper adversarial coverage, or live/shared-host proof

## Verification Evidence

The tranche was verified with:

- `make test-adversary-sim-scrapling-category-fit`
- `make test-scrapling-game-loop-mainline`
- `git diff --check`

## Outcome

The repo now has a clear answer to:

- "what should I run for the current active mainline?"

without forcing contributors to reconstruct that answer from four separate focused commands and a long testing guide.

## Follow-On Notes

1. `TEST-MAINLINE-1` does not replace the broader testing-rationalization backlog.
2. The remaining follow-on testing hygiene still stands:
   - `TEST-HYGIENE-6C`
   - `TEST-HYGIENE-3`
   - `TEST-HYGIENE-4`
   - `TEST-HYGIENE-5`
   - `TEST-HYGIENE-2`
3. No tracked artifact churn was observed from the new active-mainline aggregate itself in this slice.
