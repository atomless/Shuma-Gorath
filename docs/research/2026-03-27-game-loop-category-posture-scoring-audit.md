# Game Loop Category Posture Scoring Audit

Date: 2026-03-27
Status: Open

Related context:

- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)
- [`../../src/runtime/sim_telemetry.rs`](../../src/runtime/sim_telemetry.rs)
- [`../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)

# Problem

The current `Category Posture Achievement` section is not trustworthy for Scrapling-populated categories.

Live March 27, 2026 payloads show:

1. `benchmark_results_v1.families[non_human_category_posture]` is `insufficient_evidence` for `indexing_bot`, `ai_scraper_bot`, `automated_browser`, and `http_agent`.
2. those rows carry `current=null`, `basis=projected_recent_sim_run`, and `capability_gate=partially_supported`.
3. `operator_snapshot_v1.non_human_traffic.receipts` contains only degraded projected Scrapling category receipts with zero totals for those categories.
4. the only exact adversary-sim category row currently materialized in live data is `unknown_non_human`.

So the page is not actually measuring a trustworthy blocked share for Scrapling categories today.

# Root Cause

There are two distinct faults.

## 1. Exact Scrapling category-native receipts are missing

`RenderedRequestOutcome` only records exact `by_non_human_category` counters when the request outcome already carries a concrete `non_human_category`.

Today that category is derived from:

1. verified-identity override when present, or
2. `non_human_category_assignment_for_lane(classification.traffic_lane)`.

For generic suspicious automation, that lane maps only to `unknown_non_human`.
That is why live adversary-sim category counters still collapse to `unknown_non_human`.

The important constraint is that this must not be solved by reading the adversary harness persona label back into Shuma.
`sim_profile`, `sim_lane`, worker `fulfillment_mode`, or any equivalent sim-side declaration may help operators group runs, but they must not become category truth for the Game Loop.

The same Shuma-side evidence path that would classify real external traffic must be the only authority for category posture scoring.

## 2. The UI makes unscored rows look like zero

When the benchmark correctly returns `current=null`, the Game Loop UI still computes a zero-width meter because `ratioToTarget(null, target)` becomes `null` and the render path falls back to `0`.

That turns "unscored" into something that visually resembles "0% achieved".

## 3. Current runtime category capability is weaker than the operator-facing matrix implies

The current runtime does not materially assign the non-verified lanes that would make these category rows exact today.

In the live runtime path:

1. verified identity can crosswalk into canonical categories, and
2. non-verified botness pressure mostly lands in `SuspiciousAutomation -> unknown_non_human`.

Specific non-verified lanes such as `DeclaredCrawler` are largely present as contract/test vocabulary rather than active runtime output.

So the current category posture surface cannot truthfully score Scrapling-populated categories beyond `unknown_non_human` until Shuma gains real category inference from request and behavior evidence.

# Important Clarification

The benchmark math in `benchmark_non_human_categories.rs` is not inherently binary.

For a `blocked` posture it computes:

`short_circuited_requests / total_requests`

So if exact category-native receipts exist, the current score can be any value in `[0.0, 1.0]`.

The reason the operator has mostly seen `100%` or `0%` is not that the formula only supports extremes.
It is that the current data path either:

1. has no exact category-native receipt and therefore returns `null`, or
2. falls back to coarse exact lane truth that can be fully forwarded or fully blocked.

# Decision

The category posture surface must become explicitly honest first, then more useful.

That means:

1. projected recent-run presence must remain visible but must stay unscored,
2. `Category Posture Achievement` must render `Unscored` with no success meter when `current=null`,
3. Shuma-side category inference must be treated as a separate required capability tranche,
4. and partial blocked-share values such as `25%`, `63%`, or `87%` must only appear when Shuma itself has emitted exact category-native receipts from real request or behavior evidence.

# Why This Matters

Without exact Scrapling category-native scoring:

1. the operator cannot tell how much of each Scrapling category is truly being blocked,
2. the diagnoser cannot distinguish category-local shortfalls from missing evidence,
3. and the Game Loop cannot be trusted as a fine-grained basis for config changes.

This is a release-blocking truth gap for the strict `human_only_private` loop.

# Follow-on Required

Land a focused repair tranche that:

1. immediately repairs the rendered Game Loop so projected rows stay explicitly unscored,
2. forbids sim-side persona labels from becoming category truth,
3. audits what Shuma-side evidence exists today for truthful non-verified category inference,
4. adds exact category scoring only where that Shuma-side evidence justifies it,
5. and keeps the remaining gaps explicit where category inference is not ready yet.
