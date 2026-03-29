# Game Loop Observer-Facing Presentation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reframe the Game Loop tab as a simple observer-facing summary of recent rounds, adversaries, and defences using existing machine-first contracts and without changing the loop mechanics.

**Architecture:** Reuse the existing `GameLoopTab.svelte` surface and current operator-snapshot plus oversight payloads. Extend the dashboard adapter only where the backend already materializes fields the page currently drops, then render actor-first sections at the top and demote judge-internal detail lower in the tab.

**Tech Stack:** Svelte dashboard components, `dashboard/src/lib/domain/api-client.js`, `dashboard/src/lib/components/dashboard/monitoring-view-model.js`, focused dashboard unit and smoke tests, Game Loop tab docs, TODO closeout.

---

## Acceptance Criteria

1. The top of the Game Loop tab shows recent judged rounds using existing episode-archive truth, including completion time, participating lanes, retained vs rolled-back outcome, config family moved when available, and continue or stop state.
2. The top half of the Game Loop tab shows an adversary-facing cast that uses simulator-ground-truth and recognition comparison rows rather than category target-achievement meters.
3. The top half of the Game Loop tab shows a defence-facing cast that uses breach loci and surface-contract evidence without using simulator labels as defence truth.
4. The redesign remains presentation-only: no controller, benchmark, or loop-execution behavior changes are introduced.
5. Focused rendered proof exists through `make test-dashboard-game-loop-accountability`.

## Task 1: Lock the observer-facing data contract

**Files:**
- Modify: `dashboard/src/lib/domain/api-client.js`
- Test: `e2e/dashboard.modules.unit.test.js`

**Step 1: Write the failing test**

Add a focused unit assertion proving the adapter preserves the already-materialized observer-facing fields the redesign needs:

1. episode-archive `completed_at_ts`,
2. episode-archive `proposal`,
3. episode-archive `evidence_references`,
4. recognition `simulator_ground_truth`.

**Step 2: Run test to verify it fails**

Run: `make test-dashboard-game-loop-accountability`

Expected: the focused Game Loop dashboard test fails because the adapter does not yet preserve one or more of those fields.

**Step 3: Write minimal implementation**

Extend the adapter only for fields the backend already sends. Do not add new backend contracts.

**Step 4: Run test to verify it passes**

Run: `make test-dashboard-game-loop-accountability`

Expected: the adapter-focused Game Loop proof no longer fails on missing observer-facing fields.

## Task 2: Replace the top-of-tab story with recent rounds plus actor casts

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`
- Test: `e2e/dashboard.modules.unit.test.js`
- Test: `e2e/dashboard.smoke.spec.js`

**Step 1: Write the failing tests**

Add focused rendered expectations proving the Game Loop tab now leads with:

1. a recent-rounds summary,
2. an adversary cast,
3. and a defence cast.

Also prove the recognition area renders actual category comparison outcomes rather than target-achievement-only language.

**Step 2: Run test to verify it fails**

Run: `make test-dashboard-game-loop-accountability`

Expected: the Game Loop rendered proof fails because the current DOM still leads with judge-internal sections and legacy recognition meter language.

**Step 3: Write minimal implementation**

Refactor `GameLoopTab.svelte` so it:

1. builds recent round rows from the episode archive and oversight history,
2. builds adversary rows from recent sim run summaries plus recognition comparison rows,
3. builds defence rows from breach loci and surface-contract receipts,
4. moves those sections to the top,
5. and demotes the judge-internal panels lower without removing truthful machine detail.

**Step 4: Run test to verify it passes**

Run: `make test-dashboard-game-loop-accountability`

Expected: the Game Loop tab now renders the observer-facing story with passing unit and smoke coverage.

## Task 3: Review, refine, and document the observer framing

**Files:**
- Modify: `docs/dashboard-tabs/game-loop.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Review the rendered structure against the acceptance criteria**

Check the whole tab composition, not just the edited subtrees, for:

1. duplicate section chrome,
2. repeated headings,
3. cluttered or judge-internal-first ordering,
4. and any defence rows that accidentally rely on simulator labels.

**Step 2: Refine the presentation**

Make one tightening pass after the first green test run to simplify wording, reduce low-signal detail, and keep the actor framing crisp.

**Step 3: Update docs and backlog**

Document the new Game Loop framing, add the new plan and research entries to their indexes, and move the completed TODO item into `todos/completed-todo-history.md` with the proof cited.

**Step 4: Run the final proof**

Run: `make test-dashboard-game-loop-accountability`

Expected: focused Game Loop dashboard proof is green after the refinement pass.
