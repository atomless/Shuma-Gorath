# SIM2-GCR-10 Plan: ADR Decision Capture and Conformance Wiring

Date: 2026-02-28  
Status: Proposed

Reference research:

- [`docs/research/2026-02-28-sim2-gcr-10-adr-decision-capture-research.md`](../research/2026-02-28-sim2-gcr-10-adr-decision-capture-research.md)

## Objective

Ensure SIM2 implementation slices are constrained by explicit ADR contracts and cannot silently drift on orchestration, realtime monitoring, or lifecycle governance.

## Non-goals

1. Rewriting existing ADR history.
2. Freezing all future architecture changes (supersession remains allowed).
3. Replacing implementation-level tests with docs-only checks.

## Architecture Decisions Captured

1. `ADR-0007`: command-controller trust-boundary architecture.
2. `ADR-0008`: cursor+SSE hybrid realtime architecture and thresholds.
3. `ADR-0009`: retention/cost/security lifecycle governance.

## Delivery Phases

### Phase 1: ADR Publication and Cross-Linking

1. Publish ADR files and ensure references from research/plan/todo artifacts.
2. Add ADR references to relevant SIM2 GC sections in TODO backlog.
3. Ensure docs navigation reflects new ADR entries.

Acceptance criteria:

1. Each major GC domain has explicit ADR linkage.
2. ADR links resolve from TODO and research docs.
3. Decision provenance is clear to next implementer.

### Phase 2: Conformance Verification Hooks

1. Add TODO task for ADR-conformance verification in CI diagnostics (`SIM2-GC-11`).
2. Require implementation slices to state ADR alignment or explicit deviation/supersession intent.
3. Add review guidance to detect drift against accepted ADR contracts.

Acceptance criteria:

1. CI diagnostics can identify ADR-contract drift by domain.
2. Implementation PR slices cannot merge with unacknowledged ADR deviation.
3. Supersession flow remains explicit and documented.

### Phase 3: Supersession and Evolution Protocol

1. Define minimal supersession protocol (new ADR + explicit replaced ADR reference + migration notes).
2. Add rollback references in ADRs for high-risk shifts.
3. Keep periodic architecture review checkpoint tied to SIM2 evolution loop.

Acceptance criteria:

1. Supersession path is actionable and low-ambiguity.
2. Rollback posture is documented for each domain ADR.
3. Architecture review cadence keeps ADR set current.

## Verification Strategy

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test`

## Rollback Plan

1. If ADR linkage causes workflow friction, keep ADRs authoritative and temporarily relax CI strictness while preserving review checklist.
2. Re-enable strict conformance gate once diagnostics are stable.
3. Never remove accepted ADRs without supersession record.

## Definition of Done

1. ADR set (`0007`/`0008`/`0009`) is published and cross-linked.
2. TODO and verification paths include ADR conformance checks.
3. Supersession protocol is explicit for future architecture evolution.
