# ADR 0008: Realtime Monitoring via Cursor-Delta Baseline with Optional SSE Acceleration

- Status: Accepted
- Date: 2026-02-28
- Owners: Shuma core maintainers
- Related:
  - `docs/research/2026-02-28-sim2-gcr-4-rust-realtime-monitoring-architecture-candidates.md`
  - `docs/research/2026-02-28-sim2-gcr-9-rust-realtime-benchmark-comparison.md`

## Context

Monitoring freshness is central to both adversary simulation and real production attack visibility. Full-snapshot interval polling creates staleness and query amplification. Benchmark evidence showed 1s polling fails burst freshness targets, while tuned cursor polling and SSE both meet latency but differ significantly in query churn.

## Decision

Adopt hybrid realtime architecture:

1. Canonical transport contract is cursor-delta (`after_cursor`, `next_cursor`, bounded windows, overflow signaling).
2. Optional SSE path accelerates active live views and reuses same cursor namespace (`id`/`Last-Event-ID`).
3. Polling fallback remains mandatory for unsupported/degraded environments.
4. All paths enforce bounded buffers and explicit lag/degraded status.
5. Quantitative targets under declared envelope (`>=1000 events/s`, `>=5 active clients`):
   - freshness `p95 <= 300ms`, `p99 <= 500ms` (active path),
   - overflow/drop `== 0` in non-degraded path,
   - request budget `<=1 req/sec/client` average when streaming path is available.

## Alternatives Considered

1. Keep full snapshot interval polling.
2. Polling-only with aggressive cadence tuning.
3. SSE-only path.
4. WebSocket-first architecture.

## Consequences

### Positive

- Preserves deterministic ordering and replay semantics.
- Provides low-latency active updates with bounded query cost.
- Keeps resilience via polling fallback.
- Aligns realtime and production observability goals.

### Negative / Trade-offs

- Additional complexity to support dual delivery paths.
- Requires stronger lag/overflow diagnostics and test coverage.

## Security Impact

- Maintains existing admin auth/trust boundaries while adding long-lived stream considerations.
- Requires strict stream auth/session handling and explicit degraded signaling to avoid silent data ambiguity.

## Human Friction Impact

- Operator experience improves via faster, clearer monitoring freshness.
- Additional freshness state messaging (`fresh/degraded/stale`) required for transparency.

## Adversary Cost Placement

- Improves defender reaction speed and tuning loop quality.
- Does not directly increase attacker per-request cost, but increases defender detection velocity.

## Operational Impact

- Deploy: add cursor state path and optional SSE endpoint.
- Config: define envelope/buffer thresholds and degraded-mode behavior.
- Monitoring/alerts: track lag, overflow, reconnect/resume health, request budgets.
- Rollback: disable SSE path and continue on cursor polling baseline.

## Resource Impact

- Bandwidth: lower than high-frequency polling when SSE active.
- CPU: reduced repeated query churn vs fast polling; added stream bookkeeping.
- Memory: bounded per-client stream buffers.
- Energy/efficiency notes: hybrid path provides better cost-latency balance than polling-only fast cadence.

## Verification

- Tests:
  - cursor monotonic ordering/resume,
  - SSE reconnect continuity,
  - fallback continuity,
  - lag/overflow and threshold regression gates.
- Benchmarks (if relevant): `SIM2-GCR-9` benchmark artifact and replay target.
- Docs updated: yes (research, plan, TODO thresholds).

## Follow-ups

- Implement `SIM2-GC-6-*` and `SIM2-GC-11-*` slices aligned to thresholds.
- Wire benchmark replay into Makefile/CI threshold artifacts.
