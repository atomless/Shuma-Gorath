# RSI-GAME-ARCH-1K Post-Canary Candidate-Window Post-Implementation Review

Date: 2026-03-28  
Status: implemented

## Scope

Make the strict Scrapling loop materialize one protected post-canary candidate window automatically, under explicit canary lineage, without collapsing controller ownership and adversary-generation ownership into the same component.

## What Landed

1. Canary apply now persists explicit candidate-window lifecycle state alongside the active canary:
   - `pending`
   - requested lane
   - requested duration
   - follow-on run lineage
2. The adversary-sim supervisor now owns the follow-on run:
   - when a pending candidate window exists
   - and adversary sim is currently off
   - the next internal beat auto-starts one Scrapling follow-on run with reason `candidate_window_follow_on`
3. That follow-on is one-shot and lineage-bound:
   - the candidate window moves from `pending` to `running`
   - records the linked `follow_on_run_id`
   - and is not reissued once materialized
4. Worker-result completion now marks the linked candidate window as `materialized`, so later periodic judgment can tell the difference between:
   - missing candidate evidence
   - and candidate evidence that exists but has not yet produced a newer comparable snapshot
5. Oversight status now exposes candidate-window lifecycle truth directly, so operator-visible machine-first surfaces can say whether the next judged cycle is:
   - waiting for candidate evidence,
   - currently running the follow-on,
   - or already holding materialized candidate evidence
6. Runtime-dev now uses the shortest meaningful protected follow-on window for this path:
   - the supervisor-owned post-canary candidate run is clamped to `30s`
   - while the general adversary-sim config bound remains `30..900s`

## Why This Matters

Before this slice, the loop could already:

1. diagnose Scrapling pressure,
2. recommend a bounded patch,
3. apply a bounded canary,
4. and later reach terminal judgment timing.

But terminal judgment still fail-closed as `candidate_window_not_materialized` unless someone manually reran Scrapling while the canary was open.

This slice closes that gap in the right place:

1. oversight declares the need for candidate evidence,
2. adversary-sim supervisor generates the follow-on attack window,
3. and periodic judgment consumes the resulting evidence.

That keeps the board-game architecture clean:

1. controller changes the board,
2. adversary sim attacks the board,
3. judge compares the result.

## Verification Outcome

The focused proof path now shows the whole judged cycle, not just the canary open:

1. `make test-oversight-agent`
   - proves candidate-window request state is persisted and exposed in oversight status
2. `make test-oversight-post-sim-trigger`
   - proves completed-run post-sim trigger behavior still works around the new lifecycle
3. `make test-rsi-game-mainline`
   - proves:
     - automatic post-sim oversight apply still opens a canary,
     - adversary-sim beat auto-materializes one protected post-change Scrapling run,
     - a later periodic cycle can retain an improved canary,
     - and a later repeated cycle can roll back a regressed follow-on patch

## Remaining Follow-On

1. The core judged Scrapling loop is now autonomous in the focused proof path, but the repo still needs the later architecture-cleanup tranche:
   - `RSI-GAME-ARCH-1E`
2. The next work should keep improving the quality of Game Loop board-state truth and operator actionability rather than reopening controller-vs-supervisor ownership.
3. Runtime-dev candidate-window runs are now intentionally short at `30s`; if later proof shows that is too small for meaningful candidate evidence, the next change should be a narrowly documented cadence or workload decision rather than a hidden behavior drift.
