Date: 2026-03-24
Status: Proposed

Related context:

- [`../research/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](../research/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md)
- [`../research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](../research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md)
- [`../research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](../research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
- [`../research/2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](../research/2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md)
- [`../research/2026-03-24-game-loop-audit-trail-and-github-provenance-review.md`](../research/2026-03-24-game-loop-audit-trail-and-github-provenance-review.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md)
- [`2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md`](2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md)
- [`2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md`](2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs)

# Objective

Define the missing contract that turns Shuma's bounded loop from a useful control mechanism into a later explicit recursive self-improvement game:

1. immutable rules,
2. fixed payoffs through a sacred evaluator,
3. bounded legal move set,
4. shortfall-attribution and move-selection policy,
5. episode archive and stepping-stone memory,
6. and later run-to-homeostasis episode control.

Put more simply, the later contract should make it explicit that Shuma's recursive-improvement game has:

1. fixed rules,
2. fixed payoffs,
3. legal moves,
4. an independent judge,
5. and memory of prior episodes.

# Core Decisions

1. `operator_objectives_v1` remains the rule set and must never be loop-mutable.
2. The controller-mutability policy defines the legal move ring; admin writability alone is not game legality.
3. The machine-first benchmark stack remains the evaluator and must not be re-authored by the mutator.
4. Benchmark misses must not map straight to arbitrary patch families; Shuma needs an explicit intermediate move-selection policy.
5. Later recursive-improvement episodes should search over a bounded archive of outcomes, not just the latest config state.
6. Later recursive-improvement architecture should be modeled as attacker agent, defender agent, and independent judge rather than only two agent roles.
7. The earlier reference-stance and run-to-homeostasis methodology should be implemented as part of this game contract, not separately from it.
8. Later execution-ready player planning should consume separate contracts for judge score semantics, player protocol schemas, and held-out evaluation separation rather than letting those details remain implicit inside the broader game contract.
9. Episode memory alone is not enough; later recursive-improvement phases also need a canonical audit and provenance contract that links judge outcomes to config and later GitHub-backed code lineage.

## Task 0: Focused Verification Prep

**Files:**

- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**

1. Add a focused proof target for the game contract surface, for example `test-rsi-game-contract`.
2. Add a focused proof target for shortfall-attribution and move-selection policy behavior, for example `test-oversight-move-selection-policy`.
3. Add a focused proof target for episode-history or stepping-stone archive behavior once that work lands, for example `test-oversight-episode-archive`.

**Acceptance criteria:**

1. the later game-contract, move-selection, and episode-history work each have narrow truthful proof paths before implementation broadens.

## Task 1: `RSI-GAME-1A`

### Canonical recursive-improvement game contract

**Files:**

- Modify: `src/observability/operator_snapshot_objectives.rs`
- Modify: `src/config/controller_action_surface.rs`
- Modify: `src/admin/oversight_api.rs`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`

**Work:**

1. Define one explicit game-contract layer over the existing machine-first loop contracts.
2. Make the contract name the distinct roles of:
   1. immutable rules,
   2. evaluator scorecard,
   3. legal move ring,
   4. safety gates,
   5. regression anchors.
3. Make `operator_objectives_v1` and the canonical controller-mutability policy the formal rule boundary.
4. Make the evaluator surface explicit enough that later recursive phases cannot silently redefine the scorecard.
5. Ensure operator-facing surfaces later consume this contract instead of inferring the game structure from scattered current payloads.

**Acceptance criteria:**

1. there is one canonical answer to "what game is Shuma playing?",
2. the immutable rules, evaluator, and legal move ring are separate and explicit,
3. and later recursive-improvement phases can consume one formal game contract rather than reconstructing it from scattered modules.

**Verification:**

1. `make test-rsi-game-contract`
2. `git diff --check`

## Task 2: `RSI-GAME-1B`

### Shortfall-attribution and move-selection policy

**Files:**

- Modify: `src/observability/benchmark_results_comparison.rs`
- Modify: `src/admin/oversight_reconcile.rs`
- Modify: `src/admin/oversight_patch_policy.rs`
- Modify: `src/config/controller_action_catalog.rs`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`

**Work:**

1. Replace the current coarse benchmark-family to patch-family bridge with a ratified intermediate move-selection policy.
2. Make the policy explicit about:
   1. which shortfall patterns create which problem classes,
   2. which action families are eligible for each problem class,
   3. which direction of change is expected,
   4. what human-friction or tolerated-traffic risk each family carries,
   5. and when evidence is too weak to justify any move.
3. Classify shortfall-to-change tractability explicitly:
   1. exact bounded config moves,
   2. family-level bounded policy choice,
   3. and code or capability gaps.
4. Remove candidate-action mappings that violate the canonical mutability policy, including any benchmark-family bridge that still points to operator-owned or hard-never surfaces.
5. Preserve metric-level detail long enough that later move selection is more specific than the current `ReduceLikelyHumanFriction` versus `ReduceSuspiciousOriginCost` collapse.
6. Introduce explicit status for:
   1. exact move guidance,
   2. bounded heuristic guidance,
   3. insufficient evidence,
   4. and code-evolution-only gaps.

