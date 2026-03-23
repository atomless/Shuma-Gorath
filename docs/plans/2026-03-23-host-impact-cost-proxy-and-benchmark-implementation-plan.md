Date: 2026-03-23
Status: Proposed

Related context:

- [`../research/2026-03-23-host-impact-cost-proxy-and-benchmark-review.md`](../research/2026-03-23-host-impact-cost-proxy-and-benchmark-review.md)
- [`../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`../research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../research/2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`2026-03-20-benchmark-suite-v1-design.md`](2026-03-20-benchmark-suite-v1-design.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)
- [`../../src/runtime/effect_intents/response_renderer.rs`](../../src/runtime/effect_intents/response_renderer.rs)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)
- [`../../src/observability/benchmark_results_families.rs`](../../src/observability/benchmark_results_families.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)

# Objective

Make Shuma's closed loop capable of judging suspicious host impact with a truthful latency-shaped proxy, not only with forwarded request and byte ratios.

# Core Decisions

1. Extend the existing `suspicious_origin_cost` family instead of creating a second parallel cost family.
2. Keep the first host-impact proxy ratio-shaped so it fits the existing objective and budget model cleanly.
3. Use observed forwarded upstream duration as the next truthful host-impact signal.
4. Do not introduce speculative route weights, fake cost units, or direct CPU or memory budgets in this tranche.
5. Land the telemetry foundation before the benchmark and objective wiring.
6. Finish this work before `MON-OVERHAUL-1` so Monitoring projects the settled host-impact semantics instead of the older request-and-byte-only model.

# Tranche Plan

## `HOST-COST-1`: Bounded Host-Impact Telemetry Foundation

### Goal

Record forwarded upstream duration in the same bounded request-outcome telemetry path that already owns lane, category, request, and byte truth.

### Files

- Modify:
  - `src/runtime/request_outcome.rs`
  - `src/runtime/effect_intents/response_renderer.rs`
  - `src/observability/monitoring.rs`
  - `src/observability/hot_read_projection.rs`
  - `src/observability/operator_snapshot_live_traffic.rs`
  - `src/observability/hot_read_contract.rs`
  - `Makefile`
- Update docs:
  - `docs/observability.md`
  - `docs/api.md`
  - `docs/testing.md`

### Required behavior

1. Every forwarded request outcome should carry bounded forwarded-duration truth.
2. Monitoring scope and lane summaries should accumulate forwarded latency totals alongside request and byte totals.
3. The new telemetry must remain:
   - origin-separated,
   - lane-aware,
   - bounded,
   - and exactness-aware.
4. No raw per-request latency tail or unbounded bucket explosion should be introduced.

### Verification target

Add and use a focused make target such as `make test-host-impact-telemetry`.

It should prove:

1. forwarded requests increment forwarded latency totals,
2. short-circuited and control responses do not,
3. scope and lane summaries surface the new totals,
4. hot-read projection keeps the same truth.

## `HOST-COST-2`: Snapshot, Objective, And Benchmark Integration

### Goal

Thread the new host-impact telemetry into the machine-first snapshot and benchmark loop as a first-class suspicious-origin metric.

### Files

- Modify:
  - `src/observability/operator_snapshot.rs`
  - `src/observability/operator_snapshot_objectives.rs`
  - `src/observability/benchmark_results_families.rs`
  - `src/observability/benchmark_results.rs`
  - `src/observability/benchmark_comparison.rs`
  - `src/observability/benchmark_suite.rs`
  - `src/admin/oversight_reconcile.rs`
  - `Makefile`
- Update docs:
  - `docs/configuration.md`
  - `docs/current-system-architecture.md`
  - `docs/testing.md`

### Required behavior

1. Add a new objective budget metric:
   - `suspicious_forwarded_latency_share`
2. Compute that row from:
   - suspicious-lane forwarded latency total,
   - divided by total live forwarded latency total in the same watch window.
3. Extend the `suspicious_origin_cost` benchmark family with:
   - budget metric `suspicious_forwarded_latency_share`,
   - tracking metric `suspicious_average_forward_latency_ms`
4. Include the new metric in prior-window and candidate comparison semantics.
5. Let reconcile consume the new metric through the existing suspicious-origin family rather than inventing a second pressure model.

### Guardrails

1. Keep the new budget metric ratio-shaped and bounded.
2. Do not change proposal families or patch shaping in this tranche.
3. If forwarded latency is unavailable for a window, expose `insufficient_evidence`; do not synthesize values.

### Verification target

Add and use focused make targets such as:

1. `make test-host-impact-benchmark`
2. `make test-oversight-host-impact`

They should prove:

1. snapshot budget rows include the new metric when telemetry exists,
2. benchmark comparison marks it improved or regressed correctly,
3. suspicious-origin outside-budget pressure can now be triggered by latency-share misses,
4. and the loop still fails closed when telemetry is stale or missing.

# Sequencing And Commit Strategy

1. Land `HOST-COST-1` in one atomic implementation slice with focused tests and docs.
2. Review it immediately for boundedness, retained-size implications, and telemetry exactness.
3. Land `HOST-COST-2` in a second atomic slice with benchmark, objective, and reconcile wiring.
4. Review it immediately for:
   - controller semantics,
   - benchmark comparison truth,
   - and no drift toward speculative cost modeling.

# Operational Notes

1. This plan intentionally improves host-impact truth before Monitoring, not inside Monitoring.
2. The resulting metrics remain proxies for host impact, not literal billing or CPU cost.
3. That is acceptable for the first loop because they are traffic-attributable, bounded, comparable, and already aligned with Shuma's tuning architecture.

# Definition Of Done

This work is done when:

1. forwarded upstream duration is part of the bounded machine-first telemetry path,
2. `operator_snapshot_v1` includes a truthful host-impact budget row,
3. `benchmark_results_v1` includes the new suspicious-origin host-impact proxy,
4. prior-window and candidate comparison treat the metric correctly,
5. reconcile can use it through the existing suspicious-origin pressure family,
6. docs explain the proxy honestly,
7. and the Monitoring overhaul can safely treat host-impact cost as a settled backend truth rather than a UI invention.
