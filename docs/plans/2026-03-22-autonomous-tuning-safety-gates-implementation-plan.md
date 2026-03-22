Date: 2026-03-22
Status: Proposed

Related context:

- [`../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`../research/2026-03-22-observed-traffic-taxonomy-and-sim-representativeness-review.md`](../research/2026-03-22-observed-traffic-taxonomy-and-sim-representativeness-review.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`2026-03-21-agent-first-loop-truth-completion-implementation-plan.md`](2026-03-21-agent-first-loop-truth-completion-implementation-plan.md)
- [`2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md`](2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`2026-03-20-benchmark-suite-v1-design.md`](2026-03-20-benchmark-suite-v1-design.md)
- [`2026-03-22-observed-traffic-taxonomy-and-lane-representativeness-plan.md`](2026-03-22-observed-traffic-taxonomy-and-lane-representativeness-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Goal

Reach the first truly closed autonomous tuning loop only after Shuma can prove that the loop is optimizing against protected, representative, category-aware non-human evidence rather than synthetic convenience traffic.

# Architecture

Do not create a second controller model.

Extend the existing chain:

1. `operator_objectives_v1`,
2. `operator_snapshot_v1`,
3. `benchmark_results_v1`,
4. replay-promotion lineage,
5. `allowed_actions_v1`,
6. recommend-only reconcile,
7. later canary apply and rollback,

with one new hard gate:

8. protected tuning evidence eligibility plus category coverage proof.

That gate now has an earlier prerequisite:

9. observed-traffic taxonomy and bounded category-classification confidence.

# Guardrails

1. `synthetic_traffic` must remain ineligible for autonomous tuning evidence.
2. Raw frontier or LLM findings must remain advisory until replay-promoted or otherwise deterministically confirmed into protected evidence.
3. Autonomous tuning must not optimize against a category-blind non-human aggregate.
4. The operator must be able to express category intent before auto-apply is allowed.
5. `MON-OVERHAUL-1` remains blocked until the closed loop is proven through backend behavior and live receipts.

# Execution Sequence

## Task 1: `TRAFFIC-TAX-1`

Codify the observed non-human traffic taxonomy.

Must include:

1. the bounded category set Shuma intends to optimize over,
2. explicit unknown and insufficient-evidence states,
3. machine-first visibility of the taxonomy basis used for later tuning.

## Task 2: `TRAFFIC-TAX-2`

Codify category-classification confidence and evidence receipts.

Must include:

1. bounded confidence metadata,
2. supporting evidence references,
3. fail-closed semantics when the categorization layer is too weak for tuning.

## Task 3: `SIM-PROTECTED-1`

Codify the protected tuning evidence model.

Must include:

1. explicit `tuning_eligible` or equivalent basis metadata on adversary evidence,
2. explicit exclusion of `synthetic_traffic` from autonomous tuning,
3. explicit rule that replay-promoted frontier or LLM lineage can count as protected evidence while raw frontier attempts remain advisory,
4. snapshot and benchmark visibility of the evidence basis used for any tuning recommendation or future auto-apply.

Acceptance:

1. later reconcile and apply logic can tell the difference between contract-test evidence and tuning-grade evidence,
2. the contract makes synthetic ineligibility machine-readable rather than prose-only.

## Task 4: `SIM-COVER-1`

Define and materialize the representativeness matrix and bounded coverage receipts.

Must include:

1. the protected non-human category taxonomy derived from observed traffic,
2. the fidelity dimensions that matter for defense tuning,
3. bounded receipts showing which categories are currently covered by Scrapling runtime traffic and replay-promoted frontier or LLM lineage,
4. explicit stale, unavailable, and partial-coverage states.

Acceptance:

1. the loop can answer whether the current protected evidence set covers the categories it intends to optimize,
2. autonomous tuning can fail closed when category coverage is incomplete.

## Task 5: `OPS-OBJECTIVES-3`

Extend operator objectives with category-aware non-human intent.

Must include:

1. desired versus tolerated versus unwanted category policy,
2. explicit operator exclusions or protections for beneficial categories,
3. objective revisioning and snapshot visibility.

Acceptance:

1. the controller has a truthful utility function for category-aware optimization,
2. beneficial or allowed non-human traffic is no longer only an implicit benchmark concern.

## Task 6: `OPS-BENCH-3`

Extend benchmark results with protected-lane eligibility and category-aware comparison.

Must include:

1. category-aware benchmark family rollups,
2. protected-lane coverage status,
3. explicit blockers when only synthetic or insufficiently representative evidence is available,
4. comparison semantics suitable for canary apply and rollback decisions.

Acceptance:

1. `benchmark_results_v1` can tell the controller not just whether metrics changed, but whether the evidence basis was safe enough to trust for auto-apply.

## Task 7: `OVR-APPLY-1`

Add the first closed tuning loop: canary apply, watch window, judge, and rollback.

Must include:

1. one bounded config-family apply at a time,
2. protected-lane and category-coverage prerequisite checks,
3. watch-window comparison against the protected baseline,
4. explicit rollback on degradation or missing evidence,
5. durable decision and outcome lineage.

Acceptance:

1. Shuma can safely move from recommend-only into bounded auto-apply,
2. the loop is proven on live shared-host infrastructure before Monitoring is redesigned around it.

# Exit Criteria

This plan is complete when:

1. synthetic traffic is impossible to mistake for tuning-grade evidence,
2. Shuma can prove which non-human categories are represented in protected evidence,
3. operator objectives are category-aware,
4. benchmark results can authorize or block auto-apply on that basis,
5. the first canary apply and rollback loop works on live shared-host proof,
6. and only then `MON-OVERHAUL-1` is reopened.
