Date: 2026-03-27
Status: Open review

Related context:

- [`2026-03-27-game-loop-current-state-and-gap-review.md`](2026-03-27-game-loop-current-state-and-gap-review.md)
- [`2026-03-27-game-loop-category-posture-scoring-audit.md`](2026-03-27-game-loop-category-posture-scoring-audit.md)
- [`2026-03-27-game-loop-shared-path-locality-and-actionability-post-implementation-review.md`](2026-03-27-game-loop-shared-path-locality-and-actionability-post-implementation-review.md)
- [`../plans/2026-03-27-game-loop-board-state-refactor-plan.md`](../plans/2026-03-27-game-loop-board-state-refactor-plan.md)
- [`../plans/2026-03-27-game-loop-category-posture-truth-repair-plan.md`](../plans/2026-03-27-game-loop-category-posture-truth-repair-plan.md)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_scrapling_exploit_progress.rs`](../../src/observability/benchmark_scrapling_exploit_progress.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)
- [`../../src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)

# Purpose

Capture the newly surfaced Game Loop shortcomings that still stop the Scrapling loop from functioning as a trustworthy recursive-improvement system.

The triggering operator feedback was precise and correct:

1. `Loop Actionability` is still too vague to guide repair work.
2. `Named Breach Loci` still flatten too much and fabricate certainty in places.
3. `Category Posture Achievement` remains unusable while Scrapling-populated categories stay unscored.
4. the Game Loop does not yet prove clearly whether Scrapling actually exercised key owned surfaces such as JavaScript verification and PoW abuse.
5. the surface contract and dependency semantics are still too implicit to trust.

# Executive Summary

No, the current page is not good enough.

The Game Loop has improved, but it is still too weak in five places that matter for a real RSI loop:

1. blocker output is still a flat symptom list instead of a repair graph,
2. breach loci still describe attacker progress too generically,
3. category scoring still lacks exact shared-path truth for the categories Scrapling is meant to populate,
4. surface-exercise proof is not yet rigorous enough to explain whether Scrapling truly touched JS verification, PoW, maze, tarpit, and browser detection,
5. and surface dependencies or independence are not yet explicit enough for operator trust.

This is not only a UI issue.
Some of the vagueness is frontend projection, but some of it is a real backend proof-path and classification gap.

The most important architectural warning is this:
the current exact category path is still basically lane-level, while the required Game Loop behavior now demands exact category truth for non-verified hostile traffic.
If Shuma cannot infer those categories from real request, behavior, and browser evidence, then the next step is not to fake it.
The next step is to explicitly discuss a richer shared-path category-inference seam.

# Findings

## 1. `Loop Actionability` is still a flat blocker dump, not a repairable diagnosis

