# `RSI-GAME-HO-1C` Strict Improvement Unlock Plan

**Goal:** Define and prove the strict `human_only_private` unlock condition as repeated retained improvement toward the strict target, not merely a loop that can apply, retain, and roll back bounded config changes.

**Architecture:** Reuse the existing episode-archive and homeostasis path rather than inventing a second improvement ledger. Tighten the homeostasis summary so it explicitly reports completed-cycle outcome counts, then add a focused oversight-history proof that seeds enough strict-baseline completed episodes to satisfy the game contract’s minimum improving threshold. Wire that proof into a dedicated Make target and the active Scrapling-only mainline bundle.

**Tech Stack:** `src/observability/benchmark_comparison.rs`, `src/admin/oversight_api.rs`, `Makefile`, `scripts/tests/test_adversary_sim_make_targets.py`, `docs/testing.md`, planning indexes, and TODO history.

---

## Task 1: Write the failing unlock-condition proof first

**Files:**
- Modify: `src/observability/benchmark_comparison.rs`
- Modify: `src/admin/oversight_api.rs`
- Modify: `scripts/tests/test_adversary_sim_make_targets.py`

**Work:**
1. Add benchmark/homeostasis tests that require explicit completed-cycle outcome counts rather than only a summary status string.
2. Add an oversight-history test that requires:
   - strict `human_only_private` objectives,
   - enough completed improving cycles to satisfy the contract minimum,
   - explicit archive evidence that those considered cycles are retained improvements,
   - and a resulting `episode_archive.homeostasis.status == "improving"`.
3. Add Makefile contract expectations for a new focused strict-improvement target.

## Task 2: Tighten the archive summary to carry the real unlock evidence

**Files:**
- Modify: `src/observability/benchmark_comparison.rs`
- Modify: `src/admin/oversight_api.rs`
- Modify: `Makefile`

**Work:**
1. Extend `BenchmarkHomeostasisSummary` with explicit outcome counts for the considered completed cycles.
2. Keep the existing conservative status classification, but make it backed by those counts.
3. Add a dedicated focused target, for example `test-rsi-game-human-only-improvement`, for the strict-baseline unlock proof.
4. Update `test-scrapling-game-loop-mainline` so the active local mainline bundle includes the unlock-condition gate, not just the earlier loop-plumbing gates.

## Task 3: Close the tranche

**Files:**
- Modify: `docs/testing.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-25-rsi-game-ho-1c-strict-improvement-unlock-post-implementation-review.md`

**Verification:**

```bash
make test-rsi-game-human-only-improvement
make test-adversary-sim-make-target-contract
make test-scrapling-game-loop-mainline
git diff --check
```

## Definition Of Done

This slice is complete when:

1. the strict-baseline unlock condition is explicit and machine-checkable,
2. the archive summary proves enough completed improving cycles to satisfy the game contract’s minimum,
3. that proof distinguishes repeated retained improvement from mixed or rollback-heavy loop churn,
4. and the active local Scrapling-only mainline bundle now includes the strict improvement gate rather than stopping at repeated-cycle plumbing.
