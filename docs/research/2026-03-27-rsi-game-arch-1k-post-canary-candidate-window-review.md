Date: 2026-03-27
Status: proposed

Related context:

- [`2026-03-27-rsi-game-arch-1j-runtime-dev-effective-watch-window-post-implementation-review.md`](2026-03-27-rsi-game-arch-1j-runtime-dev-effective-watch-window-post-implementation-review.md)
- [`../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)

# Scope

Decide how the live strict Scrapling loop should materialize a protected post-canary candidate window without manual babysitting, so terminal retain-vs-rollback judgment becomes a real autonomous RSI step instead of a fail-closed cadence exercise.

# Findings

1. `RSI-GAME-ARCH-1J` solved the cadence problem:
   - runtime-dev can now reach terminal judgment without waiting a day,
   - and machine-first surfaces truthfully expose effective versus declared watch-window timing.
2. The next blocker is no longer watch-window speed.
3. Terminal judgment in `oversight_apply` still fail-closes when:
   - a canary exists,
   - the watch window elapses,
   - but there is no later comparable candidate snapshot.
4. The current rollback reason for that case is explicit:
   - `candidate_window_not_materialized`
5. The current automatic post-sim trigger happens only when adversary-sim transitions from running to off with a completed run id.
6. That trigger opens the canary, but it does not automatically schedule a second post-change Scrapling window.
7. Live proof already showed what a meaningful post-canary loop looks like:
   - after canary apply, manually rerunning Scrapling while the canary is open materializes a candidate snapshot,
   - and the next periodic supervisor cycle can then produce a real measured outcome such as `candidate_comparison_neutral`.
8. So the missing capability is not comparison logic.
9. The missing capability is candidate-window ownership:
   - who requests it,
   - who runs it,
   - how it is tied to the current canary,
   - and how the operator can see whether it is still pending, completed, or missed.

# Option Set

## Option A: Oversight agent directly reruns Scrapling after canary apply

Advantages:

1. Fastest path to a visible autonomous loop.
2. Keeps the logic close to the canary lifecycle.

Problems:

1. It collapses controller and adversary-generation ownership together.
2. It makes the judge/controller layer responsible for traffic generation.
3. It risks reentrancy and hidden coupling between canary apply and sim control semantics.

## Option B: Adversary-sim supervisor owns the post-canary follow-on run

Advantages:

1. Keeps adversary generation with the adversary-sim subsystem.
2. Lets oversight declare a need for candidate evidence without directly driving traffic generation.
3. Preserves the board-game architecture more cleanly:
   - controller changes the board,
   - adversary sim attacks the new board,
   - judge compares the outcomes.

Costs:

1. Requires a new cross-component contract:
   - active canary needs a machine-readable candidate-window request,
   - adversary-sim beat needs to honor it exactly once,
   - and operator surfaces need to show the request state.

## Option C: Keep candidate-window materialization manual or purely operator-driven

Advantages:

1. No architecture change.

Problems:

1. It does not produce a truly autonomous local RSI loop.
2. It leaves the most important live loop step dependent on manual intervention.
3. It prevents the repo from honestly claiming the live Scrapling loop is self-improving rather than merely canary-capable.

# Recommendation

Take Option B.

The oversight side should declare the need for a protected post-canary candidate window, but the adversary-sim supervisor should own actually generating that window.

That keeps the architecture cleaner:

1. defenses and config changes stay on the Shuma side,
2. adversary traffic generation stays on the attacker side,
3. and the judge compares baseline versus candidate evidence after both exist.

# Recommended Contract

1. When a canary is applied, persist an explicit pending candidate-window request tied to:
   - `canary_id`
   - `site_id`
   - `patch_family`
   - `baseline_generated_at`
   - `watch_window_end_at`
2. The adversary-sim supervisor must notice that request and schedule exactly one bounded protected Scrapling run against the changed config while the canary is open.
3. The resulting sim run must be linked back to the pending request and the canary.
4. Periodic supervisor judgment should then evaluate:
   - candidate evidence present and improved,
   - candidate evidence present and neutral or regressed,
   - or candidate evidence still missing when the watch window expires.
5. Operator surfaces should expose candidate-window request state explicitly:
   - pending
   - running
   - materialized
   - expired without candidate evidence

# Why This Is The Right Next Move

1. It preserves separation of concerns.
2. It turns the live Scrapling loop into a real two-cycle game:
   - attack current board,
   - move one bounded defense piece,
   - attack the changed board,
   - judge the delta.
3. It avoids making the controller itself the traffic generator.
4. It gives the operator explicit visibility into the one remaining live-loop blocker instead of hiding it in rollback reasons.

# Acceptance Direction

This follow-on is only complete when:

1. a canary automatically results in one protected post-change Scrapling run without manual operator action,
2. that run is explicitly linked to the active canary and candidate window,
3. periodic supervisor judgment can now produce measured `improved` or `rollback_applied` outcomes from that automatically materialized candidate evidence,
4. the system does not recursively trigger repeated post-canary runs for the same canary,
5. and operator-visible surfaces truthfully expose whether candidate evidence is pending, running, materialized, or missed.
