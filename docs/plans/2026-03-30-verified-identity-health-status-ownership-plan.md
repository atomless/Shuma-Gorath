# Verified Identity Health Status Ownership Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move the read-only `Verified Identity Health` summary from `Verification` to the bottom of `Status`, while keeping data ownership, refresh behavior, docs, and proof surfaces aligned.

**Architecture:** Treat `Verified Identity Health` as a read-only operator-health surface. `Verification` keeps only editable controls. `Status` becomes the single home for this summary and owns the `operatorSnapshot` read path needed to render it.

**Tech Stack:** Svelte dashboard, dashboard runtime refresh/store wiring, Playwright smoke tests, node unit/source-contract tests, Makefile-focused verification targets.

---

### Task 1: Write failing ownership tests

**Files:**
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`
- Modify: `Makefile`

**Steps:**
1. Update unit tests so they expect:
   - `StatusTab.svelte` to own the verified-identity health markup,
   - `VerificationTab.svelte` to stop owning that health markup,
   - `refreshStatusTab` to fetch `operatorSnapshot`,
   - `refreshVerificationTab` to stop fetching `operatorSnapshot`.
2. Update smoke tests so they expect:
   - `Verification` to show verified-identity controls only,
   - `Status` to show verified-identity health summary.
3. Rename or add focused `make` targets so target names remain truthful.
4. Run the relevant focused target and confirm it fails for the expected ownership reason.

### Task 2: Move runtime ownership

**Files:**
- Modify: `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
- Modify: `dashboard/src/routes/+page.svelte`

**Steps:**
1. Stop `refreshVerificationTab` from fetching `operatorSnapshot`.
2. Teach `refreshStatusTab` to fetch and apply `operatorSnapshot` alongside its existing read-only status data.
3. Pass `operatorSnapshot` into `StatusTab`.
4. Remove `operatorSnapshot` from `VerificationTab` wiring.

### Task 3: Move rendering ownership

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/VerificationTab.svelte`
- Modify: `dashboard/src/lib/components/dashboard/StatusTab.svelte`
- Create or modify shared helper only if needed for clean reuse.

**Steps:**
1. Remove the `Verified Identity Health` panel and its helper logic from `VerificationTab`.
2. Add the same summary to the bottom of `StatusTab`.
3. If extraction improves cleanliness, move summary parsing/formatting into a shared domain helper and consume it from `StatusTab`.
4. Keep DOM structure shallow and consistent with existing status-tab panel patterns.

### Task 4: Update docs and proof

**Files:**
- Modify: `docs/dashboard-tabs/status.md`
- Modify: `docs/dashboard-tabs/verification.md`
- Modify: `todos/completed-todo-history.md`

**Steps:**
1. Update tab docs so ownership is explicit.
2. Run focused verification:
   - `make test-dashboard-status-pane`
   - `make test-dashboard-verified-identity-pane`
   - `make dashboard-build`
3. Record the completion tranche in `todos/completed-todo-history.md`.
