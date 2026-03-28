Date: 2026-03-28
Status: active

Related context:

- [`../research/2026-03-28-rsi-game-arch-1l-persistent-iterative-loop-review.md`](../research/2026-03-28-rsi-game-arch-1l-persistent-iterative-loop-review.md)
- [`../research/2026-03-27-rsi-game-arch-1k-post-canary-candidate-window-review.md`](../research/2026-03-27-rsi-game-arch-1k-post-canary-candidate-window-review.md)
- [`../research/2026-03-28-rsi-game-arch-1k-post-canary-candidate-window-post-implementation-review.md`](../research/2026-03-28-rsi-game-arch-1k-post-canary-candidate-window-post-implementation-review.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../plans/2026-03-27-rsi-game-arch-1k-post-canary-candidate-window-plan.md`](../plans/2026-03-27-rsi-game-arch-1k-post-canary-candidate-window-plan.md)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../scripts/run_with_oversight_supervisor.sh`](../../scripts/run_with_oversight_supervisor.sh)
- [`../../docs/testing.md`](../../docs/testing.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Objective

Turn the current one-episode proof into an explicit persistent RSI contract:

1. the controller must keep opening bounded episodes while the defended board is still over budget and not yet homeostatic,
2. it must continue after either `improved` or `rollback_applied`,
3. and it must stop only for explicit machine-readable reasons.

# Core Decisions

1. Keep one bounded config mutation per judged cycle.
2. Keep adversary generation with the adversary-sim supervisor, not the oversight controller.
3. Use the existing periodic supervisor beat as the continuation driver unless proof shows a real gap.
4. Add controller code only if the new proof fails.
5. Expose explicit continuation or stop reasons rather than relying on operators to infer them from archive rows.

# Execution Tranche

## `RSI-GAME-ARCH-1L`

### Persistent iterative bounded-episode continuity

Required contract:

1. after a terminal `improved` judgment, if the latest board state is still outside budget and bounded tuning remains eligible, the next controller beat must open the next bounded canary automatically,
2. after a terminal `rollback_applied` judgment, the same must happen under the same conditions,
3. the loop must stop automatically when:
   1. the latest board state is inside budget,
   2. homeostasis is reached,
   3. bounded tuning is no longer eligible,
   4. the config ring is exhausted,
   5. or the next gap is code-only,
4. the stop or continue reason must be machine-readable,
5. and runtime-dev proof must use the existing shortest meaningful `30s` candidate rerun rather than relaxing the defended stance.

Implementation steps:

1. Add failing tests first:
   - oversight-agent proof that a retained-but-still-overbudget cycle leads to the next canary on the next periodic supervisor beat,
   - oversight-agent proof that a rolled-back-but-still-overbudget cycle does the same,
   - a stop-condition proof that inside-budget or config-ring-exhausted state does not reopen another canary,
   - and, if needed, route-level proof in the existing `test-rsi-game-mainline` bundle.
2. Check whether the current controller already satisfies those proofs:
   - if yes, keep the code path as-is and only codify the contract in tests, docs, and machine-first notes,
   - if no, add the smallest controller change needed so the next periodic beat opens the next canary automatically after terminal judgment.
3. Add or refine machine-first continuation visibility only if the proof shows operators cannot tell why the loop continued or stopped.
4. Update test and operator docs so the loop is described as a persistent bounded episode runner, not a one-shot canary tool.
5. Keep cadence changes out of this slice unless the proof shows cadence, not controller logic, is the remaining blocker.

Acceptance criteria:

1. a retained terminal episode that is still outside budget leads to a new bounded canary on the next periodic supervisor beat,
2. a rolled-back terminal episode that is still outside budget does the same,
3. an inside-budget or otherwise terminally blocked state does not reopen another canary,
4. the proof path is canonical and focused,
5. operator docs now describe the loop as repeating until under budget or homeostasis, rather than implying a one-episode stop.

Proof:

1. `make test-oversight-agent`,
2. `make test-rsi-game-mainline`,
3. `make test-rsi-game-human-only-proof`,
4. `make test`,
5. and cited machine-first evidence from:
   - `GET /admin/oversight/agent/status`
   - `GET /admin/oversight/history`
   - `GET /admin/operator-snapshot`

# Sequencing

1. Land `RSI-GAME-ARCH-1L` immediately after `RSI-GAME-ARCH-1K`.
2. Only after this persistent-loop contract is proven should the repo reopen cadence-optimization questions for runtime-dev.
3. Keep `SIM-LLM-1C3` blocked behind a truthful persistent Scrapling-first RSI loop, not just one judged episode.

# Definition Of Done

This tranche is complete when:

1. the repo has explicit proof that the shared-host Scrapling controller keeps iterating bounded episodes until a real stop condition is reached,
2. stop conditions are machine-explicit,
3. the next-episode continuation path is proven after both retained and rolled-back terminal outcomes,
4. and the docs no longer describe the Game Loop as if one judged cycle were the endpoint.
