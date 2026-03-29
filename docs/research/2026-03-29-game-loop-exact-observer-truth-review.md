Date: 2026-03-29
Status: Proposed planning driver

Related context:

- [`2026-03-28-game-loop-observer-facing-presentation-review.md`](2026-03-28-game-loop-observer-facing-presentation-review.md)
- [`2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md`](2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md)
- [`2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`../plans/2026-03-28-game-loop-observer-facing-presentation-plan.md`](../plans/2026-03-28-game-loop-observer-facing-presentation-plan.md)
- [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
- [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

# Purpose

Define the minimal machine-first contract repair needed so the Game Loop observer page can present exact judged rounds and exact simulator-owned lane categories without heuristics, blank backfills, or simulator labels leaking into Shuma runtime or tuning truth.

# Findings

## 1. The current UI is inventing observer truth because the contract drops two critical links

The current Game Loop page reconstructs the selected round by matching judged lane ids to recent runs using lane plus timestamp proximity, then falls back to the newest recent runs when no match is found.

It also assigns categories to a lane by using `observed_category_ids` when present and otherwise backfilling from round-level simulator ground truth.

Those are both observer-side inventions, not machine-first truth.

## 2. Scrapling category ownership is explicit in plans but not preserved as explicit recent-run receipt truth

Scrapling worker plans already carry bounded `category_targets`, and the worker validates that they match the allowed fulfillment-mode mapping.

But Scrapling worker results and Scrapling receipt events currently preserve:

1. the run id,
2. the lane,
3. the fulfillment mode through `sim_profile`,
4. and surface receipts,

while dropping the explicit category target list.

That forces recent-run summaries to reconstruct Scrapling categories later from `sim_profile`, which is good enough for internal bounded mapping but not strong enough for an exact observer contract.

## 3. LLM runtime already shows the stronger pattern we should reuse

`bot_red_team` recent-run truth is stronger because the LLM runtime result preserves explicit `category_targets`, the receipt event persists an `llm_runtime_summary`, and recent-run summaries can project that exact list without profile-based guessing.

Scrapling should follow the same observer-only contract pattern.

## 4. Judged episode identity already knows exact run ids earlier in the controller path

The candidate-window and continuation required-run contracts already preserve `follow_on_run_id` per lane.

But the completed episode archive currently keeps only `judged_lane_ids`, so the dashboard cannot select the exact runs that belonged to the judged episode without guessing.

This is the second missing observer link.

## 5. The fix can stay presentation-only if the new truth is kept on the observer rails

The required repair does not need to change:

1. Shuma runtime category inference,
2. benchmark scoring,
3. restriction diagnosis,
4. move selection,
5. or controller mutability.

It only needs to preserve already-bounded simulator-owned truth on the observer lineage:

1. exact Scrapling category targets per run,
2. and exact judged run ids per archived episode.

# Recommendations

## 1. Preserve explicit Scrapling category targets from plan to recent-run summary

Add an observer-only `category_targets` field to the Scrapling worker result, have the worker echo the already-validated plan targets, persist that list on the Scrapling receipt event, and let recent-run summaries prefer explicit receipt categories over `sim_profile` reconstruction.

## 2. Preserve exact judged run ids in the episode archive

Project the already-known `required_runs.follow_on_run_id` values into the archived episode record alongside `judged_lane_ids`.

That lets the Game Loop select the exact run rows for the round without lane-plus-time heuristics.

## 3. Make the dashboard fail closed on missing observer truth

The Game Loop should:

1. build the selected round from archived judged run ids,
2. build adversary rows only from lane-local observed category ids for those runs,
3. never backfill a lane from round-level simulator ground truth,
4. and render a truthful unavailable state when exact observer materialization is absent.

## 4. Keep simulator truth out of runtime and judge paths

The new explicit Scrapling category targets must remain observer-only and must not be used as:

1. runtime non-human classification truth,
2. benchmark restriction truth,
3. tuning eligibility proof,
4. or move-selection input.

They exist so the Game Loop can report what the simulator lane intentionally played, not so Shuma can self-grade from privileged labels.
