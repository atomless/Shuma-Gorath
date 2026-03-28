Date: 2026-03-28
Status: Proposed

Related context:

- [`2026-03-25-sim-llm-1c-runtime-readiness-review.md`](2026-03-25-sim-llm-1c-runtime-readiness-review.md)
- [`2026-03-25-sim-llm-1c1-live-frontier-action-generation-post-implementation-review.md`](2026-03-25-sim-llm-1c1-live-frontier-action-generation-post-implementation-review.md)
- [`2026-03-25-sim-llm-1c2-runtime-dispatch-and-ingest-post-implementation-review.md`](2026-03-25-sim-llm-1c2-runtime-dispatch-and-ingest-post-implementation-review.md)
- [`../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md`](../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md)
- [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)
- [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/observability/operator_snapshot_live_traffic.rs`](../../src/observability/operator_snapshot_live_traffic.rs)
- [`../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte)
- [`../../todos/todo.md`](../../todos/todo.md)

# SIM-LLM-1C3 Recent-Run Projection Review

## Question

With `SIM-LLM-1C1` and `SIM-LLM-1C2` landed, what is the truthful remaining `SIM-LLM-1C3` gap in the current repo?

## Conclusion

`SIM-LLM-1C3` is no longer mainly a runtime-execution problem.

The real remaining gap is recent-run and operator projection:

1. the live `bot_red_team` lane can already generate actions, dispatch a worker, execute request-mode black-box work, and ingest a typed runtime result,
2. but the machine-first recent-run surfaces still only understand Scrapling receipt events,
3. so a real LLM runtime run largely disappears before it reaches `Recent Red Team Runs`, hot-read recent-run documents, and operator-snapshot recent-run truth.

That means the repo already has a live later attacker runtime path, but it still lacks a truthful end-to-end proof surface for that actor.

## What is already real

The older March 25 review is now stale in two important ways.

`SIM-LLM-1C1` is already landed:

1. the LLM fulfillment helper now performs provider-backed generation attempts,
2. validates actions through the existing frontier-action contract,
3. and records provider-vs-fallback lineage explicitly.

`SIM-LLM-1C2` is already landed too:

1. the host-side supervisor dispatches `dispatch_mode = "llm_fulfillment_plan"`,
2. the dedicated worker emits typed `adversary-sim-llm-runtime-result.v1` payloads,
3. and the internal worker-result endpoint ingests those typed results and applies them to adversary-sim control state and lane diagnostics.

The typed runtime result in [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs) already preserves the key operator-relevant truth:

1. `backend_kind`
2. `backend_state`
3. `generation_source`
4. `provider`
5. `model_id`
6. `fallback_reason`
7. `category_targets`
8. action counts
9. `terminal_failure`
10. and per-action receipts

So the live runtime seam exists. The missing work is what happens after ingest.

## The current projection gap

The recent-run and operator-snapshot path is still Scrapling-shaped.

In [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs):

1. Scrapling worker results persist a dedicated event-log receipt row through `log_scrapling_surface_receipts_event(...)`,
2. but LLM runtime results do not persist an analogous additive recent-run receipt event.

In [`../../src/admin/api.rs`](../../src/admin/api.rs):

1. `EventLogRecord` has `scrapling_surface_receipts`,
2. `is_recent_sim_run_receipt_event(...)` only recognizes receipt events through non-empty Scrapling surface receipts,
3. and `monitoring_recent_sim_run_summaries(...)` only accumulates additive per-run proof from that Scrapling receipt path.

In [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs) and [`../../src/observability/operator_snapshot_live_traffic.rs`](../../src/observability/operator_snapshot_live_traffic.rs):

1. recent-run summaries currently expose mode/category sets and optional Scrapling owned-surface coverage,
2. but they have no additive field for LLM runtime lineage or runtime-result truth.

In [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs):

1. `llm_category_targets_for_mode(...)` exists,
2. but `observed_category_targets_for_runtime_profile(...)` only normalizes Scrapling runtime profiles,
3. so `bot_red_team` recent runs currently cannot truthfully project their observed fulfillment modes or categories through the shared recent-run path.

## Why this matters

Without this projection seam:

1. `bot_red_team` can run but still reads as effectively invisible in the recent-run evidence path,
2. the operator cannot tell whether a lane row reflects provider-backed, degraded, or failed runtime execution,
3. and the later combined-attacker strict-loop proof stays blocked because the shared machine-first surfaces do not yet tell the full truth about the second attacker.

This is exactly the kind of silent maturity overstatement the repo is trying to avoid.

## Decision

Refresh `SIM-LLM-1C3` around one additive recent-run projection slice.

The next implementation should:

1. preserve the current typed runtime result contract,
2. add one additive LLM runtime receipt summary to event-log and recent-run surfaces,
3. project truthful LLM modes/categories and runtime lineage through hot-read and operator-snapshot recent-run summaries,
4. reuse the existing generic `Recent Red Team Runs` UI rather than enabling the disabled `bot_red_team` control surface yet,
5. and prove the full path with focused Rust, Makefile, and rendered dashboard checks.

Do not treat this as lane enablement or as a second runtime rewrite.
