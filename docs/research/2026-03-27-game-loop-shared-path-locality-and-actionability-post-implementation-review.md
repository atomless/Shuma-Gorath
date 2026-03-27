Date: 2026-03-27
Status: Post-implementation review

Related context:

- [`2026-03-27-game-loop-category-posture-scoring-audit.md`](2026-03-27-game-loop-category-posture-scoring-audit.md)
- [`2026-03-27-game-loop-current-state-and-gap-review.md`](2026-03-27-game-loop-current-state-and-gap-review.md)
- [`2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`../plans/2026-03-27-game-loop-category-posture-truth-repair-plan.md`](../plans/2026-03-27-game-loop-category-posture-truth-repair-plan.md)
- [`../plans/2026-03-27-game-loop-board-state-refactor-plan.md`](../plans/2026-03-27-game-loop-board-state-refactor-plan.md)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/observability/benchmark_scrapling_evidence_quality.rs`](../../src/observability/benchmark_scrapling_evidence_quality.rs)
- [`../../src/observability/benchmark_scrapling_exploit_progress.rs`](../../src/observability/benchmark_scrapling_exploit_progress.rs)
- [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js)
- [`../../e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js)
- [`../../Makefile`](../../Makefile)

# Purpose

Review the March 27 accountability slice that targeted four immediate Game Loop needs:

1. truthful shared-path category scoring for non-verified attacker traffic,
2. more local host-cost and breach attribution,
3. stronger use of exploit-progress signals for precise config moves,
4. and a cleaner, less noisy board-state projection in the Game Loop UI.

# What Landed

## 1. Category posture now fails honest instead of pretending to know exact truth

`benchmark_non_human_categories.rs` now keeps projected or degraded category rows unscored by leaving `current = null` unless Shuma has exact category receipts.

That matters because the live classification path still does **not** justify broad exact category truth for non-verified Scrapling traffic:

1. `traffic_classification.rs` still maps non-verified suspicious automation to `unknown_non_human`,
2. `non_human_classification.rs` still turns recent sim-run category presence into degraded `projected_recent_sim_run` receipts,
3. and this slice intentionally did **not** promote that simulator-side knowledge into exact category truth.

So the repair here is truthful restraint:

1. exact category posture scores are still allowed when exact Shuma-side category receipts exist,
2. the benchmark can now score partial blocked shares from those exact receipts,
3. but projected Scrapling-populated rows remain explicitly unscored until Shuma itself can infer them from real observable request or behavior evidence.

`GameLoopTab.svelte` now renders those `current = null` rows as `Unscored` and suppresses the success bar instead of collapsing them into a misleading `0%`.

## 2. Breach loci now carry board-local host-cost and repair hints

`BenchmarkExploitLocus` in `benchmark_results.rs` now preserves:

1. `attempt_count`,
2. `cost_channel_ids`,
3. and `repair_family_candidates`.

`benchmark_scrapling_exploit_progress.rs` and `benchmark_scrapling_evidence_quality.rs` now materialize those fields from named Scrapling breach surfaces.

So the judge no longer says only "Scrapling made progress."
It can now say:

1. where the progress happened,
2. how many attempts were observed there,
3. what host-cost channels were consumed beyond the defended boundary,
4. and which bounded repair families are the first credible local responses.

## 3. Localized exploit progress can now drive bounded config tuning

This slice tightened the config path so exploit progress is not trapped as a pure code-gap by default.

`benchmark_results.rs` now allows the exploit-progress path through tuning eligibility when:

1. exploit progress is outside budget,
2. evidence quality is `high_confidence`,
3. and attribution is `surface_native_shared_path`.

`benchmark_results_comparison.rs` now lets `scrapling_exploit_progress` become `config_tuning_candidate` when named breach loci expose bounded repair families.

`oversight_patch_policy.rs` and `oversight_reconcile.rs` then preserve that localized order instead of flattening it back into a generic family list.

That means the controller can now recommend:

1. the smallest bounded move suggested by the observed breach surface,
2. in the order implied by the breach evidence,
3. while still refusing the move when exploit evidence is too weak or the bounded ring is exhausted.

## 4. The Game Loop UI now projects the board state more truthfully

`GameLoopTab.svelte`, `dashboard.modules.unit.test.js`, and `dashboard.smoke.spec.js` now present the page as separate planes rather than as one blended success score.

The rendered changes include:

1. `Origin Leakage And Human Cost` instead of the broader `Outcome Frontier`,
2. `Loop Actionability` instead of the vaguer `What The Loop Decided`,
3. `Board State` instead of the old `Where The Pressure Sits`,
4. an explicit guardrail note that leakage and human-cost rows do not prove total attacker defeat,
5. named breach loci with host-cost channels and repair families,
6. `Surface Contract Satisfaction` as a separate panel,
7. category posture rows that can read `Unscored`,
8. and top-level actionability text that now surfaces blocked state, benchmark decision, and whether a config move was actually applied.

This slice also added `dashboard-verify-freshness` to the `Makefile` so dashboard accountability checks fail fast if Playwright would otherwise be testing stale static assets.

# What Did Not Land

## 1. No new fake exact category inference

This slice did **not** add new exact Shuma-side category inference for non-verified Scrapling traffic.

That omission is intentional.

The current codebase still lacks broad evidence strong enough to say:

1. "this non-verified suspicious automation request is definitely `ai_scraper_bot`",
2. or "this one is definitely `automated_browser`",
3. purely from the same signals real external traffic would expose.

So the live Game Loop is now more honest, but not yet fully useful, for those Scrapling-populated category rows.
That remaining gap stays open until Shuma can infer exact categories from shared-path evidence rather than simulator persona knowledge.

## 2. No frontier-LLM code-evolution execution

This slice only improves the config-tuning side of the loop.
The later code-evolution ring remains planning-only and blocked behind explicit machine-first code-gap referral plus the stricter board-state doctrine.

## 3. No real human-friction calibration runtime

Human-friction remains a guardrail and later calibration ring.
This slice does not introduce a human-operated measurement loop or treat adversary-sim traffic as a proxy for human burden.

# Verdict

This slice materially improved Game Loop honesty and usefulness:

1. category rows no longer lie when exact attacker-category truth is missing,
2. exploit progress is now more localized,
3. the controller can now act on localized exploit progress when evidence quality is high enough,
4. and the dashboard makes the different scoring planes easier to interpret.

But it does **not** complete the whole problem.

The biggest remaining gap is still the same one the audit identified:
live useful category posture scoring for non-verified Scrapling traffic requires stronger Shuma-side category inference than the runtime currently has.

# Evidence

The focused proof for this slice is:

1. `make test-benchmark-results-contract`
2. `make test-rsi-score-exploit-progress`
3. `make test-rsi-score-evidence-quality`
4. `make test-rsi-score-move-selection`
5. `make test-traffic-classification-contract`
6. `make test-dashboard-game-loop-accountability`
7. `git diff --check`
