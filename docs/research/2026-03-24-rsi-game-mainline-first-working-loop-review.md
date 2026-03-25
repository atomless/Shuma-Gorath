Date: 2026-03-24
Status: Proposed

Related context:

- [`2026-03-22-live-linode-feedback-loop-proof.md`](2026-03-22-live-linode-feedback-loop-proof.md)
- [`2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md`](2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md)
- [`2026-03-24-sim-scr-challenge-2d-coverage-closure-and-gap-assignment-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2d-coverage-closure-and-gap-assignment-post-implementation-review.md)
- [`2026-03-24-sim-scr-geo-1-public-network-identity-post-implementation-review.md`](2026-03-24-sim-scr-geo-1-public-network-identity-post-implementation-review.md)
- [`../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`](../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../src/observability/operator_snapshot_live_traffic.rs`](../../src/observability/operator_snapshot_live_traffic.rs)
- [`../../scripts/tests/live_feedback_loop_remote.py`](../../scripts/tests/live_feedback_loop_remote.py)

# RSI-GAME-MAINLINE-1 Review

## Question

What is still missing before Shuma can truthfully say it has a first working self-improving loop over the now attacker-faithful Scrapling basis?

## Conclusion

The missing piece is not a new controller architecture.

Shuma already has:

1. an attacker-faithful request-native Scrapling lane with owned-surface coverage receipts,
2. a bounded shared-host oversight loop with periodic and post-sim triggers,
3. a legal move ring,
4. explicit judge semantics,
5. and a bounded episode archive.

What is still missing is one explicit proof contract that ties those pieces together and shows the loop is operating over the truthful attacker basis rather than only proving controller wiring in isolation.

## Current gap

The current live and focused proof chain still leaves a representational gap:

1. the Scrapling follow-ons prove coverage and owned-surface receipts,
2. the oversight loop proofs prove periodic and post-sim bounded execution,
3. and the episode-archive tranche proves machine-first episode lineage,
4. but there is not yet one named tranche that proves those three truths together as a single working loop.

In particular:

1. the live verifier still proves "feedback loop runs" more than "feedback loop runs over Scrapling-owned hostile pressure",
2. its unit fixtures are still synthetic-lane shaped,
3. and the proof does not yet insist on a coherent chain of:
   - Scrapling run,
   - Scrapling owned-surface coverage,
   - post-sim oversight decision,
   - and episode/archive lineage tied to that run.

## What `RSI-GAME-MAINLINE-1` should prove

This tranche should freeze and prove the smallest truthful claim:

1. the active adversary lane is `scrapling_traffic`,
2. the observed recent run and coverage summary show Scrapling-owned surfaces are covered,
3. a post-sim oversight cycle is recorded against that attacker evidence,
4. and the resulting episode/archive lineage records the move and its current outcome state.

That means the proof should be explicit about:

1. attacker basis:
   - `scrapling_traffic`
   - recent sim run receipts
   - owned-surface coverage summary
2. judge execution:
   - post-sim oversight run
   - valid bounded apply stage
   - decision and history linkage
3. episode memory:
   - archive row exists
   - it is linked to the current `sim_run_id`
   - its acceptance, watch-window, retention, and completion statuses are coherent with the recorded apply stage

## Recommended proof shape

Use two complementary proof paths:

1. cheap local proof
   - add a focused Rust test that drives a post-sim Scrapling-backed episode through the bounded loop and proves archive linkage over attacker-faithful recent-run receipts
2. live shared-host proof
   - strengthen the existing live verifier so it explicitly checks:
     - the run is Scrapling,
     - Scrapling coverage is covered,
     - and the post-sim run is represented in oversight history or episode archive

This keeps the local proof deterministic and cheap, while leaving the live proof as the higher-confidence confirmation that the deployed loop is using the same contract.

## Non-goals

`RSI-GAME-MAINLINE-1` should not:

1. reopen browser or stealth Scrapling,
2. reopen the LLM attacker or defender runtimes,
3. invent a second controller path,
4. or require the live host to retain or roll back a canary on demand in order to count as operational.

The tranche only needs to prove that the first bounded self-improving loop is truly running over the attacker-faithful Scrapling basis and that its machine-first lineage is coherent.

## Result

The right implementation is:

1. create a dedicated `RSI-GAME-MAINLINE-1` plan,
2. add a focused `make` proof target,
3. strengthen the local and live proof surfaces around Scrapling plus episode lineage,
4. and then close the backlog item only after those proofs pass.
