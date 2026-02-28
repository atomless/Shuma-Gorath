# SIM2-GCR-6 Research: Monitoring Pipeline Cost-Efficiency Patterns

Date: 2026-02-28  
Status: Recommended cost-control model selected

## Objective

Identify the best cost-efficiency patterns for Shuma’s monitoring pipeline across aggregation windows, cardinality control, sampling restrictions, compression/serialization, and query budgets.

## Repository Baseline (Current State)

1. Monitoring pipeline already includes some cardinality reduction (`normalize_telemetry_path` segment limits, bucketed IP identity, top-N limits).
2. Monitoring and event query paths still perform substantial per-refresh aggregation work and can scan broad keyspaces.
3. Admin read rate limiting exists (`expensive_admin_read_*`, dashboard refresh session limits), but there is no end-to-end quantitative cost budget contract yet.
4. Realtime work now targets cursor/SSE freshness, but explicit cost guardrails must mature in parallel.

## Primary-Source Findings

1. High-cardinality labels dramatically increase monitoring storage/query cost; unbounded dimensions should be avoided.
   Sources:
   - [Prometheus instrumentation practices](https://prometheus.io/docs/practices/instrumentation/)
   - [Prometheus metric and label naming caution](https://prometheus.io/docs/practices/naming/)
2. Precomputed recording/rollup rules are a proven pattern to reduce repeated expensive dashboard queries.
   Source: [Prometheus recording rules](https://prometheus.io/docs/prometheus/3.0/configuration/recording_rules/)
3. Sampling is explicitly a mechanism for controlling telemetry overhead; head decisions should avoid wasting expensive enrichment on non-sampled paths.
   Source: [OpenTelemetry trace SDK sampling](https://opentelemetry.io/docs/specs/otel/trace/sdk/)
4. Compression is standardized for transport and can reduce payload transfer cost when negotiated correctly.
   Sources:
   - [RFC 9110 Content-Encoding / Accept-Encoding](https://datatracker.ietf.org/doc/rfc9110/)
   - [RFC 8878 Zstandard](https://datatracker.ietf.org/doc/html/rfc8878)
5. Query/control-plane budgeting can be expressed with deterministic token-bucket style controls.
   Source: [RFC 2697 srTCM token bucket model](https://www.rfc-editor.org/rfc/rfc2697.html)

## Inferences for Shuma (Derived from Sources)

1. Cost control must be layered, not single-knob: cardinality guardrails + rollups + selective sampling + query budget + payload compression.
2. Security-critical defense outcomes must remain unsampled; only low-value/high-volume telemetry should be eligible for sampling/downsampling.
3. Cost metrics should be first-class operational telemetry (`pipeline_cost_status`) with explicit degraded-state signaling.

## Architecture Options

### Option A: Keep Existing Ad-hoc Guards

Retain current path normalization and rate limits without explicit budget contracts or rollup architecture.

### Option B: Aggressive Sampling First

Reduce volume primarily by broad sampling of monitoring events.

### Option C: Layered Budgeted Pipeline (Recommended)

Apply strict cardinality caps, rollup windows for dashboards, selective sampling with unsampleable security classes, endpoint query budgets, and compression/payload limits.

### Option D: Externalize monitoring cost problem to third-party stack

Move cost controls largely out of Shuma and rely on external observability pipeline.

## Decision Matrix

| Option | Benefits | Risks | Resource Cost | Security Impact | Rollback Complexity |
|---|---|---|---|---|---|
| A. Existing ad-hoc guards | Minimal immediate change | Cost drift and query amplification remain likely | Low initial, rising runtime/ops cost | Moderate | Low |
| B. Sampling-first | Fast volume reduction | High risk of losing security-salient evidence | Low-medium | Weak for defense assurance | Low-medium |
| C. Layered budgeted pipeline (recommended) | Balanced cost reduction with evidence integrity and deterministic controls | Requires broader implementation surface | Medium | Strong (critical events preserved) | Medium |
| D. Externalize | Potential scale efficiency | Dependency and trust-boundary complexity | High | Variable by provider | High |

## Recommendation

Adopt **Option C**.

Required controls:

1. **Cardinality budgets**
   1. Per-dimension caps with explicit overflow bucket (`other`).
   2. Reject or coarsen unbounded dimensions at ingest.
2. **Aggregation windows**
   1. Precompute dashboard rollups (for example `1m`, `5m`, `1h`) to avoid repeated full-range scans.
   2. Keep raw event path for evidence/troubleshooting.
3. **Sampling restrictions**
   1. Never sample/drop security-critical outcomes (ban/challenge failure/maze escalation/replay/security violations).
   2. Allow bounded deterministic sampling only for low-risk, high-frequency informational telemetry.
4. **Query budgets**
   1. Endpoint/session token-bucket budgets tied to monitoring read cost classes.
   2. Explicit degraded signaling when query budget is exceeded.
5. **Payload efficiency**
   1. Negotiate compression (`gzip` baseline; optional `zstd` where supported).
   2. Enforce payload-size caps with pagination/cursor continuation.

## Quantitative Targets (for TODO enforcement)

1. Default monitoring response payload target: `p95 <= 512KB` under declared realtime envelope.
2. Cardinality cap: `<=1000` distinct values per guarded dimension per hour; excess values coalesce into `other` bucket.
3. Unsampleable defense-event classes: `0` sampled/dropped records allowed.
4. Query budget: monitoring live path remains within `<=1 request/sec/client` average in non-degraded streaming-enabled mode.
5. Compression effectiveness: for payloads `>64KB`, negotiated compression should reduce transferred bytes by `>=30%` in benchmark profile.

## Plan and TODO Impact

1. New plan doc: `docs/plans/2026-02-28-sim2-gcr-6-monitoring-cost-efficiency-patterns-plan.md`.
2. Add dedicated cost-governance implementation slice (`SIM2-GC-16`) for cardinality, rollups, sampling policy, payload budgets, and query budgeting.
3. Add CI regression checks under `SIM2-GC-11` for cost thresholds and unsampleable-event guarantees.
