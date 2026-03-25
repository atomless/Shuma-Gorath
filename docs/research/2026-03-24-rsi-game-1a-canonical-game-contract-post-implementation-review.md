Date: 2026-03-24
Status: Implemented

Related plan:

- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)

# Summary

`RSI-GAME-1A` is now landed as a machine-first contract rather than a planning-only idea.

Shuma now exposes one canonical `recursive_improvement_game_contract_v1` through both:

1. `operator_snapshot_v1`
2. `oversight_history_v1`

That contract names:

1. immutable rules from `operator_objectives_v1`,
2. the independent evaluator boundary from `benchmark_results_v1` over `benchmark_suite_v1`,
3. the legal move ring as the controller-tunable subset of the controller action surface,
4. the current fail-closed safety gates,
5. and the current regression anchors plus the explicitly deferred strict reference stance anchor.

# What Landed

## 1. Canonical game-contract payload

The core contract now lives in [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs) as `recursive_improvement_game_contract_v1`.

It intentionally stops short of later `RSI-SCORE-1` partitioning work. It makes the game explicit without over-claiming the later detailed episode score vector.

## 2. Canonical legal-move-ring helper

The explicit legal move ring now lives in [`../../src/config/controller_action_surface.rs`](../../src/config/controller_action_surface.rs) as `controller_legal_move_ring_v1`.

That keeps the game contract from inferring legality indirectly from `allowed_actions_v1` group rows every time later code wants to reason about the move ring.

## 3. Projection through existing machine-first surfaces

The same contract is now projected by:

1. [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)
2. [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)

This keeps the game contract attached to the actual loop surfaces the repo already uses rather than introducing a disconnected sidecar artifact.

# Verification

- `make test-rsi-game-contract`
- `git diff --check`

# Follow-on

The next tranche should remain `RSI-GAME-1B`.

That is now a cleaner slice because the repo no longer needs to decide the game and the move-selection bridge in the same change.
