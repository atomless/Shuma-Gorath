Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`](../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../plans/2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-plan.md`](../plans/2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-plan.md)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../Makefile`](../../Makefile)

# RSI-GAME-MAINLINE: First Working Self-Improving Loop Review

## Question

What is still missing between the now-landed attacker-faithful Scrapling work and the now-landed judge-side recursive-improvement game contract?

## Conclusion

The missing piece is not more abstract game-contract work.

It is one explicit end-to-end proof that Shuma's current mainline can already do the following over the real post-sim route:

1. observe a completed attacker-faithful Scrapling run,
2. trigger the oversight agent from that completion rather than only from direct test harness calls,
3. apply one bounded legal canary move,
4. judge the watch-window outcome,
5. retain or roll back accordingly,
6. and persist the resulting episode in the archive surfaces the later game loop will use.

## What is already real

The mainline prerequisites are now landed:

1. attacker-faithful Scrapling-owned surface coverage is defined and receipt-backed,
2. the controller mutability policy and bounded legal move ring are explicit,
3. the judge-side game contract is explicit,
4. the judge scorecard is explicit,
5. and the bounded episode archive with conservative homeostasis memory is explicit.

That means the next gap is not conceptual.

It is proof-quality and route-quality.

## What is still missing

The current repo proves important pieces, but not yet the exact mainline story as one named proof:

1. [`src/admin/api.rs`](../../src/admin/api.rs) proves post-sim execution is triggered once for a completed sim run, but it does not yet prove the triggered run reaches a judged canary outcome.
2. [`src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs) proves canary apply, watch-window open, rollback, retain, and archive recording, but mostly by calling the agent directly rather than through the post-sim route path that matters to the mainline.
3. The existing focused Make targets prove the judge-side pieces individually, but there is not yet one narrow target that truthfully means "the first working game loop is proven end-to-end on the current mainline."

## Why this matters

Without this slice, the repo still has a credibility gap:

1. the attacker-faithful Scrapling work is real,
2. the judge contract is real,
3. but the first explicit self-improving loop still has to be inferred from several neighboring tests instead of being proven directly.

That is too implicit for a tranche that is supposed to mark the mainline from "useful closed loop" to "first working game loop."

## Recommended split

`RSI-GAME-MAINLINE-1` should be split into:

1. `RSI-GAME-MAINLINE-1A`
   - a local route-level proof over the existing post-sim path
   - completed Scrapling run -> post-sim oversight trigger -> bounded canary apply -> judged retain or rollback -> archive row
2. `RSI-GAME-MAINLINE-1B`
   - a stronger follow-on proof over the shared-host or higher-fidelity mainline surface once `1A` is landed
   - same contract, but at the next operational layer

## Scope of the first slice

The first slice should stay narrow:

1. do not reopen LLM attacker work,
2. do not broaden the legal move ring,
3. do not widen dashboard work,
4. do not invent a new runtime actor.

It should instead prove the existing mainline path with the current architecture.

## Result

The optimal next implementation is:

1. freeze `RSI-GAME-MAINLINE-1A` as the first explicit route-level game-loop proof,
2. land it with a narrow Make target and focused tests,
3. then move immediately to the stronger follow-on proof and later blocked player-side work.
