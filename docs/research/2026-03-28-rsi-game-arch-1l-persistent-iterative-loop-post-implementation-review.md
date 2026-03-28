# RSI-GAME-ARCH-1L Persistent Iterative Loop Post-Implementation Review

Date: 2026-03-28  
Status: implemented

## Scope

Prove and, where needed, repair the shared-host Scrapling RSI loop so it does not stop after one terminal judged cycle. Instead it must:

1. judge one bounded canary episode,
2. request a fresh bounded Scrapling rerun when the board is still outside budget,
3. let the adversary-sim supervisor materialize that rerun,
4. judge the fresh evidence,
5. and only then decide whether to open the next bounded config move or stop for an explicit machine-readable reason.

## What Landed

1. Terminal judged episodes now persist explicit continuation truth in the oversight agent state:
   - `pending` when a fresh bounded rerun is required after `improved` or `rollback_applied` and the board is still outside budget,
   - `running` once adversary-sim supervisor has started that rerun,
   - `stopped` when the loop has reached a stop reason such as `inside_budget`, `homeostasis`, `config_ring_exhausted`, or `code_evolution_referral`.
2. The adversary-sim supervisor can now wake on continuation demand, not only on already-running generation:
   - `/admin/adversary-sim/status` exposes `supervisor_attention_required`,
   - and the supervisor wrapper now starts a worker when either generation is active or a pending continuation rerun needs materialization.
3. Internal adversary-sim beats now own continuation rerun materialization in the same ownership style as candidate-window materialization:
   - if no post-canary candidate window is pending,
   - and continuation state is `pending`,
   - one fresh bounded Scrapling rerun is started with reason `loop_continuation_follow_on`.
4. The next bounded config move is now gated behind fresh evidence instead of immediate patch chaining:
   - after terminal `improved` or `rollback_applied`,
   - the controller requests a rerun first,
   - and only the later post-rerun oversight judgment may open the next bounded canary.
5. The continuation helper now follows the same persistence discipline as the candidate-window helper:
   - it prepares the `running` state in memory,
   - but does not persist that state until the adversary-sim control-state save succeeds,
   - so operator-visible continuation status cannot drift into a false `running` state if the beat loses the state write race.
6. Runtime-dev keeps the current shortest meaningful `30s` rerun duration for this path:
   - the loop is faster,
   - but the defended stance is unchanged,
   - and the repo still does not shorten below the current meaningful floor.
7. Verification surfaced and closed two support-path seams that were blocking truthful local proof rather than controller logic itself:
   - the adversarial runner now uses bounded hot-read monitoring and sim-event read paths plus separate control-plane read, write, and observation timeout budgets, so the `30s` post-canary and continuation reruns stay within runtime-dev proof budgets,
   - and the hot-read monitoring bootstrap now preserves CDP or fingerprint summaries plus cost-governance detail, so the fast adversarial and SIM2 gates keep reporting truthful protection and overflow evidence after the read-path shift.

## Why This Matters

Before this slice, the repo could truthfully prove:

1. one bounded canary could open,
2. one protected post-change candidate run could materialize,
3. and one terminal retain or rollback judgment could complete.

But that still fell short of the intended RSI contract, because the loop had no explicit obligation to keep going after a terminal judged episode. This slice closes that gap without breaking the board-game ownership model:

1. the controller still changes the board,
2. the adversary-sim supervisor still attacks the board,
3. and the judge still compares fresh evidence before another move is allowed.

The important architecture correction is that continuity is now `judge -> rerun -> judge -> next bounded move`, not direct patch chaining.

The important proof-path correction is that the loop is now verified through the same bounded hot-read machine surfaces it uses operationally, rather than through slower contributor-only observation paths that would have made the `30s` runtime-dev window look falsely insufficient.

## Verification Outcome

The proof path now covers continuation after both retained and rolled-back terminal outcomes:

1. `make test-oversight-agent`
   - proves terminal improved and rolled-back cycles persist a pending continuation rerun when still outside budget,
   - proves inside-budget terminal cycles stop cleanly,
   - and proves continuation preparation does not falsely persist `running` before control-state persistence succeeds.
2. `make test-rsi-game-mainline`
   - proves an internal adversary-sim beat auto-starts exactly one pending continuation rerun after a terminal improved cycle,
   - and proves the later fresh evidence can drive the next bounded canary rather than immediate patch chaining.
3. `make test-rsi-game-human-only-proof`
   - proves the strict human-only proof bundle still holds with the rerun-first continuity contract in place.
4. `make test`
   - proves the canonical full local verification path still passes after the continuity change.

## Remaining Follow-On

1. The strict Scrapling loop now has rerun-first persistence, but that does not mean the Game Loop page is already ideal. The remaining work is still the higher-level board-state and actionability cleanup under:
   - `RSI-GAME-BOARD-1`
   - and the broader architecture-retirement slices.
2. The current runtime-dev floor remains `30s`. If later live evidence shows that this is either too short to generate meaningful continuation evidence or still slower than necessary, the next change should be a separately researched cadence decision rather than an implicit behavior drift inside the continuity path.
