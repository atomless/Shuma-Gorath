# ADV-DIAG-1 Adversary-Sim Status Truth Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `/admin/adversary-sim/status` align with immutable shared-host simulation event truth when mutable run-state counters under-report generated traffic.

**Architecture:** Keep the existing control-plane intact and add a small event-truth projection step in the status path. The projection should use existing recent simulation evidence derived from immutable event telemetry, recover lower-bound generation and lane counters for the active or last completed run, and expose the basis of those counters in the returned payload.

**Tech Stack:** Rust admin/runtime modules, existing hot-read/event telemetry summaries, Makefile-driven focused verification, live Linode proof script.

---

### Task 1: Add the failing backend regression

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs`
- Verify via: `/Users/jamestindall/Projects/Shuma-Gorath/Makefile`

**Steps:**
1. Add a focused test that seeds adversary-sim state with zero generation counters for a known `run_id`, writes simulation-tagged monitoring events for that run, and asserts `/admin/adversary-sim/status` no longer reports impossible zero generation/lane diagnostics.
2. Run that focused test alone first and confirm it fails for the right reason.

### Task 2: Implement status event-truth projection

**Files:**
- Create: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/adversary_sim_status_truth.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/mod.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/adversary_sim_api.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/adversary_sim_diagnostics.rs`

**Steps:**
1. Add a small helper that looks up persisted recent sim-run evidence for the active or last completed `sim_run_id`.
2. Project that evidence into a cloned status state only when control-state counters or lane diagnostics are weaker than the persisted evidence.
3. Keep the projection explicitly lower-bound based:
   - request count may be recovered from persisted monitoring-event count,
   - tick count may only recover from zero to at least one when persisted run evidence exists,
   - lane diagnostics may recover generated requests, beat success lower bound, and last-generated timestamp for the selected runtime lane.
4. Expose the counter/truth basis in the payload so operators and tests can see when persisted event truth was used.

### Task 3: Add focused verification wiring

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/Makefile`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/tests/test_live_feedback_loop_remote.py`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/tests/live_feedback_loop_remote.py`

**Steps:**
1. Add a truthful narrow `make` target for adversary-sim diagnostics truth verification.
2. Extend the live shared-host verifier so it fails if a completed run still exposes impossible zero status counters after persisted recent-event evidence proves traffic.
3. Add or update the focused remote verifier unit coverage for the new live-proof expectation.

### Task 4: Verify, review, and close out

**Files:**
- Create: `/Users/jamestindall/Projects/Shuma-Gorath/docs/research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/todos/todo.md`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/todos/blocked-todo.md`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/todos/completed-todo-history.md`
- Modify if needed: `/Users/jamestindall/Projects/Shuma-Gorath/docs/research/README.md`

**Steps:**
1. Run the smallest relevant Make targets first.
2. Run the live Linode proof after the local gates pass.
3. Write the tranche review and immediately fix any shortfall it finds.
4. Move `ADV-DIAG-1` into completed history once the live proof and review both pass.
