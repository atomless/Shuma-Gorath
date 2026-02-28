# SIM2 Delta Research: Rust Telemetry Collection/Retention and Edge Sync Validation

Date: 2026-02-28  
Status: Research complete; execution recommended

## Objective

Validate whether the current SIM2 telemetry lifecycle direction remains the best fit for:

1. Rust runtime implementation realities.
2. Edge and multi-region synchronization needs.
3. Deterministic release-gate evidence requirements.

This is a focused delta pass on top of `SIM2-GCR-5/6/7` and ADR `0009`, not a full program reset.

## Existing Baseline in This Repo

1. Retention determinism, cost governance, and security/privacy controls are already captured in ADR `0009`.
2. Open TODO tracks already exist for these concerns (`SIM2-GC-15/16/17` + verification in `SIM2-GC-11`).
3. Enterprise distributed-state correctness still has open `DEP-ENT-*` work.

## Primary-Source Findings

### A. Rust telemetry pipeline mechanics

1. OpenTelemetry metrics cardinality limits are stable, with a documented default limit of `2000` and explicit overflow behavior (`otel.metric.overflow`).
   - Source: https://opentelemetry.io/docs/specs/otel/metrics/sdk/
2. Tokio bounded MPSC channels provide explicit backpressure once capacity is reached.
   - Source: https://docs.rs/tokio/latest/tokio/sync/mpsc/
3. `tracing_appender::non_blocking` can drop logs when its queue is full unless configured for backpressure behavior.
   - Source: https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/index.html
4. Rust has mature token-bucket style rate-limiting primitives (`governor`) suitable for bounded admin/monitoring query budgets.
   - Source: https://docs.rs/governor/latest/governor/

### B. Retention and stream lifecycle controls

1. Redis `XTRIM` supports `MINID` (time/ID boundary style trimming), `LIMIT` for bounded work, and consumer-reference controls (`KEEPREF`, `DELREF`, `ACKED`).
   - Source: https://redis.io/docs/latest/commands/xtrim/
2. Redis stream trimming remains O(N evicted), but with macro-node efficiencies and approximate trimming support.
   - Source: https://redis.io/docs/latest/commands/xtrim/

### C. Sync and consistency tradeoffs for edge/multi-region

1. `WAIT` improves replication safety but explicitly does not make Redis strongly consistent.
   - Source: https://redis.io/docs/latest/commands/wait/
2. Redis Active-Active streams have important caveats:
   - `XREAD` can skip entries in multi-region concurrent write scenarios.
   - strict ID mode is recommended to avoid duplicate IDs.
   - all consumer-group metadata is not replicated.
   - Source: https://redis.io/docs/latest/operate/rs/databases/active-active/develop/data-types/streams/
3. Redis Active-Active causal consistency can preserve per-key ordering but increases network/memory/performance cost (`N-2` relay effect).
   - Source: https://redis.io/docs/latest/operate/rs/databases/active-active/causal-consistency/
4. Fermyon Wasm Functions key-value store is globally replicated and low-latency, but explicitly does not support `wasi:keyvalue/atomic` due to consistency constraints.
   - Source: https://developer.fermyon.com/wasm-functions/using-key-value-store

### D. Modern edge consistency patterns (reference models)

1. Cloudflare Workers KV is eventually consistent and may take `~60s+` to converge globally.
   - Source: https://developers.cloudflare.com/kv/concepts/how-kv-works/
2. Cloudflare Durable Objects expose globally unique, single-threaded coordination with transactional storage semantics.
   - Sources:
     - https://developers.cloudflare.com/workers/platform/storage-options/
     - https://developers.cloudflare.com/durable-objects/best-practices/rules-of-durable-objects/
3. Cloudflare D1 read replication uses asynchronous replicas plus session bookmarks for sequential consistency.
   - Source: https://developers.cloudflare.com/d1/best-practices/read-replication/

### E. Streaming alternatives for future scale envelopes

