Date: 2026-03-25
Status: Proposed

Related context:

- [`2026-03-24-mainline-resequence-scrapling-before-game-loop-review.md`](2026-03-24-mainline-resequence-scrapling-before-game-loop-review.md)
- [`2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-post-implementation-review.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`](../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# SIM-LLM-1A Black-Box Contract Readiness Review

## Question

What is the next truthful non-dashboard mainline slice after:

1. attacker-faithful Scrapling request-native coverage is landed,
2. the first working self-improving loop is landed,
3. and the active verification path is cheap to run?

## Conclusion

The next backend mainline slice should be `SIM-LLM-1A`.

That slice should not jump straight to a live frontier-backed attacker runtime.
It should first make the later LLM attacker's black-box boundary a real repo contract in the existing adversarial contract surfaces and fulfillment-plan payloads.

## Why `SIM-LLM-1A` is now ready

The earlier blocker chain was correct when first written, but it is now stale at the umbrella level.

The important prerequisites are already landed:

1. bounded LLM fulfillment modes exist through `SIM-LLM-FIT-1`,
2. category-to-lane fulfillment and coverage contracts exist through `SIM-FULFILL-1` and `SIM-COVER-1`,
3. protected-evidence eligibility is landed through `SIM-PROTECTED-1`,
4. the first closed config loop is live-proven through `OVR-APPLY-1`,
5. the canonical role, protocol, and held-out-evaluation contracts are landed through `RSI-ROLES-1`, `RSI-PROTO-1`, and `RSI-EVAL-1`,
6. attacker-faithful Scrapling coverage for the currently owned request-native surfaces is now receipt-backed through `SIM-SCR-CHALLENGE-2A`, `SIM-SCR-CHALLENGE-2B`, and `SIM-SCR-CHALLENGE-2D`,
7. and `SIM-SCR-CHALLENGE-2C` remains correctly blocked because the current owned-surface matrix does not yet justify browser or stealth Scrapling.

So the old blocker "`wait for SIM-SCR-CHALLENGE-1`" is no longer the truthful next-step description.

## What remains missing

The missing piece is no longer attacker-faithful Scrapling for current owned surfaces.

The missing piece is that the later LLM attacker's black-box boundary still lives mostly in prose:

1. host-root-only entry,
2. public-host-derived hints only,
3. Shuma-blind knowledge boundary,
4. no repo/docs/web-search leakage path,
5. no judge visibility,
6. and explicit receipt requirements.

The repo already has the right implementation seam for that contract:

1. [`scripts/tests/adversarial/frontier_action_contract.v1.json`](../../scripts/tests/adversarial/frontier_action_contract.v1.json)
2. [`scripts/tests/adversarial/container_runtime_profile.v1.json`](../../scripts/tests/adversarial/container_runtime_profile.v1.json)
3. [`scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py)
4. [`src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs)

That means the next slice can be small and real:

1. extend the existing contract files rather than inventing a new ad hoc surface,
2. validate the black-box boundary in the Python loader,
3. carry it into the Rust and Python fulfillment-plan payloads,
4. and prove it with focused tests.

## Decision

Make `SIM-LLM-1A` the next active backend slice.

Specifically:

1. move `SIM-LLM-1A` into the active queue,
2. retire the stale `SIM-SCR-CHALLENGE-1` umbrella as a current blocker for this slice,
3. keep `SIM-SCR-CHALLENGE-2C` and `SIM-SCR-BROWSER-1` blocked unless a later owned-surface review produces a new receipt-backed gap,
4. and implement the LLM attacker black-box boundary as executable contract data plus focused proof.
