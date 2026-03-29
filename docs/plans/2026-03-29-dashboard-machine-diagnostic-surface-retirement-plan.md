# Dashboard Machine-Diagnostic Surface Retirement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove the now-redundant machine-diagnostic dashboard sections from `Red Team`, `Game Loop`, and `Diagnostics`, then simplify the remaining dashboard read paths, helpers, docs, and tests so the surviving UI matches the intended operator-facing architecture cleanly.

**Architecture:** Keep the internal machine loop, telemetry, benchmark, and oversight contracts intact while retiring only the dashboard surfaces that no longer earn their keep. The surviving product shape becomes: `Red Team` for adversary-sim control plus recent runs, `Game Loop` for observer-facing recent rounds plus adversary and defence casts, and `Diagnostics` for deeper subsystem inspection without the top furniture rollup.

**Tech Stack:** Svelte dashboard components, dashboard runtime refresh/store wiring, dashboard API-client normalization, focused dashboard unit and Playwright tests, Makefile-focused proof targets, dashboard tab docs, TODO/history bookkeeping.

---

## Acceptance Criteria

1. `Red Team` no longer renders:
   - `Lane State`
   - `Lane Diagnostics`
   - `Status Truth`
   - `Judged Episode Basis`
   - `Scrapling`
2. `Game Loop` no longer renders:
   - `Round Outcome`
   - `Loop Progress`
   - `Origin Leakage And Human Cost`
   - `Loop Actionability`
   - `Pressure Context`
   - `Trust And Blockers`
3. `Diagnostics` no longer renders `Defense Breakdown`.
4. Dashboard read paths are simplified truthfully:
   - `Game Loop` no longer fetches or stores `benchmarkResults` if the surviving surfaces do not use it.
   - `Red Team` no longer receives oversight snapshots if the surviving surfaces do not use them.
5. Dead dashboard-only code tied exclusively to the removed sections is deleted, including unused helpers, components, imports, and state wiring.
6. Tests and focused `make` targets no longer assert deleted surfaces and still prove the surviving contracts.
7. Docs for the three tabs describe only the surviving architecture.

## Task 1: Freeze the retirement contract in failing dashboard tests

**Files:**
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`
- Modify: `Makefile`

**Step 1: Write the failing tests**

Add or update focused assertions so they require the retired section titles and selectors to be absent, and still require the surviving section contract to be present.

**Step 2: Run the focused tests to verify RED**

Run:

1. `make test-dashboard-game-loop-accountability`
2. `make test-dashboard-red-team-truth-basis`
3. `make test-dashboard-diagnostics-pane`

Expected: the tests fail because the current UI still renders the retiring sections and still proves their old selectors and text.

## Task 2: Retire the Red Team machine-diagnostic surfaces

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
- Modify: `dashboard/src/routes/+page.svelte`

**Step 1: Remove the retiring Red Team sections**

Delete the `Lane State`, `Lane Diagnostics`, `Status Truth`, `Judged Episode Basis`, and `Scrapling` surfaces while preserving:

1. adversary-sim controls,
2. lifecycle copy,
3. progress bar,
4. and `Recent Red Team Runs`.

**Step 2: Remove now-dead Red Team props and derivations**

Delete any imports, reactive state, and props that existed only for the retired sections, including oversight inputs if the remaining UI no longer uses them.

**Step 3: Verify focused Red Team proof**

Run: `make test-dashboard-red-team-truth-basis`

Expected: the Red Team proof now verifies the leaner surviving surface rather than the retired diagnostic panels.

## Task 3: Retire the lower Game Loop machine-diagnostic sections and simplify reads

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`
- Modify: `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
- Modify: `dashboard/src/routes/+page.svelte`
- Modify: `dashboard/src/lib/domain/dashboard-state.js`
- Modify: `dashboard/src/lib/domain/api-client.js` only if adaptation or state expectations become dead

**Step 1: Remove the retiring Game Loop sections**

Keep only the top observer-facing sections and any minimal surrounding framing they still need.

**Step 2: Remove dead benchmark-driven derivations**

Delete imports, helper calls, reactive state, and renderer branches that only powered the retired lower sections.

**Step 3: Simplify the Game Loop refresh path**

If no surviving surface still needs `benchmarkResults`, stop fetching and storing it on Game Loop refresh and remove the route prop wiring accordingly.

**Step 4: Verify focused Game Loop proof**

Run: `make test-dashboard-game-loop-accountability`

Expected: the focused Game Loop proof passes with the observer-facing surface only, and no benchmark-driven dead path remains in the dashboard slice.

## Task 4: Retire Diagnostics `Defense Breakdown` and remove dead overview helpers

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/DiagnosticsTab.svelte`
- Modify: `dashboard/src/lib/components/dashboard/monitoring-view-model.js`
- Delete if unused: `dashboard/src/lib/components/dashboard/monitoring/DefenseTrendBlocks.svelte`

**Step 1: Remove the Diagnostics section**

Delete the `Defense Breakdown` surface and its render-site wiring.

**Step 2: Remove dead dashboard-only helper code**

Delete `deriveDefenseBreakdownRows` and any supporting helper code if nothing else uses it.

**Step 3: Verify focused Diagnostics proof**

Run: `make test-dashboard-diagnostics-pane`

Expected: the focused Diagnostics proof now covers the surviving diagnostics ownership without `Defense Breakdown`.

## Task 5: Clean docs, TODOs, and completion history after the architectural review

**Files:**
- Modify: `docs/dashboard-tabs/red-team.md`
- Modify: `docs/dashboard-tabs/game-loop.md`
- Modify: `docs/dashboard-tabs/diagnostics.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Update tab docs**

Describe only the surviving tab responsibilities and remove any prose that still advertises the retired sections.

**Step 2: Update planning indexes and TODOs**

Record the new research/plan docs and move the completed execution item into history when the tranche is done.

**Step 3: Final review and verification**

Run:

1. `make test-dashboard-game-loop-accountability`
2. `make test-dashboard-red-team-truth-basis`
3. `make test-dashboard-diagnostics-pane`
4. `make dashboard-build`

Expected: the leaner dashboard shape is proven, dead code is gone, docs match the surviving architecture, and the final DOM review shows no leftover empty section shells or orphaned machine-diagnostic scaffolding.
