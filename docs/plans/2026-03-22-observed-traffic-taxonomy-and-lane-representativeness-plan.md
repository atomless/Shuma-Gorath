Date: 2026-03-22
Status: Proposed

Related context:

- [`../research/2026-03-22-observed-traffic-taxonomy-and-sim-representativeness-review.md`](../research/2026-03-22-observed-traffic-taxonomy-and-sim-representativeness-review.md)
- [`2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md`](2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Goal

Make the first autonomous tuning loop depend on Shuma's own observed-traffic taxonomy and classification confidence before it depends on any claim about Scrapling or frontier or LLM lane realism.

# Guardrails

1. Do not define representativeness against vague human judgment like "looks realistic."
2. Do not require every lane to cover every category independently.
3. Keep taxonomy and confidence contracts machine-first and bounded.
4. Preserve the existing rule that `synthetic_traffic` is tuning-ineligible.

# Execution Sequence

## Task 1: `TRAFFIC-TAX-1`

Define the observed non-human traffic taxonomy.

Must include:

1. the bounded category set Shuma intends to optimize over,
2. the mapping from observed signals and verified-identity or provenance inputs into that taxonomy,
3. explicit unknown, mixed, and insufficient-evidence states,
4. machine-first visibility in snapshot or benchmark-adjacent contracts.

Acceptance:

1. Shuma can say what kinds of non-human traffic it believes it is seeing,
2. and that statement is a backend contract, not only a dashboard interpretation.

## Task 2: `TRAFFIC-TAX-2`

Add classification confidence and evidence receipts.

Must include:

1. bounded confidence or exactness metadata per category assignment,
2. evidence references showing which signals supported the assignment,
3. stale, degraded, and unavailable states when categorization is weak,
4. failure-closed semantics for later tuning if the category layer is not trustworthy enough.

Acceptance:

1. later benchmark and tuning logic can distinguish between "classified" and "best guess."

## Task 3: `SIM-COVER-1`

Redefine lane representativeness as joint coverage against the observed taxonomy.

Must include:

1. coverage receipts showing which categories Scrapling covers,
2. coverage receipts showing which categories replay-promoted frontier or LLM lineage covers,
3. the combined coverage view used by later autonomous tuning gates,
4. explicit gaps where neither lane currently represents a category well enough.

Acceptance:

1. the system can say not just that a lane ran, but which observed categories it represents well enough for tuning.

## Task 4: `SIM-PROTECTED-1`

Codify protected tuning evidence eligibility on top of the taxonomy and coverage receipts.

Must include:

1. continued exclusion of `synthetic_traffic`,
2. lane evidence eligibility tied to taxonomy-backed coverage,
3. explicit separation of advisory raw frontier findings from protected replay-promoted evidence.

## Task 5: `OPS-OBJECTIVES-3` and `OPS-BENCH-3`

Make operator objectives and benchmark judgment category-aware.

Must include:

1. operator stance by category,
2. benchmark comparison by category,
3. tuning blockers when protected category coverage is incomplete.

## Task 6: `OVR-APPLY-1`

Only after the above, add the first bounded canary apply and rollback loop.

# Exit Criteria

This plan is complete when:

1. Shuma can categorize observed non-human traffic with bounded confidence,
2. Scrapling and frontier or LLM coverage is measured against that taxonomy,
3. protected tuning evidence is category-backed rather than lane-asserted,
4. benchmark and objective contracts are category-aware,
5. and only then autonomous tuning is allowed to move beyond recommend-only.
