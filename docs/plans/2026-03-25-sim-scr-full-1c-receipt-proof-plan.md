# Receipt Proof Plan

**Goal:** Make `SIM-SCR-FULL-1C` prove the current full-power Scrapling lane explicitly across recent-run backend aggregation and operator-facing Red Team/Game Loop evidence.

**Architecture:** Reuse the existing receipt and owned-surface coverage contracts rather than inventing a new proof structure. Refresh the stale backend fixture to the browser-backed contract, derive touched/passed/failed counts from the existing receipts in the dashboard view-model, and surface those counts through the existing Red Team and Game Loop information rows.

**Tech Stack:** `src/admin/api.rs`, `dashboard/src/lib/components/dashboard/monitoring-view-model.js`, `dashboard/src/lib/components/dashboard/monitoring/ScraplingEvidencePanel.svelte`, `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`, focused Scrapling and dashboard proof targets, existing dashboard tab docs.

---

## Task 1: Write the failing proof updates

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`

**Work:**
1. Update the focused backend recent-run test to require the current contract:
   - `not_a_bot_submit` as `pass_observed`
   - covered current owned-surface summary still evaluating truthfully
2. Update the dashboard fixtures to require:
   - browser-backed `not_a_bot_submit` pass evidence
   - explicit exercised/passed/failed summary text in Red Team
   - compact passed/failed corroboration in Game Loop

## Task 2: Extend the dashboard proof surface

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/monitoring-view-model.js`
- Modify: `dashboard/src/lib/components/dashboard/monitoring/ScraplingEvidencePanel.svelte`
- Modify: `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`

**Work:**
1. Derive receipt-backed summary counts from the existing owned-surface receipts:
   - exercised surfaces
   - expected passes observed
   - expected fails observed
2. Surface those counts in Red Team using existing status-row patterns.
3. Surface a compact version of the same proof in Game Loop without duplicating the forensic table.

## Task 3: Close the tranche

**Files:**
- Modify: `docs/dashboard-tabs/red-team.md`
- Modify: `docs/dashboard-tabs/game-loop.md`
- Modify: `docs/testing.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-25-sim-scr-full-1c-receipt-proof-post-implementation-review.md`

**Verification:**

```bash
make test-adversary-sim-scrapling-coverage-receipts
make test-dashboard-scrapling-evidence
git diff --check
```

## Definition Of Done

This slice is complete when:

1. the backend recent-run proof matches the current browser-backed Scrapling contract,
2. Red Team explicitly summarizes exercised surfaces and expected pass/fail outcomes from receipts,
3. Game Loop corroborates the same proof compactly,
4. and the focused backend plus dashboard proof targets pass against the updated full-power contract.
