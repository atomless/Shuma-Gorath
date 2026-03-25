Date: 2026-03-24

# RSI-GAME-1B Shortfall Attribution And Move Selection Post-Implementation Review

## What landed

Shuma now has one explicit machine-first bridge from benchmark shortfall to bounded config guidance.

That bridge is no longer:

- a coarse benchmark-family mapping,
- followed by a hidden proposer-side family priority stack.

Instead, `benchmark_results_v1.escalation_hint` now names:

1. the shortfall problem class,
2. guidance status,
3. tractability,
4. trigger metrics,
5. ordered legal candidate families,
6. an optional recommended family,
7. and bounded per-shortfall guidance rows.

## Why this closes `RSI-GAME-1B`

`RSI-GAME-1B` was about replacing implicit move selection with a reviewable policy layer.

That is now true because:

1. benchmark shortfalls are classified into explicit problem classes,
2. the policy distinguishes exact bounded moves, bounded heuristic family choice, insufficient evidence, and code-only gaps,
3. reconcile consumes that explicit guidance,
4. and the proposer now shapes patches for the benchmark-selected family order instead of keeping a second hidden priority system.

## Proof

Focused verification:

- `make test-oversight-move-selection-policy`
- `make test-oversight-reconcile`
- `make test-controller-action-surface-parity`
- `git diff --check`

That proof now covers:

- benchmark-side shortfall guidance derivation,
- ordered legal-family selection,
- reconcile honoring a benchmark-recommended family,
- and continued parity with the controller mutability and legal move ring.

## Outcome

The next judge-side mainline can now move into `RSI-SCORE-1`, followed by `RSI-GAME-1C`.

That means:

1. the judge now has an explicit answer to how misses become bounded config guidance,
2. later scorecard and archive work can build on that explicit bridge rather than patch-policy internals,
3. and the first explicit self-improving loop is now gated by score semantics and episode memory rather than by a missing move-selection policy.
