# GitHub Actions Node24 Major Refresh Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refresh GitHub Actions workflow dependencies off the older Node 20-backed majors with the smallest truthful change set and a static contract lane that prevents regression.

**Architecture:** Keep this as a workflow-and-test-contract slice. Update existing workflow pins only; do not redesign workflow topology or triggers.

**Tech Stack:** GitHub Actions workflow YAML, Python unittest source-contract check, Makefile target, testing docs, backlog closeout, and post-implementation review.

---

## Guardrails

1. Do not widen this into a broader CI redesign.
2. Prefer the first Node 24-safe majors over later majors unless a later major is required.
3. Add a focused local contract lane so version drift becomes testable.
4. Report hosted-runner proof truthfully; do not claim a GitHub-hosted run happened if it did not.

## Task 1: Add the static workflow-version contract lane

**Files:**
- Add: `scripts/tests/test_github_workflow_node24_majors.py`
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add a focused unittest that scans `.github/workflows/*.yml`.
2. Assert the repo no longer uses:
   - `actions/checkout@v4`
   - `actions/setup-node@v4`
   - `actions/upload-artifact@v4`
3. Assert the repo now uses:
   - `actions/checkout@v5`
   - `actions/setup-node@v5`
   - `actions/upload-artifact@v6`
4. Add a Make target for this contract lane and document it.

**Acceptance criteria:**
1. Future drift back to the deprecated majors fails locally.
2. Contributors have one narrow `make` proof path for this workflow contract.

## Task 2: Update all workflow pins consistently

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/adversarial-soak.yml`
- Modify: `.github/workflows/dashboard-e2e.yml`
- Modify: `.github/workflows/release-gate.yml`
- Modify: `.github/workflows/coverage.yml`
- Modify: `.github/workflows/codeql.yml`

**Work:**
1. Update checkout uses to `actions/checkout@v5`.
2. Update setup-node uses to `actions/setup-node@v5`.
3. Update upload-artifact uses to `actions/upload-artifact@v6`.
4. Keep the rest of the workflow behavior unchanged.

**Acceptance criteria:**
1. Every workflow pin is consistent.
2. The workflow refresh is limited to the dependency version change itself.

## Task 3: Close the tranche and paper trail

**Files:**
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-24-ci-wf-1-node24-major-refresh-post-implementation-review.md`

**Work:**
1. Add the new review and plan to the indexes.
2. Move `CI-WF-1` to completed history.
3. Record the remaining hosted-runner proof state truthfully if branch-triggered runs are not available.

**Acceptance criteria:**
1. The backlog and docs indexes show the tranche as delivered.
2. The post-implementation note is explicit about local vs hosted proof.

## Verification

1. `make test-github-workflow-node24-majors`
2. `git diff --check`

## Exit Criteria

This tranche is complete when:

1. all repo workflows are off the older Node 20-backed majors,
2. the static contract lane enforces the new pins,
3. docs and backlog are updated,
4. and hosted CI status is reported honestly.
