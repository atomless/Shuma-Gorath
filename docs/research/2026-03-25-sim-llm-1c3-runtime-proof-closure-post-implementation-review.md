Date: 2026-03-25
Status: Completed

Related plan:

- [`../plans/2026-03-25-sim-llm-1c3-runtime-proof-closure-plan.md`](../plans/2026-03-25-sim-llm-1c3-runtime-proof-closure-plan.md)

# SIM-LLM-1C3 Runtime Proof Closure Post-Implementation Review

## What landed

`SIM-LLM-1C3` now closes the live `bot_red_team` recent-run and machine-first observability seam.

The repo now has one shared bounded LLM recent-run summary contract in:

- [`../../src/observability/llm_runtime_recent_run.rs`](../../src/observability/llm_runtime_recent_run.rs)

That shared shape records:

1. fulfillment mode,
2. backend kind/state,
3. generation source,
4. provider/model lineage,
5. bounded action receipts,
6. bounded outcome buckets,
7. terminal failure,
8. and an explicit recent-run status that distinguishes provider-backed, degraded, and failed-closed runs.

The runtime ingest path now persists those receipts into the canonical immutable event log in:

- [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)

So a typed LLM worker result no longer dies inside control-state counters; it also leaves a bounded receipt event that recent-run reconstruction can trust.

Recent-run and snapshot projection now carry that summary through the canonical hot-read path in:

- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)
- [`../../src/observability/operator_snapshot_live_traffic.rs`](../../src/observability/operator_snapshot_live_traffic.rs)

`bot_red_team` runtime profiles are now also normalized into observed fulfillment modes and category coverage through:

- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)

So the recent-run path now understands both Scrapling and LLM lanes instead of remaining Scrapling-only.

## Verification

- `make test-adversarial-llm-runtime-proof`
- `make test-adversary-sim-make-target-contract`
- `git diff --check`

## Outcome Against Plan

This tranche met the planned acceptance bar:

1. live `bot_red_team` worker results are now reconstructable from immutable event receipts,
2. the recent-run hot-read path exposes bounded LLM runtime lineage and receipts,
3. `operator_snapshot_v1` carries that same truth forward,
4. and the focused Make target now proves the full dispatch plus projection seam instead of stopping at typed ingest.

## Important follow-on

This slice closes the runtime proof chain for the **current live LLM request/runtime path**.

It does **not** yet mean the LLM attacker fully fulfills the browser-owned non-human categories required for the later mixed-attacker strict-loop tranche.

`browser_mode` still fails closed in the runtime worker today. That is explicit in:

- [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py)

So before `RSI-GAME-HO-2` can honestly reopen, the repo still needs a follow-on slice to land bounded executed browser-mode fulfillment for:

1. `automated_browser`
2. `browser_agent`
3. `agent_on_behalf_of_human`

The backlog should now reflect that explicit follow-on rather than treating `SIM-LLM-1C3` alone as sufficient for the mixed-attacker strict-loop proof.
