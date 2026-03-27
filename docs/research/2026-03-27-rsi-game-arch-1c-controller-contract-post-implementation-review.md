Date: 2026-03-27
Status: Implemented

Related plan:

- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)

Related code:

- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../src/observability/benchmark_comparison.rs`](../../src/observability/benchmark_comparison.rs)
- [`../../src/test_support.rs`](../../src/test_support.rs)

# Objective

Close `RSI-GAME-ARCH-1C` by making the controller contract explicit enough that reconcile stops behaving like a thin wrapper around one benchmark escalation hint.

# What Landed

1. `benchmark_results_v1` now materializes an explicit controller contract made of sibling machine-first surfaces:
   1. restriction diagnosis,
   2. recognition evaluation,
   3. and move selection.
2. typed controller blockers are now first-class benchmark output rather than implicit strings hidden inside one escalation object.
3. `oversight_reconcile` now consumes the explicit controller contract for:
   1. decision routing,
   2. diagnosis,
   3. move selection,
   4. code-evolution referral,
   5. and config-ring exhaustion.
4. manual and periodic oversight routes now expose the same controller-grade truth.
5. stale local snapshot fixtures in the oversight route tests were removed in favor of the shared controller-contract-aware test helper so route tests cannot silently drift from the controller architecture.

# Why This Was Necessary

Before this tranche, the newer restriction-vs-recognition design still depended too heavily on `benchmark.escalation_hint` as a monolithic oracle. That created two risks:

1. controller ownership remained muddled because diagnosis, recognition state, and move selection were only projections of one older object,
2. and route-level fixtures could stay green while drifting from the benchmark contract that the controller was supposed to obey.

That mismatch is exactly what showed up during implementation: the manual reconcile route still used an older local snapshot seed and therefore resolved to `observe_longer` even after the controller-grade path had moved to `config_tuning_candidate`.

# Acceptance Criteria Check

## 1. `oversight_reconcile` consumes explicit restriction-judge and move-selection contracts

Passed.

- Reconcile now branches on `benchmark.controller_contract.move_selection.decision` instead of primarily on `benchmark.escalation_hint.decision`.
- Diagnosis now reads `benchmark.controller_contract.restriction_diagnosis`.
- Recognition status is now carried explicitly into the reconcile result.

## 2. `Loop Actionability` can be powered from typed controller state rather than a flat benchmark blocker list

Passed at the backend contract level.

- `benchmark_results_v1` now emits typed `BenchmarkControllerBlocker` rows grouped by controller concern.
- The remaining UI projection work stays open as `RSI-GAME-BOARD-1F`.

## 3. Code-evolution referral, bounded config selection, and ring exhaustion remain explicit and testable

Passed.

- controller move selection keeps explicit decisions for:
  - `observe_longer`
  - `config_tuning_candidate`
  - `code_evolution_candidate`
- reconcile tests still cover:
  - bounded config recommendation,
  - code-evolution referral,
  - and config-ring exhaustion.

# Verification

- `make test-rsi-score-move-selection`
- `make test-rsi-game-mainline`
- `make test-dashboard-game-loop-accountability`
- `git diff --check`

# Follow-On Work

This tranche does not finish the operator-facing actionability repair by itself. The next immediate slice remains:

1. `RSI-GAME-BOARD-1F`
   Turn the new typed controller contract into grouped `Loop Actionability` output with exact next-fix surfaces.
2. `RSI-GAME-ARCH-1D`
   Keep normalizing breach-locus and blocker contracts so missing or derived data stays explicit end to end.
