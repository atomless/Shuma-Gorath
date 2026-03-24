# Testing Surface Rationalization Plan

Date: 2026-03-23

## Goal

Make the test surface easier to trust by separating behavior proof from source-contract proof, clarifying which targets cover current live operational functionality, and reducing routine verification churn.

## Tranches

### TEST-TIER-1: Canonical test-tier and target-scope truthfulness

Objective:

- define the repo's canonical test tiers in docs and `Makefile` help text,
- refresh stale descriptions,
- and make it obvious which targets are pre-merge/local versus live operational proofs.

Scope:

- classify targets into:
  - static and source-contract checks,
  - local behavior tests,
  - Spin runtime integration tests,
  - rendered dashboard tests,
  - live operational proofs.
- refresh stale descriptions such as the integration scenario count.
- document explicitly that `make test` is the canonical local/CI pre-merge suite, while live shared-host proof remains a separate operational tier.

Verification:

- docs-only tranche unless `Makefile` help text changes.
- `git diff --check`

### TEST-HYGIENE-6A: Replace dashboard runtime and monitoring archaeology with behavior proof

Status:

- delivered on 2026-03-24

Objective:

- replace the audit-cited dashboard Node source-regex tests with real module or runtime behavior proof,
- while leaving explicit source-contract lanes intact where file shape is itself the contract.

Scope:

- replace native-runtime source regex ownership checks with behavior proof for session restore, tab normalization, and config-mutation invalidation,
- replace refresh-runtime source regex ownership checks with behavior proof for cache clearing and freshness reset,
- replace Monitoring accountability wiring regex checks with behavior proof that the refresh runtime materializes benchmark and oversight snapshots into store state,
- add a truthful focused Make target for the new non-rendered runtime behavior checks,
- remove unrelated source-regex checks from focused pane targets whose names no longer match their real blast radius.

Verification:

- `make test-dashboard-runtime-unit-contracts`
- `make test-dashboard-monitoring-accountability`
- `make test-dashboard-verified-identity-pane`
- `git diff --check`

### TEST-HYGIENE-6B: Replace or reclassify shell-wrapper archaeology outside explicit contract lanes

Objective:

- reduce low-signal source archaeology that currently sits inside feature-proof targets,
- while preserving genuinely useful target-truthfulness guards where structure itself is the contract.

Scope:

- audit and improve shell-script and Makefile selector tests including:
  - supervisor wrapper tests,
  - integration cleanup tests.
- where source-shape checks remain necessary, move or rename them into explicit `contract` or `wiring` lanes rather than feature-behavior lanes.

Verification:

- focused `make` targets per converted area.
- avoid broad `make test` reruns unless the touched surface genuinely needs it.

### TEST-HYGIENE-6C: Reclassify feature-specific Makefile selector microtests into explicit contract lanes

Objective:

- keep genuinely useful target-truthfulness guards,
- while making sure feature-oriented proof targets do not quietly rely on selector-only or shell-text-only microtests.

Scope:

- audit small Python tests such as verified-identity, host-impact, and adversary-sim make-target selector checks,
- move retained selector checks into explicit `contract` or `wiring` lanes,
- refresh Make help and docs when target names or scope statements need to become more truthful.

Verification:

- focused `make` targets for the touched selector or wiring lanes
- `git diff --check`

### TEST-HYGIENE-2: Keep full-suite verification worktree-clean

Objective:

- finish the already queued cleanup that stops `make test` from rewriting tracked adversarial and SIM2 artifacts.

Scope:

- move generated receipts out of tracked fixture paths or make them reproducible without worktree churn.
- keep `git diff` clean after routine full-suite execution.

Verification:

- targeted adversarial/SIM2 `make` paths plus `git diff --check`.

## Proposed Execution Order

1. `TEST-TIER-1`
2. `TEST-HYGIENE-6A` (delivered)
3. `TEST-HYGIENE-6B`
4. `TEST-HYGIENE-6C`
5. existing `TEST-HYGIENE-3`
6. existing `TEST-HYGIENE-4`
7. existing `TEST-HYGIENE-5`
8. existing `TEST-HYGIENE-2`

## Notes

- Do not remove source-contract tests indiscriminately. Keep them when the contract really is file shape, target wiring, or script composition.
- Do not weaken the current live proof tier. The aim is to distinguish it more clearly, not to demote or discard it.
- Prefer rendered and subprocess proofs over source regexes whenever the behavior can be driven directly.
