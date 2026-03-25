Date: 2026-03-25
Status: Active

# `RSI-GAME-HO-1C` Strict Improvement Unlock Readiness Review

## Context

`RSI-GAME-HO-1A` and `RSI-GAME-HO-1B` now prove that:

1. the loop runs under the strict `human_only_private` stance,
2. verified non-human traffic remains denied there,
3. bounded config changes can be retained or rolled back,
4. and later cycles can run against retained earlier config.

That still leaves the user’s stronger requirement open:

1. the strict Scrapling-only loop must not be considered operational merely because one retained cycle and one rollback cycle exist,
2. the unlock condition for leaving the strict stance must be machine-checkable,
3. and that unlock must show repeated retained improvement toward the strict target rather than only plumbing exercise.

## Findings

1. The current archive summary is too implicit for an unlock gate.
   - `src/observability/benchmark_comparison.rs::BenchmarkHomeostasisSummary` currently exposes:
     - `minimum_completed_cycles_for_homeostasis`
     - `judged_cycle_count`
     - `considered_episode_ids`
     - `status`
     - `note`
   - That means the repo can say `improving`, but it does not expose how many of the considered cycles were actually improving versus regressed, flat, or guardrail-blocked.

2. The repo has no focused proof for the strict-baseline unlock condition.
   - `make test-rsi-game-mainline` proves the first working loop.
   - `make test-rsi-game-human-only-cycles` proves repeated strict-baseline iteration.
   - Neither target proves the stronger unlock condition: enough completed improving cycles to satisfy the game contract’s minimum under `human_only_private`.

3. The current active Scrapling bundle still stops short of the unlock condition.
   - `make test-scrapling-game-loop-mainline` currently bundles:
     - the Scrapling owned-surface gate,
     - malicious request-native behavior,
     - coverage receipts,
     - the first-working loop proof,
     - and repeated-cycle proof.
   - It does not yet prove the archive reaches a strict-baseline improving state.

## Conclusion

`RSI-GAME-HO-1C` should:

1. make the strict-baseline unlock condition explicit in the archive summary, not inferred from prose,
2. expose enough completed-cycle breakdown to distinguish repeated retained improvement from mixed or rollback-heavy churn,
3. add a focused strict-improvement Make target,
4. and promote that target into the active Scrapling-only mainline bundle so the current mainline proof stops at the real strict-baseline unlock rather than earlier plumbing milestones.
