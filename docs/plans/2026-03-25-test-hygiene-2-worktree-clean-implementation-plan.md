# TEST-HYGIENE-2 Worktree-Clean Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Keep routine adversarial and SIM2 verification worktree-clean by moving generated `make` outputs out of tracked fixture paths and into `.spin/adversarial/`.

**Architecture:** Preserve tracked JSON under `scripts/tests/adversarial/` as read-only contracts, manifests, schemas, and baselines. Route only generated reports and receipts through disposable local state under `.spin/`, and keep the change focused on Makefile wiring, targeted make-target proof, and docs.

**Tech Stack:** `Makefile`, focused Python Makefile wiring tests, `docs/testing.md`, backlog closeout docs.

---

## Task 1: Freeze The Output-Path Contract

**Files:**
- Modify: `docs/testing.md`
- Modify: focused Makefile-wiring tests under `scripts/tests/`

**Work:**
1. Make the intended split explicit:
   - tracked `scripts/tests/adversarial/*.json` files remain contract fixtures or baselines,
   - generated reports for routine verification land under `.spin/adversarial/`.
2. Add or extend a focused test that proves the touched `make` targets write their generated artifacts through the `.spin/adversarial/` path variables.

**Acceptance criteria:**
1. The desired worktree-clean contract is executable rather than prose only.
2. The proof stays narrow and Makefile-wiring-oriented.

## Task 2: Move Generated Make Outputs Under `.spin/adversarial/`

**Files:**
- Modify: `Makefile`

**Work:**
1. Introduce canonical `.spin/adversarial/` output variables for generated adversarial and SIM2 artifacts.
2. Update the writing `make` targets to use those variables instead of tracked paths.
3. Keep fixture/baseline inputs that are meant to stay versioned under `scripts/tests/adversarial/`.
4. Keep override ergonomics intact via existing Make/environment-variable patterns where useful.

**Acceptance criteria:**
1. Routine `make` workflows no longer rewrite tracked generated reports.
2. The tracked JSON that remains under `scripts/tests/adversarial/` is clearly input/baseline material, not routine output.

## Task 3: Verify And Close The Backlog Truthfully

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Modify: `docs/research/README.md`
- Add: post-implementation review in `docs/research/`

**Work:**
1. Run the focused adversarial and SIM2 `make` paths that used to rewrite tracked outputs.
2. Confirm the worktree stays clean apart from the intentional code/doc changes.
3. Record the tranche closeout and update the active queue truthfully.

**Acceptance criteria:**
1. Verification proves the output-path fix rather than only source edits.
2. The backlog and research index reflect the delivered state truthfully.

## Verification

1. `make test-adversarial-preflight`
2. `make test-sim2-realtime-bench`
3. `make test-sim2-ci-diagnostics`
4. `make test-sim2-operational-regressions`
5. `make test-sim2-governance-contract`
6. `git diff --check`
