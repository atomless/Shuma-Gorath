# Unified Telemetry Hot-Read Architecture Plan

Date: 2026-03-12  
Status: Proposed

Reference context:

- [Telemetry Storage and Query Efficiency Excellence Plan](./2026-03-11-telemetry-storage-query-efficiency-excellence-plan.md)
- [Deferred edge gateway status and archived historical proof notes](../deferred-edge-gateway.md)
- [Fermyon Wasm Functions key-value store guide](https://developer.fermyon.com/wasm-functions/using-key-value-store)
- [Fermyon Wasm Functions FAQ](https://developer.fermyon.com/wasm-functions/faq)
- [Spin key-value store guide](https://spinframework.dev/v2/kv-store-api-guide.md)
- [Fermyon TypeScript KV guidance](https://www.fermyon.com/blog/typescript-and-fermyon-cloud-key-value-storage)
- [Spin SQLite guide](https://spinframework.dev/v3/sqlite-api-guide)
- [Fermyon Wasm Functions PostgreSQL guide](https://developer.fermyon.com/wasm-functions/querying-postgresql)

## Objective

Define the fastest and lowest-cost telemetry storage/read architecture that remains:

1. shared across Fermyon edge and single shared hosts such as Linode,
2. aligned with Spin/Wasm platform constraints,
3. cheap for hosts to operate,
4. fast enough that dashboard and monitoring reads feel effectively immediate.

This tranche is specifically about the *hot read path* for operator monitoring, not wholesale telemetry replacement.

## Research Summary

### Platform facts

1. Fermyon Wasm Functions supports the Spin key-value interfaces, and the service describes that KV as low-latency, persisted, and globally replicated, but query rates are limited and only key-value operations are available. This is a good fit for state and bounded lookups, not query-heavy analytics assembly.
2. Fermyon Wasm Functions does **not** support the `wasi:keyvalue/atomic` interface, and the platform only guarantees read-your-writes within a request. That means shared read-modify-write patterns across concurrent edge instances are not a safe foundation for exact telemetry projections.
3. Fermyon Wasm Functions does **not** support Spin SQLite storage today, so any SQLite-first design would immediately split the architecture between local/shared-host and Fermyon edge.
4. Fermyon Wasm Functions does support external PostgreSQL and MySQL, but that introduces extra infrastructure, network cost, credentials, and a second storage model.
5. Spin KV itself exposes `get`, `set`, `delete`, `exists`, and `get-keys`; it is intentionally a simple non-relational store, and each unnecessary extra call is a real trip to storage. Fermyon’s own TypeScript KV guidance explicitly notes that `exists() + get()` is slower than a single `get()` because it causes two trips.

### Repository facts

1. Local and Linode are fast because their Spin KV reads are effectively local.
2. Fermyon edge slowness is concentrated in `/admin/monitoring?bootstrap=1...`, not in general runtime execution:
   - `/admin/session`, `/admin/config`, and `/admin/adversary-sim/status` are sub-second,
   - `/admin/monitoring/delta` is materially faster than full bootstrap,
   - `/admin/monitoring?bootstrap=1...` is the expensive path.
3. Current bootstrap/details logic still assembles too much on demand:
   - security/privacy counters are summed hour-by-hour,
   - retention health is recomputed,
   - recent event windows are reloaded from bucket indexes and records,
   - config and ban summaries are rebuilt,
   - all of this is done in the request path.
4. Current monitoring counters and retention catalogs are themselves maintained with `get` + `set` read-modify-write patterns over shared KV state. That is acceptable on local and usually masked on single-host deployments, but it is not a strong correctness basis for multi-writer edge projections.

## Architectural Conclusion

The best cross-target architecture is:

1. keep KV as the primary telemetry store,
2. keep raw event records and bucketed counters as the underlying source of truth,
3. add **materialized hot-read documents** maintained at write/flush time,
4. make dashboard bootstrap read those materialized documents in O(1-5) key reads instead of reconstructing them from many fine-grained keys,
5. reserve the existing bucketed/raw paths for detailed drill-down, deltas, streams, and forensic reads.

However, one architectural guardrail is non-negotiable: the hot-read layer must not be implemented as shared read-modify-write mutation of a projection document by concurrent writers. On Fermyon edge, the projection layer must be either:

1. overwritten from a deterministic rebuild source, or
2. derived from append-only or otherwise commutative intermediate state whose correctness does not rely on unsupported atomic KV semantics.

This avoids:

1. a Fermyon-only telemetry system,
2. unsupported SQLite dependence on Fermyon,
3. immediate migration to an external relational database,
4. dead code or duplicated logic across targets.

## Rejected Directions

### 1. Fermyon-only optimized telemetry store

Rejected because:

1. it would create a parallel system with dead code on other targets,
2. it would violate the project’s portability goals,
3. it would increase maintenance and test burden.

### 2. SQLite-first telemetry redesign

Rejected because:

1. SQLite is not supported on Fermyon Wasm Functions today,
2. it would immediately force a split architecture,
3. the current problem is not lack of relational semantics; it is too many small request-path reads.

### 3. External PostgreSQL/MySQL as the primary next step

Rejected for now because:

1. it adds host cost and operational complexity,
2. it introduces another backend path before the current KV model has been fully optimized,
3. the current problem can likely be solved more cheaply by changing read shape rather than replacing storage.

This remains a fallback if the unified KV-backed design still fails latency/cost targets after implementation.

### 4. Per-instance memory cache as the main solution

Rejected as the primary design because:

1. serverless/edge instances are not durable or affinity-stable,
2. it would not give deterministic operator truth by itself,
3. it is appropriate only as a secondary optimization layered over durable hot-read documents.

## Recommended Unified Design

### 1. Preserve one storage model

Keep the current layered telemetry model:

1. immutable raw event records,
2. bucketed counters and rollups where their exactness contract is explicitly understood,
3. retention catalogs and worker state.

Add a fourth layer:

4. **materialized hot-read documents** for the operator dashboard.

### 2. Materialize the bootstrap surface

Introduce one bounded bootstrap document per site for the dominant operator view, for example:

1. `telemetry:bootstrap:v1:<site>`

This document should contain exactly the data needed for the first dashboard/monitoring render:

1. compact summary counters,
2. compact security/privacy summary,
3. compact retention-health summary,
4. compact ban/maze/tarpit headline summaries,
5. recent-events tail metadata and the first small recent-events window,
6. cursors or continuation metadata for delta/detail follow-up.

It must be cheap to read and cheap to replace as a single bounded document.

### 3. Materialize bounded secondary summaries

Instead of recomputing expensive parts from many hourly reads on every request, maintain compact supporting documents such as:

1. `telemetry:security_privacy_summary:v1:<site>`
2. `telemetry:retention_summary:v1:<site>`
3. `telemetry:recent_events_tail:v1:<site>`
4. `telemetry:headline_rollups:v1:<site>`

Current mainline note:

1. the operator snapshot supporting document now legitimately carries a larger machine-first control-loop surface than the original hot-read baseline anticipated,
2. so its size cap must be kept explicitly bounded and may need periodic rebaseline against measured machine-first control-plane reality rather than against the monitoring bootstrap render budget,
3. and bootstrap-vs-operator-snapshot budgeting must now be treated as two separate concerns: compact operator render readiness for bootstrap, and truthful machine-first control payload for `operator_snapshot_v1`.

These are not a second source of truth. They are write/flush-time projections of the raw store.

### 4. Update hot-read documents at flush/maintenance time, not per request

The project already buffers and flushes monitoring writes. The preferred place to maintain hot-read documents is:

1. monitoring flush boundaries,
2. event-log append paths for the bounded recent-events tail,
3. retention worker updates for retention summaries,
4. explicit config/admin mutations where headline operational posture changes.

This keeps extra write cost bounded and avoids adding per-request synchronous recomputation.

Implementation guardrail:

1. do not maintain hot-read documents via concurrent incremental `get` + modify + `set` over the existing shared projection document,
2. do not assume current mutable monitoring counters or mutable bucket catalogs are sufficiently exact for multi-writer edge truth without an explicit decision,
3. prefer deterministic overwrite from already-bounded canonical inputs over shared projection mutation.

### 5. Keep raw/drill-down paths separate

Dashboard bootstrap must use materialized hot-read documents.

Detailed views may continue to use bucketed/raw reads, but they must remain:

1. bounded,
2. explicitly budgeted,
3. lazy-loaded after initial readiness,
4. shared across all targets.

### 6. Optional secondary optimization: short-lived in-memory memoization

After the durable hot-read documents exist, a small per-instance TTL memoization layer can be considered for:

1. repeated admin polling bursts,
2. repeated bootstrap reads within seconds.

This is optional and must not become the authoritative correctness layer.

## Why this design is the best fit for Fermyon *and* Linode

### Fermyon

1. It minimizes the number of KV round trips in the request path.
2. It avoids unsupported SQLite and avoids mandatory external databases.
3. It fits a globally distributed key-value store better than hour-by-hour on-demand recomputation.

### Linode / shared host

1. It stays on the same storage model already used locally and on shared hosts.
2. It reduces local CPU and I/O wasted on repeated monitoring reconstruction.
3. It improves UX there too, even if Linode currently masks the pain more than Fermyon.

## Cost Model

Primary cost goal: minimize host cost.

This design does that by trading a small amount of bounded write-time projection work for a large reduction in repeated expensive admin reads.

The intended economics are:

1. slightly more structured write-time maintenance,
2. significantly fewer request-time KV reads,
3. significantly smaller operator-facing bootstrap work,
4. no new external service bill.

## Performance Targets

Initial target envelope for the shared design:

1. `/admin/monitoring?bootstrap=1...`
   - Fermyon edge: target p95 under 2s
   - Linode/shared-host: target p95 under 750ms
2. Game Loop tab first visible rows:
   - Fermyon edge: under 1.5s after tab activation
   - Linode/shared-host: under 500ms
3. `/admin/monitoring/delta`
   - Fermyon edge: target p95 under 750ms
   - Linode/shared-host: target p95 under 250ms

These targets are aggressive by design and should only be relaxed with measured evidence.

## Implementation Phases

### Phase 1: Hot-read document contract

1. Resolve the authoritative-source and correctness contract first:
   - which telemetry values remain exact under non-atomic KV,
   - which existing counters/catalogs are best-effort only,
   - and whether hot-read documents will be rebuilt from canonical immutable inputs or from explicitly accepted approximate rollups.
2. Define the exact schema for bootstrap and supporting summary documents.
3. Define freshness, update triggers, and bounded size rules.
4. Define which fields are authoritative hot-read projections versus drill-down-only.

### Phase 2: Write-path projection

1. Update flush/event/retention paths to maintain the hot-read documents without relying on shared multi-writer projection read-modify-write.
2. Keep update logic centralized and shared across targets.
3. Add bounded repair/rebuild logic for missing or stale projections.
4. Prove concurrent edge writers cannot lose or corrupt projection state under the chosen contract.

### Phase 3: Monitoring bootstrap rewrite

1. Make `/admin/monitoring?bootstrap=1...` read the hot-read documents first.
2. Move expensive detail reconstruction out of the bootstrap path.
3. Keep full/detail reads as secondary lazy paths.

### Phase 4: Performance proof

1. Measure live Fermyon and Linode bootstrap/delta latencies before and after.
2. Add canonical Make verification for edge and shared-host telemetry-read budgets.
3. Fail verification when the request path regresses to multi-second hot reads.

## Verification

Required proof for completion:

1. live Fermyon edge measurements showing the target path is materially faster than the current `~5.3-5.7s` bootstrap,
2. live Linode/shared-host measurements showing no regression and preferably improvement,
3. regression tests proving bootstrap no longer reconstructs expensive summaries hour-by-hour,
4. docs and skills updated so deploy proofs include the improved telemetry-read expectations.

## Definition of Done

1. One shared telemetry storage/read architecture is used across Fermyon and Linode.
2. No Fermyon-only telemetry store or SQLite split has been introduced.
3. Hot monitoring/dashboard reads use durable materialized documents instead of expensive on-demand reconstruction.
4. The hot-read layer does not depend on unsafe multi-writer shared projection mutation over non-atomic KV.
5. Host cost stays low by avoiding new infrastructure and reducing repeated read amplification.
6. Edge and shared-host operator UX both improve measurably.
