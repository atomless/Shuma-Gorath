# Dashboard Scrapling Evidence Surfacing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Surface receipt-backed Scrapling adversarial evidence in the dashboard so operators can verify which personas, categories, and owned defense surfaces were exercised, while keeping detailed attack proof in `Red Team` and only a compact corroboration in `Game Loop`.

**Architecture:** Reuse the existing recent sim-run summaries and owned-surface coverage contracts already emitted by the backend. Extend the dashboard view-model and Red Team run-history projection to preserve those fields, add a dedicated Red Team evidence section for the latest Scrapling run, and add only a lightweight trust/readiness corroboration to Game Loop. Keep the slice strictly within existing dashboard primitives and layout patterns.

**Tech Stack:** Svelte dashboard components, dashboard view-model helpers, admin API hot-read payloads, focused dashboard unit + Playwright checks, Makefile verification targets.

---

## Guardrails

1. Do not invent new visual language, one-off borders, or bespoke card idioms.
2. Do not turn `Game Loop` into a second Red Team or diagnostics tab.
3. Do not claim Shuma proves more than the backend receipts actually show.
4. Preserve existing `Red Team` lifecycle and truth-basis ownership; add attack evidence alongside it rather than replacing it.
5. Keep DOM ownership clean: one section owner per real section, no needless nested section shells.

## Task 1: Manifest The Tranche

**Files:**
- Modify: `todos/todo.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`

**Work:**
1. Add an active TODO for the dashboard Scrapling evidence tranche.
2. Add this review and plan to the indexes.

**Acceptance criteria:**
1. The gap and its implementation path are discoverable from the planning chain.
2. The TODO backlog explicitly names the operator-facing Scrapling proof follow-on.

## Task 2: Write The Failing Dashboard Proof

**Files:**
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`
- Modify: `Makefile`

**Work:**
1. Add failing unit coverage that proves recent sim-run summary shaping preserves:
   - observed fulfillment modes
   - observed category ids
   - owned-surface coverage summary
2. Add failing rendered proof that `Red Team` shows:
   - Scrapling modes
   - observed categories
   - owned-surface coverage state
   - at least one per-surface receipt row
3. Add failing rendered proof that `Game Loop` shows a compact latest Scrapling evidence corroboration.
4. Expose a focused Make target for the tranche if the current targets are too broad or do not truthfully describe the new proof scope.

**Acceptance criteria:**
1. The tests fail before implementation for the missing projection.
2. The verification path is focused on Scrapling dashboard evidence, not broad dashboard churn.

## Task 3: Extend The Dashboard Projection

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/monitoring-view-model.js`
- Modify: `dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte`
- Modify: `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
- Modify: `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`
- Optionally create: `dashboard/src/lib/components/dashboard/monitoring/ScraplingEvidencePanel.svelte`
- Optionally modify: `dashboard/src/lib/domain/api-client.js`

**Work:**
1. Preserve Scrapling evidence fields in the Red Team run-row shaping path.
2. Enrich the compact run table with operator-useful proof:
   - fulfillment modes
   - observed categories
   - coverage summary
3. Add a dedicated Red Team evidence section for the latest Scrapling run with:
   - run id
   - modes
   - categories
   - coverage status
   - required, satisfied, and blocking counts
   - per-surface receipt rows including sample method, path, status, and contract satisfaction
4. Add a compact Game Loop corroboration row inside existing trust/readiness patterns.

**Acceptance criteria:**
1. Red Team becomes the clear primary dashboard surface for Scrapling attack evidence.
2. Game Loop only corroborates attacker-evidence readiness rather than duplicating Red Team.
3. The new UI reuses existing section, table, metric-list, and status-row patterns.

## Task 4: Update Docs And Close The Tranche

**Files:**
- Modify: `docs/dashboard-tabs/red-team.md`
- Modify: `docs/dashboard-tabs/game-loop.md`
- Modify: `docs/testing.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-25-dashboard-scrapling-evidence-surfacing-post-implementation-review.md`

**Work:**
1. Update tab docs so their ownership is explicit:
   - Red Team owns detailed Scrapling attack evidence
   - Game Loop owns only compact corroboration
2. Document the focused verification path.
3. Record the completed tranche and evidence.

**Acceptance criteria:**
1. The docs match the rendered contract.
2. The completion trail explains why this slice was needed and what it now proves.
