# RSI-ROLES-1 Triadic Role Contract Post-Implementation Review

Date: 2026-03-24
Status: Completed

Related context:

- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# What Landed

`RSI-ROLES-1` now freezes the triadic role split for later recursive-improvement phases:

1. the attacker is the LLM-backed adversary player in the sim harness,
2. the defender is the LLM-backed bounded diagnosis/config player,
3. the judge is the machine-first benchmark stack,
4. and Monitoring or Game Loop is only the human-readable projection of that judge.

# Why This Matters

The repo already had the core game pieces in place:

1. immutable rules,
2. a sacred evaluator,
3. a bounded legal move ring,
4. explicit move-selection policy,
5. and a bounded episode archive.

But later recursive-improvement planning still risked drifting into a vague two-agent story unless the role split itself became an explicit landed contract. This slice closes that gap and prevents later defender or attacker planning from quietly reintroducing self-judging behavior.

# Downstream Impact

After this slice:

1. `RSI-PROTO-1` can define attacker and defender schemas against a fixed role boundary instead of an implied one.
2. `RSI-EVAL-1` can define player-visible versus judge-held-out evidence against an explicit judge role.
3. `OVR-AGENT-2`, `RSI-METH-1`, and `OVR-CODE-1` must now inherit the same settled attacker/defender/judge split rather than describing it locally.

# Remaining Gaps

This slice does not yet define:

1. the player wire schemas,
2. the held-out evaluation boundary,
3. or the audit lineage vocabulary.

Those remain the next contract tranches on this path.
