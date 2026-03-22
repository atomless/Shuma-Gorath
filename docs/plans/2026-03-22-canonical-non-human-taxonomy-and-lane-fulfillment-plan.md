Date: 2026-03-22
Status: Proposed

Related context:

- [`../research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md`](../research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md)
- [`2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md`](2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Goal

Make the first autonomous tuning loop depend on:

1. a canonical non-human taxonomy defined by Shuma,
2. a shared classifier that can map both simulated and observed traffic into that taxonomy,
3. explicit Scrapling and frontier or containerized LLM lane behaviors designed to fulfill those categories,
4. explicit protected-evidence rules,
5. and a classification or fingerprinting layer that can improve over time without requiring early taxonomy churn.

# Guardrails

1. Do not define categories only from whatever traffic happened to arrive first.
2. Do not define representativeness against vague human judgment like "looks realistic."
3. Do not require every lane to cover every category independently.
4. Keep taxonomy, classification, and fulfillment contracts machine-first and bounded.
5. Preserve the existing rule that `synthetic_traffic` is tuning-ineligible.
6. Do not make taxonomy expansion part of the first closed-loop critical path.

# Execution Sequence

## Task 1: `TRAFFIC-TAX-1`

Define the canonical non-human traffic taxonomy.

Must include:

1. the bounded category set Shuma intends to optimize over before it has observed enough real adversary traffic,
2. the operator-relevant distinction between categories such as crawler, scraper, automated browser, agent-on-behalf, and verified or beneficial non-human traffic where applicable,
3. explicit unknown, mixed, and insufficient-evidence states,
4. machine-first visibility in snapshot or benchmark-adjacent contracts.

Acceptance:

1. Shuma can say what kinds of non-human traffic it intends to model and defend against,
2. and that statement is a backend contract, not only a dashboard interpretation.

## Task 2: `TRAFFIC-TAX-2`

Add the shared classification contract and confidence receipts.

Must include:

1. bounded confidence or exactness metadata per category assignment,
2. evidence references showing which signals supported the assignment,
3. the ability to classify both observed traffic and simulated traffic into the same category set,
4. stale, degraded, and unavailable states when categorization is weak,
5. failure-closed semantics for later tuning if the category layer is not trustworthy enough.
6. a design that allows fingerprinting signals, evidence weighting, and categorization quality to improve over time without changing the core category set by default.

Acceptance:

1. later benchmark and tuning logic can distinguish between "classified" and "best guess",
2. lane representativeness can be evaluated using the same classifier that later judges live traffic,
3. and category-assignment quality can improve over time without forcing category proliferation.

## Task 3: `SIM-FULFILL-1`

Implement the category-to-lane fulfillment matrix.

Must include:

1. explicit mapping of each target category to the lane or mode intended to fulfill it,
2. Scrapling modes for categories it can faithfully express,
3. frontier or containerized LLM driven browser or request modes where Scrapling is not the right fit,
4. explicit unresolved gaps when neither current lane can faithfully fulfill a category.

Acceptance:

1. Shuma can say which lane is intended to generate which non-human category before claiming representativeness.

## Task 4: `SIM-COVER-1`

Measure lane representativeness as fulfillment receipts against the canonical taxonomy.

Must include:

1. receipts showing which categories Scrapling generates and how they classify,
2. receipts showing which categories frontier or containerized LLM modes generate and how they classify,
3. the combined coverage view used by later autonomous tuning gates,
4. explicit gaps where neither lane currently represents a category well enough.

Acceptance:

1. the system can say not just that a lane ran, but which canonical categories it actually fulfills well enough for diagnosis and tuning.

## Task 5: `SIM-PROTECTED-1`

Codify protected tuning evidence eligibility on top of the taxonomy and fulfillment receipts.

Must include:

1. continued exclusion of `synthetic_traffic`,
2. lane evidence eligibility tied to taxonomy-backed coverage and fulfillment receipts,
3. explicit separation of advisory raw frontier findings from protected replay-promoted evidence.

## Task 6: `OPS-OBJECTIVES-3` and `OPS-BENCH-3`

Make operator objectives and benchmark judgment category-aware.

Must include:

1. operator stance by category,
2. benchmark comparison by category,
3. diagnosis surfaces that show how defenses perform and what cost they impose per simulated category,
4. tuning blockers when protected category coverage is incomplete,
5. classification-confidence and evidence lineage so objectives and benchmark comparisons stay interpretable as the classifier improves.

## Task 7: `OVR-APPLY-1`

Only after the above, add the first bounded canary apply and rollback loop.

# Exit Criteria

This plan is complete when:

1. Shuma has a canonical non-human taxonomy,
2. both simulated and observed traffic can be classified into it with bounded confidence,
3. Scrapling and frontier or containerized LLM lanes have explicit fulfillment paths for the target categories,
4. protected tuning evidence is category-backed rather than lane-asserted,
5. benchmark and objective contracts are category-aware,
6. classification quality can improve without invalidating that contract,
7. and only then autonomous tuning is allowed to move beyond recommend-only.
