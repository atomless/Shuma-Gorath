Date: 2026-03-24
Status: Implemented

Related context:

- [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md)
- [`../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)

# What Landed

`RSI-SCORE-1` is now explicit in the machine-first game contract.

The canonical `game_contract_v1` surface now carries a structured `judge_scorecard_v1` partition over:

1. optimization targets,
2. hard guardrails,
3. regression anchors,
4. explanatory diagnostics,
5. and homeostasis inputs.

That scorecard is projected through:

1. `operator_snapshot_v1`,
2. preview reconcile payloads,
3. and oversight history payloads.

# Why This Is Better

Before this tranche, the repo had the right ingredients for the judge, but the score semantics were still implicit across benchmark families, tuning eligibility, and replay-promotion guardrails.

Now the repo has one canonical answer to:

1. what the judge is trying to optimize,
2. what it must never trade away,
3. what remains a regression anchor even when later phases broaden search,
4. and which same scorecard entries feed later run-to-homeostasis logic.

That closes the main remaining judge-side ambiguity between `RSI-GAME-1B` and the future episode archive work.

# What Stayed Intentionally Narrow

1. The scorecard is still inside `game_contract_v1` rather than becoming a second top-level contract.
2. The judge still does not collapse Shuma into one scalar reward.
3. The held-out override surface is named, but true held-out-vs-protected evaluation separation remains future `RSI-EVAL-1`.

# Follow-On

1. `RSI-GAME-1C` should now persist baseline scorecards and watch-window outcomes as episode history.
2. `RSI-EVAL-1` should later replace the currently named held-out override placeholder with a fully ratified evaluation-ring contract.

# Verification

1. `make test-rsi-scorecard`
2. `make test-rsi-game-contract`
3. `git diff --check`
