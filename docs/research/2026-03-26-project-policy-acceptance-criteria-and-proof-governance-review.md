Date: 2026-03-26
Status: Completed

# Project Policy Acceptance-Criteria And Proof Governance Review

## Question

How should Shuma's canonical project policy guide planning, TODO writing, and completion so acceptance criteria are explicit and completion claims are backed by rigorous proof?

## External guidance

Two external references are especially relevant here:

1. The Scrum Guide says the Definition of Done is a formal description of the state of work when it meets the quality measures required for the product, and that work cannot be considered part of the increment unless it meets that standard.
2. Atlassian's acceptance-criteria guidance says acceptance criteria should define what success looks like, be clear and concise, and map cleanly to objectively verifiable tests and measurable pass/fail outcomes.

Those sources align closely with Shuma's current failure mode: we need both a shared organizational minimum for "done" and tranche-specific acceptance criteria that are observable, testable, and hard to overstate.

## Repo-specific gap

Shuma already has strong verification language in [`../../AGENTS.md`](../../AGENTS.md), but the policy is still too implicit in three places:

1. plan docs do not yet have a repo-wide requirement to define explicit acceptance criteria and proof surfaces for every non-trivial tranche,
2. TODO items do not yet have a repo-wide requirement to record the closure evidence needed before they may move to completed history,
3. completion guidance does not yet explicitly forbid describing planning completion or baseline capability as delivered feature closure.

## Decision

The canonical policy docs should say, explicitly:

1. every non-trivial plan or tranche must define acceptance criteria before implementation begins,
2. those criteria must be observable and measurable, with explicit proof surfaces and verification commands where applicable,
3. TODO items must state both the deliverable and the proof required for closure,
4. completion records must distinguish planning-only completion from shipped behavior,
5. and no work may be called complete while its defined acceptance proof is missing, contradictory, or still flaky.

## Implementation consequence

This should be codified in:

1. [`../../docs/project-principles.md`](../../docs/project-principles.md) as project-wide policy,
2. [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md) as contributor workflow,
3. [`../../AGENTS.md`](../../AGENTS.md) as agent execution discipline.
