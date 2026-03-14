# 🐙 Observability & Grafana

## 🐙 Prometheus Metrics Endpoint

Shuma-Gorath exposes Prometheus-compatible metrics at:

```
GET /metrics
```

This endpoint is unauthenticated for Prometheus compatibility. Restrict access at the network edge if required.

## 🐙 Dashboard Monitoring Summary <abbr title="Application Programming Interface">API</abbr>

The Monitoring tab now consumes a consolidated admin summary endpoint:

```
GET /admin/monitoring?hours=24&limit=10
```

This endpoint returns bounded-cardinality summaries for:
- test-mode shadow actions (`would challenge`, `would block`, `would maze`, and pass-through totals)
- honeypot hits (top crawler buckets + top paths)
- challenge rejections and attack signals (reasons + trend)
- <abbr title="Proof of Work">PoW</abbr> verification outcomes (success/failure + reasons + trend)
- rate-limiting violations (outcomes + offenders + top paths)
- <abbr title="Geolocation">GEO</abbr> violations (actions + top countries)

Use this endpoint for dashboard <abbr title="User Experience">UX</abbr> and operator <abbr title="Application Programming Interface">API</abbr> queries; use `/metrics` for external time-series scraping.

Recent event rows also carry explicit execution metadata:
- `execution_mode="enforced|shadow"`
- `shadow_source="test_mode"` when shadowed
- `intended_action="challenge|block|maze|..."` for shadow rows
- `enforcement_applied=false` for shadow rows

Dashboard monitoring uses those fields directly rather than inferring shadow semantics from free-text reasons/outcomes.

Prometheus parity scope for Monitoring widgets is tracked in:
- [`docs/monitoring-prometheus-parity-audit.md`](monitoring-prometheus-parity-audit.md)

### 🐙 Monitoring Cost Controls

- Monitoring counter writes are coalesced in a short in-memory buffer before <abbr title="Key-Value">KV</abbr> flushes to reduce hot-path read/modify/write amplification.
- Path dimensions are normalized and cardinality-capped (`<=3` segments plus wildcard tail, dynamic/high-entropy segments collapsed to `:id`) to prevent unbounded key growth.
- Retention cleanup runs through a bounded background worker over hourly bucket catalogs rather than opportunistic summary-read cleanup.
- Operational monitoring counters, raw event records, and derived daily rollups now have separate retention tiers:
  - `SHUMA_MONITORING_RETENTION_HOURS` for hourly monitoring counters and bucket indexes,
  - `SHUMA_EVENT_LOG_RETENTION_HOURS` for requested raw event retention, with high-risk raw operator views capped to `72h`,
  - `SHUMA_MONITORING_ROLLUP_RETENTION_HOURS` for derived daily monitoring rollups.
