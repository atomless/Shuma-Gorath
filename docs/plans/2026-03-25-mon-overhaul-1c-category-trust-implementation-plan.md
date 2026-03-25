# MON-OVERHAUL-1C Category And Trust Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `Game Loop` show the next layer of machine-first accountability truth: explicit numeric budget usage, category target achievement, and trust/actionability readiness.

**Architecture:** Reuse the existing `GameLoopTab.svelte` surface and the already-materialized `benchmark_results_v1` plus `operator_snapshot_v1` payloads. Do not add new backend contracts. Keep the current top line, refine the outcome and pressure sections, and strengthen the trust section from existing snapshot data.

**Tech Stack:** `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`, focused dashboard tests, existing dashboard styles and primitives, backlog/docs closeout.

---

## Task 1: Freeze The Game Loop Projection Contract

**Files:**
- Modify: `docs/dashboard-tabs/game-loop.md`
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`

**Work:**
1. Extend the focused Game Loop proofs to require:
   - explicit numeric budget usage rows for the real objective budgets,
   - explicit category target-achievement rows,
   - richer trust/actionability projection for coverage, protected evidence, and verified-identity guardrails.
2. Keep the current high-level overall top line intact.
3. Keep the proof focused on rendered operator-visible behavior, not source archaeology.

**Acceptance criteria:**
1. The tests fail before the UI is updated.
2. The desired Game Loop contract is executable rather than prose only.

## Task 2: Implement Numeric Budget Usage And Category Target Achievement

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`
- Modify: `dashboard/style.css`

**Work:**
1. Keep the existing top-level status cards.
2. Refine `Outcome Frontier` so true numeric budgets read as target-vs-current budget usage with a stronger visual signal than wording-only status.
3. Add category target-achievement rows derived from the category posture benchmark family and current objective posture targets.
4. Keep the visual language inside existing dashboard patterns and tokens; if a new shared style is required, extend the canonical stylesheet narrowly rather than inventing one-off inline styling.

**Acceptance criteria:**
1. Numeric budgets are readable without relying on `inside_budget` text alone.
2. Category posture is shown as target achievement, not fake configured budgets.
3. The tab remains recognizably within the existing dashboard design system.

## Task 3: Strengthen Trust And Actionability

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`
- Modify: `docs/dashboard-tabs/game-loop.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Modify: `docs/research/README.md`
- Add: post-implementation review in `docs/research/`

**Work:**
1. Expand the trust section to project:
   - classification readiness,
   - coverage state,
   - protected replay state,
   - tuning eligibility,
   - verified-identity guardrail state where available.
2. Keep the trust surface bounded and accountability-oriented.
3. Close the backlog slice truthfully and promote the next queue item.

**Acceptance criteria:**
1. The tab makes it clearer why the loop is or is not trustworthy enough to act.
2. The backlog and docs reflect the delivered state truthfully.

## Verification

1. `make test-dashboard-game-loop-accountability`
2. `git diff --check`
