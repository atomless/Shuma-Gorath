Date: 2026-03-25
Status: Proposed planning driver

Related context:

- [`2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md`](2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md)
- [`2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md`](2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md)
- [`../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md`](../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Purpose

Decide whether `RSI-GAME-HV-1` should follow the strict Scrapling-only proof directly, or whether it should stay delayed until the later LLM attacker is in the loop and Shuma has also proven repeated improvement under combined Scrapling plus LLM pressure while still in `human_only_private`.

# Findings

## 1. `RSI-GAME-HO-1` is the right place to prove the strict loop works, but not the right place to justify loosening stance

`RSI-GAME-HO-1` should prove:

1. the canonical stance model is wired correctly,
2. full-power Scrapling is pressuring the strict stance truthfully,
3. the Game Loop can generate recommendations, apply bounded config changes, rerun, and retain or roll back under `human_only_private`.

That is a real milestone.

But it does not yet answer the later question:

1. does the loop still improve toward the strict target once both non-agent and LLM-backed attacker pressure are included?

Conclusion:

1. `RSI-GAME-HO-1` is sufficient to prove the strict loop exists and works,
2. but not sufficient to justify loosening away from `human_only_private`.

## 2. `humans_plus_verified_only` adds little to loop verification if opened before the later LLM attacker is in the loop

Opening `humans_plus_verified_only` immediately after `RSI-GAME-HO-1` would mostly test:

1. whether the stance model can be relaxed,
2. and whether verified identities can receive lower-friction treatment.

It would not add much to the core verification question, because `RSI-GAME-HO-1` already proves the loop plumbing itself under the strict stance.

Conclusion:

1. `RSI-GAME-HV-1` should not be used as a substitute for fuller loop verification.

## 3. The stricter proof should stay on `human_only_private` until both Scrapling and LLM pressure are included

If the goal is to trust the loop before any loosening, the stricter proof should show:

1. repeated config tweaks suggested by the loop,
2. those tweaks applied and judged across many runs,
3. and measurable positive movement toward the strict target,
4. while both Scrapling and the later LLM attacker contribute pressure in the same strict stance.

Conclusion:

1. there should be a second strict-baseline proof tranche after the LLM attacker runtime lands.

## 4. The later LLM runtime should therefore come before `RSI-GAME-HV-1`, not after it

The earlier sequence still put `RSI-GAME-HV-1` before the remaining LLM runtime reopening.

That is the wrong order for the stronger validation standard.

Conclusion:

1. after `RSI-GAME-HO-1`, the next meaningful slice is the remaining LLM attacker runtime proof closure,
2. then a combined-pressure strict-baseline proof,
3. and only then the later relaxed `humans_plus_verified_only` sweep.

# Decisions

1. Keep `RSI-GAME-HO-1` as the first strict-baseline proof tranche over full-power Scrapling.
2. Reopen the remaining LLM attacker runtime work after `RSI-GAME-HO-1`, not after `RSI-GAME-HV-1`.
3. Add a second strict-baseline tranche after the LLM attacker lands:
   1. repeated `human_only_private` cycles with both Scrapling and LLM runs in the loop,
   2. retained config changes,
   3. and measured improvement toward the strict target.
4. Keep `RSI-GAME-HV-1` blocked until that combined-pressure strict-baseline proof exists.

# Result

The corrected order is:

1. `STANCE-MODEL-1`
2. `SIM-SCR-FULL-1`
3. `RSI-GAME-HO-1`
4. `SIM-LLM-1C3`
5. `RSI-GAME-HO-2`
6. only then `RSI-GAME-HV-1`