1. NATS JetStream supports retention policies, dedupe windows, pull-consumer flow control, and at-least-once baseline with exactly-once mechanisms.
   - Sources:
     - https://docs.nats.io/nats-concepts/jetstream
     - https://docs.nats.io/nats-concepts/jetstream/streams
     - https://docs.nats.io/nats-concepts/jetstream/consumers
2. Kafka supports idempotent producers + transactions for exactly-once style processing semantics and has established geo-replication via MirrorMaker 2.
   - Sources:
     - https://kafka.apache.org/30/design/design/
     - https://kafka.apache.org/28/operations/geo-replication-cross-cluster-data-mirroring/

## Inferences for Shuma (Derived)

1. The current ADR `0009` direction is still fundamentally correct for Shuma’s immediate goals.
2. For release-gate evidence correctness, single-writer authority per evidence partition remains safer than multi-writer active-active stream writes.
3. Bounded backpressure controls must be explicit in Rust runtime paths, because default logging/queue choices can silently drop high-volume telemetry.
4. Edge-read acceleration is useful, but consistency must be explicit and session/cursor constrained (bookmark-style), not assumed.
5. Multi-region write-anywhere designs can be pursued later, but should be entered with explicit ordering/duplication and consumer-metadata caveat tests.

## Architecture Options (Delta Decision Matrix)

| Option | Summary | Benefits | Risks | Resource Cost | Deterministic Evidence Fit | Recommendation |
|---|---|---|---|---|---|---|
| A. Single-writer authority + bounded replication/read acceleration | Keep one authoritative write path per evidence partition; add edge-read/session-cursor semantics | Strong ordering clarity, simpler incident reconstruction, lower correctness risk | Added latency for far writers unless carefully routed | Medium | Strong | **Recommended now** |
| B. Redis Active-Active multi-writer for telemetry evidence | Allow concurrent writes from multiple regions into same logical stream | Better write locality, region survivability | `XREAD` skip caveats, metadata replication complexity, higher causal consistency cost | Medium-high | Medium-weak unless heavily constrained | Not recommended for release-gate evidence path |
| C. JetStream ingress log + Redis materialized operational views | Durable log-first ingest then derive monitoring views | Strong replay/debuggability, good flow control | Added system complexity and operations | High | Strong | Candidate for post-launch/high-scale phase |
| D. Kafka geo-replicated telemetry backbone | Enterprise-grade replication and EOS tooling | Mature ecosystem for large-scale data pipelines | Heavy operational footprint relative to current product phase | High | Strong | Not justified pre-launch unless requirements change |

## Recommendation

1. Continue with ADR `0009` implementation path (`GC-15/16/17`) as planned.
2. Add explicit Rust backpressure/loss-budget validation as a first-class benchmark domain.
3. Treat edge sync as a contract question, not only infra question:
   - authoritative write ownership,
   - allowed staleness windows,
   - replay/bookmark semantics,
   - deterministic lineage guarantees.
4. Keep multi-writer Active-Active out of release-blocking evidence paths until ordering/metadata caveats are proven safe under workload and failure injection.

## Required TODO Alignment

1. `SIM2-GC-15`: include stream/index retention mechanics that avoid read-path scans and use bounded purge work.
2. `SIM2-GC-16`: include Rust backpressure, drop accounting, and query-budget enforcement evidence.
3. `SIM2-GC-17`: include classification/scrubbing + leak-canary protections with measurable enforcement.
4. `SIM2-GC-11`: include new benchmark and failure-injection tests for sync lag, ordering, and drop/loss behavior.
5. `DEP-ENT-*`: include distributed-state convergence and outage tests tied to explicit SLOs.

## Definition of “Validated” for This Delta

Validation is complete when:

1. The architecture choice for write authority and edge read semantics is explicit and test-backed.
2. Rust runtime backpressure and telemetry loss behavior are measured under declared envelopes.
3. Retention/cost/security controls are benchmarked with threshold gates, not only code-path presence.
4. Enterprise sync behavior has measurable convergence and failure-mode evidence.
