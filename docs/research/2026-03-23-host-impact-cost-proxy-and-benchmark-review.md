# Host-Impact Cost Proxy And Benchmark Review

Date: 2026-03-23
Status: active research

Related context:

- [`2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](./2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`2026-03-19-controller-readiness-telemetry-foundation-review.md`](./2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`2026-03-20-benchmark-suite-v1-research-synthesis.md`](./2026-03-20-benchmark-suite-v1-research-synthesis.md)
- [`../plans/2026-03-20-benchmark-suite-v1-design.md`](../plans/2026-03-20-benchmark-suite-v1-design.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)
- [`../../src/observability/benchmark_results_families.rs`](../../src/observability/benchmark_results_families.rs)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)

External references:

- OpenTelemetry HTTP metrics semantic conventions: [https://opentelemetry.io/docs/specs/semconv/http/http-metrics/](https://opentelemetry.io/docs/specs/semconv/http/http-metrics/)
- Prometheus histogram guidance: [https://prometheus.io/docs/practices/histograms/](https://prometheus.io/docs/practices/histograms/)
- Google SRE canarying guidance: [https://sre.google/workbook/canarying-releases/](https://sre.google/workbook/canarying-releases/)

# Purpose

Define the next truthful extension to Shuma's feedback loop so it can judge not only how much suspicious traffic still reaches the origin, but also how much host-impact that suspicious traffic is imposing while it does so.

This note is intentionally about the machine-first control contract, not about Monitoring presentation.

# Executive Summary

Shuma already has a usable first-wave cost model, but it is still mostly an origin reach and bandwidth proxy.

Today the loop can measure:

1. how much suspicious traffic is still forwarded,
2. how many suspicious bytes are still served by the origin,
3. how much suspicious traffic is short-circuited locally,
4. and how much byte share Shuma is absorbing instead of the origin.

That is enough to launch the first bounded config loop, but it is not yet enough to claim that Shuma is measuring the full host impact from unwanted non-human traffic.

The main missing truth is latency- or work-shaped origin impact.

Shuma already records upstream forward latency in Prometheus-style metrics, but that signal is not yet part of the machine-first operator snapshot or benchmark contract. The current diagnoser therefore cannot use it when deciding whether suspicious traffic is still imposing too much host work.

The clean next move is not speculative "cost units", CPU heuristics, or hand-assigned route weights. The clean next move is to add one bounded, aggregatable, benchmark-grade host-impact proxy built from observed request outcomes:

1. record forwarded upstream duration on request outcomes,
2. aggregate forwarded latency totals by scope and lane in the same bounded telemetry path that already owns requests and bytes,
3. materialize a new suspicious-origin metric based on suspicious share of forwarded latency,
4. and let the existing benchmark and reconcile machinery consume that metric without inventing a second tuning model.

# What Shuma Measures Today

Current suspicious-origin cost is derived from:

1. `suspicious_forwarded_request_rate`,
2. `suspicious_forwarded_byte_rate`,
3. `suspicious_short_circuit_rate`,
4. `suspicious_locally_served_byte_share`.

These come from:

1. per-request rendered outcomes in [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs),
2. bounded request-outcome summaries in [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs),
3. budget-distance rows in [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs),
4. and the `suspicious_origin_cost` benchmark family in [`../../src/observability/benchmark_results_families.rs`](../../src/observability/benchmark_results_families.rs).

This is already good at answering:

1. how much suspicious traffic still consumes origin request handling,
2. how much suspicious byte volume still leaves the origin,
3. and whether Shuma is shifting work onto cheaper local responses.

It is not yet good at answering:

1. how much origin time suspicious traffic is consuming,
2. whether suspicious traffic is disproportionately expensive per forwarded request,
3. or whether a config change reduces request count but leaves a large long-tail latency burden behind.

# Gap Analysis

## 1. Forward latency exists, but outside the controller contract

The runtime already records forward latency in [`../../src/observability/metrics.rs`](../../src/observability/metrics.rs), and the response renderer already measures it in [`../../src/runtime/effect_intents/response_renderer.rs`](../../src/runtime/effect_intents/response_renderer.rs).

But that signal currently stays outside:

1. `request_outcome` summaries,
2. `operator_snapshot_v1`,
3. `benchmark_results_v1`,
4. and the reconcile loop.

Result:

1. the loop can optimize for fewer forwarded requests and bytes,
2. but it cannot yet optimize for less forwarded origin time.

## 2. The current objective model is ratio-shaped

`operator_objectives_v1` currently uses `max_ratio` budgets in [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs).

That is good news for a first host-impact extension because it means the cleanest new benchmark metric is not raw milliseconds, but a bounded ratio derived from milliseconds.

The most truthful first metric is:

1. suspicious share of total forwarded origin latency over the current window.

That keeps the objective model stable, comparable, and bounded.

## 3. Fake work weights would violate the repo's telemetry-as-map rule

It would be tempting to assign static weights to route families or content types and call the result "origin work cost".

That would be a mistake right now.

Reasons:

1. the weights would be speculative rather than observed,
2. they would differ wildly across sites,
3. and they would drift the controller away from the repository rule that telemetry is the map.

So the first slice should not invent weighted cost units.

## 4. Direct CPU and memory accounting is not yet the right first contract

Direct host CPU, memory, and energy metrics remain desirable later, but they are not the right first benchmark extension.

Reasons:

1. they are deployment- and runtime-specific,
2. they are harder to attribute cleanly per traffic lane,
3. and they are easier to misread when shared-host background work changes independently of traffic posture.

The current feedback loop needs a traffic-attributable host-impact proxy first.

# External Guidance And Why It Matters Here

## OpenTelemetry

OpenTelemetry's HTTP metric conventions explicitly standardize request duration and request or response body size for both server and client paths. That matches Shuma's gap exactly: request and byte size are first-order transport or work proxies, and duration is the next truthful work proxy once bytes alone are not enough.

This supports adding forwarded duration into Shuma's machine-first contract instead of inventing a bespoke cost vocabulary first.

## Prometheus

Prometheus' histogram guidance says request durations and response sizes are the right kinds of observations for histograms, and it strongly prefers histograms when values must be aggregated across instances or windows.

That supports a bounded latency-bucket or duration-sum approach over per-request raw tails, and it also argues against client-local quantile semantics as the first operator contract.

## Google SRE

Google's canary guidance is about judging change against service-impact metrics rather than intuition. For Shuma, that means the tuning loop should not stop at "fewer suspicious requests reached origin" if suspicious traffic still consumes a large share of forwarded origin time. The candidate-vs-baseline judgment should eventually include that latency-shaped host-impact proxy as part of the protected watch-window evidence.

# Recommended Host-Impact Model v1

## Core decision

Extend the existing `suspicious_origin_cost` family rather than creating a parallel cost family.

Add one new budgetable metric:

1. `suspicious_forwarded_latency_share`

Meaning:

1. suspicious-lane forwarded latency sum,
2. divided by total live forwarded latency sum,
3. over the same bounded watch window.

This answers:

1. what share of the origin's total forwarded time is being consumed by suspicious traffic.

That is a better host-impact proxy than suspicious-request count alone.

## Supporting tracking metrics

Also add bounded, non-budget-first tracking metrics:

1. `suspicious_average_forward_latency_ms`
2. optionally later `suspicious_high_latency_forwarded_request_rate`

These are useful for operator interpretation and later Monitoring, but they do not need to become first-wave objective rows on day one.

## Telemetry basis

Materialize the latency proxy through the same request-outcome path that already carries:

1. origin separation,
2. lane classification,
3. non-human category assignment,
4. response byte counts,
5. and execution mode.

That keeps the host-impact model:

1. bounded,
2. explainable,
3. lane-aware,
4. and consistent with the rest of the snapshot and benchmark architecture.

# Recommended Sequence

## `HOST-COST-1`

Add bounded forwarded-latency telemetry to request outcomes and request-outcome summaries.

Deliverables:

1. request-outcome field for forwarded upstream duration,
2. scope and lane aggregates for forwarded latency totals,
3. exactness and basis carried through hot-read summaries,
4. no unbounded raw latency tail storage.

## `HOST-COST-2`

Thread that telemetry into `operator_snapshot_v1`, `operator_objectives_v1`, and `benchmark_results_v1`.

Deliverables:

1. new budget metric `suspicious_forwarded_latency_share`,
2. snapshot budget-distance row generation,
3. suspicious-origin benchmark-family extension,
4. prior-window and candidate comparison support for the new metric,
5. reconcile consumption through the existing suspicious-origin pressure path.

# Explicit Non-Goals For This Slice

Do not include:

1. static route-family work weights,
2. speculative per-path origin cost models,
3. CPU or memory budgets as first-wave tuning inputs,
4. energy accounting,
5. or Monitoring redesign.

Those can follow later if and only if Shuma can materialize them as truthful, traffic-attributable, bounded signals.

# Conclusion

The right next improvement is not "full cost accounting".

The right next improvement is:

1. keep the current request and byte proxies,
2. add a latency-shaped host-impact proxy derived from observed forwarded work,
3. and let the existing snapshot, benchmark, and reconcile contracts consume it before Monitoring is redesigned.

That is the smallest honest extension that makes the closed loop materially better at optimizing for reduced host impact from unwanted non-human traffic.
