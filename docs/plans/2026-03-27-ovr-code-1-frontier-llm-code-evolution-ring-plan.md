Date: 2026-03-27
Status: Proposed, execution blocked

Related context:

- [`../research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](../research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`../research/2026-03-27-game-loop-current-state-and-gap-review.md`](../research/2026-03-27-game-loop-current-state-and-gap-review.md)
- [`2026-03-27-game-loop-board-state-refactor-plan.md`](2026-03-27-game-loop-board-state-refactor-plan.md)
- [`2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md`](2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md)
- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Define the later frontier-LLM code-evolution ring so it begins from explicit machine-first code-gap evidence instead of vague dashboard impressions, stays separate from bounded config tuning, and preserves the strict human-only reference stance as a regression anchor.

# Core Decisions

1. Code evolution is a separate ring from bounded config tuning and must not reopen the controller action surface by stealth.
2. The ring may begin only from explicit `code_evolution_referral` or `config_ring_exhausted` plus sufficient benchmark evidence quality.
3. The frontier LLM must consume sacred machine-first inputs, not simulator hints, private lane metadata, or raw UI impressions.
4. The independent judge remains authoritative before and after any suggested code change.
5. The strict `human_only_private` reference stance remains a mandatory regression anchor even when later code evolution optimizes broader target stances.
6. GitHub should remain the canonical lineage spine for later code proposals, review, checks, merge, revert, and optional artifact provenance.

# Sacred Inputs

Any future `OVR-CODE-1` execution must consume, at minimum:

1. `benchmark_results_v1`,
2. `operator_snapshot_v1`,
3. `oversight_history_v1`,
4. bounded episode-archive and failed-move lineage,
5. named breach loci, evidence quality, urgency, and repair-surface context,
6. explicit code-gap referral or config-ring exhaustion reason,
7. the canonical mutability policy and allowed-action surface so the model knows what config was already legal and exhausted,
8. and the strict-reference regression contract.

It must **not** consume:

1. simulator persona declarations as truth,
2. private repo-only knowledge about attacker identity that real traffic would not reveal to Shuma at runtime,
3. or freeform permission to rewrite objectives, trust topology, or hard-never controller surfaces.

# Expected Outputs

The first reopenable version should be recommendation-only.

Minimum output contract:

1. a machine-readable diagnosis summary of the code gap,
2. one or more bounded code-change hypotheses tied to named breach loci,
3. expected benchmark-family impact and possible human-friction risks,
4. an explicit verification plan naming the `make` proofs that must pass,
5. and an auditable lineage record that can later map to GitHub review or PR artifacts.

Later optional expansions may include:

1. concrete patch proposals,
2. draft PR generation,
3. and merge or revert orchestration,

but only after the audit and provenance contracts are mature enough to keep those actions accountable.

# Phased Plan

## `OVR-CODE-1A`

### Sacred handoff contract

Define the exact machine-first inputs, blocked fields, provenance ids, and strict-reference regression requirements.

Acceptance criteria:

1. the handoff starts only from explicit code-gap referral or config-ring exhaustion,
2. sacred inputs and forbidden inputs are listed explicitly,
3. the strict `human_only_private` regression anchor is mandatory,
4. and blocked backlog items reference this handoff contract.

## `OVR-CODE-1B`

### Recommendation-only diagnosis and patch planning

Define the first safe frontier-LLM output shape as recommendation-only.

Acceptance criteria:

1. output schema names diagnosis, proposed files or modules, expected benchmark impact, human-friction risks, and verification plan,
2. the proposal cannot silently widen the controller move ring or mutate operator objectives,
3. and the plan is explicit that the independent judge still decides whether a code change improved the board state.

## `OVR-CODE-1C`

### Later GitHub-backed patch and PR execution

Define the later optional code-writing and PR-generation ring over GitHub lineage.

Acceptance criteria:

1. GitHub-backed lineage requirements are explicit,
2. review, checks, merge, and revert remain first-class gates,
3. and this phase stays blocked until the audit/provenance contracts are execution-ready.

# Definition Of Done For This Planning Slice

This planning slice is complete when:

1. the repo has one canonical frontier-LLM code-evolution ring plan,
2. `OVR-CODE-1` blocked backlog text points to this plan,
3. the plan makes the sacred inputs, outputs, and regression anchors explicit,
4. and execution remains blocked until the Scrapling game loop is proven strongly enough that code evolution would be judged from trustworthy machine-first evidence rather than from noisy impressions.
