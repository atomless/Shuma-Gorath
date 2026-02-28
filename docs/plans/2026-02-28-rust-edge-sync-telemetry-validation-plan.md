# SIM2 Rust + Edge Sync Telemetry Validation Plan

Date: 2026-02-28  
Status: Proposed (implementation-ready)

Related:

- [`docs/research/2026-02-28-rust-edge-sync-telemetry-delta-research.md`](../research/2026-02-28-rust-edge-sync-telemetry-delta-research.md)
- [`docs/adr/0009-telemetry-lifecycle-retention-cost-security.md`](../adr/0009-telemetry-lifecycle-retention-cost-security.md)
- [`docs/adr/0008-realtime-monitoring-cursor-sse-hybrid.md`](../adr/0008-realtime-monitoring-cursor-sse-hybrid.md)
- [`todos/todo.md`](../../todos/todo.md)

## Objective

Run a focused validation and benchmark program that proves Shuma’s data collection and retention architecture is optimal for current goals in Rust and edge-sync conditions, without weakening release-gate determinism.

## Non-goals

1. Replacing SIM2 architecture with a new data platform before GC implementation.
2. Introducing vendor-heavy telemetry infra during pre-launch unless benchmark evidence demands it.
3. Allowing stochastic or eventually-consistent-only data paths to become release-blocking evidence sources.

## Validation Questions

1. Does current retention architecture (`GC-15`) stay deterministic under bursty ingest and purge failure/retry conditions?
2. Do cost controls (`GC-16`) keep payload/query/cardinality budgets inside target envelopes while preserving unsampleable evidence?
3. Do security/privacy controls (`GC-17`) fail closed without creating unacceptable operational friction?
4. Under edge/multi-instance conditions (`DEP-ENT-*`), do monitoring and lifecycle semantics preserve required ordering, lineage, and convergence?

## Architecture Stance for This Plan

1. Keep single-writer authority per evidence partition in release-gate paths.
2. Allow edge-read acceleration only with explicit cursor/bookmark consistency semantics.
3. Keep adaptive/emergent lanes non-blocking until deterministic replay confirmation.
4. Keep Makefile targets as the only canonical verification surface.

## Execution Sequence

### Phase 1: Contracts and Bench Harness Skeleton

1. Publish a benchmark contract doc section in SIM2 operator docs with:
   - declared workload envelope,
   - required metrics,
   - pass/fail thresholds,
   - artifact schema.
2. Add Make targets (skeleton no-op or minimal harness first):
   - `make test-sim2-retention-bench`
   - `make test-sim2-cost-bench`
   - `make test-sim2-sync-bench`
3. Add CI artifact slots for benchmark outputs (JSON + markdown summary).

Exit criteria:

1. Targets exist and run deterministically.
2. Artifact schema is stable and versioned.

### Phase 2: Retention Determinism Validation (`GC-15` + `GC-11`)

1. Implement benchmark scenarios for:
   - steady ingest,
   - burst ingest,
   - purge backlog catch-up,
   - purge partial-failure recovery.
2. Validate retention metrics:
   - `purge_lag_hours`,
   - `pending_expired_buckets`,
   - `oldest_retained_ts`,
   - purge worker tick runtime.
3. Add regression tests proving no monitoring/admin read path triggers full keyspace cleanup scans.

Thresholds:

1. Purge lag under normal envelope: `<= 1 hour` beyond retention window.
2. Purge worker per-tick budget: `<= 500ms`.
3. Healthy state: `pending_expired_buckets == 0`.

Exit criteria:

1. Retention thresholds enforced in automated gate.
2. Failure diagnostics name exact violating metric and bucket window.

### Phase 3: Cost Envelope Validation (`GC-16` + `GC-11`)

1. Run cardinality pressure tests with guarded dimensions and overflow bucket behavior.
2. Run payload budget tests with cursor windowing + compression negotiation.
3. Run query-budget tests across polling/cursor/SSE paths with concurrent operator clients.
4. Verify unsampleable event class policy (`0` sampled/dropped for protected classes).

Thresholds:

1. Guarded dimension cap: `<=1000` values/hour with deterministic `other` overflow.
2. Default monitoring response payload: `p95 <= 512KB`.
3. Streaming-enabled request budget: `<=1 req/sec/client` average.
4. Compression effectiveness on payloads `>64KB`: `>=30%` transferred-byte reduction.
5. Unsampleable event loss: `0`.

