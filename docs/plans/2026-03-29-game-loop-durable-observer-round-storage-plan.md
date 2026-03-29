# Game Loop Durable Observer Round Storage Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the Game Loop top-of-tab round, adversary, and defence casts stay exact and durable after judged runs age out of the bounded recent sim-run hot read.

**Architecture:** Add a separate compact observer-round archive keyed by `episode_id`, populate it once when a judged round is recorded, expose it through `/admin/oversight/history`, and keep `operator_snapshot` on its current compact machine-first path. Completed-round Game Loop casts will read from the durable archive; only current in-flight evidence remains tied to live recent-run state.

**Tech Stack:** Rust admin/observability hot-read and oversight stores, dashboard API adapters, Svelte Game Loop rendering, Make-based focused proof targets.

---

## Design Constraints

- The top Game Loop sections must never invent or stitch round truth from unrelated recent runs.
- Simulator labels remain observer-only and must not enter runtime defence or tuning truth.
- The fix must be storage-efficient:
  - do not expand the hot operator snapshot with repeated per-round cast payloads,
  - do not rely on broader recent-run caps as the fix,
  - do not replay raw event logs on every dashboard refresh.
- Missing observer data must be recorded explicitly as missing, not guessed.

## Acceptance Criteria

1. A completed judged round keeps its exact adversary cast even when one or more `judged_run_ids` are no longer present in `operator_snapshot.adversary_sim.recent_runs`.
2. A completed judged round keeps its exact Scrapling defence cast even when the Scrapling run has aged out of the recent-run buffer.
3. The durable observer storage records only compact round-observer fields needed by the Game Loop top sections; it must not duplicate full benchmark scorecards or unrelated LLM runtime detail.
4. `operator_snapshot` remains on the compact hot-read path and is not expanded with full per-round observer cast payloads.
5. `/admin/oversight/history` returns the durable observer-round archive in a form the dashboard can use without any lane/time heuristics or per-refresh event-log replay.
6. When an exact judged run summary is unavailable at archive-write time, the stored round must say so explicitly via basis/missing-run fields rather than synthesizing category or surface claims.

## Proof Surface

- Persistence proof:
  - dedicated observer-round archive store contains exact rows for the judged episode.
- API proof:
  - `/admin/oversight/history` returns both the judged episode archive and the matching observer-round rows.
- Dashboard proof:
  - the Game Loop top sections render the archived round cast from the durable observer archive even when the corresponding `recent_runs` entries are absent.
- Guardrail proof:
  - `operator_snapshot` contract tests confirm that the new observer storage did not move the heavy cast payload into the hot snapshot path.

## Verification Path

- `make test-admin-machine-contracts`
- `make test-dashboard-game-loop-accountability`
- `make test`

If the current Makefile does not expose a focused proof for the new durable observer archive path, add a truthfully named focused target first and use that target in addition to the commands above.

## Task 1: Add A Dedicated Durable Observer-Round Archive

**Files:**

- Create: `src/admin/oversight_observer_round_archive.rs`
- Modify: `src/admin/mod.rs`
- Modify: `src/admin/oversight_api.rs`

**Implementation outline:**

1. Introduce a bounded archive state keyed by site id and episode id.
2. Keep its row limit aligned to the judged episode archive limit unless there is a measured reason to differ.
3. Define compact stored structs:
   - episode-level record,
   - run-level observer summary,
   - Scrapling surface-row summary.
4. Keep fields to exact ids, counts, and receipt-backed values needed by the Game Loop top sections.
5. Do not embed full `MonitoringRecentSimRunSummary`, full `ScraplingOwnedSurfaceCoverageSummary`, or unrelated benchmark/controller state in this archive.

## Task 2: Materialize The Archive At Episode Completion

**Files:**

- Modify: `src/admin/oversight_api.rs`
- Modify: `src/admin/api.rs`
- Modify: `src/admin/adversary_sim_worker_plan.rs` only if helper reuse is needed

**Implementation outline:**

1. At the point where a completed judged episode record is created, load exact summaries for that episode’s `judged_run_ids`.
2. Build compact observer rows from those run summaries:
   - all judged runs contribute run-level observer rows,
   - Scrapling runs contribute compact defence-surface rows derived from the stored owned-surface coverage receipts.
3. Persist a basis marker:
   - `exact_judged_run_receipts` when every judged run is found,
   - `partial_missing_run_receipts` plus `missing_run_ids` when any run is absent.
4. Never guess categories or surfaces for a missing run id.

## Task 3: Expose The Durable Archive Through The Game Loop Read Path

**Files:**

- Modify: `src/admin/oversight_api.rs`
- Modify: `dashboard/src/lib/domain/api-client.js`
- Modify: `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`

**Implementation outline:**

1. Extend `/admin/oversight/history` with an additive `observer_round_archive` payload.
2. Keep `operator_snapshot` unchanged for this purpose.
3. Update the dashboard adapter to normalize the new archive.
4. Change the Game Loop top sections so completed-round casts come from the durable observer archive matched by `episode_id`, not by rehydrating `judged_run_ids` from current `recent_runs`.
5. Retain the current exact current-evidence path separately for in-flight candidate/continuation state if still needed.

## Task 4: Add Focused Regression Proof

**Files:**

- Modify: `src/admin/api.rs`
- Modify: `src/admin/oversight_api.rs`
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`
- Modify: `Makefile` if a focused target is missing

**Implementation outline:**

1. Add an API-level test that creates:
   - a judged episode,
   - a missing aged-out recent-run buffer entry for the Scrapling run,
   - and verifies that `/admin/oversight/history` still returns the durable observer round summary.
2. Add a dashboard proof where:
   - the latest judged round’s Scrapling run id is absent from `operator_snapshot.adversary_sim.recent_runs`,
   - but the Game Loop still renders the archived adversary and defence cast correctly from `oversightHistory`.
3. Keep one negative proof showing that if the archive explicitly marks a run missing, the UI says it is unavailable rather than guessing.

## Task 5: Document The New Observer Contract

**Files:**

- Modify: `docs/dashboard-tabs/game-loop.md`
- Modify: `docs/adversarial-operator-guide.md` only if operator-facing wording needs the clarified storage boundary
- Modify: `todos/completed-todo-history.md`

**Implementation outline:**

1. Document that completed-round casts come from a durable judged-round observer archive.
2. Document that `recent_runs` remains a transient live window and is not the durable source of completed-round history.
3. Keep the no-fabrication policy explicit.

## Recommended Execution Order

1. Add the archive module and compact structs.
2. Materialize compact observer rows at episode completion.
3. Expose the archive through `/admin/oversight/history`.
4. Switch the Game Loop top sections to the durable archive.
5. Add focused API and dashboard regression coverage.
6. Run focused Make proofs, then `make test`.

## Explicitly Rejected Shortcuts

- Raising `HOT_READ_RECENT_SIM_RUNS_MAX_RECORDS` as the primary fix.
- Querying or replaying raw event logs on every Game Loop refresh.
- Copying full observer cast payloads into the hot `operator_snapshot.episode_archive`.
- Reintroducing lane/time stitching or round-level category backfills.
