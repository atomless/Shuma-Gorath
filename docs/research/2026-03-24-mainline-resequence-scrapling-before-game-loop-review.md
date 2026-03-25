Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`](../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md)
- [`../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`../plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md`](../plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Mainline Resequence: Scrapling Before Game Loop Review

## Question

Given the clarified requirement for attacker-faithful Scrapling, what should the main execution order be now?

## Conclusion

The mainline should move to:

1. attacker-faithful Scrapling expansion,
2. receipt-backed proof that Scrapling touches all defense surfaces it owns and can pass the ones a real attacker should be able to pass,
3. first explicit self-improving game-loop execution over that truthful attacker basis,
4. only then the later LLM attacker and defender runtime tracks and the deferred operator-surface cleanup work.

## Why this is the right resequence

The previous active queue still reflected dashboard and diagnostics follow-on work as the immediate next steps.

That is now the wrong order.

The user has clarified the real priority:

1. Scrapling must become fully malicious for the surfaces it owns,
2. the self-improving loop must then be proven working,
3. and only after that should the later LLM attacker and defender roles be reopened.

That matches the earlier sequencing rule already added to the game-loop plans:

1. judge-side planning may continue,
2. but fuller player-side execution must wait for attacker-faithful Scrapling.

## LLM attacker implication

The later LLM attacker must remain genuinely black-box.

Its starting knowledge should be:

1. the host site's root entrypoint,
2. the category or category family it is supposed to fulfill,
3. any bounded public hints discoverable from the host itself, such as `robots.txt`, sitemap references, or traversal-visible pages,
4. and the instruction to behave maliciously where that category implies malicious behavior.

It should not receive:

1. Shuma-specific internals,
2. Shuma route maps,
3. Shuma defense descriptions,
4. Shuma repo or documentation knowledge obtained out of band from the attacked host,
5. web-search access that could reveal Shuma-specific internals,
6. or any hidden judge or defender state.

That requirement belongs in the later `SIM-LLM-1A` black-box contract.

## Mainline consequence

The active backlog should stop implying that dashboard cleanup is the next mainline tranche.

Instead, the next mainline should be:

1. `SIM-SCR-CHALLENGE-2A` defense-surface matrix and success contract,
2. `SIM-SCR-CHALLENGE-2B` malicious request-native Scrapling interaction implementation,
3. `SIM-SCR-CHALLENGE-2D` receipt-backed coverage and ownership closure,
4. `SIM-SCR-CHALLENGE-2C` browser or stealth Scrapling adoption only if those receipts prove a remaining Scrapling-owned surface still requires it,
5. `RSI-GAME-MAINLINE-1` first working self-improving loop over that truthful attacker base.

## Result

Shuma's immediate priority is no longer generic UI backlog progress.

It is:

1. make Scrapling attacker-faithful,
2. prove it,
3. then make the game loop work over that truthful adversary pressure,
4. then reopen later LLM attacker and defender runtime work.
