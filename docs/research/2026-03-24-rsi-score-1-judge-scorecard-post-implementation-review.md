# RSI-SCORE-1 Judge Scorecard Post-Implementation Review

Date: 2026-03-24
Status: Completed

Related context:

- [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md)
- [`../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)

# What Landed

`recursive_improvement_game_contract_v1.evaluator_scorecard` is now an explicit machine-first judge scorecard rather than only a broad evaluator boundary.

The landed contract now names:

1. `optimization_targets`
2. `hard_guardrails`
3. `regression_inputs`
4. `diagnostic_contexts`
5. `comparison_contract`

The initial scorecard freezes:

1. numeric budget optimization for likely-human friction and suspicious-origin request, byte, and latency cost,
2. category target achievement for canonical non-human posture outcomes,
3. beneficial non-human no-harm as the current hard guardrail,
4. representative adversary regression and prior-window progress as regression inputs,
5. and explicit rollback plus 10-cycle homeostasis inputs.

It also explicitly forbids hidden scalarization so later players cannot collapse the judge into one opaque reward.

# Why This Is Better

Before this slice, the repo had a clear judge boundary but still lacked one canonical answer to:

1. what exactly is optimized,
2. what can block apparent progress,
3. what remains regression-only,
4. and what counts toward homeostasis.

The new scorecard removes that ambiguity without widening the current bounded config loop.

# Verification

- `make test-rsi-scorecard-contract`
- `make test-rsi-game-contract`
- `git diff --check`

# Follow-On

The next judge-side mainline slice is `RSI-GAME-1C`, which should archive episode baselines, proposed moves, retain or rollback outcomes, and homeostasis-relevant receipts against this now-settled scorecard contract.
