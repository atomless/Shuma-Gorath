# SIM-SCR-FULL-1C2 Surface-Contract Controller-Grade Post-Implementation Review

Date: 2026-03-26
Status: Closed

Related context:

- [`../../src/observability/benchmark_scrapling_surface_contract.rs`](../../src/observability/benchmark_scrapling_surface_contract.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/observability/benchmark_suite.rs`](../../src/observability/benchmark_suite.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../../docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Scope Reviewed

This closeout reviewed `SIM-SCR-FULL-1C2`: make Scrapling defense-surface contract truth controller-grade so aggregate suspicious-origin suppression cannot make the loop look healthy or tuning-ready while the latest required Scrapling surfaces remain unsatisfied.

# What Landed

1. `benchmark_results_v1` now materializes a first-class `scrapling_surface_contract` family.
2. That family is derived from the latest visible Scrapling recent run:
   - `inside_budget` only when every required owned surface is satisfied,
   - `outside_budget` when any required surface remains blocking,
   - `insufficient_evidence` when the latest Scrapling run has no owned-surface coverage summary.
3. Tuning eligibility now fails closed on blocking Scrapling surface truth:
   - `scrapling_surface_contract_not_ready`
   - plus one blocker per named blocking surface such as `scrapling_surface_blocking:maze_navigation`.
4. Reconcile now preserves those blockers as an `observe_longer` refusal path instead of letting aggregate leakage pressure alone justify a bounded config recommendation.
5. The benchmark suite contract now names the new family explicitly so the machine-first family registry and admin benchmark surfaces remain truthful.

# Review Result

The tranche now matches the intended contract:

1. required Scrapling defense-surface misses are no longer just Red Team corroboration,
2. they now enter benchmark truth as a named family,
3. they block controller-grade tuning readiness directly,
4. and reconcile no longer treats low suspicious leakage as enough while required Scrapling surfaces are still blocking.

This is the right second repair before `RSI-SCORE-2` because it closes the specific false-health path the audit identified without prematurely claiming the full exploit-first judge redesign is already done.

# Shortfalls Found

This tranche intentionally does not yet deliver the full exploit-first judge:

1. it does not yet score terrain-local breach depth, novelty, or urgency,
2. it does not yet separate judge, diagnoser, and move selector into the richer `RSI-SCORE-2` contracts,
3. and it does not yet prove whether the current operator-facing “fully blocked” Scrapling picture is truthful or misleading.

Those remain the next open work:

1. `SIM-SCR-FULL-1C3` for the operator truth audit,
2. then `RSI-SCORE-2A` onward for exploit-progress, evidence-quality, urgency, and move-selection refinement.

# Verification

- `make test-adversary-sim-scrapling-coverage-receipts`
- `make test-benchmark-results-contract`
- `make test-benchmark-suite-contract`
- `make test-benchmark-comparison-contract`
- `make test-oversight-reconcile`
- `git diff --check`
