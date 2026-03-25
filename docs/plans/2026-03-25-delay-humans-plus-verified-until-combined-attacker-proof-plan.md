Date: 2026-03-25
Status: Proposed

Related context:

- [`../research/2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-review.md`](../research/2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-review.md)
- [`2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`2026-03-25-sim-llm-1c-runtime-decomposition-plan.md`](2026-03-25-sim-llm-1c-runtime-decomposition-plan.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Refine the post-`STANCE-MODEL-1` sequence so `humans_plus_verified_only` stays delayed until after:

1. full-power Scrapling strict-baseline proof,
2. the remaining LLM attacker runtime proof closure,
3. and a second strict `human_only_private` proof tranche that includes both Scrapling and LLM attacker pressure.

# Core Decisions

1. `RSI-GAME-HO-1` proves the strict loop works with full-power Scrapling, but it does not unlock stance relaxation by itself.
2. `SIM-LLM-1C3` should reopen after `RSI-GAME-HO-1`, not stay behind `RSI-GAME-HV-1`.
3. `RSI-GAME-HO-2` should prove repeated retained improvement under `human_only_private` with both Scrapling and LLM attacker runs included in the loop.
4. `RSI-GAME-HV-1` remains a later relaxed verified-identity sweep, but it should open only after `RSI-GAME-HO-2`.

# Execution Shape

## `RSI-GAME-HO-1`: First strict-baseline proof over full-power Scrapling

Keep the current contract:

1. `STANCE-MODEL-1` lands,
2. `SIM-SCR-FULL-1` lands,
3. the machine-first loop proves retained config-change improvement under `human_only_private` with full-power Scrapling pressure.

This tranche proves the strict loop is real and useful.

## `SIM-LLM-1C3`: Remaining LLM runtime proof closure

After `RSI-GAME-HO-1`, reopen the remaining LLM attacker runtime proof slice.

Required contract:

1. runtime receipts are projected truthfully,
2. recent-run visibility is complete,
3. and the later LLM attacker is a real black-box runtime participant rather than a partial harness seam.

## `RSI-GAME-HO-2`: Combined-attacker strict-baseline proof

This tranche should begin only after `SIM-LLM-1C3` is satisfied.

### `RSI-GAME-HO-2A`

Run the strict `human_only_private` loop with both:

1. Scrapling runs,
2. and LLM attacker runs

as game-loop pressure sources.

### `RSI-GAME-HO-2B`

Repeat many config-change cycles under that combined pressure until the proof is stronger than plumbing:

1. recommendations are generated,
2. bounded config changes are applied,
3. later mixed-attacker runs occur against those changed configs,
4. watch windows retain or roll back truthfully,
5. and repeated retained changes produce measured positive movement toward the strict target.

### `RSI-GAME-HO-2C`

Define the unlock condition for any later relaxed verified-identity stance:

1. repeated retained improvements under `human_only_private`,
2. with both Scrapling and LLM attacker pressure included,
3. and evidence that the loop is improving the strict target rather than only surviving replay.

## `RSI-GAME-HV-1`: Later `humans_plus_verified_only` sweep

Keep this explicitly later.

It should remain blocked until `RSI-GAME-HO-2C` is satisfied.

When it opens, it should be framed as:

1. a later comparative stance experiment,
2. not a prerequisite for proving the loop itself.

# Backlog Integration

1. Keep `STANCE-MODEL-1`, `SIM-SCR-FULL-1`, and `RSI-GAME-HO-1` in the active queue.
2. Keep `SIM-LLM-1C3` blocked only until those earlier strict-baseline prerequisites are complete.
3. Add blocked `RSI-GAME-HO-2` as the required combined-attacker strict-baseline follow-on after `SIM-LLM-1C3`.
4. Re-block `RSI-GAME-HV-1` behind `RSI-GAME-HO-2`, not just behind `RSI-GAME-HO-1`.

# Definition Of Done

This planning tranche is satisfied when:

1. the repo explicitly places `SIM-LLM-1C3` before `RSI-GAME-HV-1`,
2. `RSI-GAME-HO-2` is defined as the combined Scrapling plus LLM strict-baseline proof,
3. and `humans_plus_verified_only` is blocked until that combined-pressure proof exists.
