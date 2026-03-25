Date: 2026-03-24
Status: Proposed

Related context:

- [`../research/2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](../research/2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md)
- [`../research/2026-03-24-game-loop-audit-trail-and-github-provenance-review.md`](../research/2026-03-24-game-loop-audit-trail-and-github-provenance-review.md)
- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`2026-03-24-llm-player-role-decomposition-plan.md`](2026-03-24-llm-player-role-decomposition-plan.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md`](2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Freeze the three remaining cross-cutting contracts that the later full recursive-improvement game still needs:

1. judge scorecard semantics,
2. player protocol schemas,
3. protected-vs-held-out evaluation separation.

# Core Decisions

1. `RSI-SCORE-1` should define the canonical episode scorecard over optimization targets, hard guardrails, regression anchors, and homeostasis-relevant comparisons.
2. `RSI-PROTO-1` should define canonical wire schemas for attacker observations and actions plus defender inputs and outputs.
3. `RSI-EVAL-1` should define which evaluation contexts and evidence rings are player-visible versus judge-only held-out.
4. These contracts should remain separate from:
   1. the broader game contract,
   2. the broader role contract,
   3. the broader methodology contract,
   4. and the later audit and provenance contract.

## Task 1: `RSI-SCORE-1`

### Canonical judge scorecard over targets, guardrails, anchors, and homeostasis inputs

Implementation note:

- `RSI-SCORE-1` is now landed in the machine-first game contract.
- `recursive_improvement_game_contract_v1.evaluator_scorecard` now explicitly partitions:
  - `optimization_targets`
  - `hard_guardrails`
  - `regression_inputs`
  - `diagnostic_contexts`
  - `comparison_contract`
- The first scorecard freezes:
  - numeric budget optimization for likely-human friction and suspicious-origin request, byte, and latency cost
  - category target achievement for canonical non-human posture outcomes
  - beneficial non-human no-harm as the current hard guardrail
  - representative adversary regression plus prior-window progress as regression inputs
  - explicit comparison inputs for rollback or retain and 10-cycle homeostasis
  - `scalarization=forbidden` so the judge cannot collapse these tradeoffs into one opaque scalar reward

**Files:**

- Modify: `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`
- Modify: `docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**

1. Define the canonical episode score vector for the independent judge.
2. Partition metric families into:
   1. optimization targets,
   2. hard no-harm gates,
   3. regression-anchor contexts,
   4. explanatory-only diagnostics.
3. Define how current-vs-prior comparisons feed:
   1. episode judgment,
   2. rollback or retain,
   3. homeostasis detection.
4. Make explicit that the judge does not collapse the system into one opaque scalar if that would hide tradeoffs between host-cost reduction and human or tolerated-traffic harm.

**Acceptance criteria:**

1. the repo has one canonical answer to "what exactly is the judge scoring in a recursive-improvement episode?",
2. optimization targets and hard guardrails are distinct,
3. and homeostasis inputs are derived from the same scorecard contract rather than separate later intuition.

## Task 2: `RSI-PROTO-1`

### Canonical attacker and defender protocol schemas under the independent judge

**Files:**

- Modify: `docs/plans/2026-03-24-llm-player-role-decomposition-plan.md`
- Modify: `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**

1. Define canonical observation and action schema families for the LLM attacker agent.
2. Define canonical input and output schema families for the LLM defender agent.
3. Define shared envelope semantics where useful:
   1. episode identifiers,
   2. trace or receipt identifiers,
   3. refusal or escalation statuses,
   4. provenance and version markers.
4. Keep role-specific meaning distinct:
   1. attacker actions are not defender proposals,
   2. defender refusals are not judge verdicts.

**Acceptance criteria:**

1. the repo has one canonical answer to "what shapes do the player-side messages and traces have?",
2. both player roles are protocol-complete enough for later implementation planning,
3. and the players remain subordinate to the judge contract rather than inventing their own result semantics.

Implementation note:

1. `RSI-PROTO-1` is now landed.
2. The canonical player protocol now distinguishes:
   1. shared envelope fields:
      1. `protocol_revision`
      2. `episode_id`
      3. `message_id`
      4. `parent_message_id`
      5. `trace_id`
      6. `turn_index`
      7. `role`
      8. `message_kind`
      9. `visibility_ring`
      10. `receipt_refs`
      11. `created_at`
   2. attacker observation families:
      1. `episode_init`
      2. `host_discovery`
      3. `tool_result`
      4. `request_receipt`
      5. `episode_terminal`
   3. attacker action families:
      1. `discover`
      2. `request_plan`
      3. `request_dispatch`
      4. `form_submit`
      5. `wait`
      6. `episode_finish`
      7. `episode_refusal`
   4. defender input families:
      1. `episode_context`
      2. `judge_snapshot`
      3. `move_guidance`
      4. `recent_episode_memory`
      5. `mutability_contract`
   5. defender output families:
      1. `config_proposal`
      2. `refusal`
      3. `need_more_evidence`
      4. `code_gap_escalation`
      5. `code_evolution_referral`
3. Role semantics remain distinct:
   1. attacker actions are not defender proposals,
   2. defender refusals are not judge verdicts,
   3. and neither player-side message family is allowed to redefine outcome truth.

## Task 3: `RSI-EVAL-1`

### Protected-vs-held-out evaluation separation for recursive-improvement episodes

**Files:**

- Modify: `docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`
- Modify: `docs/plans/2026-03-24-llm-player-role-decomposition-plan.md`
- Modify: `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**

1. Define the evaluation rings explicitly:
   1. player-visible protected evidence,
   2. judge-visible held-out evaluation contexts,
   3. regression-anchor contexts.
2. Define what the attacker may learn from or retain.
3. Define what the defender may plan against or see in episode history.
4. Define what remains withheld so the judge can detect overfitting and preserve real evaluation value.
5. Define how protected evidence, held-out evals, and regression anchors interact in later run-to-homeostasis episodes.

**Acceptance criteria:**

1. the repo has one canonical answer to "what can the players see versus what only the judge can use for evaluation?",
2. protected evidence is preserved without collapsing held-out evaluation into the training surface,
3. and later autonomy cannot quietly optimize against every benchmark context it is supposed to be judged by.

Implementation note:

1. `RSI-EVAL-1` is now landed.
2. The canonical evaluation rings are now:
   1. `player_visible_protected_evidence`
      1. current episode context and operator-selected target stance
      2. current non-held-out judge snapshot and move guidance
      3. replay-promoted or equivalently confirmed protected evidence
      4. bounded recent episode memory with held-out details redacted
      5. host-derived public attacker observations and action receipts
   2. `judge_held_out_evaluation`
      1. held-out benchmark contexts
      2. hidden adversary or replay cohorts reserved for scoring
      3. hidden human and tolerated-traffic no-harm checks
      4. withheld regression cases not exposed as player training inputs
   3. `regression_anchor_contexts`
      1. strict reference-stance contexts
      2. other judge-scored anchor suites that may emit summary verdicts without exposing raw case inventories
3. The contract now explicitly requires:
   1. players may learn from protected evidence,
   2. players must not receive the raw held-out context inventory,
   3. judge-side negative held-out results may override apparently positive player-visible trends,
   4. regression anchors remain mandatory score inputs without becoming player optimization surfaces.

# Sequencing

1. Keep the current mainline operator-facing work first: `MON-OVERHAUL-1`, `CTRL-SURFACE-1..3`, and `TUNE-SURFACE-1`.
2. Keep the broader judge and role decomposition first: `RSI-GAME-1A..1C`, `RSI-ROLES-1`, and the player-role planning.
3. Land `RSI-SCORE-1` before `OVR-AGENT-2B`, `OVR-AGENT-2C`, and `RSI-METH-1`.
4. Land `RSI-PROTO-1` before `SIM-LLM-1A` and `OVR-AGENT-2A` are treated as execution-ready.
5. Land `RSI-EVAL-1` before `SIM-LLM-1B`, `SIM-LLM-1C`, `OVR-AGENT-2B`, `OVR-AGENT-2C`, and `OVR-CODE-1` are treated as execution-ready.
6. Land `RSI-AUDIT-1` after these contracts so provenance can bind to settled scorecard, protocol, and evaluation revisions instead of inventing independent ids and meanings.

# Definition Of Done

This plan is complete when:

1. the judge scorecard is explicit,
2. the player wire protocols are explicit,
3. the protected-vs-held-out evaluation boundary is explicit,
4. and the later recursive-improvement path no longer depends on implicit protocol or scoring assumptions.
