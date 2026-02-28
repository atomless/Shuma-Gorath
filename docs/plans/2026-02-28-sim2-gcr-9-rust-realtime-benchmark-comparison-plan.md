# SIM2-GCR-9 Plan: Rust Realtime Benchmark Comparison Follow-through

Date: 2026-02-28  
Status: Proposed

Reference research:

- [`docs/research/2026-02-28-sim2-gcr-9-rust-realtime-benchmark-comparison.md`](../research/2026-02-28-sim2-gcr-9-rust-realtime-benchmark-comparison.md)

## Objective

Translate benchmark evidence into implementation-ready constraints for realtime monitoring delivery, with quantified SLOs and verification gates.

## Non-goals

1. Final ADR publication (handled by `SIM2-GCR-10`).
2. Full end-to-end load lab creation in this tranche.
3. Removing fallback polling path.

## Architecture Decisions from Benchmark Evidence

1. Default 1s polling cadence is non-compliant for burst freshness requirements.
2. Hybrid architecture (cursor baseline + SSE acceleration) is preferred over polling-only and SSE-only extremes.
3. Quantitative SLO thresholds must be encoded in TODO acceptance criteria and CI diagnostics.

## Delivery Phases

### Phase 1: Benchmark Contract and Makefile Entry

1. Promote benchmark harness into repository-tracked artifact with deterministic params.
2. Add Make target for realtime candidate benchmark replay and machine-readable output capture.
3. Document benchmark envelope assumptions and interpretation guidance.

Acceptance criteria:

1. Contributors can reproduce benchmark outputs with one Make command.
2. Benchmark artifacts include freshness, query-cost proxy, and buffer/overflow metrics.
3. Output format is stable enough for CI artifact ingestion.

### Phase 2: Quantitative Threshold Wiring

1. Apply benchmark-derived thresholds to `SIM2-GC-6` acceptance criteria.
2. Apply benchmark-derived regression checks to `SIM2-GC-11` acceptance criteria.
3. Define degraded-mode exceptions explicitly (for polling fallback) and require operator-visible degraded status.

Acceptance criteria:

1. `p95 <= 300ms` and `p99 <= 500ms` thresholds are explicit in TODO/plan contracts for active live path.
2. Overflow/drop tolerance is explicitly zero in non-degraded path.
3. Query-budget and ordering constraints are measurable and testable.

### Phase 3: Verification and Diagnostics

1. Add verification tasks requiring benchmark rerun when realtime architecture changes.
2. Add CI diagnostics fields for freshness percentile, lag/overflow counts, and request/connection cost proxies.
3. Ensure failure output names violated threshold and scenario envelope.

Acceptance criteria:

1. Realtime regressions fail with threshold-level diagnostics.
2. CI artifacts make stale/overflow/order regressions actionable.
3. Benchmark and realtime gates remain aligned with Makefile-first workflow.

## Verification Strategy

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test-dashboard-e2e` (with `make dev` running)
4. `make test`
5. `make test-sim2-realtime-bench` (new target from this plan)

## Rollback Plan

1. If SSE path destabilizes, keep cursor polling fallback and explicitly report degraded freshness state.
2. If benchmark target flakes, keep historical artifact and temporarily run benchmark in advisory mode with logged waiver.
3. Do not relax ordering/overflow invariants during rollback.

## Definition of Done

1. Benchmark evidence is reproducible from repository artifacts.
2. Realtime TODO acceptance criteria include explicit quantitative thresholds.
3. Verification contract includes threshold-aware diagnostics.
4. Architecture handoff is ready for ADR codification in `SIM2-GCR-10`.
