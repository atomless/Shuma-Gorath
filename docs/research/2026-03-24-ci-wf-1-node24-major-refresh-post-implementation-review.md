Date: 2026-03-24
Status: Completed

Related implementation:

- [`../../.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
- [`../../.github/workflows/adversarial-soak.yml`](../../.github/workflows/adversarial-soak.yml)
- [`../../.github/workflows/dashboard-e2e.yml`](../../.github/workflows/dashboard-e2e.yml)
- [`../../.github/workflows/release-gate.yml`](../../.github/workflows/release-gate.yml)
- [`../../.github/workflows/coverage.yml`](../../.github/workflows/coverage.yml)
- [`../../.github/workflows/codeql.yml`](../../.github/workflows/codeql.yml)
- [`../../scripts/tests/test_github_workflow_node24_majors.py`](../../scripts/tests/test_github_workflow_node24_majors.py)
- [`../../Makefile`](../../Makefile)

# CI-WF-1 Node24 Major Refresh Post-Implementation Review

## What landed

`CI-WF-1` now refreshes the repo workflows off the older Node 20-backed action majors:

1. `actions/checkout@v4` -> `actions/checkout@v5`
2. `actions/setup-node@v4` -> `actions/setup-node@v5`
3. `actions/upload-artifact@v4` -> `actions/upload-artifact@v6`

It also adds a focused static contract lane in [`scripts/tests/test_github_workflow_node24_majors.py`](../../scripts/tests/test_github_workflow_node24_majors.py), exposed through `make test-github-workflow-node24-majors`, so future workflow drift back to the older majors fails locally.

## Why this closes the tranche

The backlog item was specifically about getting off the older Node 20-backed majors before hosted-runner deprecations become emergency maintenance. This slice does that with the smallest honest change set:

1. the workflow topology is unchanged,
2. every relevant workflow pin is refreshed consistently,
3. and the repo now has an explicit local proof for that version contract.

## Verification evidence

- `make test-github-workflow-node24-majors`
- `git diff --check`

## Hosted CI status truth

The local static contract is green, but hosted GitHub Actions proof for this branch still depends on actual workflow runs. Some workflows do not trigger automatically on feature-branch pushes, so hosted-runner proof may remain pending or unavailable until the workflows are triggered through their configured GitHub events.
