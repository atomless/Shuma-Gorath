Date: 2026-03-25
Status: Proposed

Related context:

- [`2026-03-25-sim-llm-1a-black-box-contract-post-implementation-review.md`](2026-03-25-sim-llm-1a-black-box-contract-post-implementation-review.md)
- [`2026-03-24-rsi-eval-1-held-out-evaluation-boundary-post-implementation-review.md`](2026-03-24-rsi-eval-1-held-out-evaluation-boundary-post-implementation-review.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# SIM-LLM-1B Episode Harness Readiness Review

## Question

Now that `SIM-LLM-1A` made the later attacker black-box boundary executable, what is the smallest truthful next backend slice?

## Conclusion

The next backend slice should be `SIM-LLM-1B`, and it should remain contract-first.

That means:

1. define the attacker's episode start state,
2. define its bounded terminal states,
3. define what memory and curriculum inputs are allowed,
4. define what remains judge-held and therefore invisible,
5. and carry that through the same bounded fulfillment-plan surfaces rather than reopening a live runtime actor.

## Why this is now the right next step

After `SIM-LLM-1A`, the repo has one explicit answer to:

1. what the later attacker may know,
2. what it must not know,
3. and how its black-box position differs from Shuma-aware testing.

What is still missing is one equally explicit answer to:

1. what an attacker episode is,
2. when it starts clean,
3. when it stops,
4. what prior information it may carry forward,
5. and how that memory stays subordinate to the held-out evaluation boundary.

Without that, the later attacker is still under-specified in one important way:
it could quietly drift into hidden long-memory search or judge leakage while still technically remaining black-box on first entry.

## Existing seams make this a small real slice

The repo already has the right implementation seam:

1. the existing adversarial contract file in [`../../scripts/tests/adversarial/frontier_action_contract.v1.json`](../../scripts/tests/adversarial/frontier_action_contract.v1.json),
2. the Python fulfillment-plan loader in [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py),
3. the Rust fulfillment-plan payload in [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs),
4. and the internal beat payload proof in [`../../src/admin/api.rs`](../../src/admin/api.rs).

So `SIM-LLM-1B` should remain a contract tranche over those same surfaces.

## Decision

Make `SIM-LLM-1B` the next active backend slice.

Keep it narrow:

1. no live frontier attacker runtime,
2. no broader dashboard work,
3. no new parallel schema family,
4. just the canonical episode harness and bounded memory contract over the existing fulfillment-plan path.
