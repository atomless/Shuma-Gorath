Date: 2026-03-25
Status: Completed

Related context:

- [`2026-03-25-rsi-game-ho-1c-strict-improvement-unlock-readiness-review.md`](2026-03-25-rsi-game-ho-1c-strict-improvement-unlock-readiness-review.md)
- [`../plans/2026-03-25-rsi-game-ho-1c-strict-improvement-unlock-plan.md`](../plans/2026-03-25-rsi-game-ho-1c-strict-improvement-unlock-plan.md)

# What landed

`RSI-GAME-HO-1C` is now complete.

Shuma now makes the strict `human_only_private` unlock condition explicit rather than leaving it as prose:

1. `BenchmarkHomeostasisSummary` now exposes explicit counts for improving, regressed, flat, and guardrail-blocked completed cycles,
2. `oversight_history_v1` now has a focused proof showing enough strict-baseline retained improving cycles to satisfy the game contract minimum,
3. and the active local Scrapling-only mainline bundle now includes that strict improvement gate through `make test-rsi-game-human-only-improvement`.

# Why this was the right closeout

Before this slice, the repo could prove:

1. the loop ran under `human_only_private`,
2. repeated cycles reused retained config state,
3. and later cycles could still retain or roll back truthfully.

It still could not prove the stronger operational claim the user asked for: that repeated strict-baseline cycles had materially improved enough to satisfy the game contract’s own minimum rather than merely exercise retain/rollback plumbing.

This slice closes that gap by reusing the canonical archive and homeostasis path rather than inventing a second improvement ledger.

# Validation note

One initial run of `make test-scrapling-game-loop-mainline` saw a transient missing `puzzle_submit_or_escalation` receipt inside the HTTP-agent Scrapling worker test. A focused rerun of `make test-adversary-sim-scrapling-coverage-receipts` and a subsequent full bundle rerun both passed without code changes in the Scrapling lane. No stability change was folded into this tranche; treat that as a watch item if it recurs.

# Next active work

The strict Scrapling-only proof chain is now complete. The next active slice is:

1. `SIM-LLM-1C3` to close the runtime proof chain and recent-run projection for the live `bot_red_team` actor before the second strict mixed-attacker proof tranche opens.
