Date: 2026-03-27
Status: In progress

Related context:

- [`../research/2026-03-27-game-loop-category-posture-scoring-audit.md`](../research/2026-03-27-game-loop-category-posture-scoring-audit.md)
- [`../research/2026-03-27-game-loop-shared-path-locality-and-actionability-post-implementation-review.md`](../research/2026-03-27-game-loop-shared-path-locality-and-actionability-post-implementation-review.md)
- [`2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)
- [`../../src/runtime/sim_telemetry.rs`](../../src/runtime/sim_telemetry.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

# Objective

Repair `Category Posture Achievement` so it is truthful under strict human-only semantics: projected rows must read as unscored, and exact category scoring for Scrapling traffic must come only from Shuma-side category inference rather than sim persona labels.

Implementation status update (2026-03-27):

1. `RSI-SCORE-2F1` is now landed:
   - degraded or projected rows stay `current = null`,
   - the dashboard renders them as `Unscored`,
   - and the success meter is suppressed instead of reading as `0%`.
2. The audit portion of `RSI-SCORE-2F2` is also now explicit:
   - current non-verified suspicious automation still flows mainly through `unknown_non_human`,
   - recent Scrapling category presence still enters only as degraded `projected_recent_sim_run`,
   - so no new exact Scrapling-populated category truth was added in this slice.
3. The remaining open work is the "first exact receipts" part of `RSI-SCORE-2F2`, and it must stay blocked until Shuma can infer those categories from real shared-path evidence rather than simulator persona knowledge.

# Core Decisions

1. Do not reintroduce fabricated per-category envelopes from aggregate sim totals.
2. Do not use signed adversary-sim metadata, worker fulfillment modes, or any equivalent sim-side declaration as category truth for scoring.
3. Exact category attribution is only allowed when Shuma itself can infer that category from request and behavior evidence that real external traffic could also produce.
4. Projected recent-run category presence remains visible for coverage and evidence-quality purposes, but it must stay unscored.
5. The UI must distinguish `Unscored` from `0% achieved`.

# Implementation Shape

## `RSI-SCORE-2F1` Honest unscored rendering and blocker truth

Required contract:

1. projected or degraded rows in `non_human_category_posture` stay explicitly unscored,
2. the dashboard renders `Unscored` with no success meter when `current=null`,
3. and the Game Loop makes it clear that missing exact category evidence is a scoring blocker rather than a measured `0%`.

Acceptance criteria:

1. dashboard tests prove unscored rows do not render as zero-achievement bars,
2. benchmark tests prove degraded projected rows stay `current=null`,
3. and the repo has focused proof through:
   1. `make test-benchmark-results-contract`
   2. `make test-dashboard-game-loop-accountability`

## `RSI-SCORE-2F2` Shuma-side category inference audit and first exact receipts

Required contract:

1. the repo explicitly identifies which Scrapling-populated categories Shuma can already infer from real request and behavior evidence, if any,
2. exact category receipts are only added where that evidence is sufficient without reading sim persona labels,
3. categories without sufficient Shuma-side inference remain unscored with explicit blocker truth,
4. and exact category scoring is still shared-path truth for sim and real traffic alike.

Acceptance criteria:

1. research or implementation evidence names the exact Shuma-side signals used for each newly exact category,
2. benchmark tests prove a category can score a non-binary partial blocked share from exact category receipts once Shuma-side inference is present,
3. no test or implementation path depends on `sim_profile`, `sim_lane`, or worker fulfillment mode as category truth,
4. and the repo has focused proof through:
   1. `make test-traffic-classification-contract`
   2. `make test-adversary-sim-scrapling-category-fit`
   1. `make test-benchmark-results-contract`
   2. `make test-dashboard-game-loop-accountability`

# Sequencing

1. Land the honest unscored rendering repair first.
2. Audit current Shuma-side category inference capability for non-verified traffic.
3. Add exact category scoring only where that audit proves real shared-path inference is possible.
4. Keep the remaining category gaps explicit if inference is still not ready for some Scrapling personas.
5. Update docs, backlog, and post-implementation review only after focused verification passes.

# Definition Of Done

This repair tranche is complete when:

1. degraded projected rows stay visible but render as `Unscored` rather than as `0%`,
2. exact category scoring appears only where Shuma-side inference justifies it,
3. no category score is sourced from sim persona labels,
4. the focused `make` proof targets above pass,
5. and the repo docs and TODO chain explicitly record both the repair and any remaining inference gaps.
