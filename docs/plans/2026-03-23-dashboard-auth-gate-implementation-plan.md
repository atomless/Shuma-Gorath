# Dashboard Auth Gate Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove the logged-out flash of the dashboard shell by gating `/dashboard/index.html` behind an auth-pending client route until session restoration completes.

**Architecture:** Keep the first fix local to the dashboard route and route-controller bootstrap flow. Render a neutral auth-pending state by default, prove the bug with a rendered browser test, then reveal the shell only after authenticated session restore succeeds.

**Tech Stack:** SvelteKit static dashboard, shared dashboard runtime modules, Playwright dashboard smoke tests, dashboard source-contract unit tests, Makefile verification targets

---

### Task 1: Capture the scoped auth-gate work item

**Files:**
- Modify: `todos/todo.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`

**Steps:**
1. Add a single scoped TODO for the dashboard auth-gate flash fix.
2. Add this plan and its paired research note to the flat docs indexes.
3. Keep the item scoped to auth-flow truthfulness only.

### Task 2: Add failing proof for the flash

**Files:**
- Modify: `e2e/dashboard.smoke.spec.js`
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `Makefile`

**Steps:**
1. Add a rendered dashboard test that navigates to `/dashboard/index.html` while logged out, delays the `/admin/session` response, and proves the dashboard shell is not visible before redirect.
2. Add a focused source-contract assertion that the route renders an auth-pending gate before the shell.
3. Add a focused `make` target for this auth-gate proof.
4. Run the focused target and verify the new test fails for the current implementation.

### Task 3: Implement the auth-pending gate

**Files:**
- Modify: `dashboard/src/routes/+page.svelte`
- Modify: `dashboard/src/lib/runtime/dashboard-route-controller.js`

**Steps:**
1. Introduce an explicit route-local auth bootstrap state.
2. Keep the dashboard shell hidden until session restoration succeeds.
3. Preserve logout/session-expiry redirect semantics for already-mounted authenticated sessions.
4. Keep the change local; do not alter global session semantics or unrelated polling/auth flows.

### Task 4: Update docs and close out

**Files:**
- Modify: `docs/dashboard.md`
- Add: `docs/research/2026-03-23-dashboard-auth-gate-post-implementation-review.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Steps:**
1. Document the dashboard auth-pending behavior in the dashboard docs.
2. Write a post-implementation review covering the root cause, the chosen fix, and any shortfalls.
3. Move the TODO to completed history with evidence.

### Task 5: Verify and push atomically

**Steps:**
1. Run the focused auth-gate `make` target.
2. Run `git diff --check`.
3. Commit the planning/doc slice separately from the implementation slice when practical.
4. Push each validated atomic commit to `origin/main`.
