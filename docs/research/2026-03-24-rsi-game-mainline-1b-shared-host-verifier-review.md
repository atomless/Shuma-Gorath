Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`../../scripts/tests/live_feedback_loop_remote.py`](../../scripts/tests/live_feedback_loop_remote.py)
- [`../../scripts/tests/test_live_feedback_loop_remote.py`](../../scripts/tests/test_live_feedback_loop_remote.py)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)

# RSI-GAME-MAINLINE-1B Shared-Host Verifier Review

## Question

What is the next strongest truthful harness for `RSI-GAME-MAINLINE-1B` after the landed local route-level proof in `RSI-GAME-MAINLINE-1A`?

## Conclusion

The right next harness is the existing shared-host feedback-loop verifier layer in:

1. [`scripts/tests/live_feedback_loop_remote.py`](../../scripts/tests/live_feedback_loop_remote.py)
2. [`scripts/tests/test_live_feedback_loop_remote.py`](../../scripts/tests/test_live_feedback_loop_remote.py)

That is already the repo-owned operational harness for:

1. wrapper-chain proof,
2. public machine-first status reads,
3. internal supervisor trigger execution,
4. and completed adversary-sim linkage into the shared-host loop.

So `RSI-GAME-MAINLINE-1B` should deepen that harness rather than create a third local proof shape.

## Important constraint

`RSI-GAME-MAINLINE-1B` should not pretend the live remote can cheaply close a real watch window inside a short smoke budget.

That is blocked by current contract:

1. [`src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs) enforces `window_hours >= 1`.
2. [`src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs) uses `operator_objectives_watch_window_seconds(...)`, which is `window_hours * 3600`.

So a real live canary judgment cannot be forced to complete in a few seconds without mutating operator objectives or cheating the protected watch-window rule.

## What this means for `1B`

`1B` should become:

1. a stronger shared-host verifier proof,
2. over the same post-sim -> bounded move -> judged terminal outcome -> archive contract,
3. but exercised in the unitized shared-host verifier harness rather than by waiting an hour on a live host.

That keeps the proof honest:

1. it uses the next operational layer up from the local Rust route test,
2. it still proves the shared-host/public-status/internal-supervisor orchestration semantics,
3. and it does not lie about what the live remote can finish inside a focused verification target.

## Recommended shape

1. Extend the live shared-host verifier logic so it understands:
   - terminal apply stages,
   - completed episode-archive rows,
   - and the follow-on periodic judgment step after a post-sim watch window opens.
2. Prove that behavior first in `scripts/tests/test_live_feedback_loop_remote.py`.
3. Keep the actual live remote proof focused on the real shared-host wrapper and trigger surfaces unless and until the project introduces a safe operator-approved way to run a short live watch window.

## Result

The next implementation should:

1. land `RSI-GAME-MAINLINE-1B` against the shared-host verifier behavior layer,
2. add archive-aware post-sim terminal judgment proof there,
3. and leave a later live-terminal proof as a separate follow-on if the project wants one.
