# SIM-LLM-1C Runtime Decomposition Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the vague `SIM-LLM-1C` umbrella with a truthful sequence of runtime slices that turns the later LLM attacker from a plan-only concept into a real executed actor over the settled black-box and episode contracts.

**Architecture:** Reuse the existing `llm_fulfillment_plan`, adversarial contract files, and container black-box runner rather than inventing a second runtime stack. Separate the remaining work into three seams: live frontier action generation, supervisor/ingest wiring, and runtime proof plus recent-run projection. Keep `bot_red_team` disabled until the full path is real.

**Tech Stack:** Rust adversary-sim runtime and internal API, Rust supervisor, Python adversarial runner and container worker, existing frontier-action contract and runtime profile, Makefile verification, backlog/docs closeout.

---

## Guardrails

1. Do not claim a live LLM attacker exists until action generation, dispatch, ingest, and proof all exist.
2. Do not widen the actor beyond the settled host-root-only and Shuma-blind contract.
3. Do not let deterministic fallback pretend to be provider-backed generation; receipt lineage must make the source explicit.
4. Do not enable `bot_red_team` in the operator surface until the full runtime chain is proven.

## Task 1: Reopen `SIM-LLM-1C` Truthfully In The Backlog

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`
- Modify: `docs/plans/2026-03-24-llm-player-role-decomposition-plan.md`
- Modify: `docs/plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`

**Work:**
1. Keep the `SIM-LLM-1` and `SIM-LLM-1C` umbrellas blocked.
2. Add explicit runtime sub-slices:
   - `SIM-LLM-1C1` live frontier action-generation backend
   - `SIM-LLM-1C2` supervisor dispatch and result-ingest runtime path
   - `SIM-LLM-1C3` runtime receipt projection and proof closure
3. Make `SIM-LLM-1C1` the next active backend slice.
4. Keep later sub-slices blocked behind the earlier ones.

**Acceptance criteria:**
1. The backlog no longer implies the full runtime actor is one immediate executable chunk.
2. The next active step is explicit and truthful.

## Task 2: `SIM-LLM-1C1` Live Frontier Action-Generation Backend

**Files:**
- Later code targets: `scripts/tests/adversarial_runner/`, `scripts/tests/adversarial_container_runner.py`, `scripts/tests/adversarial_runner/governance.py`, `Makefile`, focused Python tests
- Reference contracts: `scripts/tests/adversarial/frontier_action_contract.v1.json`, `scripts/tests/adversarial/container_runtime_profile.v1.json`

**Work:**
1. Add one canonical provider-backed attack-generation adapter that:
   - selects configured frontier providers,
   - uses the settled black-box and episode contracts as prompt or payload context,
   - generates attacker actions from only host-derived observations,
   - validates those actions through the existing frontier-action contract,
   - and records provider or fallback lineage explicitly.
2. Keep deterministic fallback available only as a clearly labeled degraded path.
3. Add focused proof that:
   - provider-backed generation is attempted when keys exist,
   - degraded fallback is explicit when keys do not exist,
   - and no forbidden Shuma-specific knowledge enters the generation context.

**Acceptance criteria:**
1. The repo has a real action-generation path, not just provider metadata.
2. Generated actions remain black-box, bounded, and contract-validated.

## Task 3: `SIM-LLM-1C2` Supervisor Dispatch And Runtime Result Ingest

**Files:**
- Later code targets: `scripts/supervisor/adversary_sim_supervisor.rs`, `src/admin/adversary_sim_api.rs`, `src/admin/adversary_sim_lane_runtime.rs`, `src/admin/adversary_sim_worker_plan.rs`, focused Rust and supervisor tests
- Reference runtime: `scripts/tests/adversarial_container_runner.py`, `scripts/tests/adversarial_container/worker.py`

**Work:**
1. Add supervisor handling for `dispatch_mode = "llm_fulfillment_plan"`.
2. Reuse the existing container black-box runner to execute the actor plan.
3. Add a typed internal result payload for LLM runtime results rather than trying to stuff them into the Scrapling worker result schema.
4. Persist the runtime outcome into control-state and recent-run materialization surfaces.
5. Preserve fail-closed behavior when action generation, container launch, or result validation fails.

**Acceptance criteria:**
1. `bot_red_team` can actually execute a bounded actor run from the live beat path.
2. The result path is typed and separate from the Scrapling worker contract.

## Task 4: `SIM-LLM-1C3` Runtime Receipt Projection And Proof Closure

**Files:**
- Later code targets: `src/admin/api.rs`, `src/observability/hot_read_documents.rs`, `src/observability/non_human_classification.rs`, focused Rust/dashboard-proof targets if the rendered surface changes
- Modify if needed: `docs/testing.md`, `Makefile`

**Work:**
1. Project LLM attacker runtime outcomes into recent-run and machine-first observability surfaces.
2. Preserve:
   - action-source lineage,
   - observed categories,
   - bounded execution receipts,
   - coverage or blocking state,
   - terminal failure reasons.
3. Add end-to-end proof that the runtime:
   - generates actions,
   - executes them,
   - persists results,
   - and materializes them in the recent-run path Shuma already uses.
4. Only after this proof closes should the lane be reconsidered for operator enablement.

**Acceptance criteria:**
1. The later LLM attacker runtime is proven end to end rather than only at contract or dispatch level.
2. The observability path truthfully distinguishes provider-backed, degraded, and failed runs.

## Recommended Implementation Order

1. backlog and sequencing truth refresh
2. `SIM-LLM-1C1`
3. `SIM-LLM-1C2`
4. `SIM-LLM-1C3`
5. only then reconsider enabling `bot_red_team` or collapsing the umbrella blocker
