Date: 2026-03-24
Status: Completed

Related context:

- [`../2026-03-24-rsi-game-mainline-first-working-loop-review.md`](2026-03-24-rsi-game-mainline-first-working-loop-review.md)
- [`../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/test_support.rs`](../../src/test_support.rs)
- [`../../Makefile`](../../Makefile)
- [`../../docs/testing.md`](../../docs/testing.md)

# RSI-GAME-MAINLINE-1A First Working Loop Post-Implementation Review

## What landed

`RSI-GAME-MAINLINE-1A` is now real.

The repo now has one focused proof path showing the current mainline can already do the first working loop in two adjacent steps:

1. the completed adversary-sim path triggers the post-sim oversight run once and dedupes by `sim_run_id`,
2. the post-sim oversight route can then apply one bounded canary, close the watch window, and persist a completed retained episode into the archive surfaces.

## What changed

1. [`src/admin/api.rs`](../../src/admin/api.rs)
   - added `make_internal_oversight_request(...)`
   - added `post_sim_oversight_route_can_apply_improve_and_archive_first_working_game_loop`
2. [`src/test_support.rs`](../../src/test_support.rs)
   - added shared canary/objective snapshot seed helpers for route-level oversight proofs
3. [`Makefile`](../../Makefile)
   - added `make test-rsi-game-mainline`
4. [`docs/testing.md`](../../docs/testing.md)
   - documented the new focused proof path

## What the proof now covers

1. automatic post-sim trigger from completed sim state
2. route-level `post_adversary_sim` execution, not direct `execute_agent_cycle(...)` only
3. bounded canary apply under the legal move ring
4. terminal watch-window judgment via the periodic supervisor route
5. completed episode archive projection through machine-first status and history surfaces

## Important implementation note

The first failing version of this slice exposed a real architectural seam:

1. the adversary-sim beat path refreshes the operator snapshot from underlying telemetry,
2. so a hand-seeded hot-read document is not enough to prove a post-sim apply directly through the beat hook,
3. and the first mainline proof is cleaner when split into:
   - automatic post-sim hook proof
   - route-level post-sim working-loop proof

That is not a weakness in the slice.

It is a more truthful decomposition of the current mainline.

## Remaining follow-on

`RSI-GAME-MAINLINE-1B` is still needed.

The next stronger proof should move the same contract into the next operational layer rather than keep extending this local route-level harness indefinitely.

## Verification

1. `make test-rsi-game-mainline`
2. `make test-oversight-agent`
3. `make test-oversight-episode-archive`
4. `git diff --check`
