Date: 2026-03-25
Status: Proposed

Related context:

- [`../research/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md`](../research/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md)
- [`2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md`](2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md)
- [`2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`2026-03-25-scrapling-full-attacker-capability-principle-plan.md`](2026-03-25-scrapling-full-attacker-capability-principle-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Define the post-`STANCE-MODEL-1` execution order so Shuma does not relax stance or reopen LLM runtime work until:

1. Scrapling fully represents the non-agent or non-LLM adversary spectrum assigned to it through all attacker-relevant upstream Scrapling capability that belongs to that lane,
2. and the strict `human_only_private` loop has repeatedly improved through real config-change iteration.

# Core Decisions

1. `human_only_private` remains the first real operating stance after the stance-model redesign lands.
2. Scrapling must mature beyond the current request-native baseline before that strict stance is treated as operationally proven.
3. The proof bar is repeated config-change improvement across many completed cycles, not one canary cycle.
4. `humans_plus_verified_only` is a later gated second stance, not the next automatic move.
5. LLM attacker and defender runtime work remains downstream of the same gate.

# Execution Shape

## `SIM-SCR-FULL-1`: Full-power Scrapling for the non-agent adversary spectrum

This tranche should follow `STANCE-MODEL-1`.

It should treat the current request-native Scrapling lane as a truthful baseline, but not the maturity target.

### `SIM-SCR-FULL-1A`

Freeze the full-power capability matrix for the non-agent or non-LLM adversary spectrum assigned to Scrapling.

Required contract:

1. identify the full attacker-relevant upstream Scrapling capability set Shuma expects to own in Scrapling,
2. include request-native, browser, stealth, challenge-interaction, and bypass-style capability where relevant,
3. and record explicit exclusions only where they are intentionally out of scope for Scrapling.

### `SIM-SCR-FULL-1B`

Implement the remaining Scrapling capability required by that matrix.

Required contract:

1. Shuma should no longer rely on the current polite subset,
2. Scrapling should be capable of touching and, where realistic, passing the defenses a real non-agent adversary should be able to pass,
3. and failing the defenses it should fail.

### `SIM-SCR-FULL-1C`

Add receipt-backed proof for the full-power Scrapling lane.

Required contract:

1. prove which defenses it touched,
2. which it passed where expected,
3. which it failed where expected,
4. and which non-human categories and defense surfaces it actually exercised.

## `RSI-GAME-HO-1`: Strict `human_only_private` operational proof over repeated cycles

This tranche should begin only after `SIM-SCR-FULL-1` is satisfied.

### `RSI-GAME-HO-1A`

Make `human_only_private` the actual active game-loop stance for the current machine-first loop.

Required contract:

1. verified non-human traffic remains denied under this stance,
2. benchmarks and Game Loop project the strict stance truthfully,
3. and the existing loop machinery runs against this corrected stance.

### `RSI-GAME-HO-1B`

Run repeated strict-baseline cycles until the loop is operationally proven.

Required contract:

1. Scrapling runs,
2. recommendations are generated,
3. bounded config changes are applied,
4. later Scrapling runs occur against the changed config,
5. watch windows judge retain or rollback truthfully,
6. and this repeats enough times to show non-noisy improvement toward the strict target rather than a single lucky cycle.

### `RSI-GAME-HO-1C`

Define the unlock condition for leaving the strict stance.

Required contract:

1. multiple completed cycles under `human_only_private`,
2. actual retained config changes,
3. measured improvement toward the strict target,
4. and clear evidence that the loop is not merely exercising rollback or recommendation plumbing.

## `RSI-GAME-HV-1`: Later `humans_plus_verified_only` sweep

This is explicitly a later tranche.

It should remain blocked until `RSI-GAME-HO-1C` is satisfied.

When it opens, it should:

1. compare against the proven strict baseline,
2. measure verified-identity handling against real prior strict-baseline data,
3. and not rely on hypothetical benefits.

# Backlog Integration

1. Keep `STANCE-MODEL-1` as the immediate next design and implementation prerequisite.
2. After `STANCE-MODEL-1`, make `SIM-SCR-FULL-1` the next mainline instead of any LLM runtime slice.
3. After `SIM-SCR-FULL-1`, make `RSI-GAME-HO-1` the next mainline.
4. Block `RSI-GAME-HV-1` until `RSI-GAME-HO-1` proves real repeated improvement.
5. Re-block any remaining active LLM runtime work behind this stricter non-LLM gate.

# Definition Of Done

This planning tranche is satisfied when:

1. the repo explicitly says the next post-stance-model work is full-power Scrapling, not LLM runtime,
2. the strict `human_only_private` operational proof is defined as repeated improvement through real config-change cycles,
3. the later `humans_plus_verified_only` sweep is blocked on that proof,
4. and LLM lanes are also kept behind that same gate.
