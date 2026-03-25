# RSI-AUDIT-1A Shared Lineage Schema Post-Implementation Review

Date: 2026-03-24
Status: Completed

Related context:

- [`../plans/2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md`](../plans/2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# What Landed

`RSI-AUDIT-1A` now freezes the shared lineage vocabulary across current config episodes and later code episodes:

1. `episode_id`
2. `proposal_id`
3. `proposal_kind`
4. `origin_role`
5. `game_contract_revision`
6. `scorecard_revision`
7. `protocol_revision`
8. `evaluation_revision`
9. `evidence_refs`
10. `baseline_score_refs`
11. `result_score_refs`
12. `enactment_status`
13. `rollback_or_revert_status`

# Why This Matters

The player protocol and evaluation boundary are now landed, so the repo needed one stable audit vocabulary that could bind those revisions to actual recursive-improvement episodes. Without this slice, later config and code proposals would still risk diverging into separate provenance languages.

# Downstream Impact

After this slice:

1. `OVR-AGENT-2B` can treat shared episode lineage as a settled prerequisite rather than a future contract.
2. `RSI-AUDIT-1B` can focus purely on GitHub-backed code lineage instead of redefining basic episode ids.
3. `RSI-AUDIT-1C` can focus on retrieval and operator projection rather than schema invention.

# Remaining Gaps

The remaining audit work is now narrower:

1. GitHub-backed code provenance in `RSI-AUDIT-1B`
2. machine-first retrieval and operator projection in `RSI-AUDIT-1C`