- the 2026-03-14 post-compaction retention rebaseline kept those default windows unchanged: compact raw rows are materially smaller, but live retained-byte evidence shows hot-read documents and retention metadata still dominate the retained footprint on the measured shared-host sample.
- Monitoring summary, delta, stream, and normal monitoring-details reads use bucket indexes and key catalogs instead of whole-keyspace `get_keys()` scans.
- Monitoring query budgets account for requested window, bucket count, key count, residual scans, and dense-bucket penalty rather than only `hours * limit`.
- The first live shared-host baseline and the current hot-path compression decision are archived in [`docs/research/2026-03-11-shared-host-telemetry-storage-query-evidence.md`](research/2026-03-11-shared-host-telemetry-storage-query-evidence.md).
- The hot-read telemetry contract is now explicit in code: current bootstrap candidates derived from immutable event records or direct state snapshots are treated as exact, while summaries derived from mutable shared counters or retention catalogs are treated as best-effort until the unified hot-read architecture replaces those sources with a non-racy projection path. See [`src/observability/hot_read_contract.rs`](../src/observability/hot_read_contract.rs) and [`docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md).
- The durable hot-read document contract is now explicit too: bootstrap and supporting summary documents have versioned storage keys, freshness/rebuild budgets, bounded size caps, and an explicit drill-down-only field list so the later bootstrap rewrite can stay fast on Fermyon and still share one storage/read shape with Linode. See [`src/observability/hot_read_documents.rs`](../src/observability/hot_read_documents.rs).
- Hot-read documents are now maintained centrally from existing write paths rather than rebuilt opportunistically in request handlers: counter flushes refresh the monitoring summary and supporting summaries, event appends refresh the recent tail, retention worker passes refresh retention state only when the worker actually runs, and config or ban mutations refresh the bootstrap posture. The central projection logic lives in [`src/observability/hot_read_projection.rs`](../src/observability/hot_read_projection.rs).
- Canonical bootstrap monitoring reads (`/admin/monitoring?hours=24&limit=10&bootstrap=1`) now consume the materialized monitoring summary and bootstrap documents directly. Custom windows, forensic reads, delta cursors, streams, and deeper monitoring drill-down still use the bounded bucket/raw paths instead of a parallel telemetry store.

### 🐙 Metrics Included

- `bot_defence_requests_total`
- `bot_defence_bans_total{reason="..."}`
- `bot_defence_blocks_total`
- `bot_defence_challenges_total`
- `bot_defence_challenge_served_total`
- `bot_defence_challenge_solved_total`
- `bot_defence_challenge_incorrect_total`
- `bot_defence_challenge_expired_replay_total`
- `bot_defence_cdp_detections_total`
- `bot_defence_allowlisted_total`
- `bot_defence_test_mode_actions_total`
- `bot_defence_monitoring_shadow_actions_total{action="challenge|block|maze|..."}`
- `bot_defence_monitoring_shadow_pass_through_total`
- `bot_defence_maze_hits_total`
- `bot_defence_maze_token_outcomes_total{outcome="entry|validated|invalid|expired|replay|binding_mismatch|depth_exceeded|budget_exceeded|checkpoint_missing|micro_pow_failed"}`
- `bot_defence_maze_checkpoint_outcomes_total{outcome="accepted|method_not_allowed|binding_mismatch|invalid"}`
- `bot_defence_maze_budget_outcomes_total{outcome="acquired|saturated|response_cap_exceeded"}`
- `bot_defence_maze_proof_outcomes_total{outcome="required|passed|failed"}`
- `bot_defence_maze_entropy_variants_total{variant="...",provider="internal|operator",metadata_only="true|false"}`
- `bot_defence_active_bans`
- `bot_defence_test_mode_enabled`
- `bot_defence_botness_signal_state_total{signal="...",state="active|disabled|unavailable"}`
- `bot_defence_defence_mode_effective_total{module="rate|geo|js",configured="off|signal|enforce|both",signal_enabled="true|false",action_enabled="true|false"}`
- `bot_defence_edge_integration_mode_total{mode="off|additive|authoritative"}`
- `bot_defence_provider_implementation_effective_total{capability="...",backend="internal|external",implementation="..."}`
- `bot_defence_rate_limiter_backend_errors_total{route_class="main_traffic|admin_auth"}`
- `bot_defence_rate_limiter_outage_decisions_total{route_class="...",mode="fallback_internal|fail_open|fail_closed",action="fallback_internal|allow|deny",decision="allowed|limited"}`
- `bot_defence_rate_limiter_usage_fallback_total{route_class="...",reason="backend_error|backend_missing"}`
- `bot_defence_rate_limiter_state_drift_observations_total{route_class="...",delta_band="delta_0|delta_1_5|delta_6_20|delta_21_plus"}`
- `bot_defence_policy_matches_total{level="L*...",action="A*...",detection="D*..."}`
- `bot_defence_policy_signals_total{signal="S_*"}`
- `bot_defence_monitoring_challenge_failures_total{reason="incorrect|expired_replay|sequence_violation|invalid_output|forbidden"}`
- `bot_defence_monitoring_pow_verifications_total{outcome="success|failure"}`
- `bot_defence_monitoring_pow_failures_total{reason="invalid_proof|missing_seed_nonce|sequence_violation|expired_replay|binding_timing_mismatch"}`
- `bot_defence_monitoring_rate_violations_total{outcome="limited|banned|fallback_allow|fallback_deny"}`
- `bot_defence_monitoring_geo_violations_total{action="block|challenge|maze"}`

## 🐙 Prometheus Scrape Example

```yaml
scrape_configs:
  - job_name: shuma-gorath
    static_configs:
      - targets: ["your-domain.example.com"]
    metrics_path: /metrics
```

## 🐙 Grafana Integration

1. Add Prometheus as a data source
2. Build panels for requests total, bans by reason, active bans, challenges/blocks over time, test mode status, and composability visibility (signal-state and effective-mode counters)

## 🐙 Botness Visibility

- Botness-driven challenge/maze events include:
  - active signal summary (`signals=...`)
  - full state summary (`signal_states=key:state:contribution,...`)
  - runtime metadata summary (`modes=rate=... geo=... js=... edge=...`)
  - provider summary (`providers=rate_limiter=... ban_store=... challenge_engine=... maze_tarpit=... fingerprint_signal=...`)
- Policy-driven event outcomes append canonical taxonomy metadata:
  - `taxonomy[level=L* action=A* detection=D* signals=S_*...]`
- Use this event context with the two composability metrics to distinguish:
  - intentional disabled signals (`state=disabled`),
  - unavailable inputs (`state=unavailable`), and
  - active contributors (`state=active`).

## 🐙 Edge Provider Cutover Monitoring

Use this when moving from internal-only to additive/authoritative edge integration.

### 1. Mode and Provider Selection Checks

- Confirm configured edge mode is being observed:
  - `bot_defence_edge_integration_mode_total{mode="off|additive|authoritative"}`
- Confirm active provider implementation by capability:
  - `bot_defence_provider_implementation_effective_total{capability="...",backend="...",implementation="..."}`
  - For `rate_limiter` external mode, expect `implementation="external_redis_with_internal_fallback"`.
  - For `ban_store` external mode, expect `implementation="external_redis_with_internal_fallback"`.
  - For `fingerprint_signal` external mode, expect `implementation="external_akamai_with_internal_fallback"`.
- Use `increase(...)` windows in PromQL to verify recent behavior rather than cumulative lifetime totals.

Example PromQL (last 10 minutes):

```promql
sum by (mode) (increase(bot_defence_edge_integration_mode_total[10m]))
```

```promql
sum by (capability, backend, implementation) (
  increase(bot_defence_provider_implementation_effective_total[10m])
)
```

### 2. Signal Health Checks

- Watch unavailable signal-state growth during external cutover:
  - `bot_defence_botness_signal_state_total{state="unavailable"}`
- For fingerprint migrations specifically, confirm provider implementation remains `external_akamai_with_internal_fallback` and investigate sudden drops in detection throughput (`bot_defence_cdp_detections_total`) after enablement.

Example PromQL (last 10 minutes):

```promql
sum by (signal, state) (increase(bot_defence_botness_signal_state_total[10m]))
```

### 3. Outcome Sanity Checks

Correlate provider/mode transitions with:

- `bot_defence_challenges_total`
- `bot_defence_blocks_total`
- admin event outcomes that include:
  - `signal_states=...`
  - `modes=... edge=...`
  - `providers=...`

If challenge/block behavior changes sharply without matching traffic or threat context, roll back to internal/`off` and investigate.

### 4. Edge-Backed Rate-Limiter Degradation Monitoring

During external rate-limiter operation, watch:

- backend errors:
  - `bot_defence_rate_limiter_backend_errors_total`
- degraded decisions and selected outage modes:
  - `bot_defence_rate_limiter_outage_decisions_total`
- usage-read fallback behavior:
  - `bot_defence_rate_limiter_usage_fallback_total`
- external vs local shadow drift bands:
  - `bot_defence_rate_limiter_state_drift_observations_total`

Example PromQL (last 10 minutes):

```promql
sum by (route_class) (increase(bot_defence_rate_limiter_backend_errors_total[10m]))
```

```promql
sum by (route_class, mode, action, decision) (
  increase(bot_defence_rate_limiter_outage_decisions_total[10m])
)
```

```promql
sum by (route_class, delta_band) (
  increase(bot_defence_rate_limiter_state_drift_observations_total[10m])
)
```

### 5. Minimum Alerting Guidance

During any edge provider rollout, alert on:

- sustained increase in unavailable signal state,
- unexpected provider implementation label changes,
- sudden challenge/block jumps versus baseline.

## 🐙 Spin Cloud Monitoring

```bash
spin cloud logs
spin cloud apps info
spin cloud apps metrics
```
