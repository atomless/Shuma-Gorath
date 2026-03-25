# SIM-LLM-1A Black-Box Contract Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn the later LLM attacker-agent's host-root-only, public-knowledge-only, Shuma-blind boundary into an executable repo contract over the existing adversarial contract surfaces.

**Architecture:** Reuse the current bounded LLM fulfillment contract path rather than creating a parallel LLM-only framework. Extend the existing frontier-action contract and fulfillment-plan adapters, mirror the black-box boundary into the Rust fulfillment-plan payload, and prove the result with focused LLM-fit tests. Keep this slice contract-first: do not reopen the full runtime actor.

**Tech Stack:** `scripts/tests/adversarial/frontier_action_contract.v1.json`, `scripts/tests/adversarial_runner/llm_fulfillment.py`, `src/admin/adversary_sim_llm_lane.rs`, focused unit tests, and active backlog docs.

---

## Task 1: Reopen `SIM-LLM-1A` And Retire The Stale Scrapling Umbrella Blocker

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`
- Modify: `docs/plans/2026-03-24-llm-player-role-decomposition-plan.md`
- Modify: `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`

**Work:**
1. Move `SIM-LLM-1A` from blocked to active work.
2. Update current sequencing notes so the next backend mainline after `TEST-MAINLINE-1` is `SIM-LLM-1A`, not deferred dashboard cleanup.
3. Retire the stale `SIM-SCR-CHALLENGE-1` blocker wording for current owned request-native surfaces.
4. Keep `SIM-SCR-CHALLENGE-2C` and `SIM-SCR-BROWSER-1` conditional only for future receipt-backed gaps.

**Acceptance criteria:**
1. The active queue truthfully shows `SIM-LLM-1A` as next.
2. Current plans no longer imply that the attacker-faithful Scrapling prerequisite is still unsatisfied for existing owned request-native surfaces.

## Task 2: Extend The Existing LLM Fulfillment Contract With The Black-Box Boundary

**Files:**
- Modify: `scripts/tests/adversarial/frontier_action_contract.v1.json`
- Modify: `scripts/tests/adversarial_runner/llm_fulfillment.py`
- Modify: `scripts/tests/test_llm_fulfillment.py`

**Work:**
1. Add one canonical `llm_attacker_black_box` section to the existing frontier-action contract.
2. Capture:
   - host-root-only entry,
   - category objective required,
   - malicious-category priming required,
   - public-host hint sources only,
   - allowed observation families,
   - forbidden knowledge sources,
   - explicit denial of web-search, repo visibility, and judge visibility,
   - receipt requirements.
3. Load and validate that section in the Python fulfillment helper.
4. Carry the normalized black-box boundary into the Python fulfillment plan payload.

**Acceptance criteria:**
1. The later attacker boundary is machine-readable in repo contracts, not prose only.
2. The Python fulfillment plan now exposes that boundary explicitly.

## Task 3: Mirror The Black-Box Boundary Into The Rust Fulfillment Plan

**Files:**
- Modify: `src/admin/adversary_sim_llm_lane.rs`
- Modify: `src/admin/api.rs`

**Work:**
1. Add a Rust-side black-box boundary struct to the LLM fulfillment plan.
2. Mirror the same contract semantics carried by the Python plan:
   - public-knowledge-only,
   - Shuma-blind,
   - host-root-only,
   - no web search or repo visibility,
   - no judge visibility,
   - receipt expectations.
3. Extend the existing Rust tests and internal-beat rendered proof to assert the new boundary is present.

**Acceptance criteria:**
1. Rust and Python fulfillment plans carry the same black-box contract concept.
2. The internal beat path proves the contract reaches the rendered machine payload.

## Task 4: Keep Verification Focused And Cheap

**Files:**
- Modify only if needed: `Makefile`
- Modify only if needed: `docs/testing.md`

**Work:**
1. Reuse the existing focused `test-adversarial-llm-fit` gate if it remains the smallest truthful proof path.
2. Only add a narrower target if the current target becomes untruthfully broad for this slice.

**Acceptance criteria:**
1. The slice is proven through the smallest truthful `make` path.
2. No new misleading verification target is introduced.

## Recommended Implementation Order

1. backlog and plan truth refresh
2. failing focused tests for the black-box boundary
3. contract-file and Python loader changes
4. Rust fulfillment-plan mirror
5. focused verification
6. post-implementation review and backlog closeout
