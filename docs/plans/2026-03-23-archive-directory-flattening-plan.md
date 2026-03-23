# Archive Directory Flattening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove `docs/research/` and `docs/plans/` entirely by moving their dated documents back into the flat top-level `docs/research/` and `docs/plans/` directories, while preserving the current-vs-historical-vs-outdated distinction through curated indexes and entry docs instead of nested folders.

**Architecture:** Treat dated filenames plus curated README/index sections as the documentation information architecture. Keep the filesystem flat inside `docs/research/` and `docs/plans/`; keep semantic status in the indexes and deferred-edge explainer.

**Tech Stack:** Markdown docs, repo-local indexes, link repair, TODO history

---

### Task 1: Record The Follow-On Review And Execution Boundary

**Files:**
- Create: `docs/research/2026-03-23-archive-directory-flattening-review.md`
- Create: `docs/plans/2026-03-23-archive-directory-flattening-plan.md`

**Steps:**
1. Capture why the first docs cleanup slice is no longer enough.
2. Freeze the scope around flattening archived dated docs, not a broader content rewrite.

### Task 2: Move Archived Dated Docs Back To The Flat Top-Level Layout

**Files:**
- Move every dated Markdown file from `docs/research/**` into `docs/research/`
- Move every dated Markdown file from `docs/plans/**` into `docs/plans/`
- Remove: `docs/research/README.md`
- Remove: `docs/research/archive/outdated/README.md`
- Remove: `docs/plans/README.md`
- Remove: `docs/plans/archive/outdated/README.md`

**Steps:**
1. Move the historical delivered research notes into `docs/research/`.
2. Move the outdated deferred-edge research notes into `docs/research/`.
3. Move the historical delivered plan notes into `docs/plans/`.
4. Move the outdated deferred-edge plan notes into `docs/plans/`.
5. Remove the now-redundant archive README files and empty archive directories.

### Task 3: Rebuild The Information Architecture In The Top-Level Indexes

**Files:**
- Modify: `docs/index.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/deferred-edge-gateway.md`

**Steps:**
1. Remove archive-directory entry points from the main docs index.
2. Add explicit sections in the research and plans indexes for historical baselines and outdated deferred-edge notes.
3. Make the deferred-edge explainer point directly at the flattened dated docs.

### Task 4: Repair The Link Graph

**Files:**
- Modify: every affected doc/history/backlog file that still points at `docs/research/**` or `docs/plans/**`

**Steps:**
1. Update active docs, historical docs, and backlog/history references to the new flat paths.
2. Re-run a repo-local search to confirm no active repository docs still depend on the removed archive directories.

### Task 5: Record Completion And Verify

**Files:**
- Modify: `todos/completed-todo-history.md`

**Steps:**
1. Add a completion record for the flattening tranche.
2. Verify with `git diff --check`.
3. Keep verification docs-only; do not run behavior tests because this slice changes only documentation paths and indexes.
