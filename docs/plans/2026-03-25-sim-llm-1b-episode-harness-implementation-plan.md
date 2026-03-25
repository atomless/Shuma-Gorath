# SIM-LLM-1B Episode Harness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the later LLM attacker episode lifecycle and bounded memory contract executable in the existing bounded fulfillment-plan surfaces.

**Architecture:** Reuse the existing adversarial contract path again. Extend the current frontier-action contract with one `llm_attacker_episode_harness` section, validate and emit it in the Python fulfillment helper, mirror it in the Rust fulfillment plan, and prove it through the existing focused `test-adversarial-llm-fit` gate. Keep live runtime orchestration out of scope.

**Tech Stack:** `scripts/tests/adversarial/frontier_action_contract.v1.json`, `scripts/tests/adversarial_runner/llm_fulfillment.py`, `src/admin/adversary_sim_llm_lane.rs`, focused tests, backlog/docs closeout.

---

## Task 1: Freeze The Episode Harness Contract

**Files:**
- Modify: `scripts/tests/adversarial/frontier_action_contract.v1.json`
- Modify: `scripts/tests/adversarial_runner/llm_fulfillment.py`
- Modify: `scripts/tests/test_llm_fulfillment.py`

**Work:**
1. Add one canonical `llm_attacker_episode_harness` section.
2. Capture:
   - required initial context,
   - environment reset requirement,
   - bounded action horizon semantics,
   - terminal conditions,
   - failure and completion states.
3. Capture bounded memory rules:
   - allowed retained episode-summary memory,
   - allowed curriculum inputs,
   - player-visible protected evidence allowed,
   - judge-held-out evaluation forbidden.
4. Validate and emit that section in the Python fulfillment loader and plan payload.

**Acceptance criteria:**
1. The attacker episode shape is machine-readable rather than prose only.
2. The memory boundary is explicit and consistent with the landed evaluation-visibility rules.

## Task 2: Mirror The Episode Harness Into The Rust Fulfillment Plan

**Files:**
- Modify: `src/admin/adversary_sim_llm_lane.rs`
- Modify: `src/admin/api.rs`

**Work:**
1. Add a Rust-side episode harness struct.
2. Mirror the same lifecycle and memory semantics carried by the Python payload.
3. Extend the existing Rust tests and internal-beat proof to assert the episode harness is present.

**Acceptance criteria:**
1. Rust and Python now expose the same attacker episode-harness concept.
2. The machine-visible internal beat proves the contract reaches the runtime payload.

## Task 3: Close The Backlog Slice Cleanly

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/completed-todo-history.md`
- Add: post-implementation review in `docs/research/`

**Work:**
1. Mark `SIM-LLM-1B` complete.
2. Promote the next backend slice truthfully.
3. Keep the full runtime actor blocked.

**Acceptance criteria:**
1. The backlog tells the truth about what landed.
2. The next slice is explicit rather than implied.
