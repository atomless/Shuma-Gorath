# SIM2-GCR-4 Plan: Rust Realtime Monitoring Architecture Candidates

Date: 2026-02-28  
Status: Proposed

Reference research:

- [`docs/research/2026-02-28-sim2-gcr-4-rust-realtime-monitoring-architecture-candidates.md`](../research/2026-02-28-sim2-gcr-4-rust-realtime-monitoring-architecture-candidates.md)

## Objective

Define and stage a realtime monitoring architecture that gives near-realtime operator freshness while preserving deterministic ordering, bounded resource cost, and graceful fallback behavior.

## Non-goals

1. Replacing deterministic release gates with streaming-only telemetry.
2. Introducing WebSocket bidirectional control channels in this tranche.
3. Removing polling fallback before benchmark evidence supports it.

## Architecture Decisions

1. Canonical update model is cursor-based incremental delivery.
2. SSE is optional acceleration path, sharing the same cursor namespace.
3. All delivery paths must enforce bounded queues/buffers and explicit lag signaling.
4. Monitoring/IP-ban live surfaces should bypass stale cache semantics via explicit freshness policy.

## Delivery Phases

### Phase 1: Cursor and Delta Contract

1. Define event sequence cursor schema and monotonic ordering guarantees.
2. Add delta response shape (`events`, `next_cursor`, `has_more`, `overflow`).
3. Add request parameters (`after_cursor`, `limit`) with strict bounds.
4. Add no-store cache policy for live monitoring responses.

Acceptance criteria:

1. Delta responses are monotonic and cursor-resumable.
2. Payload windows are bounded and deterministic.
3. No stale cache artifact can masquerade as current monitoring state.

### Phase 2: Cursor Polling Path

1. Implement new polling endpoint for monitoring and IP-ban deltas.
2. Add conditional request support (`If-None-Match`/304 where applicable).
3. Update dashboard runtime to use cursor polling and preserve fallback to full snapshot only on explicit compatibility path.

Acceptance criteria:

1. Polling path reduces transferred payload and query amplification under steady load.
2. Cursor retries do not duplicate or silently drop events within retention window.
3. Operator refresh yields bounded-latency visibility under declared envelope.

### Phase 3: Optional SSE Path

1. Add SSE endpoint (`text/event-stream`) backed by same cursor/ordering contract.
2. Support resume from `Last-Event-ID`.
3. Keep client-side fallback to cursor polling on connection failure or unsupported environment.

Acceptance criteria:

1. SSE events maintain cursor-order consistency with polling path.
2. Disconnect/reconnect resumes correctly with bounded replay window.
3. SSE failure degrades gracefully to polling without hidden telemetry gaps.

### Phase 4: Backpressure and Lag Signaling

1. Add bounded server-side fan-out/buffer strategy.
2. Emit explicit lag/overflow status when consumer falls behind.
3. Surface lag/freshness state to dashboard monitoring UI (`fresh`, `degraded`, `stale`).

Acceptance criteria:

1. No unbounded queue growth under bursty adversary load.
2. Lagged/overflow conditions are explicit in API/UI.
3. System remains stable under load envelope used by `SIM2-GCR-9` benchmarks.

### Phase 5: Benchmark Handshake with `SIM2-GCR-9`

1. Finalize benchmark scenarios comparing cursor polling vs SSE.
2. Capture p50/p95/p99 freshness latency, CPU, memory, query cost, and drop/lag behavior.
3. Use benchmark outputs to finalize selected path and ADR decisions.

Acceptance criteria:

1. Candidate selection is evidence-backed, not preference-only.
2. Benchmarks are reproducible and tied to declared load envelope.
3. Architecture decision is ready for ADR capture in `SIM2-GCR-10`.

## Verification Strategy

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test-dashboard-e2e` (with `make dev` running)
4. `make test`

## Rollback Plan

1. Keep full-snapshot endpoint as temporary fallback while cursor path stabilizes.
2. If SSE path is unstable, disable SSE feature flag and continue on cursor polling baseline.
3. Preserve cursor semantics even during rollback so benchmark evidence remains comparable.

## Definition of Done

1. Cursor-based realtime contract is defined and implemented as canonical baseline.
2. Optional SSE acceleration path is compatible, bounded, and fallback-safe.
3. Backpressure, ordering, and lag visibility are explicit and test-covered.
4. Candidate decision is benchmark-ready for `SIM2-GCR-9`.
