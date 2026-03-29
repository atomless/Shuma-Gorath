# Game Loop Exact Observer Truth Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove Game Loop observer-side heuristics by preserving exact judged run ids and explicit lane-owned Scrapling category targets through the existing machine-first observer contracts.

**Architecture:** Extend the existing observer lineage rather than changing runtime or controller semantics. Preserve explicit Scrapling category targets on worker-result and receipt-event paths, preserve judged run ids in the completed episode archive, and then make the dashboard consume only those exact observer fields with truthful fail-closed rendering when data is absent.

**Tech Stack:** Rust admin and observability contracts, Python Scrapling worker, Svelte dashboard, focused Rust/Python/dashboard tests, Game Loop docs and TODO bookkeeping.

---

## Acceptance Criteria

1. Recent sim run summaries preserve explicit Scrapling-owned category targets from worker receipts instead of reconstructing them only from `sim_profile`.
2. Completed episode archive rows preserve exact judged run ids in addition to judged lane ids.
3. The Game Loop tab selects the active observer round from archived judged run ids, not lane-plus-time heuristics or recent-run fallback slices.
4. The Game Loop adversary cast renders only lane-local categories for the judged runs in that round and never backfills a lane from round-level simulator ground truth.
5. The repair remains presentation-only: no simulator category labels are introduced into runtime classification, benchmark scoring, restriction diagnosis, or controller move selection.
6. Focused proof exists through:
   - `make test-adversary-sim-scrapling-worker`
   - `make test-oversight-episode-archive`
   - `make test-operator-snapshot-foundation`
   - `make test-dashboard-game-loop-accountability`

## Task 1: Freeze the missing observer links in tests

**Files:**
- Modify: `scripts/tests/test_scrapling_worker.py`
- Modify: `src/admin/api.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`

**Step 1: Write the failing tests**

Add focused assertions proving:

1. Scrapling worker results include explicit `category_targets`.
2. Recent sim run summaries preserve explicit Scrapling category targets.
3. Episode archive rows preserve exact `judged_run_ids`.
4. The Game Loop tab does not attribute a category to a lane unless that run explicitly carries the category.
5. The Game Loop tab uses archived judged run ids instead of lane-plus-time coincidence.

**Step 2: Run tests to verify they fail**

Run:

1. `make test-adversary-sim-scrapling-worker`
2. `make test-oversight-episode-archive`
3. `make test-operator-snapshot-foundation`
4. `make test-dashboard-game-loop-accountability`

Expected: the new assertions fail because the current contracts still drop exact Scrapling category targets and judged run ids, and the dashboard still uses heuristic reconstruction.

## Task 2: Preserve explicit Scrapling category truth through the observer lineage

**Files:**
- Modify: `src/admin/adversary_sim_worker_plan.rs`
- Modify: `scripts/supervisor/scrapling_worker.py`
- Modify: `src/admin/adversary_sim_api.rs`
- Modify: `src/admin/api.rs`
- Modify: `src/observability/hot_read_documents.rs`
- Modify: `src/observability/hot_read_projection.rs`
- Modify: `src/observability/operator_snapshot_live_traffic.rs`

**Step 1: Add the observer-only field to the Scrapling result contract**

Extend `ScraplingWorkerResult` with explicit `category_targets`.

**Step 2: Emit the field from the real worker**

Return the already-validated plan category targets in every Scrapling worker result path.

**Step 3: Persist and project the field**

Store those category targets on Scrapling receipt events and project them into recent sim run summaries, hot-read payloads, and operator snapshot recent-run rows.

**Step 4: Verify focused tests**

Run:

1. `make test-adversary-sim-scrapling-worker`
2. `make test-operator-snapshot-foundation`

Expected: explicit Scrapling category truth now survives the observer path end to end.

## Task 3: Preserve exact judged run ids in the archive

**Files:**
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `src/admin/oversight_api.rs`
- Modify: `dashboard/src/lib/domain/api-client.js`

**Step 1: Add the failing archive test if it is still missing**

Assert that completed episode rows carry exact `judged_run_ids`.

**Step 2: Project the field from required runs**

Use the already-known `required_runs.follow_on_run_id` values when building completed episode records.

**Step 3: Adapt the dashboard archive contract**

Preserve `judged_run_ids` in the dashboard adapter without changing the rest of the archive semantics.

**Step 4: Verify focused tests**

Run:

1. `make test-oversight-episode-archive`
2. `make test-dashboard-game-loop-accountability`

Expected: archived judged episodes now identify the exact runs the Game Loop should use.

## Task 4: Remove Game Loop heuristic stitching

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`
- Modify: `dashboard/src/lib/components/dashboard/monitoring-view-model.js`
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`

**Step 1: Build the selected round from exact judged run ids**

Replace lane-plus-time matching and fallback slices with exact run-id selection from the archived judged episode row.

**Step 2: Build the adversary cast from lane-local category truth only**

Remove round-level simulator-category backfill. When a judged run has no explicit lane-local categories, render a truthful unavailable note instead of guessing.

**Step 3: Tighten fixtures and rendering proof**

Replace invalid lane/category fixtures with canonical mappings and assert that impossible pairings do not render.

**Step 4: Verify focused tests**

Run: `make test-dashboard-game-loop-accountability`

Expected: the rendered Game Loop now shows only exact observer truth and no invented lane/category or round/run associations.

## Task 5: Review the full tab, document, and close the tranche

**Files:**
- Modify: `docs/dashboard-tabs/game-loop.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Review the whole rendered tab**

Inspect the top-of-tab observer sections and the lower machine-first sections together to confirm:

1. no duplicate chrome,
2. no stale heuristic wording,
3. no impossible lane/category pairings,
4. and no simulator-label leakage into defence-native copy.

**Step 2: Refine wording after the first green pass**

Make one deliberate simplification pass so unavailable observer truth reads clearly without clutter.

**Step 3: Update docs and backlog**

Document the exact-observer-truth rule, update the research and plan indexes, and move the completed TODO item into history with proof cited.

**Step 4: Run final focused verification**

Run:

1. `make test-adversary-sim-scrapling-worker`
2. `make test-oversight-episode-archive`
3. `make test-operator-snapshot-foundation`
4. `make test-dashboard-game-loop-accountability`

Expected: all focused observer-truth proof paths are green after the refinement pass.
