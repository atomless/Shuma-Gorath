Date: 2026-03-24
Status: Proposed

Related context:

- [`../research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](../research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md)
- [`../research/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](../research/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md)
- [`../research/2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](../research/2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md)
- [`../research/2026-03-24-game-loop-audit-trail-and-github-provenance-review.md`](../research/2026-03-24-game-loop-audit-trail-and-github-provenance-review.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md)
- [`2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md`](2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md)
- [`2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Carry the reference-stance and run-to-homeostasis methodology into the later recursive-improvement phases without prematurely broadening the current bounded config loop.

# Core Decisions

1. The development reference stance is a later controller-program choice, not a product-default change.
2. The first reference stance should be `Human-only / private`.
3. Later recursive-improvement runs should execute as bounded episodes that continue until homeostasis rather than as single-shot recommendations.
4. Relaxed preset sweeps should be added only after the strict reference stance has already proven useful.
5. Later code evolution must continue to pass the strict reference stance as a regression anchor.
6. Homeostasis inputs and later episode judgment must come from the canonical judge scorecard and held-out-evaluation boundaries rather than from agent-defined heuristics.

# Execution Shape

## `RSI-METH-1`: Recursive-improvement methodology contract

This later methodology slice should land before or together with execution-ready `OVR-AGENT-2` planning.

It should consume the canonical recursive-improvement game contract from [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md) rather than defining its own implicit rules, evaluator, or move-selection surface.

It should also consume the later scorecard and evaluation-separation contracts from [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md) so homeostasis and preset sweeps are judged through the same canonical score semantics and held-out contexts as the broader game.

It should define:

1. the development reference stance identifier,
2. the episode lifecycle states,
3. the homeostasis detector inputs,
4. the preset sweep regimen,
5. the regression-anchor obligations for code change proposals,
6. and the assumption that later run-to-homeostasis execution operates over an attacker/defender/judge game contract rather than a two-agent duel.

## Episode lifecycle

Later recursive-improvement runs should no longer be modeled as single reconcile invocations only.

They should support a bounded episode contract:

1. initialize target stance,
2. collect baseline,
3. iterate candidate cycles,
4. record each completed watch-window judgment,
5. continue while eligible and still improving,
6. stop on target reached, homeostasis, or hard guardrail.

## Homeostasis detector

The later loop should add a rolling detector over the last 10 completed cycles.

Initial required behavior:

1. consider only completed cycles with watch-window judgment,
2. derive improvement or regression relative to baseline or prior accepted state,
3. use a confidence-aware threshold rather than raw deltas,
4. classify recent behavior as `improving`, `mixed`, or `homeostasis`.

This detector should initially remain machine-first and operator-visible in Monitoring rather than hidden inside agent prose.

Its inputs should be drawn from the canonical judge scorecard, and later withheld evaluation contexts must remain capable of overruling apparently positive player-visible trends when held-out evidence shows overfitting or hidden harm.

## Preset sweep regimen

After the strict reference stance stabilizes, later recursive-improvement planning should add a small preset sweep set.

Initial sweep candidates:

1. `Search-visible, AI-restricted`
2. `Agent-friendly, scraper-hostile`
3. `General public website` or equivalent renamed balanced public-web preset

The sweep should:

1. run as a secondary regimen after strict reference stance work,
2. record whether suggested config and later code changes transfer cleanly,
3. and keep the strict reference stance as the base benchmark.

## Code-evolution regression anchor

When `OVR-CODE-1` is reopened, it should require:

1. success on the target relaxed stance,
2. and no unacceptable regression on the strict reference stance.

This should be explicit in both planning and benchmark acceptance criteria.

# Backlog Integration

1. Update `OVR-AGENT-2` planning to consume the reference-stance and run-to-homeostasis contract.
2. Update `OVR-CODE-1` planning to require strict-reference-stance regression proof.
3. Make both later phases also consume the canonical recursive-improvement game contract and move-selection policy.
4. Make the methodology consume `RSI-SCORE-1` and `RSI-EVAL-1` so reference-stance episodes do not drift into player-defined scoring or fully visible evaluation surfaces.
5. Make later code-evolution execution also consume `RSI-AUDIT-1` so strict-reference regression anchors link cleanly to GitHub PR, merge, and revert lineage rather than only prose.
6. Keep this methodology blocked until the broader later controller phases are reopened; do not retrofit the current proven bounded config loop into an indefinite autonomous runner prematurely.

Current note:

1. `RSI-SCORE-1` is now landed.
2. Homeostasis inputs should therefore be taken from the explicit `comparison_contract.homeostasis_input_ids` surface in `recursive_improvement_game_contract_v1.evaluator_scorecard` rather than improvised later per-agent heuristics.

# Definition Of Done

This planning tranche is satisfied when:

1. the later controller phases explicitly distinguish development reference stance from product stances,
2. homeostasis is defined as a formal stopping rule over recent completed cycles,
3. relaxed preset sweeps are sequenced after strict-stance stabilization,
4. and later code evolution is bound to the strict reference stance as a regression anchor.
