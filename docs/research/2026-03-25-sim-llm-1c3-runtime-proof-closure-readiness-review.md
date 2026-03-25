Date: 2026-03-25
Status: Proposed

Related context:

- [`2026-03-25-sim-llm-1c-runtime-readiness-review.md`](2026-03-25-sim-llm-1c-runtime-readiness-review.md)
- [`2026-03-25-sim-llm-1c2-runtime-dispatch-and-ingest-post-implementation-review.md`](2026-03-25-sim-llm-1c2-runtime-dispatch-and-ingest-post-implementation-review.md)
- [`../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md`](../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md)
- [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../todos/todo.md`](../../todos/todo.md)

# SIM-LLM-1C3 Runtime Proof Closure Readiness Review

## Question

With `SIM-LLM-1C1` and `SIM-LLM-1C2` landed, is the repo ready for the final runtime-proof slice that closes recent-run and machine-first observability for the live `bot_red_team` actor?

## Conclusion

Yes.

The remaining gap is now tightly scoped and implementation-ready:

1. the LLM worker result is already typed and ingested,
2. the recent-run hot-read path already exists and is already the canonical machine-first route for adversary runtime truth,
3. and the missing work is now projection and proof closure rather than another open-ended runtime architecture seam.

## What is already real

The earlier runtime seams are landed:

1. `bot_red_team` now emits live `llm_fulfillment_plan` payloads through the internal beat path,
2. the supervisor dispatches a dedicated LLM runtime worker,
3. the worker executes bounded request-mode black-box traffic and returns typed action receipts,
4. and the Rust ingest path updates control-state and lane diagnostics from `adversary-sim-llm-runtime-result.v1`.

So the repo no longer needs another execution path for this slice. It needs to make the existing one observable through the same recent-run surfaces Shuma already trusts.

## The remaining truth gap

Two concrete issues remain after `SIM-LLM-1C2`:

1. no immutable event-log receipt currently preserves the LLM runtime lineage and bounded action receipts for later recent-run reconstruction,
2. and `monitoring_recent_sim_run_summaries(...)` still only understands Scrapling-owned receipt projection.

That means `bot_red_team` can execute, but its provider lineage, fulfillment mode, observed categories, and bounded receipts are still largely invisible to the machine-first recent-run path.

## Ready implementation shape

The cleanest closure is:

1. add one bounded shared LLM recent-run summary type,
2. log one immutable LLM runtime receipt event on worker-result ingest,
3. reuse the existing recent-run hot-read pipeline to aggregate that summary,
4. project it into `operator_snapshot_v1`,
5. and add one truthful focused Make target for this exact closure seam.

This stays aligned with existing project patterns:

1. immutable event log as canonical recent-run source,
2. bounded hot-read summaries rather than ad hoc side stores,
3. one recent-run row type with optional lane-specific details instead of a second LLM-only projection stack.

## Post-slice expectation

If this slice lands cleanly, the repo should be able to prove:

1. a live `bot_red_team` run executes,
2. a receipt event is persisted,
3. recent-run hot reads expose its bounded runtime lineage and receipts,
4. and `operator_snapshot_v1` carries that same truth forward.

That closes the projection seam.

It does **not** by itself guarantee that browser-mode execution is already mature enough for the later mixed-attacker strict-loop tranche. That follow-on must be judged explicitly after this slice lands.
