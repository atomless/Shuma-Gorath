# SIM2 Plan 2: Coverage Contract Governance

Date: 2026-02-27  
Status: Proposed

Reference research:

- [`docs/research/2026-02-27-sim2-shortfall-2-coverage-contract-governance.md`](../research/2026-02-27-sim2-shortfall-2-coverage-contract-governance.md)

## Objective

Make SIM2 full-coverage requirements immutable and drift-resistant by moving from profile-local convention to canonical contract enforcement.

## Non-goals

1. Replacing scenario manifest format.
2. Broad redesign of all quantitative gates.

## Architecture Decisions

1. Add canonical coverage contract artifact (`coverage_contract.v1.json`) as source of truth.
2. Require `full_coverage` manifest gate requirements to satisfy canonical contract exactly (or explicitly supersede with versioned contract update).
3. Emit coverage contract version/hash in reports for auditability.
4. Add drift checks across plan docs, manifest, and runner validation.

## Delivery Phases

### Phase 1: Canonical Contract Definition

1. Define canonical coverage categories and minimum evidence values.
2. Include explicit rows for tarpit progression and event-stream minimums.

Acceptance criteria:

1. Contract file is versioned and validated in `make test-adversarial-manifest`.
2. All categories required by SIM2 plan are represented.

### Phase 2: Runner and Manifest Enforcement

1. Update validation so `full_coverage` cannot omit canonical categories.
2. Fail gate evaluation on category mismatch, missing keys, or below-minimum values.

Acceptance criteria:

1. Coverage mismatch produces explicit per-key diagnostics.
2. `coverage_gates` output includes canonical contract reference.

### Phase 3: Drift-Check Automation

1. Add drift-check test that compares canonical contract vs profile coverage requirements.
2. Add docs consistency check to ensure plan contract table stays synchronized with canonical contract.

Acceptance criteria:

1. Contract drift fails CI with clear action message.
2. No manual checklist-only path remains for contract parity.

### Phase 4: Reporting and Operator Clarity

1. Add report fields for contract version/hash and missing/extra key summaries.
2. Update operator documentation for interpreting coverage-contract failures.

Acceptance criteria:

1. Failed coverage runs are immediately actionable by category.
2. Docs explain contract update workflow and when changes are allowed.

## Verification Strategy

1. `make test-adversarial-manifest`
2. `make test-adversarial-coverage`
3. `make test-adversarial-fast`
4. `make test` (with `make dev` running)

## Operational and Security Notes

1. Improves release-gate integrity by preventing silent coverage reduction.
2. Strengthens auditability for pre-release adversarial signoff.

## Definition of Done

1. Coverage requirements are enforced from one canonical contract source.
2. Full-coverage profile cannot pass with plan-contract omissions.
3. Drift checks are automated and mandatory in CI.
