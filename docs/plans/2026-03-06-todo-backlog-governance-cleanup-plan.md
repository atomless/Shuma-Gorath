# TODO Backlog Governance Cleanup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Simplify backlog management by keeping `todos/todo.md` focused on active work, moving blocked items to `todos/blocked-todo.md`, and relocating durable policy guidance into canonical policy docs.

**Architecture:** Treat the backlog as a three-file system: active work in `todos/todo.md`, blocked or contingent work in `todos/blocked-todo.md`, and completed work in `todos/completed-todo-history.md`. Keep only execution-ready tasks in the active queue, move stale completion stubs out, and store cross-cutting governance rules in `docs/project-principles.md` and `AGENTS.md` rather than in tranche prose.

**Tech Stack:** Markdown documentation, repository governance docs, backlog curation

---

### Task 1: Define the backlog split and policy moves

**Files:**
- Modify: `todos/todo.md`
- Create: `todos/blocked-todo.md`
- Modify: `docs/project-principles.md`
- Modify: `AGENTS.md`

**Step 1: Identify active vs blocked vs completed backlog content**

- Keep active near-term work in `todos/todo.md`.
- Move explicitly gated/contingent work to `todos/blocked-todo.md`.
- Remove completed deployment tranche stubs from `todos/todo.md`.
- Move durable policy statements out of tranche prose where they belong in canonical docs.

**Step 2: Update policy docs for durable governance rules**

- Add explicit backlog-governance wording to `AGENTS.md` so future TODO scans include `todos/blocked-todo.md`.
- Add only the minimum new principle text needed in `docs/project-principles.md`:
  - operator-facing monitoring must reflect real runtime telemetry,
  - runtime start paths should not silently mutate persisted config outside explicit seed/migration flows.

**Step 3: Verify no backlog references break**

- Ensure `todos/todo.md` and `todos/blocked-todo.md` both point clearly to `todos/completed-todo-history.md` and `todos/security-review.md`.
- Ensure any moved tranche IDs remain discoverable by name in one place only.

### Task 2: Rewrite the active backlog for clarity

**Files:**
- Modify: `todos/todo.md`

**Step 1: Remove archive-only material**

- Delete completed stubs for `DEP-GW-1` and `DEP-GW-POST`.
- Remove redundant tranche-local policy prose where a linked plan or canonical doc already governs it.

**Step 2: Fix misleading or conflicting identifiers**

- Rename the active dashboard connection-state tranche so it no longer reuses `SIM2-R4-5`, which is already archived for a different completed tranche.

**Step 3: Reorder active work around the immediate road ahead**

- Keep active:
  - monitoring/config lifecycle stabilization,
  - shared-host deployment readiness,
  - privacy follow-up,
  - shared-host discovery,
  - enterprise deployment/state hardening,
  - remaining hardening/coverage items.
- Move blocked or contingent SIM items out of the active queue.

### Task 3: Create the blocked backlog

**Files:**
- Create: `todos/blocked-todo.md`

**Step 1: Add a concise purpose section**

- Explain that the file is for gated, contingent, or not-yet-approved work.
- Note that items should move back into `todos/todo.md` only when the blocking condition clears.

**Step 2: Move the clearly blocked items**

- Move:
  - `SIM-SCR-LANE-1`,
  - `SIM-BREACH-REPLAY-1`,
  - `SIM-DEPLOY-2`,
  - `SIM-LLM-1`,
  - other explicitly gated or contingent items that are not execution-ready.

**Step 3: Preserve the gating rationale**

- Keep the blocking condition short and specific.
- Prefer one-line references to the relevant plan/research doc instead of copying tranche policy prose.

### Task 4: Sanity-check the edited backlog

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `docs/project-principles.md`
- Modify: `AGENTS.md`

**Step 1: Check for duplicate IDs and completed items still in active backlog**

- Search for duplicate tranche IDs.
- Search for references to completed tranches still listed as active.

**Step 2: Check for obvious duplication with canonical docs**

- Confirm `todos/todo.md` no longer carries long-lived policy text that now exists in canonical docs.

**Step 3: Docs-only verification**

- Because this slice is docs/backlog-only, do not run the full test suite.
- Perform targeted repo searches to confirm the new backlog file is referenced where needed and that the active backlog is internally consistent.
