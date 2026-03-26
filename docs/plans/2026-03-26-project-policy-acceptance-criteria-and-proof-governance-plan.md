Date: 2026-03-26
Status: Proposed

Related context:

- [`../research/2026-03-26-project-policy-acceptance-criteria-and-proof-governance-review.md`](../research/2026-03-26-project-policy-acceptance-criteria-and-proof-governance-review.md)
- [`2026-03-26-acceptance-gate-and-completion-claim-discipline-plan.md`](2026-03-26-acceptance-gate-and-completion-claim-discipline-plan.md)
- [`../../docs/project-principles.md`](../../docs/project-principles.md)
- [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md)
- [`../../AGENTS.md`](../../AGENTS.md)

# Objective

Codify explicit acceptance-criteria and proof-governance rules in Shuma's canonical project policy so planning, TODO writing, and completion claims all use the same rigorous standard.

# Core Decisions

1. The project must have an explicit organizational minimum Definition of Done and explicit tranche-level acceptance criteria.
2. Acceptance criteria must be outcome-focused, observable, and measurable rather than aspirational.
3. Every non-trivial TODO must name both the deliverable and the proof needed before closure.
4. Completion records must distinguish planning completion from shipped feature completion.
5. Missing or contradictory proof means the work stays open.

# Execution Shape

## Task 1: Strengthen project-wide principles

Update [`../../docs/project-principles.md`](../../docs/project-principles.md) so the project principles explicitly require:

1. acceptance criteria for non-trivial work,
2. proof surfaces and pass/fail verification,
3. and truthful completion claims.

## Task 2: Strengthen contributor workflow

Update [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md) so contributors are told, directly:

1. what must be written into plans and TODOs before implementation,
2. what must be cited before marking work complete,
3. and how planning-only work must be described.

## Task 3: Strengthen agent workflow

Update [`../../AGENTS.md`](../../AGENTS.md) so agents are required to:

1. define tranche acceptance criteria and proof plans during the planning chain,
2. write TODOs that include closure evidence,
3. and refuse completion claims when the acceptance bar is not fully proven.

## Task 4: Record the policy tranche

Update indexes and completion history so the new governance rule is discoverable and leaves an auditable paper trail.

# Definition Of Done

This policy tranche is satisfied when:

1. the three canonical policy docs all explicitly require acceptance criteria and proof-driven completion,
2. those docs distinguish shared Definition of Done from tranche-specific acceptance criteria,
3. they forbid treating planning completion as feature completion,
4. and the docs indexes and completion history reflect the new governance rule.
