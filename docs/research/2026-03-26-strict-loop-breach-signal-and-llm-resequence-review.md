Date: 2026-03-26
Status: Proposed planning driver

Related context:

- [`2026-03-26-game-loop-scoring-and-diagnoser-audit.md`](2026-03-26-game-loop-scoring-and-diagnoser-audit.md)
- [`2026-03-26-ideal-rsi-game-loop-scoring-review.md`](2026-03-26-ideal-rsi-game-loop-scoring-review.md)
- [`2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md`](2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md)
- [`2026-03-26-sim-scr-full-1b-browser-and-proxy-capability-post-implementation-review.md`](2026-03-26-sim-scr-full-1b-browser-and-proxy-capability-post-implementation-review.md)
- [`../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/observability/benchmark_adversary_effectiveness.rs`](../../src/observability/benchmark_adversary_effectiveness.rs)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

# Purpose

Settle the next proof-order question for the strict `human_only_private` loop:

1. do not weaken the strict stance merely to manufacture a breach signal,
2. first prove whether the current Scrapling picture is truthful or misleading,
3. and only then decide whether the LLM attacker lane must move earlier in the mainline.

# Findings

## 1. Current Scrapling is stronger than the Game Loop can make it look

The active Scrapling runtime is no longer the old three-persona request-native lane.

Today it cycles:

1. `crawler`
2. `bulk_scraper`
3. `browser_automation`
4. `stealth_browser`
5. `http_agent`

via [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs), and the worker imports `FetcherSession`, `DynamicSession`, and `StealthySession` in [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py).

The current Scrapling-owned category contract now covers:

1. `indexing_bot`
2. `ai_scraper_bot`
3. `automated_browser`
4. `http_agent`

in [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs).

So the default explanation for a bleak Game Loop reading should not be "Scrapling is still the polite request-only lane."

## 2. The Game Loop still mixes planes that invite an over-bleak reading

The category target-achievement rows are posture-alignment math.

For `blocked` categories, they currently read as:

1. `short_circuited_requests / total_requests`
2. with a target of `1.0`

from [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs).

Meanwhile the compact Scrapling evidence row in [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte) is only a latest-run surface-summary projection.

That means the Game Loop can currently encourage the reading "Scrapling is totally unsuccessful" even though:

1. the category rows,
2. the compact latest-run evidence row,
3. and the coarse `representative_adversary_effectiveness` proxy

are not the same truth and do not answer the same question.

## 3. Weakening the strict stance is the wrong way to prove the loop

Relaxing `human_only_private` or introducing a looser baseline merely to provoke a config-change signal would invalidate the thing being proved.

It would show that Shuma can react under a weaker policy, not that it can defend and adapt under the intended strict one.

## 4. Positive control is still required, but it must be isolated from the strict proof claim

If no adversary lane can ever produce a breach signal, the adaptive loop cannot be proven.

But the right remedy is:

1. first maximize attacker faithfulness,
2. then verify that the scoring and projection are truthful,
3. then add stronger adversaries if needed,
4. and only use seeded or synthetic breach-positive controls as a separate diagnostic ring, never as the proof that the strict loop is operational.

# Recommendations

## 1. Add a dedicated Scrapling truth-audit gate before stricter loop claims

The next immediate proof step should be:

1. compare machine-first Scrapling receipts,
2. compare Red Team truth,
3. compare Game Loop projection,
4. and explicitly answer whether the current "fully blocked" operator reading is true or a presentation artifact.

## 2. Forbid weaker-baseline proof as part of `RSI-GAME-HO-1`

The strict-loop tranche should not count a looser policy baseline as evidence.

If a positive-control ring is needed, it should be explicitly labeled as:

1. diagnostic,
2. seeded,
3. and not evidence that the strict production-intended stance has been proven.

## 3. Promote `SIM-LLM-1C3` earlier only if Scrapling still cannot generate controller-grade breach signals

If, after:

1. full-power Scrapling,
2. truthful receipt-backed interpretation,
3. and the exploit-first `RSI-SCORE-2` judge,

the system still cannot produce localized breach signals under the unchanged strict stance, then the LLM lane should become the next adversary-strength tranche before further strict-loop proof claims.

That promotion should be evidence-driven, not speculative.

# Planning Implications

1. add an explicit `SIM-SCR-FULL-1C3` truth-audit step,
2. update strict-loop planning to forbid weakening `human_only_private` as proof,
3. and make `SIM-LLM-1C3` promotion conditional on Scrapling still failing to produce controller-grade breach signals under the strict stance.
