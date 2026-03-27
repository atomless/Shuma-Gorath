Date: 2026-03-27
Status: Active review driver

Related context:

- [`2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`2026-03-27-game-loop-architecture-alignment-gap-review.md`](2026-03-27-game-loop-architecture-alignment-gap-review.md)
- [`2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md`](2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md)
- [`../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_scrapling_evidence_quality.rs`](../../src/observability/benchmark_scrapling_evidence_quality.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)

# Objective

Capture the live architecture gap that is still preventing the Scrapling Game Loop from behaving like a real RSI loop, even after the March 27 restriction-vs-recognition split.

# Live Evidence

Using the fresh local runtime plus freshly seeded dashboard data, the live `/admin/benchmark-results` payload currently shows:

1. `overall_status = outside_budget`,
2. `problem_class = scrapling_exploit_progress_gap`,
3. named breach loci at:
   1. `Public Path Traversal`,
   2. `Challenge Routing`,
   3. `Rate Pressure`,
   4. `Geo Or IP Policy`,
4. but `tuning_eligibility.status = blocked`,
5. and `move_selection.decision = observe_longer`.

The blockers on that same live payload are:

1. `degraded_category_receipts_present`,
2. `insufficient_category_evidence`,
3. `non_human_classification_not_ready`,
4. `scrapling_exploit_evidence_quality_low`.

That live combination is the key failure.

The board is already saying:

1. Scrapling made real incursions,
2. the breach loci are localized enough to name,
3. bounded repair families are already materialized,
4. yet the controller still refuses to act.

# What The Code Shows

## 1. Restriction tuning is still contaminated by recognition-side blockers

[`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs) still builds `tuning_eligibility()` from `non_human_traffic.restriction_readiness`.

That readiness currently carries blockers such as:

1. `degraded_category_receipts_present`,
2. `insufficient_category_evidence`.

Those are recognition-side category quality gaps.

They are not the same thing as restriction-side board truth.

So the current code is still allowing the recognition quest to block the restriction quest.

That is not aligned with the new Game Loop doctrine.

## 2. Restriction tuning still depends on simulator-derived persona diversity

[`../../src/observability/benchmark_scrapling_evidence_quality.rs`](../../src/observability/benchmark_scrapling_evidence_quality.rs) currently computes `persona_diversity_status` from `run.observed_fulfillment_modes.len()`.

Those `observed_fulfillment_modes` are materialized in [`../../src/admin/api.rs`](../../src/admin/api.rs) from `observed_category_targets_for_runtime_profile(...)`, which is derived from the adversary-sim runtime profile rather than from Shuma runtime defense evidence alone.

That is acceptable for harness evaluation.

It is not acceptable as a prerequisite for restriction tuning.

So the current restriction-grade confidence gate is still using simulator-known lane metadata.

That is exactly the leakage the newer architecture forbids.

## 3. The current latest-run-only gate understates real recent-window pressure

The latest visible Scrapling run is currently crawler-only and covers only the request-native subset.

But the immediately preceding recent runs show:

1. multi-mode execution,
2. the broader browser-inclusive required surface set,
3. and repeated blocking at `maze_navigation`, `js_verification_execution`, and `browser_automation_detection`.

So the live evidence already has a richer recent-window story than the latest-run-only evidence-quality gate currently acknowledges.

The controller is therefore underestimating the maturity of the present breach signal.

# Main Findings

1. The Game Loop is still not pure enough: restriction tuning is being blocked by recognition-evaluation defects.
2. The Game Loop is still not pure enough: restriction tuning confidence is still using simulator-derived persona metadata.
3. The controller is therefore staying in `observe_longer` even when the board already has localized, repeated breach evidence plus bounded repair-family candidates.
4. This is now a more urgent correctness problem than legacy-surface retirement, because it directly prevents the RSI loop from making or testing bounded moves.

# Required Direction

The next repair must do four things:

1. remove recognition-side category-readiness blockers from the restriction-tuning eligibility path when surface-native board-state breach evidence is already strong enough,
2. replace simulator-derived persona-diversity gating with restriction-grade recent-window support derived from breach evidence rather than harness labels,
3. keep simulator labels available only on the recognition-evaluation rail,
4. and prove that the controller can become eligible on restriction-grade Scrapling pressure without reopening simulator-label leakage into runtime or tuning.

# Architectural Decision

This is still on the right overall architecture.

It does not currently require a whole new Game Loop model.

It does require a new explicit purity repair tranche before further retirement cleanup:

1. restriction tuning must become independent of recognition blockers,
2. exploit-evidence quality must be judged from board-state evidence,
3. and only after that should the repo continue the retirement of older category-first surfaces.

# Consequence For Sequencing

The previous claim that `RSI-GAME-ARCH-1E` is the next active architecture slice is no longer precise enough.

There is now a higher-priority follow-on slice:

1. restriction-tuning purity and recent-window exploit-evidence repair,
2. then legacy-surface retirement.