Exit criteria:

1. Threshold breaches fail CI with class-specific diagnostics.
2. Cost controls do not regress freshness SLOs from ADR `0008`.

### Phase 4: Security/Privacy Validation (`GC-17` + `GC-11`)

1. Add classification-enforcement tests across ingest/persist boundaries.
2. Add secret-canary injection tests for telemetry and frontier artifacts.
3. Add pseudonymization default-coverage checks for non-forensic views.
4. Add retention-tier tests for high-risk raw artifacts.

Thresholds:

1. Secret canary persistence leakage: `0` accepted cases.
2. High-risk raw artifact retention default: `<=72h`.
3. Pseudonymization coverage in non-forensic paths: `100%` of configured sensitive identifiers.
4. Incident hook visibility: within one refresh/stream cycle.

Exit criteria:

1. Security/privacy regressions fail deterministically with explicit taxonomy.
2. Operator diagnostics remain actionable and minimally ambiguous.

### Phase 5: Edge Sync and Distributed-State Validation (`DEP-ENT-*` + `GC-11`)

1. Add two-instance and fault-injection benchmark matrix:
   - normal operation,
   - replication lag,
   - backend outage,
   - partial instance failure.
2. Validate convergence and ordering semantics for monitoring and ban-state views.
3. Validate lifecycle command/evidence lineage continuity across reconnection and failover paths.

Thresholds (initial proposal; finalize in implementation slice):

1. Ban-sync convergence SLO: explicit target captured and enforced per deployment mode.
2. No silent split-brain: conflicting controllers or stale read paths must surface degraded/failure state.
3. Monitoring cursor lineage: monotonic progression with explicit overflow taxonomy.

Exit criteria:

1. Enterprise sync mode has measurable, enforced convergence guarantees.
2. Failure modes are explicit, operator-visible, and rollback-safe.

## Benchmark Matrix (Minimum Required)

1. Workload profiles:
   - `baseline`: `1000 events/s`, `5` clients
   - `elevated`: `3000 events/s`, `10` clients
   - `stress`: `5000 events/s`, `20` clients (scheduled/manual lane if needed)
2. Traffic mixes:
   - sim-heavy,
   - mixed sim/manual,
   - non-sim production-like.
3. Failure injections:
   - storage delay,
   - partial write failure,
   - replay/race on control-plane operations,
   - temporary backend unavailability.

## Required Artifacts per Benchmark Run

1. Percentiles (`p50/p95/p99`) for freshness and lag.
2. Overflow/drop counts by event class.
3. Cardinality pressure and overflow bucket stats.
4. Purge lifecycle telemetry (`lag`, `pending buckets`, watermark progression).
5. Security/privacy policy events (classification rejects, canary detections, incident hooks).
6. Sync/convergence timeline for distributed mode.

## TODO Mapping

1. `SIM2-GC-15`: Phases 1-2.
2. `SIM2-GC-16`: Phases 1 and 3.
3. `SIM2-GC-17`: Phases 1 and 4.
4. `SIM2-GC-11`: all benchmark-gate and diagnostics wiring.
5. `DEP-ENT-1..5`: Phase 5 convergence/failure validation.

## Verification Path

1. `make setup`
2. `make dev` (separate session)
3. `make test-sim2-retention-bench`
4. `make test-sim2-cost-bench`
5. `make test-sim2-sync-bench`
6. `make test`
7. `make build`

## Rollback Strategy

1. Roll back benchmark gates by domain (retention/cost/security/sync), not all at once.
2. Never disable deterministic release-gate checks for evidence lineage while tuning thresholds.
3. If a new benchmark lane is unstable, mark it non-blocking temporarily but keep artifact capture mandatory until stabilized.

## Definition of Done

1. Data collection/retention approach is validated with source-backed rationale and workload evidence.
2. Rust runtime backpressure and loss behavior is measurable and policy-enforced.
3. Edge sync/distributed behavior is bounded, observable, and gate-tested.
4. SIM2 gates fail with specific diagnostics when thresholds drift.
5. Architecture remains aligned with ADR `0008` and ADR `0009` (or superseded explicitly).
