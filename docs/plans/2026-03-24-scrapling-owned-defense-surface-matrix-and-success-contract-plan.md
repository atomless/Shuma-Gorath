# Scrapling Owned Defense-Surface Matrix And Success Contract Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Freeze the first explicit owned defense-surface matrix for the Scrapling lane so later request-native expansion, browser or stealth adoption, and receipt-backed proof all work from one truthful contract.

**Architecture:** Keep category ownership and defense-surface ownership separate. Preserve the current request-native Scrapling category contract while adding a second machine-readable matrix that states which defense surfaces Scrapling owns, whether request-native behavior is enough, and what counts as faithful success on each surface.

**Tech Stack:** Planning docs, adversarial coverage contract fixture, coverage-contract validator, focused Python unit coverage, Makefile verification.

---

## Guardrails

1. Do not claim current Scrapling already passes Shuma `not_a_bot`, puzzle, or PoW.
2. Do not silently widen request-native Scrapling to browser-only surfaces.
3. Do not blur category ownership and defense-surface ownership into one matrix.
4. Keep every owned-surface claim grounded in outside-attacker behavior, not library marketing.

## Task 1: Write The Human-Readable Owned-Surface Contract

**Files:**
- Modify: `docs/research/2026-03-24-scrapling-owned-defense-surface-matrix-and-success-contract-review.md`
- Modify: `docs/plans/2026-03-24-scrapling-owned-defense-surface-matrix-and-success-contract-plan.md`
- Modify: `docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`

**Work:**
1. Freeze the first owned request-native surface set:
   - `honeypot`
   - `rate_limit`
   - `geo_ip_policy`
   - `challenge_routing`
   - `not_a_bot`
   - `challenge_puzzle`
   - `proof_of_work`
2. Define success semantics for each surface:
   - `must_touch`
   - `must_fail_or_escalate`
   - later `must_pass_when_publicly_solved` where justified
3. Explicitly exclude browser-only surfaces from this first request-native matrix.

**Acceptance criteria:**
1. The repo has one clear human-readable answer to “what defenses does request-native Scrapling actually own?”
2. The contract distinguishes touching a surface from solving it.

## Task 2: Freeze The Machine-Readable Contract

**Files:**
- Modify: `scripts/tests/adversarial/coverage_contract.v2.json`
- Modify: `scripts/tests/check_adversarial_coverage_contract.py`
- Modify: `scripts/tests/test_adversarial_coverage_contract.py`

**Work:**
1. Add a `scrapling_owned_defense_surfaces` section to the coverage contract fixture.
2. For each surface, record:
   - owner status,
   - required interaction class,
   - expected success contract,
   - request-native vs browser-or-stealth requirement,
   - and notes.
3. Add validator and unit-test coverage so drift fails fast.

**Acceptance criteria:**
1. The owned-surface matrix is machine-readable, not only prose.
2. `make test-adversarial-coverage-contract` proves the contract.

## Task 3: Sync The Planning Chain

**Files:**
- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Work:**
1. Add the new review and plan to the indexes.
2. Thread the owned-surface matrix concept into the adversary roadmap and the main loop-closure plan.
3. Move `SIM-SCR-CHALLENGE-2A` to completed history once the contract and proof are landed.

**Acceptance criteria:**
1. The first owned-surface contract is discoverable across the planning chain.
2. The active backlog can move cleanly to `SIM-SCR-CHALLENGE-2B` afterward.

## Verification

1. `make test-adversarial-coverage-contract`
2. `git diff --check`

## Exit Criteria

This tranche is complete when:

1. Scrapling-owned defense surfaces are frozen in both prose and machine-readable form,
2. each owned surface has an explicit success contract,
3. request-native vs later browser-or-stealth responsibility is explicit,
4. and the active mainline can move into implementation against that frozen matrix.
