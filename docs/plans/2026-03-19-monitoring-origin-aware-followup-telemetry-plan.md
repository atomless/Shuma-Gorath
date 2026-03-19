Date: 2026-03-19
Status: Active implementation plan

Related context:

- [`../research/2026-03-19-defence-funnel-origin-integrity-review.md`](../research/2026-03-19-defence-funnel-origin-integrity-review.md)
- [`../research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../research/2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`2026-03-19-monitoring-defence-funnel-plan.md`](./2026-03-19-monitoring-defence-funnel-plan.md)
- [`2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md`](./2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)

# Purpose

Define the immediate follow-on slice after the first defence-funnel implementation:

1. make legacy `not_a_bot`, `challenge`, and `pow` follow-up telemetry origin-aware,
2. switch the operator-facing summaries to live-only truth by default,
3. and safely re-enable the richer defence-funnel stages that were temporarily withheld while those counters were origin-blended.

# Why This Slice Must Happen Before The Monitoring Overhaul

The Monitoring overhaul should not be forced to invent workarounds for backend truth gaps.

Without this slice:

1. top-level `challenge`, `not_a_bot`, and `pow` summaries could still blend external traffic with adversary-sim traffic,
2. the funnel would either overclaim or remain artificially sparse,
3. and later bounded controller benchmarks would inherit the same ambiguity.

# Design Decision

## 1. Keep the family telemetry paths, but add explicit traffic-origin separation

The older family-specific counters remain useful, but they must stop behaving like anonymous global totals.

For:

1. `challenge`
2. `not_a_bot`
3. `pow`

record the same semantics with an explicit `traffic_origin` cohort so the backend can distinguish:

1. `live`
2. `adversary_sim`

without adding a second analytics subsystem.

## 2. Operator summaries become live-only by default

The Monitoring operator surface is about defended external traffic.

So the backend summaries consumed by Monitoring should default to the `live` origin for:

1. challenge failure totals, reasons, offenders, and trends,
2. `not_a_bot` served, submitted, outcomes, and latency,
3. PoW totals, successes, failures, reasons, offenders, outcomes, and trends.

This does not mean adversary-sim evidence is discarded. It means it must no longer silently inhabit the same rows as live operator truth.

## 3. Re-enable the richer funnel stages only after origin integrity exists

Once the family counters are origin-aware, the funnel may safely populate:

1. `not_a_bot.passed_requests`
2. `not_a_bot.failed_requests`
3. `not_a_bot.escalated_requests`
4. `challenge.failed_requests`
5. `pow.*`

using the live-only family summaries.

# Implementation Rules

1. Reuse the existing monitoring keyspace and summary machinery rather than creating new ad hoc rollups.
2. Keep the read model bounded.
3. Preserve adversary-sim evidence in storage and summaries that are meant for Red Team or later tuning work.
4. Do not widen bootstrap payloads just to make this slice work.

# Verification Plan

Use:

1. `make test-monitoring-telemetry-foundation-unit`

Add or update proofs that:

1. `challenge`, `not_a_bot`, and `pow` operator summaries ignore adversary-sim-origin counters,
2. the richer funnel stages return once their backing counters are origin-safe,
3. hot-read summary and bootstrap documents carry the updated live-only summary truth,
4. adversary-sim evidence still remains visible through the origin-aware counter path rather than disappearing.

# Outcome

When this lands, Shuma will have a cleaner controller-grade backend contract:

1. live operator truth,
2. separate adversary-sim evidence,
3. and richer family-stage funnel semantics without hidden origin blending.
