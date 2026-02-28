# SIM2-GCR-8 Plan: GC-6/GC-8/GC-11/GC-14 Quantitative Implementation Sequence

Date: 2026-02-28  
Status: Proposed

Reference research:

- [`docs/research/2026-02-28-sim2-gcr-8-gc6-gc8-gc11-gc14-synthesis.md`](../research/2026-02-28-sim2-gcr-8-gc6-gc8-gc11-gc14-synthesis.md)

## Objective

Execute SIM2 gap-closure core domains with explicit quantitative gates and dependency-safe sequencing.

## Non-goals

1. Running all high-risk domains in one unreviewable slice.
2. Promoting emergent findings to blocking status without deterministic confirmation.
3. Relaxing trust boundaries to accelerate implementation.

## Recommended Execution Order

1. `GC-6` realtime contract and delivery path.
2. `GC-8` frontier actor execution and safety envelopes.
3. `GC-11` verification/gate expansion for new realtime/frontier paths.
4. `GC-14` hybrid promotion governance and threshold enforcement.

## Domain-by-Domain Plan

### Phase A: `GC-6` Realtime Foundation

1. Implement cursor contract + delta endpoints + optional SSE path.
2. Wire freshness/lag/degraded state propagation to UI and API.
3. Enforce quantitative thresholds (`p95<=300ms`, `p99<=500ms`, overflow/drop `0`, query budget contract).

Exit criteria:

1. Thresholds measurable in benchmark/CI artifacts.
2. Active monitoring and IP-bans paths meet freshness envelope.

### Phase B: `GC-8` Frontier Real-Actor Execution

1. Enforce runtime hardening and signed capability-envelope execution.
2. Guarantee action lineage completeness and reject-by-default validation.
3. Enforce kill-switch SLO (`p95<=10s`) and explicit outage/degraded state visibility.

Exit criteria:

1. Policy-violation execution rate `0` in mandatory tests.
2. Executed action lineage completeness `100%`.

### Phase C: `GC-11` Verification Hardening

1. Expand matrix for realtime, frontier, retention, cost, and security domains.
2. Wire threshold-aware failure diagnostics and artifact outputs.
3. Add ADR conformance checks for domains governed by `0007`/`0008`/`0009`.

Exit criteria:

1. CI fails deterministically on threshold breaches.
2. Diagnostics identify exact violated threshold/domain.

### Phase D: `GC-14` Hybrid Governance Lock-in

1. Enforce promotion thresholds (`>=95% deterministic confirmation`, `<=20% false discovery`, `<=48h owner disposition SLA`).
2. Keep release-blocking authority deterministic-only.
3. Publish operator governance workflow and lineage visibility updates.

Exit criteria:

1. Emergent lane remains non-blocking until deterministic confirmation.
2. Promotion decisions are auditable and threshold-backed.

## Verification Strategy

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test-dashboard-e2e` (with `make dev` running)
4. `make test`
5. `make test-sim2-realtime-bench`

## Rollback Strategy

1. Revert by domain phase, not entire tranche, to preserve completed guardrails.
2. If realtime path regresses, disable SSE and retain cursor polling baseline.
3. If frontier path regresses, keep deterministic lanes while disabling frontier execution path.
4. Never bypass `GC-11` threshold gates during rollback.

## Definition of Done

1. `GC-6`, `GC-8`, `GC-11`, `GC-14` have explicit quantitative acceptance thresholds in TODO contracts.
2. Sequence dependencies are documented and reflected in implementation order.
3. CI/gates enforce threshold and ADR conformance contracts.
4. Operator documentation reflects deterministic vs emergent governance semantics.
