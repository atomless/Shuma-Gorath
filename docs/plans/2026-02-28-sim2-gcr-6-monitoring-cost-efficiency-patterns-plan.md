# SIM2-GCR-6 Plan: Monitoring Pipeline Cost-Efficiency Controls

Date: 2026-02-28  
Status: Proposed

Reference research:

- [`docs/research/2026-02-28-sim2-gcr-6-monitoring-cost-efficiency-patterns-research.md`](../research/2026-02-28-sim2-gcr-6-monitoring-cost-efficiency-patterns-research.md)

## Objective

Implement layered cost controls so realtime monitoring remains affordable and stable without losing security-critical evidence.

## Non-goals

1. Sampling or dropping security-critical defense outcomes.
2. Forcing external observability dependency in this tranche.
3. Regressing freshness SLOs to reduce cost.

## Architecture Decisions

1. Cost governance uses layered controls (cardinality + rollups + selective sampling + query budgets + compression).
2. Unsampleable event classes are policy-enforced.
3. Cost thresholds are explicit and test/CI enforced.

## Delivery Phases

### Phase 1: Cardinality and Schema Guardrails

1. Define guarded dimensions and hourly cardinality caps.
2. Add overflow-bucket behavior for excess dimensions.
3. Add ingest-time validation/coarsening for out-of-budget dimensions.

Acceptance criteria:

1. Guarded dimensions cannot exceed configured caps without explicit overflow accounting.
2. Overflow behavior is deterministic and observable.
3. No unbounded label explosion path remains in monitoring ingest.

### Phase 2: Rollups and Query Cost Reduction

1. Add precomputed rollup windows for dashboard-facing queries.
2. Route default monitoring views to rollups where appropriate.
3. Keep raw-event detail path for forensic drill-down.

Acceptance criteria:

1. Default monitoring queries avoid repeated full-window recomputation.
2. Rollup and raw views remain lineage-consistent.
3. Query CPU/read amplification drops measurably under benchmark envelope.

### Phase 3: Sampling Policy and Event-Class Protection

1. Define unsampleable defense-event class list.
2. Add deterministic sampling only for approved low-risk telemetry classes.
3. Emit sampled-vs-unsampled counters by class.

Acceptance criteria:

1. Unsampleable classes are never sampled/dropped.
2. Sampling policy is explicit, testable, and reviewable.
3. Sampling counters support audit and debugging workflows.

### Phase 4: Payload and Query Budget Controls

1. Add response-size caps and cursor pagination for large payloads.
2. Add compression negotiation and reporting for monitoring responses.
3. Extend endpoint/session query budgets with cost-class awareness.

Acceptance criteria:

1. Default payload targets remain within budget (`p95 <= 512KB`).
2. Compression yields measurable transfer savings for large payloads.
3. Budget overruns are explicit and surface degraded status.

### Phase 5: Verification and CI Diagnostics

1. Add regression tests for cardinality caps and overflow buckets.
2. Add tests for unsampleable-class protection.
3. Add benchmark diagnostics for payload, compression ratio, and request-budget metrics.

Acceptance criteria:

1. Cost-threshold regressions fail deterministically in CI.
2. CI artifacts identify failing cost dimension (`cardinality`, `payload`, `budget`, `sampling`).
3. Makefile verification remains canonical path for cost checks.

## Verification Strategy

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test`
4. `make test-sim2-realtime-bench`

## Rollback Plan

1. If rollup path regresses correctness, fall back to raw path while keeping cardinality and budget controls active.
2. If compression path introduces compatibility issues, disable optional encoding and keep payload caps/pagination.
3. Never relax unsampleable event protections during rollback.

## Definition of Done

1. Cost controls are explicit, layered, and test-covered.
2. Realtime freshness and evidence integrity remain intact under cost governance.
3. CI enforces quantitative cost thresholds and reports actionable diagnostics.
4. Operators can observe cost health and degraded states directly.
