# RSI-EVAL-1 Held-Out Evaluation Boundary Post-Implementation Review

Date: 2026-03-24
Status: Completed

Related context:

- [`../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# What Landed

`RSI-EVAL-1` now freezes the evaluation-visibility boundary for later recursive-improvement episodes:

1. `player_visible_protected_evidence`
2. `judge_held_out_evaluation`
3. `regression_anchor_contexts`

# Why This Matters

Without this slice, later players could quietly overfit every context they are supposed to be judged by. The repo already had strong protected-evidence instincts, but it still needed one explicit answer to:

1. what players may learn from,
2. what only the judge may score with,
3. and how strict-reference anchor contexts remain mandatory without becoming fully visible optimization targets.

# Downstream Impact

After this slice:

1. `SIM-LLM-1B` and `SIM-LLM-1C` must treat protected evidence as learnable but held-out judge contexts as off-limits.
2. `OVR-AGENT-2B` and `OVR-AGENT-2C` must consume summary verdicts without inheriting raw held-out case inventories.
3. `RSI-AUDIT-1A` can now bind lineage to a settled `evaluation_revision` boundary rather than a speculative one.

# Remaining Gaps

The next remaining cross-cutting contract gap is audit and provenance lineage:

1. shared episode and proposal ids,
2. config and later code enactment lineage,
3. and GitHub-backed code provenance where applicable.
