Date: 2026-03-28
Status: Proposed planning driver

Related context:

- [`2026-03-27-game-loop-current-state-and-gap-review.md`](2026-03-27-game-loop-current-state-and-gap-review.md)
- [`2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md`](2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md)
- [`../plans/2026-03-28-rsi-game-ho-2-combined-attacker-orchestration-plan.md`](../plans/2026-03-28-rsi-game-ho-2-combined-attacker-orchestration-plan.md)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`../../src/observability/operator_snapshot_non_human.rs`](../../src/observability/operator_snapshot_non_human.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)

# Purpose

Define how to reframe the Game Loop tab as an observer-facing summary of recent rounds, adversaries, and defences without changing the loop mechanics, controller policy, or judge semantics.

# Findings

## 1. The current page still tells the story from inside the loop

`GameLoopTab.svelte` currently opens with machine-first status cards and then spends its main top-level sections on:

1. `Recent Loop Progress`,
2. `Origin Leakage And Human Cost`,
3. `Loop Actionability`,
4. `Board State`,
5. and `Trust And Blockers`.

That is truthful, but it reads like the judge narrating its own internals rather than an outside observer watching recent rounds play out.

## 2. The round history contract already exists, but the dashboard is dropping some of the most useful fields

The episode archive record in [`src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs) already includes:

1. `completed_at_ts`,
2. `judged_lane_ids`,
3. `proposal`,
4. `watch_window_result`,
5. `retain_or_rollback`,
6. and `cycle_judgment`.

The current dashboard adapter preserves only a subset of that archive row, so the Game Loop tab cannot currently render a simple, actor-readable recent-round summary even though the backend already materializes the truth needed for it.

## 3. The adversary-side cast already exists in machine contracts, but the page is projecting the wrong rail

The operator snapshot recognition rail already materializes:

1. `simulator_ground_truth` categories,
2. `comparison_rows`,
3. current exact/degraded/collapsed counts,
4. and evidence references.

That means the system already knows, after the fact:

1. which non-human categories appeared in recent sim runs,
2. and how Shuma classified them.

But the Game Loop tab is still rendering `non_human_category_posture` target-achievement meters rather than the actual category-vs-inference comparison rows, so the page gives a worse story than the backend truth already supports.

## 4. The defence-side cast already exists without leaking simulator labels into runtime truth

The defence-facing observer story can be built from existing surface-native contracts:

1. `mixed_attacker_restriction_progress` breach loci,
2. `restriction_diagnosis.breach_loci`,
3. and Scrapling owned-surface coverage receipts from recent run summaries.

Those rows already expose:

1. surface labels,
2. attempt counts,
3. request samples,
4. evidence status,
5. host-cost channels,
6. repair-family locality,
7. and surface-state or dependency truth.

That is enough to describe what the defence surfaces saw and how they fared without giving those surfaces simulator-known category labels.

## 5. The right presentation shift is actor-first and simple

The page should lead with a compact round history and then present two casts:

1. the adversaries that showed up,
2. and the defence surfaces that encountered them.

The judge's internal rails should remain available, but lower on the page and no longer as the first story a human sees.

# Recommendations

## 1. Lead with recent rounds, not judge internals

The top of the page should summarize recent judged rounds with only:

1. when the round completed,
2. which lanes participated,
3. whether the move was retained or rolled back,
4. which config family moved,
5. and whether the loop stopped, continued, or broke homeostasis.

## 2. Replace recognition meters with an adversary cast

The adversary panel should use `simulator_ground_truth` plus `comparison_rows` to show:

1. which categories appeared,
2. which lane brought them,
3. what the run attempted at a simple level,
4. and what Shuma inferred.

That is a much more honest observer view than target-achievement bars for canonical categories that may not even have appeared.

## 3. Add a defence cast built from surface-native evidence only

The defence panel should render only surface-native fields:

1. surface or breach-locus label,
2. observed sample or attempt count,
3. surface state or evidence status,
4. and whether the surface held, leaked, or remained unreached.

## 4. Keep this tranche presentation-only

This work should:

1. reuse existing backend machine contracts wherever possible,
2. extend the dashboard adapter only for already-materialized fields,
3. avoid changing controller logic, benchmark logic, or loop orchestration,
4. and prove the redesign through focused Game Loop dashboard tests rather than through loop-mechanics tests.
