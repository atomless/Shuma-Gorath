Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md`](../plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md)
- [`../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Game Loop Sequencing: Require Attacker-Faithful Scrapling Review

## Question

Should Shuma require attacker-faithful Scrapling coverage for Scrapling-owned defense surfaces before moving forward with the fuller attacker/defender game loop?

## Conclusion

Yes.

But the sequencing needs one important distinction:

1. the machine-first judge and game-contract planning can continue,
2. while the fuller player-side game loop must stay blocked until Scrapling is attacker-faithful and receipt-backed for the surfaces it owns.

## Why this is the right boundary

The broader recursive-improvement game depends on truthful players and a truthful judge.

If the attacker side is underpowered on a major lane:

1. the defender optimizes against the wrong pressure,
2. the judge still measures something real, but not the real competitive environment,
3. and the loop risks converging on false improvements.

That is especially dangerous for Scrapling because it is intended to be the first primary adaptive adversary lane. If Scrapling is not using the strongest realistic hostile behaviors for the surfaces it owns, then the attacker side of the game is distorted before the later LLM attacker even arrives.

## What must be true before the fuller game loop is execution-ready

For every defense surface the Scrapling lane is supposed to represent:

1. the relevant attacker-grade Scrapling behavior must be operational,
2. that behavior must be receipt-backed and verified in Shuma,
3. and any remaining gap must be explicit and intentionally assigned to another lane.

This is not the same thing as "adopt every Scrapling feature."

It is the narrower and more correct prerequisite:

1. adopt all attacker-relevant Scrapling capability needed for Scrapling-owned surfaces,
2. and prove it.

## Consequence for sequencing

The right sequencing split is:

### Work that can continue

1. judge-side game-contract planning,
2. scorecard planning,
3. protocol planning,
4. held-out evaluation planning,
5. audit and provenance planning.

### Work that should remain blocked

1. fuller attacker-agent runtime execution,
2. fuller defender-agent runtime execution,
3. later bounded autonomous run-to-homeostasis phases,

until Scrapling-owned surfaces are attacker-faithful and receipt-backed.

## Concrete prerequisite wording

Broader game-loop execution should remain blocked until:

1. `SIM-SCR-CHALLENGE-1` is complete,
2. and, where the defense-surface matrix shows it is necessary for Scrapling-owned surfaces, `SIM-SCR-BROWSER-1` is also complete,
3. with any still-uncovered surfaces explicitly assigned to another lane rather than silently tolerated.

## Result

Shuma should not treat the existence of a Scrapling lane as enough.

Before the fuller attacker/defender game loop becomes execution-ready, Scrapling must be:

1. attacker-faithful for the surfaces it owns,
2. receipt-backed,
3. and explicitly bounded where it does not own a surface.
