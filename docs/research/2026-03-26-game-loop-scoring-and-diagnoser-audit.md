Date: 2026-03-26
Status: Proposed planning driver

Related context:

- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md)
- [`2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md`](2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md)
- [`2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md`](2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md)
- [`2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md`](2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md)
- [`../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/observability/benchmark_adversary_effectiveness.rs`](../../src/observability/benchmark_adversary_effectiveness.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../docs/dashboard-tabs/game-loop.md`](../../docs/dashboard-tabs/game-loop.md)

# Purpose

Audit how the current Game Loop actually scores the active config against adversaries, what evidence the diagnoser uses before suggesting config changes, and where current operator-facing output can be read more strongly than the backend truth supports.

# Findings

## [P1] The diagnoser only recommends config changes from likely-human friction and suspicious-origin cost pressure

The current controller-grade reconcile path does not inspect Scrapling defense-surface contract truth when deciding whether to suggest config changes.

Today the decision path is:

1. refuse if required input sections are stale,
2. refuse if benchmark snapshot metadata is contradictory,
3. return `within_budget` if the overall benchmark status is `inside_budget`,
4. return `observe_longer` if the benchmark is `near_limit` or explicitly says `observe_longer`,
5. return `no_change` unless the benchmark escalation hint says `config_tuning_candidate`,
6. otherwise choose a problem class and shape a bounded patch.

That logic lives in [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs).

The decisive detail is that `primary_problem_class(...)` only reads:

1. `likely_human_friction_rate`,
2. `suspicious_forwarded_request_rate`,
3. `suspicious_forwarded_byte_rate`,
4. `suspicious_forwarded_latency_share`,
5. and the benchmark trigger family ids `likely_human_friction` or `suspicious_origin_cost`.

It does not read Scrapling owned-surface coverage, required-surface misses, or Red Team pass or fail truth.

That design is reinforced one layer earlier in [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs):

1. `likely_human_friction` is classified as `config_tuning_candidate`,
2. `suspicious_origin_cost` is classified as `config_tuning_candidate`,
3. `non_human_category_posture` is classified as `code_evolution_candidate`,
4. `representative_adversary_effectiveness` is classified as `code_evolution_candidate`,
5. and `beneficial_non_human_posture` is also `code_evolution_candidate`.

Conclusion:

1. the diagnoser currently suggests config changes only from aggregate human-friction and suspicious-origin budget pressure,
2. and it does not yet use the richer attacker-proof surfaces that operators naturally expect it to use.

## [P1] The fallback adversary-sim category receipts copy whole-run totals into every observed category

The current category-achievement rows can be stronger than the underlying evidence when adversary-sim lacks explicit `request_outcomes.by_non_human_category` rows.

In [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs), `summarize_non_human_classification(...)` first tries to use explicit adversary-sim category rows. But when those rows are absent, it falls back to `sim_receipts_from_recent_runs(...)`.

That fallback:

1. iterates `run.observed_category_ids`,
2. assigns `run.monitoring_event_count` to each observed category as `total_requests`,
3. derives forwarded and short-circuited counts from the aggregate sim scope through `projected_recent_run_outcomes(...)`,
4. and inserts that same projected run envelope into every category observed by that run.

The tests in the same file codify this behavior explicitly. The `classification_summary_projects_scrapling_recent_run_category_receipts` test asserts that a single run with:

1. `monitoring_event_count = 9`,
2. `forwarded_requests = 2`,
3. `short_circuited_requests = 7`,
4. and observed categories `indexing_bot`, `ai_scraper_bot`, and `http_agent`

produces an `ai_scraper_bot` receipt with all `9` total requests, `2` forwarded requests, and `7` short-circuited requests.

Conclusion:

1. in the fallback path, per-category achievement is not independently measured per category,
2. it is projected from a shared run envelope,
3. and that can make multiple category rows look separately proven as `100%` achieved when they are really sharing one aggregate run outcome.

## [P1] Strict human-only benchmark pressure can look perfect before Scrapling meaningfully satisfies its defense-surface contract

Under the strict `human_only_private` stance, the suspicious-origin benchmark family is intentionally sourced from the aggregate `adversary_sim` scope in [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs).

That family is built in [`../../src/observability/benchmark_results_families.rs`](../../src/observability/benchmark_results_families.rs) from:

1. `suspicious_forwarded_request_rate`,
2. `suspicious_forwarded_byte_rate`,
3. `suspicious_forwarded_latency_share`,
4. plus tracking-only short-circuit, locally served byte share, and average forwarded latency metrics.

The benchmark payload currently includes only five families:

1. `suspicious_origin_cost`,
2. `likely_human_friction`,
3. `representative_adversary_effectiveness`,
4. `beneficial_non_human_posture`,
5. and `non_human_category_posture`.

There is no benchmark family driven by Scrapling defense-surface contract satisfaction.

That means Shuma can currently observe:

1. perfect aggregate suspicious-origin leakage suppression,
2. perfect or near-perfect category posture achievement,
3. and still have Scrapling required-surface misses in Red Team that never enter the controller-grade config recommendation path.

Conclusion:

1. the current loop can treat early short-circuiting as success even when the attacker has not yet truthfully exercised the surfaces the game loop is supposed to learn from,
2. and that is not strong enough for the strict human-only proof the repo now says it wants.

## [P2] Representative adversary effectiveness is only a coarse proxy and is explicitly not controller-tunable

The current `representative_adversary_effectiveness` family in [`../../src/observability/benchmark_adversary_effectiveness.rs`](../../src/observability/benchmark_adversary_effectiveness.rs) is built from coarse recent-run proxies:

1. runs with `defense_delta_count == 0` and `ban_outcome_count == 0`,
2. runs whose monitoring-event volume exceeds defense plus ban counts,
3. and runs with any defense or ban event at all.

Its capability gate is only `partially_supported`, and the comparison layer classifies it as `code_evolution_only`, not a config-tuning input.

Conclusion:

1. this family is useful corroboration,
2. but it is not the authoritative attacker-proof score operators might assume from the label.

## [P2] The Game Loop tab currently places three different scoring planes side by side without enough semantic separation

The Game Loop tab currently renders:

1. `Budget Usage` from the `suspicious_origin_cost` and `likely_human_friction` families,
2. `Category Target Achievement` from the `non_human_category_posture` family,
3. and a compact `Latest Scrapling Evidence` row from recent-run `owned_surface_coverage`.

Those projections live together in [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte), while the docs in [`../../docs/dashboard-tabs/game-loop.md`](../../docs/dashboard-tabs/game-loop.md) describe the Scrapling row as only bounded corroboration.

That distinction is technically real, but the current layout still makes it easy to read:

1. `Target Blocked | Achieved 100.0%`
2. and `0 / N surfaces` or other Scrapling evidence summaries

as if they were one unified attacker-success metric.

Conclusion:

1. the current page makes a reasonable operator misreading likely,
2. and the scoring planes need to be separated more explicitly if the game loop is going to be trusted as the judge for config changes.

# What The Diagnoser Actually Uses Today

The current criteria for suggesting a config change are:

1. evidence freshness and snapshot-alignment checks must pass,
2. at least one benchmark family must be `outside_budget`,
3. the benchmark escalation hint must classify that family as `config_tuning_candidate`,
4. controller-legal candidate action families must still exist for that problem class,
5. and the bounded patch policy must be able to shape a legal smaller move from the current config.

In practice, that means the current diagnoser is driven by:

1. likely-human friction overspend,
2. suspicious-origin reach overspend,
3. or suspicious-origin latency overspend.

It is not currently driven by:

1. Scrapling required-surface misses,
2. Scrapling pass-where-expected versus fail-where-expected truth,
3. or category posture misses that are only visible through projected sim receipts.

# Required Follow-On Work

1. Replace projected recent-run category receipts with category-native adversary-sim evidence or explicitly degraded status so per-category benchmark rows stop copying a whole run into every observed category when category-native telemetry is absent.
2. Feed Scrapling defense-surface contract truth into controller-grade scoring or readiness gates so aggregate suspicious-origin suppression cannot by itself declare the loop healthy.
3. Separate aggregate leakage pressure, per-category target achievement, and Scrapling defense-surface satisfaction more explicitly in Game Loop rendering and copy.

# Audit Conclusion

The current game loop is not meaningless, but it is judging a narrower question than the page currently suggests.

Today it mainly answers:

1. is likely-human friction inside budget,
2. is aggregate suspicious-origin leakage inside budget,
3. and are coarse category and adversary proxies at least directionally acceptable.

It does not yet answer the stronger question the user now wants the loop to answer:

1. has full-power Scrapling truthfully exercised the defense surfaces it should,
2. did it pass and fail the right ones,
3. and is the diagnoser recommending config changes from that attacker-proof shortfall rather than from coarse aggregate suppression alone.

That gap needs to close before the strict `human_only_private` loop can be called operationally trustworthy.
