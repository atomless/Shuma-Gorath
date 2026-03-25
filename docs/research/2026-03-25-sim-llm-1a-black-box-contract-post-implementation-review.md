Date: 2026-03-25
Status: Completed

Related plan:

- [`../plans/2026-03-25-sim-llm-1a-black-box-contract-implementation-plan.md`](../plans/2026-03-25-sim-llm-1a-black-box-contract-implementation-plan.md)

# SIM-LLM-1A Post-Implementation Review

## What landed

`SIM-LLM-1A` is now an executable repo contract instead of prose only.

The later LLM attacker's black-box boundary is now carried through the existing bounded fulfillment surfaces:

1. the canonical contract data in [`../../scripts/tests/adversarial/frontier_action_contract.v1.json`](../../scripts/tests/adversarial/frontier_action_contract.v1.json),
2. the Python fulfillment loader and plan builder in [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py),
3. the Rust fulfillment-plan payload in [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs),
4. and the internal beat rendered proof in [`../../src/admin/api.rs`](../../src/admin/api.rs).

The boundary now states machine-readably that the later attacker is:

1. `outside_attacker`,
2. host-root-only,
3. category-objective-first,
4. publicly informed only by host-derived hints,
5. Shuma-blind,
6. and denied repo visibility, judge visibility, and web-search access.

## Verification

- `make test-adversarial-llm-fit`
- `git diff --check`

## Outcome Against Plan

The tranche met the plan:

1. backlog and sequencing were refreshed so `SIM-LLM-1A` became the active backend slice,
2. the stale Scrapling umbrella blocker was retired for the current owned request-native matrix,
3. the black-box contract was added to the existing adversarial contract surface rather than a new parallel one,
4. the Python fulfillment plan now validates and emits the boundary,
5. the Rust fulfillment plan now mirrors it,
6. and the internal beat payload proves the boundary reaches the machine-visible runtime surface.

## Remaining Gap

This slice does not yet define:

1. episode structure,
2. bounded strategy memory,
3. or curriculum or archive inputs for the later attacker.

That remains the next backend slice: `SIM-LLM-1B`.

## Follow-On

The next backend mainline slice should be `SIM-LLM-1B`, while deferred dashboard cleanup remains below the backend path.
