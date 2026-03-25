Date: 2026-03-24
Status: Proposed

Related context:

- [`../../.github/workflows/ci.yml`](../../.github/workflows/ci.yml)
- [`../../.github/workflows/adversarial-soak.yml`](../../.github/workflows/adversarial-soak.yml)
- [`../../.github/workflows/dashboard-e2e.yml`](../../.github/workflows/dashboard-e2e.yml)
- [`../../.github/workflows/release-gate.yml`](../../.github/workflows/release-gate.yml)
- [`../../.github/workflows/coverage.yml`](../../.github/workflows/coverage.yml)
- [`../../.github/workflows/codeql.yml`](../../.github/workflows/codeql.yml)
- [actions/checkout releases](https://github.com/actions/checkout/releases)
- [actions/setup-node releases](https://github.com/actions/setup-node/releases)
- [actions/upload-artifact releases](https://github.com/actions/upload-artifact/releases)

# GitHub Actions Node24 Major Refresh Review

## Question

How should Shuma refresh its GitHub Actions workflow dependencies off the older Node 20-backed majors without widening into a risky workflow redesign?

## Current repo state

The repo still pins:

1. `actions/checkout@v4`
2. `actions/setup-node@v4`
3. `actions/upload-artifact@v4`

across the current hosted workflows in `.github/workflows/`.

That matches the open `CI-WF-1` backlog item, but the task wording is now slightly behind the upstream release state.

## What upstream says now

The official release notes show:

1. `actions/checkout@v5` is the first Node 24-backed checkout major and requires runner `v2.327.1` or newer.
2. `actions/setup-node@v5` is the first Node 24-backed setup-node major and also requires runner `v2.327.1` or newer.
3. `actions/upload-artifact@v6` is the first upload-artifact major that defaults to Node 24 and requires runner `v2.327.1` or newer.

There are later majors upstream too:

- `actions/checkout@v6`
- `actions/setup-node@v6`
- `actions/upload-artifact@v7`

But the first Node 24-safe majors are enough to satisfy the hosted-runner deprecation problem.

## Recommended upgrade choice

Use the first Node 24-safe majors, not the newest available majors:

1. `actions/checkout@v5`
2. `actions/setup-node@v5`
3. `actions/upload-artifact@v6`

Why this is the cleanest choice:

1. it solves the Node 20-backed major problem directly,
2. it minimizes workflow behavior drift,
3. it avoids pulling in unrelated later-major behavior changes unless Shuma actually needs them,
4. and it still gives the repo an explicit static contract lane that can prevent future regression to deprecated majors.

## Recommended proof shape

Because most of these workflows do not run automatically on feature-branch pushes, the local proof should be:

1. a focused static workflow-contract test that scans every workflow for the expected action majors,
2. a Makefile target that owns that proof,
3. and a truthful completion note that local contract proof is green while hosted GitHub Actions runs on this branch may still be pending or unavailable.

That is better than pretending a hosted run happened when it did not.

## Result

`CI-WF-1` should land as a narrow workflow-contract tranche:

1. refresh all workflow pins to `checkout@v5`, `setup-node@v5`, and `upload-artifact@v6`,
2. add a focused static workflow-version contract lane,
3. update docs and backlog,
4. and report hosted CI status truthfully rather than overselling local verification.
