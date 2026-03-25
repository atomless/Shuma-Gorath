Date: 2026-03-24

Related research:

- [`../research/2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-review.md`](../research/2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-review.md)
- [`2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-plan.md`](2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-plan.md)

# SIM-SCR-CHALLENGE-2B Plan

## Objective

Make the live Scrapling request-native worker behave like a malicious attacker for every request-native surface the owned-surface matrix assigns to the current Scrapling personas.

## Tasks

1. Widen the worker-plan contract.
   - Extend `ScraplingWorkerPlan` with:
     - per-mode `surface_targets`,
     - a bounded `runtime_paths` object for owned request-native public routes.
   - Populate those fields from the Rust control plane at beat generation time.
   - Add plan-contract assertions in Rust tests.

2. Expand the real Python Scrapling personas.
   - Keep the ordinary passing traffic already used for:
     - `crawler`,
     - `bulk_scraper`,
     - `http_agent`.
   - Add attacker-faithful malicious request-native interactions by mode:
     - `bulk_scraper`: `not_a_bot`, puzzle,
     - `http_agent`: `not_a_bot`, puzzle, PoW verify, tarpit progress,
     - `crawler`: light challenge-routing pressure only.
   - Keep all requests inside the shared-host scope contract and free of privileged headers or secrets.

3. Add focused proof.
   - Add a new focused make target for malicious request-native Scrapling behavior.
   - Extend `test_scrapling_worker.py` with route-level assertions proving the new malicious submits happen while ordinary success traffic still exists.
   - Update target-scope truth tests and `docs/testing.md`.

4. Close the paper trail.
   - Update relevant indexes and TODO references.
   - After verification, move `SIM-SCR-CHALLENGE-2B` into `todos/completed-todo-history.md`.
   - Write a post-implementation review capturing whether `2C` is still required.

## Verification

- `make test-adversary-sim-scrapling-malicious-request-native`
- `git diff --check`
