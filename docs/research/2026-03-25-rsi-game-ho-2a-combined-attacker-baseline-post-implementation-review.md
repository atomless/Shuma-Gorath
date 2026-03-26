Date: 2026-03-25
Status: Completed

Related context:

- [`2026-03-25-rsi-game-ho-2a-combined-attacker-baseline-readiness-review.md`](2026-03-25-rsi-game-ho-2a-combined-attacker-baseline-readiness-review.md)
- [`../plans/2026-03-25-rsi-game-ho-2a-combined-attacker-baseline-plan.md`](../plans/2026-03-25-rsi-game-ho-2a-combined-attacker-baseline-plan.md)

# RSI-GAME-HO-2A Combined-Attacker Baseline Post-Implementation Review

## What landed

`RSI-GAME-HO-2A` is now complete.

Shuma now has a focused strict `human_only_private` proof that bridges the previously separate attacker stories:

1. a Scrapling-driven strict-baseline cycle can retain a bounded config change,
2. a later `bot_red_team` run can then open the next canary on that retained config,
3. and both lanes remain visible in the same recent-run machine-first evidence surface.

The route-level proof landed in:

- [`../../src/admin/api.rs`](../../src/admin/api.rs)

The focused verification surface landed in:

- [`../../Makefile`](../../Makefile)
- [`../../scripts/tests/test_adversary_sim_make_targets.py`](../../scripts/tests/test_adversary_sim_make_targets.py)
- [`../../docs/testing.md`](../../docs/testing.md)

through:

1. `make test-rsi-game-human-only-mixed-baseline`

## Important implementation result

This tranche did **not** require a new runtime mechanism.

The red test showed that the mixed-attacker path was already supported by the existing loop machinery. The initial failure came from proof setup: appending simulation events refreshes the hot-read operator snapshot, which overwrote the deliberately seeded apply-ready benchmark snapshot used by the test harness.

So the right fix was:

1. keep the existing runtime unchanged,
2. order the proof setup so recent-run evidence exists without clobbering the seeded strict-loop benchmark state,
3. and add the missing focused proof target instead of inventing new mixed-attacker runtime branches.

That is a good result because it means the first mixed-attacker seam was primarily a missing proof contract, not a hidden architecture gap.

## Verification

- `make test-rsi-game-human-only-mixed-baseline`
- `make test-adversary-sim-make-target-contract`
- `git diff --check`

## What remains

`RSI-GAME-HO-2A` proves the first mixed-attacker strict-baseline seam. It does **not** yet prove:

1. repeated retained improvement under combined attacker pressure,
2. or the later unlock condition for any relaxed verified-identity stance.

So the next active tranche is:

1. `RSI-GAME-HO-2B`

That slice should extend this first mixed-attacker seam into repeated retained config-change improvement rather than stopping at the first truthful combined-attacker canary handoff.
