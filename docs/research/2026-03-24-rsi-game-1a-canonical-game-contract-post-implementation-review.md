Date: 2026-03-24

# RSI-GAME-1A Canonical Game Contract Post-Implementation Review

## What landed

Shuma now has one explicit machine-first answer to the question:

> What game is the current bounded loop playing?

That answer is materialized as `game_contract_v1`.

The contract now names:

1. the immutable rule surface,
2. the fixed payoffs and independent judge,
3. the legal move ring,
4. the safety gates,
5. the regression anchors.

It is derived from the already-landed backend truths rather than inventing a parallel game model:

- `operator_objectives_v1` as the immutable rule surface,
- the machine-first benchmark stack as the judge,
- `allowed_actions_v1` plus controller mutability as the legal move ring,
- rollout and protected-evidence guardrails as safety gates,
- and no-harm plus protected-evidence constraints as regression anchors.

## Why this closes `RSI-GAME-1A`

`RSI-GAME-1A` was not about adding another operator narrative or later player runtime.

It was specifically about freezing the current bounded loop into one formal contract before later recursive phases widen the system.

That is now true because:

1. the game contract is machine-visible,
2. it is derived from the same backend-owned rules and mutability surfaces the live loop already uses,
3. it projects through both `operator_snapshot_v1` and the bounded oversight execution/history payloads,
4. and the focused proof path now asserts those surfaces rather than leaving the contract implicit.

## Proof

Focused verification:

- `make test-rsi-game-contract`
- `git diff --check`

That proof covers:

- contract naming in `operator_snapshot_objectives`,
- legal move ring identity on `allowed_actions_v1`,
- projection through `operator_snapshot_v1`,
- and projection through the bounded oversight execution/history contract.

## Outcome

The next judge-side mainline can now move directly into `RSI-GAME-1B`.

That means:

1. shortfall attribution and move selection no longer need to infer the broader game from scattered modules,
2. later scorecard, archive, and player-runtime work can consume one explicit upstream contract,
3. and the first explicit self-improving loop is now blocked by the remaining judge-side slices, not by a missing definition of the game itself.
