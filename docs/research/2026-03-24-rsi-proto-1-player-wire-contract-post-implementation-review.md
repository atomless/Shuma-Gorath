# RSI-PROTO-1 Player Wire Contract Post-Implementation Review

Date: 2026-03-24
Status: Completed

Related context:

- [`../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`](../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# What Landed

`RSI-PROTO-1` now freezes one canonical player-wire contract for later recursive-improvement runtimes:

1. shared envelope fields for protocol revision, episode lineage, trace lineage, role, message kind, visibility ring, and receipt references,
2. attacker observation families,
3. attacker action families,
4. defender input families,
5. defender output families.

# Why This Matters

The role contract alone was not enough. Later attacker and defender runtimes still needed one explicit answer to what they send and receive, otherwise:

1. each runtime could invent its own wire format,
2. refusal or escalation semantics could drift between players,
3. and later audit lineage would have to retrofit protocol revisions after the fact.

This slice closes that gap and makes later player implementations subordinate to one shared envelope and vocabulary.

# Downstream Impact

After this slice:

1. `SIM-LLM-1A` must use the canonical attacker observation and action families instead of inventing a bespoke attacker schema.
2. `OVR-AGENT-2A` must use the canonical defender input and output families instead of inventing a local recommendation schema.
3. `RSI-AUDIT-1A` can now bind episode and proposal lineage to a settled `protocol_revision`.

# Remaining Gaps

This slice does not yet define:

1. which evidence rings are player-visible versus judge-held-out,
2. or the full audit lineage schema over config and later code episodes.

Those remain the next contract tranches on this path.
