Date: 2026-03-25
Status: Completed

Related plan:

- [`../plans/2026-03-25-sim-llm-1b-episode-harness-implementation-plan.md`](../plans/2026-03-25-sim-llm-1b-episode-harness-implementation-plan.md)

# SIM-LLM-1B Post-Implementation Review

## What landed

`SIM-LLM-1B` is now an executable attacker episode-harness contract instead of prose only.

The later LLM attacker now carries an explicit machine-readable episode lifecycle and bounded memory policy through the same fulfillment-plan surfaces already used for the black-box boundary:

1. the canonical contract data in [`../../scripts/tests/adversarial/frontier_action_contract.v1.json`](../../scripts/tests/adversarial/frontier_action_contract.v1.json),
2. the Python fulfillment loader and plan builder in [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py),
3. the Rust fulfillment-plan payload in [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs),
4. and the internal beat rendered proof in [`../../src/admin/api.rs`](../../src/admin/api.rs).

The contract now states machine-readably that the later attacker episode includes:

1. required initial context fields,
2. a fresh-environment reset policy,
3. a bounded action horizon,
4. canonical terminal and failure states,
5. allowed retained memory and curriculum inputs,
6. explicit allowance for player-visible protected evidence,
7. and an explicit prohibition on held-out evaluation visibility.

## Verification

- `make test-adversarial-llm-fit`
- `git diff --check`

## Outcome Against Plan

The tranche met the plan:

1. the episode harness is now frozen in the canonical contract file rather than prose only,
2. the Python fulfillment path validates and emits the same lifecycle and bounded-memory rules,
3. the Rust fulfillment payload mirrors those rules,
4. the internal beat proves the episode harness reaches the machine-visible runtime payload,
5. and the memory boundary is now explicit and aligned with the landed held-out evaluation rules.

## Remaining Gap

This slice still does not reopen a live first-class LLM attacker runtime.

That later work remains `SIM-LLM-1C`, and it stays blocked until the later attacker runtime is intentionally reopened over the now-landed black-box and episode contracts rather than assumed to be execution-ready just because those contracts now exist.

## Follow-On

The next active queue returns to the deferred operator-surface cleanup while the full LLM attacker runtime remains explicitly blocked.
