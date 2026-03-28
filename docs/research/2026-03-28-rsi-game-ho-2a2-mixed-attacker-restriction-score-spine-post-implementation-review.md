Date: 2026-03-28
Status: Implemented

Related context:

- [`2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md`](2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md)
- [`../plans/2026-03-28-rsi-game-ho-2-combined-attacker-orchestration-plan.md`](../plans/2026-03-28-rsi-game-ho-2-combined-attacker-orchestration-plan.md)
- [`../../src/observability/benchmark_mixed_attacker_restriction_progress.rs`](../../src/observability/benchmark_mixed_attacker_restriction_progress.rs)
- [`../../src/observability/benchmark_mixed_attacker_evidence_quality.rs`](../../src/observability/benchmark_mixed_attacker_evidence_quality.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../Makefile`](../../Makefile)

# Objective

Land `RSI-GAME-HO-2A2` by replacing the Scrapling-native restriction score spine with a truthful mixed-attacker restriction rail that can consume both Scrapling and `bot_red_team` evidence without leaking simulator labels into runtime or tuning.

# What Landed

1. Added a new mixed-attacker restriction family in [`../../src/observability/benchmark_mixed_attacker_restriction_progress.rs`](../../src/observability/benchmark_mixed_attacker_restriction_progress.rs).
   - Scrapling still contributes owned-surface exploit loci from Shuma-side receipts.
   - `bot_red_team` now contributes restriction-grade loci by mapping runtime action receipts into named board surfaces.
2. Added mixed-attacker exploit-evidence quality in [`../../src/observability/benchmark_mixed_attacker_evidence_quality.rs`](../../src/observability/benchmark_mixed_attacker_evidence_quality.rs).
   - protected evidence and tuning eligibility can now become true from mixed board-state evidence rather than Scrapling-only evidence.
3. Re-centered controller-grade scoring in [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs), [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs), [`../../src/observability/benchmark_urgency.rs`](../../src/observability/benchmark_urgency.rs), [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs), [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs), and [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs).
   - the primary restriction problem class is now `mixed_attacker_restriction_gap`;
   - protected evidence basis is now `live_mixed_attacker_runtime`;
   - and the optimization target contract now points at the mixed-attacker family instead of the old Scrapling-only family.
4. Updated the Game Loop projection in [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte) plus the rendered and adapter proof in [`../../e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js) and [`../../e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js).
   - the dashboard now corroborates the new mixed-attacker restriction rail instead of silently continuing to present the old Scrapling-only family as the controller-grade score.
5. Added a dedicated verification path in [`../../Makefile`](../../Makefile): `make test-rsi-game-mixed-restriction-score-spine`.

# Acceptance Check

## Mixed restriction family

Passed.

- benchmark assembly now materializes `mixed_attacker_restriction_progress`;
- and the family truthfully includes both Scrapling and `bot_red_team` contribution when both lanes have recent evidence.

## LLM restriction-grade evidence

Passed.

- LLM runtime receipts are no longer recent-run visibility only;
- the mixed family now converts them into named board loci such as `public_path_traversal`, `challenge_routing`, `maze_navigation`, `pow_verify_abuse`, and `tarpit_progress_abuse`.

## Controller readiness

Passed.

- protected evidence and tuning eligibility can now key off mixed-attacker board-state evidence rather than a Scrapling-only spine.

## Proof surfaces

Passed.

- `make test-benchmark-results-contract`
- `make test-rsi-game-mixed-restriction-score-spine`
- `make test-dashboard-game-loop-accountability`

# Remaining Gaps

`RSI-GAME-HO-2` is still not complete.

1. `RSI-GAME-HO-2A3`
   - operator/admin lineage and dashboard wording still need to show which lanes contributed to the judged mixed-attacker episode rather than forcing operators to infer it from recent-run coincidence.
2. `RSI-GAME-HO-2B`
   - repeated retained improvement under mixed-attacker pressure is still unproven.
3. `RSI-GAME-ARCH-1E`
   - the now-unused Scrapling-only evidence-quality module is still present and warning-only; retire it only under the explicit replacement-proof cleanup tranche rather than silently bundling architectural cleanup into this score-spine slice.

# Verification

- `make test-benchmark-results-contract`
- `make test-rsi-game-mixed-restriction-score-spine`
- `make test-dashboard-game-loop-accountability`
- `git diff --check`
