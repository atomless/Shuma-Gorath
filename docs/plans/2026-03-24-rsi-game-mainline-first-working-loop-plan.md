Date: 2026-03-24
Status: Proposed

Related context:

- [`../research/2026-03-24-rsi-game-mainline-first-working-loop-review.md`](../research/2026-03-24-rsi-game-mainline-first-working-loop-review.md)
- [`2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`](2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md)
- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-plan.md`](2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-plan.md)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)

# RSI-GAME-MAINLINE First Working Loop Plan

## Goal

Prove the first explicit working self-improving loop on the current mainline:

1. attacker-faithful Scrapling completes a run,
2. the post-sim route triggers the oversight agent,
3. the agent applies one bounded legal canary move,
4. the watch window is judged,
5. the system retains or rolls back,
6. and the episode is persisted to the machine-first archive surfaces.

## Architecture

Do not build a new loop.

Use the already-landed pieces together:

1. attacker-faithful Scrapling-owned surface receipts,
2. the current post-sim trigger path in `src/admin/api.rs`,
3. the existing canary apply and watch-window machinery in `src/admin/oversight_agent.rs`,
4. the episode archive in `src/admin/oversight_api.rs`,
5. and the existing status/history machine-first surfaces.

## Guardrails

1. Keep the first slice local and route-level.
2. Do not reopen later LLM attacker or defender work.
3. Do not widen controller mutability or move selection in this tranche.
4. Do not treat dashboard work as part of this proof.
5. Reuse existing test-seeding patterns or extract shared test support rather than duplicating a third fork of canary snapshot helpers.

## Task 1: Split The Mainline Tranche

**Files:**

- Modify: `todos/todo.md`
- Modify: `docs/plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`

**Work:**

1. Replace umbrella `RSI-GAME-MAINLINE-1` with:
   1. `RSI-GAME-MAINLINE-1A` local route-level working-loop proof
   2. `RSI-GAME-MAINLINE-1B` stronger follow-on mainline proof after `1A`
2. Make `1A` the active tranche now.
3. Keep `1B` explicit so the mainline does not regress back into a vague umbrella after the first proof lands.

**Acceptance criteria:**

1. The backlog names a concrete first slice rather than one umbrella label.
2. The planning chain makes clear what `1A` proves and what remains for `1B`.

## Task 2: `RSI-GAME-MAINLINE-1A`

### Local route-level proof of the first working loop

**Files:**

- Modify: `src/admin/api.rs`
- Modify: `src/admin/oversight_agent.rs` or shared internal test support if extraction is needed
- Modify: `Makefile`
- Modify: `docs/testing.md`
- Modify: `docs/api.md`

**Work:**

1. Add a focused route-level proof that starts from a completed Scrapling-owned sim run.
2. Trigger the oversight agent through the post-sim route path rather than only by direct `execute_agent_cycle(...)` calls.
3. Prove one bounded canary move is applied under the existing legal move ring.
4. Prove the follow-on judged cycle reaches a terminal retain or rollback outcome.
5. Prove `oversight_history_v1`, `oversight_agent_status_v1`, or `operator_snapshot_v1` surface the resulting completed episode archive row.
6. Add or refine a focused Make target, for example `test-rsi-game-mainline`, that truthfully means the first working mainline loop is proven.

**Acceptance criteria:**

1. The repo has one narrow proof that the current mainline already forms a working self-improving loop.
2. The proof uses the real post-sim trigger path, not only direct test harness calls.
3. The proof shows canary apply plus judged retain/rollback plus archive persistence.

**Verification:**

1. `make test-rsi-game-mainline`
2. `make test-oversight-agent`
3. `make test-oversight-episode-archive`
4. `git diff --check`

## Task 3: `RSI-GAME-MAINLINE-1B`

### Stronger follow-on mainline proof

**Files:**

- Modify: the most truthful existing proof harness once `1A` is landed
- Modify: `docs/testing.md`

**Work:**

1. Extend the first proof into the next operational layer after `1A` lands.
2. Keep the same contract:
   1. attacker-faithful Scrapling pressure
   2. post-sim trigger
   3. bounded move
   4. judged terminal outcome
   5. archive persistence
3. Use the next strongest truthful harness rather than broadening the local unit proof indefinitely.

**Acceptance criteria:**

1. Shuma has both a narrow local proof and a stronger operational proof of the first working loop.

## Recommended implementation order

1. Land the planning split and focused proof target name.
2. Land `RSI-GAME-MAINLINE-1A`.
3. Review whether helper extraction is needed before `1B`.
4. Then move to `RSI-GAME-MAINLINE-1B`.
