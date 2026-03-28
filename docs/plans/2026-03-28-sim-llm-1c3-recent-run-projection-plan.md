Date: 2026-03-28
Status: Implemented

Related context:

- [`../research/2026-03-28-sim-llm-1c3-recent-run-projection-review.md`](../research/2026-03-28-sim-llm-1c3-recent-run-projection-review.md)
- [`../research/2026-03-28-sim-llm-1c3-recent-run-projection-post-implementation-review.md`](../research/2026-03-28-sim-llm-1c3-recent-run-projection-post-implementation-review.md)
- [`../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md`](../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md)
- [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)
- [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/observability/operator_snapshot_live_traffic.rs`](../../src/observability/operator_snapshot_live_traffic.rs)
- [`../../dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte)

# SIM-LLM-1C3 Recent-Run Projection Plan

## Objective

Close the remaining `SIM-LLM-1C3` gap by making the live `bot_red_team` runtime truthfully visible through the same recent-run and operator-snapshot surfaces the repo already uses for Scrapling, without leaking simulator privilege into runtime or pretending the lane is more mature than it is.

## Core Decisions

1. Keep the existing typed `LlmRuntimeResult` contract as the source of runtime lineage truth.
2. Additive-only data-model change:
   - do not break or rename the current Scrapling recent-run shape,
   - add an optional LLM runtime summary field instead.
3. Persist one dedicated LLM recent-run receipt event at worker-result ingest time.
4. Project observed LLM fulfillment modes and categories from LLM receipt truth, not from a guessed runtime profile string.
5. Reuse the existing `Recent Red Team Runs` dashboard panel for rendered proof.
6. Do not enable the disabled `bot_red_team` lane control in Red Team as part of this slice.

## Execution Tranche

## `SIM-LLM-1C3`

### LLM runtime receipt projection and recent-run truth closure

Required contract:

1. a successful or failed LLM runtime worker-result ingest must persist one additive recent-run receipt event with:
   1. run/tick identity,
   2. lane/profile identity,
   3. runtime lineage,
   4. action counts,
   5. terminal failure or error truth,
   6. and bounded per-action receipts,
2. recent-run accumulation must recognize that additive LLM receipt event as recent-run evidence,
3. recent-run summaries must project:
   1. observed fulfillment modes,
   2. observed categories,
   3. and an additive `llm_runtime_summary`,
4. operator-snapshot recent-run rows must preserve that same additive LLM runtime truth,
5. the generic dashboard recent-run rendering path must surface the LLM row truthfully without implying the lane is operator-enabled,
6. and the proof path must distinguish provider-backed, degraded, and failed LLM runs instead of flattening them into lane presence.

Implementation steps:

1. Add failing tests first:
   - Rust test proving LLM worker-result ingest persists an additive recent-run receipt event,
   - Rust test proving `monitoring_recent_sim_run_summaries(...)` projects LLM modes/categories plus runtime lineage,
   - Rust test proving operator snapshot preserves the additive LLM recent-run summary,
   - dashboard unit test proving the generic recent-runs panel renders LLM runtime lineage or categories from the additive summary,
   - and Makefile selector-contract updates if a new focused target is added.
2. Add the additive runtime-summary data models:
   - event-log receipt summary for LLM runtime,
   - hot-read recent-run summary field,
   - operator-snapshot recent-run summary field.
3. Persist one dedicated LLM receipt event during worker-result ingest.
4. Extend recent-run accumulation to:
   - recognize LLM receipt events,
   - merge LLM fulfillment modes/categories,
   - and carry forward bounded runtime lineage.
5. Reuse the current generic dashboard recent-run view-model and table to surface that new truth with minimal additive presentation.
6. Update testing and operator docs to describe the LLM recent-run truth basis.

Acceptance criteria:

1. a `bot_red_team` runtime result now leaves additive recent-run evidence instead of disappearing after control-state ingest,
2. recent-run summaries truthfully show LLM fulfillment modes and categories for request-mode and browser-mode receipts,
3. recent-run summaries truthfully distinguish provider-backed, degraded, and failed LLM runtime lineage,
4. operator snapshot carries the additive LLM recent-run truth end to end,
5. dashboard proof shows a recent `bot_red_team` row with the projected runtime truth in the existing panel,
6. the lane can remain disabled in controls without making recent-run evidence invisible,
7. and the slice does not regress Scrapling recent-run coverage.

Proof:

1. `make test-adversarial-llm-runtime-dispatch`
2. one new focused make target for recent-run and operator projection proof, if needed
3. the smallest relevant dashboard proof target, likely by extending `make test-dashboard-red-team-truth-basis`
4. `make test`
5. `git diff --check`

## Definition Of Done

This slice is complete when:

1. the later LLM attacker runtime is visible end to end in recent-run and operator-snapshot truth,
2. the visibility is explicit about runtime lineage and failure semantics,
3. the shared recent-run path now truthfully handles both Scrapling and LLM lanes,
4. and the repo no longer needs to describe `SIM-LLM-1C3` as an abstract runtime-proof gap.
