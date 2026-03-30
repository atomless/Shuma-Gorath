# Scrapling 0.4.3 Upgrade Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade the repo-owned Scrapling worker runtime from `0.4.2` to `0.4.3` and preserve truthful proof of the pinned runtime contract before the realism chain starts.

**Architecture:** Keep this tranche intentionally narrow. Change only the runtime pin, the exact readiness check, the focused proof surface that guards pin/readiness drift, and the planning or backlog truth needed to capture upstream realism-relevant features. Do not broaden the worker behavior or new control surfaces in the same slice.

**Tech Stack:** Bash runtime bootstrap helpers, Python bootstrap tests, Makefile verification, docs and backlog metadata.

---

## Task 1: Record the upstream delta and the non-goals

**Files:**
- Create: `docs/research/2026-03-30-scrapling-0-4-3-upgrade-and-realism-impact-review.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`

**Work:**
1. Record the current pin location and the latest upstream release truth.
2. Capture the relevant upstream `v0.4.2..v0.4.3` changes.
3. State explicitly that browser-path support and background-XHR receipt work are realism follow-ons, not part of the dependency bump.

**Acceptance criteria:**
1. The repo contains a dated research note citing the upstream release and the realism-relevant changes.
2. The active backlog truth makes it explicit that the upgrade lands ahead of `SIM-REALISM-1A`, but the new upstream capabilities are not silently adopted in this tranche.

## Task 2: Upgrade the pinned Scrapling runtime contract

**Files:**
- Modify: `scripts/bootstrap/scrapling_runtime.sh`

**Work:**
1. Change the canonical Scrapling worker runtime pin from `0.4.2` to `0.4.3`.
2. Keep the readiness gate exact: the installed package version must still match the pinned version.

**Acceptance criteria:**
1. One canonical version constant defines the worker runtime pin.
2. The readiness check fails closed when the installed version does not match the canonical pin.

## Task 3: Add a focused proof against pin/readiness drift

**Files:**
- Modify: `scripts/tests/test_setup_runtime_spin_install.py`

**Work:**
1. Add a focused test that proves the bootstrap helper's canonical version pin and readiness check stay synchronized.

**Acceptance criteria:**
1. The bootstrap proof surface fails if the runtime pin changes without the readiness guard changing with it.
2. The proof remains inside the existing `test-setup-runtime-bootstrap` path rather than inventing a second disconnected target.

**Proof:**
1. `make test-setup-runtime-bootstrap`

## Task 4: Reprovision locally and prove the real worker still runs

**Files:**
- Modify only as needed if proof exposes drift

**Work:**
1. Re-run the canonical runtime bootstrap so `.venv-scrapling` installs the upgraded Scrapling package.
2. Run the smallest truthful Scrapling worker proofs that cover both bootstrap and real worker behavior.

**Acceptance criteria:**
1. The repo-owned Scrapling runtime is locally upgraded to `0.4.3`.
2. The real worker proof still passes against the upgraded runtime.

**Proof:**
1. `make setup-runtime`
2. `make test-setup-runtime-bootstrap`
3. `make test-adversary-sim-scrapling-worker`
4. `make test-adversary-sim-scrapling-browser-capability`

## Task 5: Close out the backlog and completion record

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Work:**
1. Remove the active upgrade TODO after completion.
2. Add a completion record that distinguishes the dependency bump from later realism behavior work.

**Acceptance criteria:**
1. The active TODO queue no longer carries the upgrade after the work is verified.
2. The completion history records the shipped pin bump, proof, and deferred realism implications.
