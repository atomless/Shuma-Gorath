Date: 2026-04-02
Status: Proposed

Related context:

- [`2026-03-28-sim-llm-1c3-recent-run-projection-review.md`](2026-03-28-sim-llm-1c3-recent-run-projection-review.md)
- [`2026-03-28-sim-llm-1c3-recent-run-projection-post-implementation-review.md`](2026-03-28-sim-llm-1c3-recent-run-projection-post-implementation-review.md)
- [`../plans/2026-03-28-sim-llm-1c3-recent-run-projection-plan.md`](../plans/2026-03-28-sim-llm-1c3-recent-run-projection-plan.md)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/llm_surface_observation.rs`](../../src/observability/llm_surface_observation.rs)
- [`../../src/admin/oversight_observer_round_archive.rs`](../../src/admin/oversight_observer_round_archive.rs)
- [`../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte)

# Agentic Recent-Run Coverage Gap Review

## Question

Why does the `Coverage` column in `Recent Red Team Runs` stay empty for `bot_red_team` rows even when Agentic runs already expose lane-relevant surface observations elsewhere in the repo?

## Findings

1. The current `Coverage` column is wired only to `owned_surface_coverage`, which is still a Scrapling-only recent-run field.
2. Recent-run aggregation explicitly computes that field only from `scrapling_surface_receipts` in [`../../src/admin/api.rs`](../../src/admin/api.rs).
3. The existing LLM recent-run projection tranche (`SIM-LLM-1C3`) made Agentic runtime lineage, categories, modes, and action receipts visible, but it did not add an additive Agentic surface-coverage summary to the shared recent-run shape.
4. The repo already has a truthful LLM surface-observation seam in [`../../src/observability/llm_surface_observation.rs`](../../src/observability/llm_surface_observation.rs), and that seam is already reused by the oversight observer archive in [`../../src/admin/oversight_observer_round_archive.rs`](../../src/admin/oversight_observer_round_archive.rs).
5. The current empty `Coverage` cells therefore do not mean Agentic traffic has no board-surface evidence. They mean the Red Team recent-run path still has no additive Agentic coverage projection.

## Root Cause

The shared recent-run model is still asymmetrical:

1. Scrapling rows carry a dedicated surface-closure summary (`owned_surface_coverage`).
2. Agentic rows carry runtime lineage (`llm_runtime_summary`) but no sibling compact coverage summary.
3. The dashboard table is therefore being truthful to the current payload, but the payload is missing one additive Agentic coverage surface.

## Important Constraint

This follow-on must not repeat the earlier latest-receipt mismatch.

`LlmRuntimeRecentRunSummary::merge_summary(...)` accumulates counts whole-run, but it still overwrites `latest_action_receipts` with the newest summary. So any Agentic coverage summary derived only from the merged `llm_runtime_summary.latest_action_receipts` would become latest-tick truth rather than whole-run truth.

The correct source is therefore:

1. aggregate per-receipt LLM surface observations during recent-run accumulation,
2. summarize those observations at the end of the run window,
3. and then project that additive whole-run coverage summary into hot-read/operator/dashboard surfaces.

## Recommendation

Add one new optional Agentic recent-run coverage summary rather than overloading Scrapling's field or adding Red Team-only UI logic.

That summary should:

1. be derived from the existing LLM surface-observation helper,
2. aggregate across all LLM receipt events in the bounded run window,
3. preserve honest observed/progress counts without pretending Scrapling-style required-surface closure exists where it does not,
4. and leave `monitoring_event_count` / `defense_delta_count` semantics unchanged, because those are still limited to true external monitoring events rather than receipt events.
