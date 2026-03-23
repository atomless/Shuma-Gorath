# README Entrypoint Repair Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bring the top-level `README.md` Documentation section into parity with the reorganized docs tree so the repository front door reflects the current shared-host-first mainline rather than stale deferred-edge emphasis.

**Architecture:** Treat the root `README.md` as the repo’s highest-traffic navigation surface. It should summarize the current docs topology, not compete with or contradict `docs/README.md`.

**Tech Stack:** Markdown docs, repo-local indexes, TODO history

---

### Task 1: Record The README Audit Addendum

**Files:**
- Create: `docs/research/2026-03-23-readme-entrypoint-audit-addendum.md`
- Create: `docs/plans/2026-03-23-readme-entrypoint-repair-plan.md`

**Steps:**
1. Record the fact that the earlier docs reorganization missed `README.md`.
2. Freeze scope around the Documentation section rather than opening a larger README rewrite.

### Task 2: Rewrite The Documentation Section As A Current Front Door

**Files:**
- Modify: `README.md`

**Steps:**
1. Add a short “Start here” group that points to `docs/README.md`, `docs/current-system-architecture.md`, and the current shared-host-first operating docs.
2. Add a “Current mainline” group for the shared-host-first architecture and feedback-loop proof chain.
3. Keep operator/product references and contributor workflow links, but remove stale emphasis on one-off historical edge proof notes.
4. Keep the deferred edge skills and docs available only under an explicitly deferred label.

### Task 3: Record Completion And Verify

**Files:**
- Modify: `todos/completed-todo-history.md`

**Steps:**
1. Add a completion record for the README front-door repair.
2. Verify with `git diff --check`.
3. Keep verification docs-only; do not run behavior tests because this slice changes only documentation.