The current page renders `benchmarkResults.escalation_hint.blockers` as one comma-separated text line in [`GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte).

That flat list currently mixes together at least four different problem types:

1. shared classification readiness problems such as `non_human_classification_not_ready`,
2. category evidence problems such as `insufficient_category_evidence`,
3. exploit or surface proof problems such as `scrapling_exploit_evidence_quality_low`,
4. and specific required-surface misses such as `scrapling_surface_blocking:maze_navigation`.

Those are not interchangeable blockers.

Some are root-cause truth gaps.
Some are downstream consequences.
Some are concrete surfaces that need repair.
Some are controller gate conditions.

Presented as one list, the operator cannot tell:

1. which problem should be fixed first,
2. which items are merely consequences of earlier missing proof,
3. which specific repair surface should be changed,
4. or whether the controller is blocked because evidence is weak or because the bounded move ring is genuinely exhausted.

That is not good enough for a self-improving loop.

## 2. `Named Breach Loci` still flatten too much and can fabricate certainty

`benchmark_scrapling_exploit_progress.rs` currently derives breach loci only from owned-surface receipts whose `coverage_status == pass_observed`.

That already means the current breach view is not a full board-state picture.
It is only the positive-progress subset of the latest owned-surface receipt summary.

It then adds:

1. a generic stage label such as `exposure`, `interactive`, or `control_bypass`,
2. static cost-channel mappings,
3. static repair-family mappings,
4. and the sample request tuple.

That is useful as a first cut, but it is still too generic for precise repair.

More importantly, the frontend currently coerces missing numeric attempt data into zero in both:

1. [`dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
2. [`dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)

So the operator can be shown `0 attempts` even when the real state is "not materialized".

That is false certainty and it is unacceptable in an RSI loop.

## 3. `Category Posture Achievement` is still not usable for Scrapling-populated categories

The existing audit remains correct:

1. the formula in [`benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs) can produce partial values,
2. but the current exact non-verified data path still does not feed it useful exact category receipts for the current Scrapling categories.

The reason is straightforward:

1. [`traffic_classification.rs`](../../src/runtime/traffic_classification.rs) still maps generic suspicious non-verified automation only to `unknown_non_human`,
2. [`request_outcome.rs`](../../src/runtime/request_outcome.rs) derives non-human category truth from that coarse lane mapping when no verified-identity override exists,
3. [`non_human_classification.rs`](../../src/observability/non_human_classification.rs) still projects recent Scrapling run category presence into degraded `projected_recent_sim_run` receipts,
4. and the Game Loop therefore still has no exact shared-path category truth for `indexing_bot`, `ai_scraper_bot`, `automated_browser`, and `http_agent` under non-verified hostile traffic.

This remains the single biggest scoring gap.

The current `Unscored` honesty repair was necessary, but it is not enough.
The user is right that leaving all of Scrapling's current categories unscored is not acceptable for a functioning loop.

## 4. Scrapling is already attempting more than the page proves

The current worker code shows that Scrapling is already attempting several of the surfaces the operator worries are untouched:

1. browser personas fetch the PoW or JS surface and run `_checkCDPAutomation`, which is how the current worker attempts both `js_verification_execution` and `browser_automation_detection` in [`scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py),
2. the `http_agent` persona posts directly to `pow_verify` for `pow_verify_abuse`,
3. the `http_agent` persona posts directly to `tarpit_progress` for `tarpit_progress_abuse`,
4. and browser personas attempt maze traversal through the maze entrypoint plus link click.

So if the Game Loop currently leaves the operator unsure whether those surfaces were actually exercised, that is a proof-path or contract problem, not simply "the worker never tried".

That proof-path gap has to close.

## 5. Surface dependency and independence semantics are still too implicit

The current canonical owned-surface contract in [`scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs) treats:

1. `maze_navigation`,
2. `js_verification_execution`,
3. `browser_automation_detection`,
4. `pow_verify_abuse`,
5. `tarpit_progress_abuse`

as distinct surfaces with distinct success contracts.

The current contract does **not** encode tarpit as downstream of maze.
Instead, tarpit is currently modeled as a request-native challenge-abuse surface owned by `http_agent`.

That may be a correct contract or it may be the wrong contract.
But right now the operator has to infer that from code or historical plans.

That is too implicit.

If a surface is independent, the Game Loop must say so.
If a surface is blocked only because an earlier prerequisite surface was never reached, the Game Loop must say that instead.

## 6. A likely architecture fork is emerging

No large architecture change is justified yet.

But one likely fork is coming into view:

the current exact category path is still essentially lane-level, while the Game Loop now needs exact category truth for non-verified hostile traffic.

If exact category blocked-share values for `indexing_bot`, `ai_scraper_bot`, `automated_browser`, and `http_agent` cannot be produced from current shared-path signals such as:

1. request shape,
2. route family,
3. response kind,
4. behavior sequence,
5. browser-execution evidence,
6. and policy outcome lineage,

then the repo should stop and discuss a dedicated shared-path hostile-category inference layer rather than continue polishing UI around a missing backend capability.

# Required Follow-on

The next repair tranche must add five explicit execution slices:

1. `SIM-SCR-FULL-1C4` Audit and repair Scrapling surface-exercise proof for JS verification, PoW, maze, tarpit, and browser detection.
2. `SIM-SCR-FULL-1C5` Tighten owned-surface dependency and contract rigor so required, blocking, and not-reached states are explicit and trustworthy.
3. `RSI-SCORE-2F3` Land exact shared-path category posture scoring for the current Scrapling-populated categories or stop and escalate the architectural blocker explicitly.
4. `RSI-GAME-BOARD-1F` Replace flat actionability blockers with typed blocker groups, causal ordering, and exact next-fix surfaces.
5. `RSI-GAME-BOARD-1G` Replace vague breach-locus projection with exact surface-local breach facts and honest missing-data rendering.

# Why This Matters

Until these gaps close, the Game Loop is still not good enough to guide recursive improvement safely.

It can show pressure.
It can show some breach evidence.
It can show some controller state.

But it still cannot reliably answer the question a real RSI loop must answer:

"Exactly what failed, where did it fail, what smallest change should we try next, and how will we know that change improved the board?"
