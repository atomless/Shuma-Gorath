# CI-WF-1 Node24 Action Refresh Post-Implementation Review

Date: 2026-03-24

## What landed

`CI-WF-1` now refreshes the repo's official JavaScript GitHub Actions off the
Node20-backed majors in the workflow files under
[`/.github/workflows/`](../../.github/workflows):

1. `actions/checkout@v4` -> `actions/checkout@v6`
2. `actions/setup-node@v4` -> `actions/setup-node@v6`
3. `actions/upload-artifact@v4` -> `actions/upload-artifact@v7`

It also adds a focused local contract test in:

- [`scripts/tests/test_ci_workflow_action_versions.py`](../../scripts/tests/test_ci_workflow_action_versions.py)

exposed through:

- `make test-ci-workflow-action-versions`

so the repo now fails fast if those workflow files drift back to the older
majors.

## Why this tranche mattered

The active backlog explicitly called out the Node20-backed official action
majors as upcoming maintenance debt ahead of the hosted-runner cutoff. This
repo had those older majors pinned across the main CI, release-gate, dashboard,
coverage, soak, and CodeQL workflows.

This tranche makes the workflow pins consistent with the current Node24
transition path and turns that requirement into a repo-local contract.

## Verification

The tranche was verified with:

- `make test-ci-workflow-action-versions`
- `git diff --check`

## Assessment

The delivered change is deliberately narrow:

1. only the official GitHub JavaScript actions called out by the backlog were
   updated,
2. the repo now has a focused local proof path for those version pins,
3. workflow execution behavior still needs hosted-runner confirmation after
   push, but the static version drift is removed.

## Follow-on

After the push, the relevant GitHub Actions workflows should be watched to
confirm there are no runner-compatibility or workflow-behavior regressions on
hosted infrastructure. If a later official major supersedes these Node24-backed
pins, the same local contract should be updated rather than allowing silent
workflow drift.
