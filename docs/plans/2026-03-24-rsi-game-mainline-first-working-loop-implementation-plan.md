# RSI-GAME-MAINLINE-1 First Working Loop Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prove that the existing bounded shared-host oversight loop is operating as the first working self-improving loop over the attacker-faithful Scrapling basis rather than only as controller wiring in isolation.

**Architecture:** Reuse the existing oversight agent, apply loop, operator snapshot, Scrapling owned-surface coverage summary, and episode archive. Do not invent a new controller path. Instead, add one dedicated proof contract that ties the attacker basis, post-sim judge execution, and episode lineage together in both local focused proof and the shared-host live verifier.

**Tech Stack:** Rust oversight and operator-snapshot modules, Python live verifier tooling, Makefile proof targets, planning and testing docs.

---

## Guardrails

1. Do not add a second self-improvement runtime; this tranche proves the existing bounded loop.
2. Do not weaken the attacker-faithful requirement by falling back to synthetic-lane proof.
3. Do not require live proof to force a retained or rolled-back canary; valid bounded episode states must remain truthful to current live conditions.
4. Keep the proof machine-first: attacker evidence, judge execution, and archive lineage must all be read from real Shuma surfaces rather than invented test-only payloads.

## Task 1: Freeze the explicit `RSI-GAME-MAINLINE-1` contract

**Files:**
- Modify: `docs/plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `todos/todo.md`

**Work:**
1. Add the dedicated `RSI-GAME-MAINLINE-1` plan and review to the planning chain.
2. State explicitly that this tranche proves:
   - Scrapling attacker basis,
   - post-sim oversight execution,
   - and episode/archive lineage coherence.
3. Keep later dashboard work and LLM runtime work downstream of this proof.

**Acceptance criteria:**
1. The backlog no longer treats `RSI-GAME-MAINLINE-1` as only a sentence fragment.
2. The repo has one discoverable contract for what counts as the first working loop.

## Task 2: Add focused local failing proof first

**Files:**
- Modify: `src/admin/oversight_agent.rs`
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add a focused local proof test that seeds attacker-faithful Scrapling recent-run receipts into the operator snapshot, runs a post-sim oversight cycle, and proves:
   - `scrapling_traffic` is the attacker basis,
   - the owned-surface coverage summary is `covered`,
   - the oversight run records `latest_sim_run_id`,
   - and the episode archive row records coherent acceptance and completion lineage.
2. Add a focused Make target such as `test-rsi-game-mainline`.
3. Verify the new proof fails before implementation logic is added or helper surfaces are updated.

**Acceptance criteria:**
1. There is one cheap deterministic proof for the first working loop.
2. That proof explicitly depends on Scrapling-owned surface coverage and episode lineage, not only on generic oversight execution.

## Task 3: Strengthen the shared-host live verifier

**Files:**
- Modify: `scripts/tests/live_feedback_loop_remote.py`
- Modify: `scripts/tests/test_live_feedback_loop_remote.py`
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Make the live verifier explicitly enable `scrapling_traffic`.
2. Add an explicit live preflight for `ADVERSARY_SIM_SCRAPLING_PUBLIC_NETWORK_IDENTITIES` so missing attacker-faithful public-network identity configuration fails immediately rather than surfacing later as a vague partial-coverage result.
3. After the completed sim run, fetch `operator_snapshot_v1` and assert:
   - the matching recent run is Scrapling,
   - the coverage summary schema is correct,
   - the coverage summary is `covered`,
   - and the current `sim_run_id` is visible in recent runs.
4. Fetch oversight history and assert there is a coherent episode row for the current post-sim run, with acceptance or completion semantics that match the recorded apply stage.
5. Update the unit fixtures so they are Scrapling-shaped rather than synthetic-lane shaped.

**Acceptance criteria:**
1. The live/shared-host verifier proves the loop is operating over the truthful attacker lane.
2. Missing live public-network identity readiness is reported as an explicit preflight blocker instead of a late ambiguous coverage miss.
3. The live proof now ties attacker evidence, judge execution, and episode lineage together.

## Task 4: Close the tranche and sync the paper trail

**Files:**
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-24-rsi-game-mainline-first-working-loop-post-implementation-review.md`

**Work:**
1. Move `RSI-GAME-MAINLINE-1` to completed history once proof is landed.
2. Record why the tranche matters: it is the first explicit proof that Shuma's loop is operating over the attacker-faithful Scrapling basis.
3. Update the research and plan indexes.
4. Add the post-implementation review with any newly exposed gaps.

**Acceptance criteria:**
1. The planning chain shows `RSI-GAME-MAINLINE-1` as a delivered proof tranche.
2. Any remaining next-step gaps are explicit rather than implied.

## Verification

1. `make test-rsi-game-mainline`
2. `make test-live-feedback-loop-remote-unit`
3. `git diff --check`

If remote access is available and the tranche remains truthfully scoped, also run:

4. `make test-live-feedback-loop-remote`

## Exit Criteria

This tranche is complete when:

1. a focused local proof shows the bounded loop operating over Scrapling-owned surface coverage and recording episode lineage,
2. the shared-host live verifier explicitly proves Scrapling basis plus episode linkage,
3. the proof chain is documented and discoverable,
4. and `RSI-GAME-MAINLINE-1` can move out of the active backlog.
