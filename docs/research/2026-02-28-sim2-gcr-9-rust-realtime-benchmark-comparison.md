# SIM2-GCR-9 Research: Rust Realtime Candidate Benchmark Comparison

Date: 2026-02-28  
Status: Benchmark evidence captured

## Objective

Run Rust-focused prototype benchmarks comparing realtime monitoring delivery candidates (cursor polling vs streaming candidate) and capture latency/CPU/memory/query-cost tradeoffs for architecture selection.

## Benchmark Prototype and Reproducibility

Artifacts:

1. Benchmark harness: `docs/research/artifacts/2026-02-28-sim2-gcr-9-realtime-benchmark.rs`
2. Raw output: `docs/research/artifacts/2026-02-28-sim2-gcr-9-realtime-benchmark-output.txt`

Command used:

```bash
rustc docs/research/artifacts/2026-02-28-sim2-gcr-9-realtime-benchmark.rs -O -o /tmp/sim2_gcr9_realtime_bench && /tmp/sim2_gcr9_realtime_bench
```

Modeled candidates:

1. `poll_default`: cursor polling every 1000ms, delta limit 600.
2. `poll_fast`: cursor polling every 250ms, delta limit 400.
3. `sse`: streaming candidate with bounded queue capacity 1024 and consumer drain every 250ms.

## Scenarios

1. **Steady**: 200 events/s for 120s, 5 operator clients.
2. **Burst**: 1000 events/s for 30s, 5 operator clients.

Metrics captured:

1. Delivery completeness (`delivered`, `overflow/drop`)
2. Freshness (`p50/p95/p99 latency`)
3. Query/control cost proxy (`calls_or_connections`)
4. CPU proxy (`cpu_ms` for simulation loop)
5. Memory proxy (`approx queue window bytes`)

## Results

### Steady: 200 events/s for 120s

| Candidate | Delivered | Overflow/Drop | Calls/Connections | p50 ms | p95 ms | p99 ms | Mean ms | CPU ms | Approx Memory |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `poll_default` | 120,000 | 0 | 605 | 496 | 946 | 986 | 498.50 | 2 | 768,000 B |
| `poll_fast` | 120,000 | 0 | 2,405 | 121 | 236 | 246 | 123.50 | 1 | 512,000 B |
| `sse` | 120,000 | 0 | 5 | 121 | 236 | 246 | 123.50 | 2 | 1,310,720 B |

### Burst: 1000 events/s for 30s

| Candidate | Delivered | Overflow/Drop | Calls/Connections | p50 ms | p95 ms | p99 ms | Mean ms | CPU ms | Approx Memory |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `poll_default` | 90,005 | 929,995 overflow | 155 | 6,499 | 11,899 | 12,419 | 6,499.14 | 1 | 768,000 B |
| `poll_fast` | 150,000 | 0 | 605 | 124 | 237 | 247 | 124.50 | 0 | 512,000 B |
| `sse` | 150,000 | 0 | 5 | 124 | 237 | 247 | 124.50 | 1 | 1,310,720 B |

## Interpretation

1. `poll_default` (1s cadence) is not acceptable for near-realtime under burst envelope; it overflows and exhibits multi-second to 12s tail lag.
2. `poll_fast` reaches latency comparable to streaming, but increases query-call pressure by roughly two orders of magnitude versus `sse` (`605` calls vs `5` connections in burst scenario).
3. `sse` achieves low latency with minimal request churn, at the cost of higher persistent per-client buffer memory.
4. Cursor semantics remain necessary even with `sse`, because reconnect/resume and fallback paths depend on deterministic sequence continuity.

## Decision Matrix

| Option | Benefits | Risks | Resource Cost | Security Impact | Rollback Complexity |
|---|---|---|---|---|---|
| Polling-only (1s default) | Simplest operationally | Fails freshness/overflow under burst | Lower call count than fast polling, but unacceptable latency | Low complexity | Low |
| Polling-only (250ms tuned) | Meets freshness targets in prototype | High query-call volume and higher backend read amplification | High query cost | Moderate (more exposed read surface frequency) | Low |
| SSE-only | Lowest call churn + low latency | Stream lifecycle complexity, lag handling, fallback requirements | Higher persistent memory | Moderate (long-lived stream boundary) | Medium |
| Hybrid cursor baseline + SSE acceleration (recommended) | Low-latency with controlled query cost and robust fallback | More implementation complexity than single path | Balanced when bounded | Strong if same auth/cursor contract on both paths | Medium |

## Recommendation

Adopt **hybrid cursor baseline + optional SSE acceleration**.

1. Keep canonical cursor-delta polling contract as the deterministic truth and fallback.
2. Use SSE for active live views to reduce read-amplification while preserving low-latency updates.
3. Enforce bounded stream buffers and explicit lag/overflow status.
4. Treat 1s polling cadence as non-compliant for realtime SLO under declared burst envelope.

## Quantitative Thresholds Derived for TODO Enforcement

Proposed thresholds for `SIM2-GC-6`/`SIM2-GC-11` under benchmark envelope (`1000 events/s`, `5 active clients`):

1. Freshness SLO: `p95 <= 300ms`, `p99 <= 500ms` for active live path.
2. Reliability: `overflow/drop == 0` within declared bounded queue window.
3. Query budget: active live updates must not exceed `1 request/sec/client` average when streaming path is available; fallback polling may exceed this only in degraded mode with explicit `degraded` state.
4. Ordering: zero cursor regressions (`next_cursor` must be monotonic; reconnect resume must not reorder).
5. Backpressure visibility: lag/overflow states must be surfaced in API and UI within one refresh/stream cycle.

## Limitations

1. Prototype is an in-memory Rust simulation, not full Spin end-to-end load against production runtime.
2. CPU values are relative loop costs, not whole-system CPU utilization.
3. Network and serialization overhead were approximated through call-count and queue-size proxies.

These limitations do not change the core outcome: default polling cadence is insufficient; low-latency freshness requires either high-frequency cursor polling (high read cost) or streaming acceleration.

## Plan and TODO Impact

1. New plan doc: `docs/plans/2026-02-28-sim2-gcr-9-rust-realtime-benchmark-comparison-plan.md`.
2. Upgrade `SIM2-GC-6` and `SIM2-GC-11` acceptance criteria with benchmark-derived quantitative thresholds.
3. Carry selected hybrid architecture decision into `SIM2-GCR-10` ADR capture.
