Date: 2026-03-27
Status: Implemented

Related context:

- [`2026-03-27-game-loop-architecture-alignment-gap-review.md`](2026-03-27-game-loop-architecture-alignment-gap-review.md)
- [`2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
- [`../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# What Landed

`RSI-GAME-ARCH-1B` is now landed.

The repo no longer treats `non_human_category_posture` as a primary restriction objective for undeclared hostile traffic.

The main changes are:

1. [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs) now removes `category_target_achievement` from the Game Loop optimization targets, rollback inputs, and homeostasis inputs.
2. The same scorecard now re-scopes category posture into the diagnostic partition as `category_recognition_evaluation`, making the side-quest role explicit in the canonical machine-first contract.
3. [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs) now ignores `non_human_category_posture` when it is the only outside-budget family deciding top-level restriction status or escalation.
4. Category-only outside-budget pressure now resolves to `observe_longer` with `recognition_evaluation_gap` instead of reading like a primary restriction miss.
5. [`../../src/observability/benchmark_suite.rs`](../../src/observability/benchmark_suite.rs) now describes the family as recognition evaluation against simulator-ground-truth intent rather than as the main restriction question.
6. [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js) now adapts the operator snapshot recognition-evaluation summary so the dashboard can project the side quest explicitly.
7. [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte) now renders `Recognition Evaluation` instead of `Category Posture Achievement` and shows recognition summary counts separately from the main board-state restriction surfaces.

# Why This Was Necessary

The old architecture still let category posture act like the attacker scoreboard in three places:

1. the recursive-improvement objective contract,
2. the benchmark overall-status and escalation path,
3. and the operator-facing Game Loop page.

That was no longer consistent with the March 27 doctrine:

1. restriction is the main quest,
2. recognition is a side quest,
3. and simulator-known labels must never drive runtime or bounded tuning.

# Acceptance Check

## Objective truth

Met.

Undeclared hostile traffic is no longer optimized primarily through exact hostile-category posture targets because category posture is no longer part of:

1. optimization targets,
2. rollback inputs,
3. or homeostasis inputs.

## Benchmark truth

Met for the architectural reset.

`benchmark_suite_v1` and `benchmark_results_v1` already had the board-progression, host-cost, and human-friction families.
This tranche made those surfaces the effective primary restriction spine by removing category posture from the top-level restriction judgment path.

## Retirement readiness

Met.

`non_human_category_posture` still exists, but only with an explicitly secondary recognition-evaluation role.

# Verification

The following focused proofs passed on this tree:

1. `make test-operator-objectives-contract`
2. `make test-benchmark-results-contract`
3. `make test-rsi-score-move-selection`
4. `make dashboard-build`
5. `make test-dashboard-game-loop-accountability`

# What Remains Open

`RSI-GAME-ARCH-1B` is closed, but `RSI-SCORE-2F3` is still open.

The remaining restriction-first gap is not category posture anymore.
It is the stricter weighting model the user asked for:

1. Shuma confidence must become a more explicit restriction input,
2. non-restriction of high-confidence hostile traffic must weigh more heavily,
3. and low-confidence but high-cost traffic still needs an explicit abuse backstop or anomaly floor.

So this tranche resets the architecture correctly, but it does not yet complete the later confidence-weighted urgency model.
