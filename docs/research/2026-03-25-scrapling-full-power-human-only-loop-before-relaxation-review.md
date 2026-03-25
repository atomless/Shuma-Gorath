Date: 2026-03-25
Status: Proposed planning driver

Related context:

- [`2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md`](2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](2026-03-24-reference-stance-and-run-to-homeostasis-review.md)
- [`2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md`](2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md)
- [`../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md`](../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md)
- [`../plans/2026-03-25-scrapling-full-attacker-capability-principle-plan.md`](../plans/2026-03-25-scrapling-full-attacker-capability-principle-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Purpose

Decide what Shuma must prove after `STANCE-MODEL-1` and before:

1. relaxing from `human_only_private` to `humans_plus_verified_only`,
2. or reopening any later LLM attacker or defender runtime work.

# Findings

## 1. The first working Scrapling-driven game loop proof exists, but it is not the same as full operational proof

Shuma already has:

1. attacker-faithful request-native Scrapling proof,
2. a first working post-sim -> canary -> judged retain or rollback -> archive loop proof,
3. and the machine-first judge-side game contracts.

That is a major milestone.

But it is still narrower than the stronger operational claim:

1. Scrapling fully represents the non-agent or non-LLM adversary spectrum Shuma wants it to own,
2. and the loop has repeatedly improved the strict baseline through actual config change and rerun cycles.

Conclusion:

1. "first working loop" is not yet the same thing as "strict stance operationally proven".

## 2. The current request-native Scrapling baseline is still too narrow to unlock stance relaxation

The repo already upgraded Scrapling from a timid request lane into a truthful request-native attacker baseline.

That is good, but the user is right that the unlock condition for relaxing stance should be stronger:

1. Scrapling should use the full set of attacker-relevant upstream Scrapling capability needed to represent the non-agent or non-LLM adversary spectrum Shuma assigns to it,
2. not only the current request-native subset.

Conclusion:

1. stance relaxation should wait for full-power Scrapling maturity, not merely the first request-native baseline.

## 3. The strict `human_only_private` stance is only meaningful if repeated loop cycles actually improve it

A strict baseline is valuable only if Shuma can show:

1. the loop generates recommendations,
2. those recommendations become bounded config changes,
3. later Scrapling runs occur against the changed config,
4. the watch windows judge retain or rollback truthfully,
5. and repeated cycles, run many times rather than once or twice, measurably improve the strict-target scorecard rather than only firing one lucky canary.

Conclusion:

1. the unlock condition must be repeated-cycle improvement under the strict stance, not only one end-to-end loop proof.

## 4. The transition to `humans_plus_verified_only` should be treated as a gated second experiment, not the next default move

`humans_plus_verified_only` is still the right second stance.

But it should not open just because:

1. verified identity exists,
2. or one strict-baseline loop can run at all.

It should open only after the strict baseline has been operationally validated.

Conclusion:

1. the relaxation itself should be an explicit later gate.

## 5. LLM runtime work should stay behind the same proof gate

If Shuma has not yet:

1. given Scrapling full non-agent adversary power,
2. and shown repeated strict-baseline improvement under that pressure,

then later LLM runtime work is premature.

Conclusion:

1. the next queue after `STANCE-MODEL-1` should remain Scrapling-only and machine-first.

# Decisions

1. After `STANCE-MODEL-1`, the next mainline should stay non-LLM.
2. The next mainline tranche should be full-power Scrapling maturity for the non-agent or non-LLM adversary spectrum Shuma assigns to Scrapling.
3. After full-power Scrapling lands, Shuma should run repeated `human_only_private` loop cycles until real improvement is proven through actual config-change iteration over many completed cycles.
4. Do not transition to `humans_plus_verified_only` until a later combined-attacker strict-baseline proof exists.
5. The remaining LLM attacker runtime should reopen after the first strict Scrapling-only proof, but before any relaxed verified-identity sweep.

# Result

The correct post-`STANCE-MODEL-1` order is:

1. full-power Scrapling,
2. repeated strict human-only loop proof with Scrapling,
3. then the remaining LLM attacker runtime closure,
4. then a second strict human-only proof with both Scrapling and LLM pressure,
5. and only after that a later `humans_plus_verified_only` sweep.
