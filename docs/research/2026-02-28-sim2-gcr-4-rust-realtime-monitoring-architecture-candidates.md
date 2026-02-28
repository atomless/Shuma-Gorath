# SIM2-GCR-4 Research: Rust Realtime Monitoring Architecture Candidates

Date: 2026-02-28  
Status: Candidate set selected for benchmark phase

## Objective

Identify the best Rust-first architecture candidates for near-realtime Monitoring/IP-ban freshness, with explicit guarantees for ordering, resume semantics, backpressure, and bounded memory/CPU cost.

## Repository Baseline (Current State)

1. Dashboard monitoring refresh currently runs on interval polling (`monitoring=30s`, `ip-bans=45s`) with browser cache TTL defaults (`60s`) in runtime refresh layer (`dashboard/src/lib/state/dashboard-store.js`, `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`).
2. Monitoring API currently returns broad summary + detail snapshots (`/admin/monitoring`) and constructs detail payload by scanning recent event records and related keys on each request (`src/admin/api.rs`).
3. Current model is simple and resilient but creates freshness lag and repeated full-snapshot query cost.
4. Current model does not expose a first-class cursor contract for deterministic incremental updates.

## Primary-Source Findings

1. Server-Sent Events are standardized over HTTP with `text/event-stream` and support resume through `id` + `Last-Event-ID`.
   Source: [WHATWG HTML SSE](https://html.spec.whatwg.org/dev/server-sent-events.html)
2. WebSocket protocol requires explicit origin checks and server-side implementation limits for resource safety.
   Source: [RFC 6455](https://datatracker.ietf.org/doc/rfc6455/)
3. Caching semantics: `no-store` forbids cache storage; this is important for realtime operator views that must avoid stale reuse.
   Source: [RFC 9111](https://datatracker.ietf.org/doc/html/rfc9111)
4. Conditional request semantics (`If-None-Match`/`304`) allow efficient incremental polling without retransmitting unchanged payloads.
   Source: [RFC 7232](https://www.rfc-editor.org/rfc/rfc7232)
5. Bounded Tokio `mpsc` channels preserve send order and provide explicit backpressure when capacity is reached.
   Source: [Tokio bounded mpsc docs](https://docs.rs/tokio/latest/src/tokio/sync/mpsc/bounded.rs.html)
6. Tokio unbounded channels can exhaust memory under lagging consumers.
   Source: [Tokio `unbounded_channel` docs](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.unbounded_channel.html)
7. Tokio broadcast channels provide bounded fan-out with explicit lag detection (`RecvError::Lagged`) when consumers fall behind.
   Source: [Tokio broadcast docs](https://docs.rs/tokio/latest/tokio/sync/broadcast/)
8. Spin SDK supports streaming HTTP responses (`OutgoingResponse` + `ResponseOutparam`), making SSE technically viable in runtime model.
   Source: [Spin SDK `OutgoingResponse`](https://docs.rs/spin-sdk/latest/spin_sdk/http/struct.OutgoingResponse.html)

## Inferences for Shuma (Derived from Sources)

1. Cursor-based incremental polling is the lowest-risk deterministic baseline because it preserves request/response semantics while reducing payload and query cost.
2. SSE is a viable acceleration path in Rust/Spin, but must be layered on the same cursor/ordering contract rather than replacing it.
3. WebSocket is currently higher complexity than needed for one-way monitoring updates and introduces additional trust-boundary/connection-state management cost.

## Architecture Options

### Option A: Full Snapshot Interval Polling (Current Shape)

Keep fixed-interval polling and return full monitoring payload each refresh.

### Option B: Cursor-Delta Polling + Conditional GET

Introduce monotonic event cursor and delta endpoint (`after_cursor`, `limit`, `next_cursor`), with conditional GET support for unchanged windows.

### Option C: SSE Stream with Cursor Resume

Provide `text/event-stream` endpoint using event `id` and resume via `Last-Event-ID`, backed by bounded event buffers.

### Option D: WebSocket Stream

Use persistent bidirectional socket for update push and acknowledgements.

### Option E: Hybrid (Cursor-Delta Baseline + Optional SSE Acceleration) (Recommended Candidate Set)

Make cursor-delta polling canonical and add SSE as optional fast path sharing the same sequence and resume contract.

## Decision Matrix

| Option | Benefits | Risks | Resource Cost | Security Impact | Rollback Complexity |
|---|---|---|---|---|---|
| A. Full snapshot polling | Minimal implementation change | Staleness and repeated heavy payload/query cost remain | Low code, higher steady-state runtime cost | Low incremental risk | Low |
| B. Cursor-delta polling | Deterministic ordering, bounded payload, simple operational model | Requires new cursor schema and delta endpoint | Medium | Strong; HTTP request model unchanged | Low-medium |
| C. SSE stream | Lower latency push path, native resume semantics (`Last-Event-ID`) | Long-lived connection management and lag handling required | Medium-high | Moderate; long-lived stream boundary must be hardened | Medium |
| D. WebSocket | Flexible bidirectional channel | Highest complexity; origin/auth/state management overhead | High | Higher attack surface and resource-management burden | High |
| E. Hybrid cursor + optional SSE (recommended) | Deterministic baseline + low-latency optional acceleration; graceful fallback | More moving parts than single path | Medium-high | Strong if both paths share same auth/cursor policy | Medium |

## Recommendation

Adopt **Option E** as the target architecture, with explicit sequencing:

1. Build canonical cursor-delta polling path first.
2. Benchmark cursor polling versus SSE in `SIM2-GCR-9` under equivalent load envelopes.
3. Keep WebSocket out of current scope unless benchmark evidence later shows SSE is insufficient.

## Required Contract Elements

1. **Monotonic ordering contract**
   1. Every monitoring event gets a strictly monotonic sequence cursor.
   2. Delta responses and streams carry `next_cursor` and overflow indicators.
2. **Backpressure contract**
   1. Server-side queues must be bounded.
   2. Slow-consumer lag must be explicit (`lagged`/`overflow`) rather than silently unbounded.
3. **Freshness contract**
   1. Responses for monitoring/IP-ban live views use no-store semantics.
   2. Client cache usage must not mask current operator state.
4. **Resume contract**
   1. Polling resumes via `after_cursor`.
   2. SSE resumes via `Last-Event-ID` mapped to same cursor namespace.

## Plan and TODO Impact

1. New plan doc: `docs/plans/2026-02-28-sim2-gcr-4-rust-realtime-monitoring-architecture-candidates-plan.md`.
2. Expand `SIM2-GC-6` with explicit cursor contract, delta endpoint, optional SSE path, and bounded buffer/backpressure tasks.
3. Expand `SIM2-GC-11` with cursor/stream ordering, resume, lag, and overflow regression tests.
4. Keep final architecture selection contingent on quantitative benchmark evidence from `SIM2-GCR-9`.
