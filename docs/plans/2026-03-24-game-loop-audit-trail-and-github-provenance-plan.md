Date: 2026-03-24
Status: Proposed

Related context:

- [`../research/2026-03-24-game-loop-audit-trail-and-github-provenance-review.md`](../research/2026-03-24-game-loop-audit-trail-and-github-provenance-review.md)
- [`../research/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](../research/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md)
- [`../research/2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](../research/2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md)
- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md)
- [`2026-03-24-llm-player-role-decomposition-plan.md`](2026-03-24-llm-player-role-decomposition-plan.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../../src/observability/operator_snapshot_recent_changes.rs`](../../src/observability/operator_snapshot_recent_changes.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../docs/api.md`](../../docs/api.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Freeze the missing audit and provenance contract for Shuma's later recursive-improvement path so:

1. current config-loop changes remain auditable,
2. later recursive-improvement episodes have stable lineage and receipt vocabulary,
3. and later code evolution leans on GitHub for branch, PR, review, check, merge, revert, and build-provenance lineage instead of inventing a parallel review system inside Shuma.

# Core Decisions

1. The machine-first judge remains authoritative for outcome truth: scorecards, benchmark deltas, no-harm gates, retain or rollback, and homeostasis inputs.
2. Current config-loop lineage remains primarily internal to Shuma through the decision ledger, snapshot `recent_changes`, and later episode archive.
3. Later code-evolution lineage should lean on GitHub wherever possible:
   1. compare views,
   2. pull requests,
   3. pull-request reviews,
   4. status checks,
   5. protected-branch merge gates,
   6. merge or revert commits,
   7. and later artifact attestations when build outputs exist.
4. Shuma should store stable GitHub references and normalized receipt summaries, not duplicate full PR discussion or CI log content.
5. Recursive-improvement planning needs one shared provenance vocabulary across config proposals, later defender recommendations, and later code-evolution proposals.
6. This audit and provenance contract should remain separate from:
   1. the broader game contract,
   2. the scorecard contract,
   3. the player-protocol contract,
   4. and the held-out evaluation contract.

## Task 1: `RSI-AUDIT-1A`

### Canonical episode and proposal lineage schema across config and code moves

**Files:**

- Modify: `src/observability/operator_snapshot_recent_changes.rs`
- Modify: `src/admin/oversight_api.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `docs/api.md`
- Modify: `docs/dashboard.md`

**Work:**

1. Define one canonical provenance vocabulary shared by:
   1. current config recommend/apply/watch cycles,
   2. later defender-agent recommendation episodes,
   3. later run-to-homeostasis episodes,
   4. and later code-evolution proposals.
2. Freeze common lineage fields such as:
   1. `episode_id`,
   2. `proposal_id`,
   3. `proposal_kind`,
   4. `origin_role`,
   5. `game_contract_revision`,
   6. `scorecard_revision`,
   7. `protocol_revision`,
   8. `evaluation_revision`,
   9. evidence refs,
   10. baseline and result score refs,
   11. enactment status,
   12. rollback or revert status.
3. Make the current config ledger and later episode archive consume the same vocabulary instead of drifting into separate local naming.
4. Keep the canonical schema machine-first first; later dashboard or reporting surfaces should project it rather than define it.

**Acceptance criteria:**

1. the repo has one canonical answer to "what ids and lineage fields define a recursive-improvement episode or proposal?",
2. current config-change history and later code-evolution work no longer need separate conceptual audit vocabularies,
3. and later provenance work can be built without retrofitting ids after the fact.

## Task 2: `RSI-AUDIT-1B`

### GitHub-backed provenance contract for later code-evolution proposals

**Files:**

- Modify: `docs/plans/2026-03-24-llm-player-role-decomposition-plan.md`
- Modify: `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`
- Modify: `docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`
- Modify: `docs/api.md`
- Modify: later `OVR-CODE-1` planning when reopened

**Work:**

1. Define GitHub as the canonical code-lineage substrate for later code-evolution proposals wherever feasible.
2. Freeze the minimum GitHub-backed receipt surface Shuma should carry for code proposals:
   1. repository identifier,
   2. branch or compare refs,
   3. pull request number or URL,
   4. review status summary,
   5. check-suite refs and latest conclusions,
   6. head, merge, and revert commit SHAs,
   7. optional issue or escalation refs,
   8. optional artifact-attestation refs when build outputs exist.
3. Define which GitHub controls later code evolution must respect:
   1. protected branches,
   2. required pull-request reviews,
   3. required status checks,
   4. required deployments where used.
4. Make explicit that GitHub process success does not equal judge success:
   1. merge approval is not benchmark approval,
   2. passing CI is not no-harm proof,
   3. and revert lineage must still link back to judge-visible regression or failure reasons.
5. Keep Shuma from duplicating GitHub's native artifacts beyond stable refs and normalized summary fields.

**Acceptance criteria:**

1. the repo has one canonical answer to "how does a later code-evolution proposal map onto GitHub lineage and merge governance?",
2. code-side provenance leans on GitHub instead of inventing a shadow PR and review system inside Shuma,
3. and later code-evolution planning can cleanly distinguish GitHub process lineage from judge outcome lineage.

## Task 3: `RSI-AUDIT-1C`

### Machine-first audit retrieval and operator projection over config and code provenance

**Files:**

- Modify: `src/admin/oversight_api.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `docs/api.md`
- Modify: `docs/dashboard-tabs/game-loop.md`
- Modify: later operator-facing audit projection planning when reopened

**Work:**

1. Define how audit lineage should be retrieved machine-first for both:
   1. config proposals and applies,
   2. later code proposals and enactments.
2. Define the minimum operator-visible projection over that lineage:
   1. suggested,
   2. accepted,
   3. applied,
   4. retained,
   5. rolled back,
   6. merged,
   7. reverted.
3. Ensure later projections can show both:
   1. the GitHub lineage ref,
   2. and the judge outcome tied to that lineage.
4. Keep detailed PR review or CI output outside Shuma unless a later explicit product need justifies richer mirroring.

**Acceptance criteria:**

1. later recursive-improvement surfaces can expose a truthful audit trail without scraping GitHub ad hoc,
2. the operator can see both enactment lineage and benchmark outcome lineage,
3. and Shuma still avoids becoming a second source of truth for full PR or CI detail.

# Sequencing

1. Keep the attacker-faithful mainline first: `SIM-SCR-CHALLENGE-2A..2D`, `CTRL-SURFACE-1..3`, `RSI-GAME-1A`, `RSI-GAME-1B`, `RSI-SCORE-1`, `RSI-GAME-1C`, and `RSI-GAME-MAINLINE-1`.
2. Land the broader recursive-improvement role and protocol contract after that: `RSI-ROLES-1`, `RSI-PROTO-1`, and `RSI-EVAL-1`.
3. Land `RSI-AUDIT-1A` before `OVR-AGENT-2B` or `OVR-AGENT-2C` are treated as execution-ready.
4. Land `RSI-AUDIT-1B` before `OVR-CODE-1` is treated as execution-ready.
5. Land `RSI-AUDIT-1C` before later operator-facing loop history or code-evolution audit surfaces are considered complete.
6. Return to deferred operator-surface cleanup and follow-ons only after the first working loop is proven against the truthful attacker basis.

# Definition Of Done

This plan is complete when:

1. recursive-improvement episodes have one canonical lineage vocabulary,
2. GitHub is explicitly adopted as the code-lineage authority wherever feasible,
3. the machine-first judge remains the outcome authority,
4. and later defender or code-evolution phases can be audited without inventing a second review and merge system inside Shuma.
