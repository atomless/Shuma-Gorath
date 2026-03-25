# SIM-LLM-1C3 Runtime Proof Closure Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the final `SIM-LLM-1C3` seam by making live `bot_red_team` runtime results reconstructable from immutable event receipts and visible through the canonical recent-run and operator-snapshot paths.

**Architecture:** Reuse the existing event-log -> recent-sim-run hot-read -> operator-snapshot projection chain. Add one bounded shared LLM recent-run summary shape rather than inventing an LLM-only observability path.

**Tech Stack:** Rust event-log record model, Rust recent-run aggregation, Rust operator-snapshot projection, existing typed `LlmRuntimeResult`, Makefile verification, docs/backlog closeout.

---

## Guardrails

1. Do not create a second ad hoc store for LLM runtime receipts.
2. Do not widen the recent-run path beyond bounded summaries and bounded receipts.
3. Do not collapse provider-backed, degraded, and failed-closed runs into one ambiguous status.
4. Do not pretend browser-mode execution is solved if this slice only closes the observability seam.

## Task 1: Add one shared bounded LLM recent-run summary contract

**Files:**
- Add: `src/observability/llm_runtime_recent_run.rs`
- Modify: `src/observability/mod.rs`
- Modify: `src/observability/hot_read_documents.rs`
- Modify: `src/observability/operator_snapshot_live_traffic.rs`

**Work:**
1. Define one shared bounded summary for:
   - fulfillment mode,
   - backend kind/state,
   - generation source,
   - provider/model lineage,
   - fallback reason,
   - bounded action receipts,
   - action outcome buckets,
   - terminal failure,
   - and an explicit recent-run status (`provider_backed`, `degraded`, `failed_closed`).
2. Thread that shared summary through recent-run and operator-snapshot row types.

**Acceptance criteria:**
1. The repo has one reusable LLM recent-run summary contract rather than duplicate structs.
2. The summary is bounded and serializable through existing hot-read surfaces.

## Task 2: Persist LLM runtime receipts into the canonical recent-run source

**Files:**
- Modify: `src/admin/adversary_sim_api.rs`
- Modify: `src/admin/api.rs`
- Modify: `src/observability/non_human_lane_fulfillment.rs`
- Modify if needed: telemetry field-classification schema in `src/admin/api.rs`

**Work:**
1. Log one immutable event record for each typed LLM runtime result.
2. Record:
   - normalized runtime profile,
   - lane,
   - bounded receipt summary,
   - and the provider/degraded/failure lineage.
3. Extend recent-run aggregation to:
   - treat LLM runtime receipt events as recent-run evidence,
   - normalize `bot_red_team` modes into observed categories,
   - and project the bounded LLM recent-run summary forward.

**Acceptance criteria:**
1. `bot_red_team` results are reconstructable from immutable event-log records.
2. Recent-run rows expose the LLM runtime summary without disturbing Scrapling projection.

## Task 3: Add focused proof and close the tranche

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `Makefile`
- Modify: `scripts/tests/test_adversary_sim_make_targets.py`
- Modify: `docs/testing.md`
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `todos/completed-todo-history.md`
- Add: tranche post-implementation review

**Work:**
1. Add focused proof for:
   - bot-red-team profile normalization into categories,
   - recent-run aggregation of bounded LLM runtime receipts,
   - and full beat -> worker-result -> recent-run -> operator-snapshot closure.
2. Add a truthful focused Make target for this exact seam.
3. Update backlog/docs when the tranche lands.

**Acceptance criteria:**
1. `SIM-LLM-1C3` is provable through one dedicated Make target.
2. The repo can truthfully say the current live `bot_red_team` runtime is visible end to end through recent-run and operator-snapshot machine contracts.
