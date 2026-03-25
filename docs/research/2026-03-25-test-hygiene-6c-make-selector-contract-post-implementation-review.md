# TEST-HYGIENE-6C Post-Implementation Review

Date: 2026-03-25
Status: Completed

Related plan:

- [`../plans/2026-03-25-test-hygiene-6c-make-selector-contract-plan.md`](../plans/2026-03-25-test-hygiene-6c-make-selector-contract-plan.md)
- [`2026-03-25-test-hygiene-6c-make-selector-contract-review.md`](2026-03-25-test-hygiene-6c-make-selector-contract-review.md)

## What landed

`TEST-HYGIENE-6C` is now delivered as an explicit target-truthfulness cleanup slice.

The repo now separates feature-behavior proof from Makefile selector proof in the three families that were still mixed:

1. adversary-sim
2. verified identity
3. host impact

Each family now has its own explicit contract lane:

1. `make test-adversary-sim-make-target-contract`
2. `make test-verified-identity-make-target-contract`
3. `make test-host-impact-make-target-contract`

The feature-behavior targets no longer hide selector-only Python microtests inside their execution path, and the retained selector suites were updated so they point at those explicit contract lanes instead of enforcing the old mixed structure.

## Verification

- `make test-adversary-sim-make-target-contract`
- `make test-verified-identity-make-target-contract`
- `make test-host-impact-make-target-contract`
- `make test-adversary-sim-scrapling-category-fit`
- `make test-verified-identity-guardrails`
- `make test-oversight-host-impact`
- `make test-make-selector-contract-targets`
- `git diff --check`

## Outcome Against Plan

The tranche met the planned requirements:

1. a focused split-contract test now proves the explicit lane structure,
2. Makefile target names and blast radius are more truthful,
3. selector-only proof is no longer hidden inside behavior-oriented feature targets,
4. and the affected behavior targets still pass after the selector microtests were removed from them.

## Remaining Gap

The selector-microtest cleanup track is now complete.

The broader testing-hygiene follow-ons that remain are separate dashboard or behavior tranches:

1. `TEST-HYGIENE-3`
2. `TEST-HYGIENE-4`
3. `TEST-HYGIENE-5`