**Acceptance criteria:**

1. benchmark shortfalls no longer imply a move only through broad static family priorities,
2. the move-selection policy is explicit and reviewable,
3. and the loop can explain why a given shortfall justifies a given bounded move family.

**Verification:**

1. `make test-oversight-move-selection-policy`
2. `make test-oversight-reconcile`
3. `make test-controller-action-surface-parity`
4. `git diff --check`

## Task 3: `RSI-GAME-1C`

### Episode archive, stepping-stone memory, and progress-to-homeostasis inputs

**Files:**

- Modify: `src/admin/oversight_api.rs`
- Modify: `src/admin/oversight_agent.rs`
- Modify: `src/observability/benchmark_comparison.rs`
- Modify: `src/observability/operator_snapshot.rs`
- Modify: `docs/api.md`
- Modify: `docs/dashboard.md`

**Work:**

1. Persist a bounded episode archive over completed loop cycles.
2. Record at least:
   1. target stance or evaluation context,
   2. baseline scorecard,
   3. proposed move,
   4. accepted or refused status,
   5. watch-window result,
   6. rollback or retain status,
   7. benchmark deltas,
   8. and hard-guardrail triggers.
3. Make recent accepted and rejected episodes available as the stepping-stone substrate for later recursive-improvement phases.
4. Make the archive machine-first first; later Monitoring should project it, but the archive must not begin as a UI-only concept.
5. Feed the last 10 completed cycle judgments into the later run-to-homeostasis detector described in the reference-stance methodology.
6. Preserve stable episode and proposal identifiers plus provenance refs so the later audit contract can link current config episodes and later code-evolution lineage to the same machine-first history.

**Acceptance criteria:**

1. later recursive-improvement phases can reason over a bounded archive of actual move outcomes,
2. homeostasis no longer depends on vague trend reading,
3. and Monitoring can eventually project recent loop progress against the same episode history the controller uses.

**Verification:**

1. `make test-oversight-episode-archive`
2. `make test-oversight-apply`
3. `git diff --check`

## Task 4: `RSI-ROLES-1`

### Attacker, defender, and independent-judge role contract for later autonomous phases

**Files:**

- Modify: `docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`
- Modify: `todos/blocked-todo.md`
- Modify: later `OVR-AGENT-2` and `OVR-CODE-1` planning when reopened

**Work:**

1. Define the formal roles of:
   1. LLM-backed attacker agent in the sim harness,
   2. LLM-backed defender agent over bounded moves,
   3. and the machine-first benchmark stack as the independent judge.
2. Make `RSI-METH-1`, `OVR-AGENT-2`, and `OVR-CODE-1` consume that triadic role contract rather than float as self-defining later phases.
3. Make Monitoring remain the human-readable projection of the independent judge rather than one of the players.
4. Make later code evolution optimize permissive target stances only while continuing to pass the strict reference stance as a regression anchor.
5. Ensure later autonomy cannot widen the rules, evaluator, or move set by role confusion or implicit self-judging.

**Acceptance criteria:**

1. `RSI-METH-1`, `OVR-AGENT-2`, and `OVR-CODE-1` all inherit one shared attacker/defender/judge role contract,
2. the judge remains independent of both later agent roles,
3. Monitoring remains the human-readable projection of that judge,
4. and later autonomy cannot widen the rules, evaluator, or move set by implication.

# Sequencing

1. Finish `CTRL-SURFACE-1..3` before `RSI-GAME-1A`.
2. Keep `MON-OVERHAUL-1` and `TUNE-SURFACE-1A` on the current operator-facing path; do not block those surfaces on later recursive-game machinery.
3. Land `RSI-GAME-1A` and `RSI-GAME-1B` before reopening `OVR-AGENT-2`.
4. Land `RSI-GAME-1C` before implementing `RSI-METH-1`.
5. Land `RSI-ROLES-1` before any later dual-agent or triadic autonomous-loop design is treated as execution-ready.
6. Reopen `OVR-CODE-1` only after `OVR-AGENT-2`, the game contract, and the strict-reference regression-anchor expectations are all explicit.
7. Land `RSI-SCORE-1`, `RSI-PROTO-1`, and `RSI-EVAL-1` before any later player-side runtime planning is treated as protocol-complete.
8. Land `RSI-AUDIT-1` before later autonomous defender runtime work is treated as operationally accountable, and before `OVR-CODE-1` is treated as execution-ready.

# Definition Of Done

This plan is complete when:

1. Shuma has one canonical recursive-improvement game contract,
2. benchmark shortfalls flow through an explicit move-selection policy instead of only coarse pressure heuristics,
3. a bounded episode archive exists for later stepping-stone search and homeostasis detection,
4. and the later recursive-improvement phases consume that shared contract instead of reconstructing their own implicit rules.
