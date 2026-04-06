# Scrapling 0.4.4 Upgrade And Robots Policy Pin Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade the repo-owned Scrapling worker runtime from `0.4.3` to `0.4.4`, explicitly pin adversary Spider policy as `robots_txt_obey = False`, and preserve attacker-faithful behavior with truthful proof.

**Architecture:** Keep this tranche narrow and additive. Adopt upstream stability and dependency updates that strengthen runtime correctness, but do not silently relax attacker pressure or broaden behavior beyond the current bounded adversary contracts.

**Tech Stack:** Bash runtime bootstrap helpers, Python Scrapling worker runtime, Python bootstrap tests, adversary worker tests, Makefile verification, docs/backlog metadata.

---

## Task 1: Capture upstream `0.4.4` deltas and Shuma adoption stance

**Files:**
- Create: `docs/research/2026-04-06-scrapling-0-4-4-upgrade-and-robots-policy-review.md`
- Modify: `docs/research/README.md`
- Modify: `todos/todo.md`

**Work:**
1. Record latest upstream release truth (`v0.4.4`, release date, authoritative source links).
2. Capture meaningful upstream deltas vs `0.4.3`:
   - Spider `robots_txt_obey` support and `robots_disallowed_count`,
   - ProxyRotator/browser page-pool leak fixes,
   - dependency changes under `fetchers`.
3. Freeze explicit adoption policy under attacker-faithfulness:
   - adopt patch/runtime stability updates,
   - keep adversary crawler robots obedience disabled (`robots_txt_obey = False`),
   - do not conflate polite-crawler semantics with attacker simulation.

**Acceptance criteria:**
1. A dated research note exists with authoritative upstream evidence and a concrete adoption decision.
2. The note explicitly documents why adversary crawler robots compliance remains disabled.

## Task 2: Upgrade the canonical runtime pin and fail-closed readiness contract

**Files:**
- Modify: `scripts/bootstrap/scrapling_runtime.sh`

**Work:**
1. Bump canonical runtime pin from `0.4.3` to `0.4.4`.
2. Keep readiness fail-closed on exact installed version equality.
3. Preserve runtime dependency readiness checks relevant to active worker sessions.

**Acceptance criteria:**
1. One canonical version constant defines the `0.4.4` pin.
2. Readiness still fails closed when installed version diverges from the pin.

## Task 3: Explicitly pin adversary spider robots policy to non-obedient mode

**Files:**
- Modify: `scripts/supervisor/scrapling_worker.py`

**Work:**
1. In the `ShumaScraplingSpider` class, set `robots_txt_obey = False` explicitly.
2. Add a succinct inline comment explaining this is an attacker-faithfulness policy pin (not default reliance).

**Acceptance criteria:**
1. The adversary spider class explicitly encodes robots non-obedience.
2. The policy remains stable if upstream default changes in future versions.

## Task 4: Add focused proof for version pin coherence and robots policy pin

**Files:**
- Modify: `scripts/tests/test_setup_runtime_spin_install.py`
- Modify: `scripts/tests/test_scrapling_worker.py`
- Modify: `Makefile` only if a focused existing path is missing

**Work:**
1. Extend runtime bootstrap test coverage so pin + readiness version equality stays synchronized for `0.4.4`.
2. Add a focused worker contract test proving the crawler spider keeps `robots_txt_obey = False`.
3. Keep proofs on existing canonical Make paths where possible.

**Acceptance criteria:**
1. Tests fail if runtime pin/readiness drift.
2. Tests fail if adversary spider robots policy pin is removed or changed.

**Proof:**
1. `make test-setup-runtime-bootstrap`
2. `make test-adversary-sim-scrapling-worker`

## Task 5: Verify runtime behavior and close backlog truthfully

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Modify docs only as needed for operator/runtime guidance

**Work:**
1. Re-run runtime bootstrap and targeted worker proofs against `0.4.4`.
2. Confirm no regression in request-native and browser persona execution paths.
3. On completion, move the TODO to completed history with exact proof evidence.

**Acceptance criteria:**
1. Local runtime installs and verifies `scrapling==0.4.4`.
2. Adversary worker proofs pass with robots policy pin retained.
3. Backlog reflects completion without overstating behavior changes.

**Proof:**
1. `make setup-runtime`
2. `make test-setup-runtime-bootstrap`
3. `make test-adversary-sim-scrapling-worker`
4. `make test-adversary-sim-scrapling-browser-capability`
5. `make test-code-quality`
