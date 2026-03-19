Date: 2026-03-19
Status: Active implementation plan

Related context:

- [`../research/2026-03-17-operator-decision-support-telemetry-audit.md`](../research/2026-03-17-operator-decision-support-telemetry-audit.md)
- [`../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`../research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../research/2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`2026-03-18-monitoring-traffic-lane-and-denominator-contract.md`](./2026-03-18-monitoring-traffic-lane-and-denominator-contract.md)
- [`2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](./2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md)
- [`2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md`](./2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../todos/todo.md`](../../todos/todo.md)

# Purpose

Define the minimum telemetry addition needed to turn Shuma's existing human-friction evidence into truthful rates before the Monitoring UI overhaul.

This plan is intentionally narrow. It does not redesign Monitoring and it does not yet build the full defence-effectiveness funnel. It only settles the denominator and summary contract for `MON-TEL-1-3`.

# Why This Is The Next Slice

Shuma already has:

1. coarse live-versus-adversary-sim request totals,
2. coarse lane totals,
3. existing subsystem summaries for `not_a_bot`, `pow`, and challenge failure outcomes,
4. and richer bounded backend breakdowns for `response_kind`, `policy_source`, and `route_action_family`.

What it still lacks is one truthful way to answer:

1. how much likely-human traffic saw friction,
2. how much unknown-interactive traffic saw friction,
3. how much combined interactive traffic saw friction,
4. and which friction families drove that burden.

The cheapest correct answer is not another bespoke friction subsystem. It is one extra request-outcome breakdown family that preserves lane-specific friction numerators so the backend can derive rates from the lane denominators it already owns.

# Design Decision

## 1. Add one bounded `lane_response_kind` counter family

Extend the existing request-outcome counter family with one new low-cardinality nested cohort:

1. `lane_response_kind`

It should be keyed by the existing lane cohort plus normalized `response_kind`.

This is the minimum new telemetry needed because:

1. lane totals already exist,
2. scope-level `response_kind` rows already exist,
3. but Shuma cannot currently tell which friction response kinds landed in which human-adjacent lanes.

No new raw event rows, request-time scans, or per-path/per-IP breakdowns should be introduced.

## 2. Derive a compact `human_friction` summary from request outcomes

Add a bounded summary that is derived from:

1. existing lane totals,
2. the new `lane_response_kind` counts,
3. and the existing `not_a_bot` / `pow` subsystem summaries where those remain the richer source for solve and abandonment details.

The summary should answer issuance-rate questions only. Solve/fail/escalation details already remain in the existing subsystem summaries.

# Summary Contract

Add a compact top-level summary family:

1. `human_friction`

Recommended row shape:

```rust
pub(crate) struct HumanFrictionSegmentRow {
    pub execution_mode: String,
    pub segment: String,
    pub denominator_requests: u64,
    pub not_a_bot_requests: u64,
    pub challenge_requests: u64,
    pub js_challenge_requests: u64,
    pub maze_requests: u64,
    pub friction_requests: u64,
    pub not_a_bot_rate: f64,
    pub challenge_rate: f64,
    pub js_challenge_rate: f64,
    pub maze_rate: f64,
    pub friction_rate: f64,
}
```

Where:

1. `segment` is one of:
   - `likely_human`
   - `unknown_interactive`
   - `interactive`
2. `execution_mode` keeps shadow and enforced traffic separate.
3. the summary is restricted to:
   - `traffic_origin = live`
   - `measurement_scope = ingress_primary`

That keeps the summary narrowly operator-relevant and prevents adversary-sim or defence-followup traffic from polluting human-friction benchmarks.

# Friction Response Set

For this tranche, count the following request-outcome response kinds as human-friction issuance:

1. `not_a_bot`
2. `challenge`
3. `js_challenge`
4. `maze`

Do not include these in `human_friction` yet:

1. `block_page`
2. `plain_text_block`
3. `redirect`
4. `drop_connection`
5. `tarpit`

Those responses matter, but they belong more naturally in the next normalized defence-effectiveness funnel tranche because they mix hard denial and cost-shifting semantics rather than pure interactive friction.

# Derivation Rules

## 1. Denominators

Use existing lane totals from request outcomes for:

1. live
2. ingress-primary
3. the current execution mode

Segment totals:

1. `likely_human` = lane total for `likely_human`
2. `unknown_interactive` = lane total for `unknown_interactive`
3. `interactive` = sum of `likely_human` and `unknown_interactive`

## 2. Numerators

Use the new `lane_response_kind` breakdown counts over the same scope:

1. `not_a_bot_requests`
2. `challenge_requests`
3. `js_challenge_requests`
4. `maze_requests`

Then:

1. `friction_requests` = sum of the four friction families

## 3. Rates

If `denominator_requests == 0`, all rates must be `0.0`.

Otherwise:

1. `family_rate = family_requests / denominator_requests`
2. `friction_rate = friction_requests / denominator_requests`

# Design Guardrails

1. Do not derive these rates in the dashboard. The backend summary is the contract.
2. Do not widen the request-outcome summary into a high-cardinality analytics cube.
3. Do not add per-path, per-country, or per-IP denominator variants in this tranche.
4. Do not treat `likely_human` as ground truth; it remains an operator-facing runtime classification.
5. Keep adversary-sim traffic out of this summary entirely.

# Verification Plan

Use the focused Make lane during implementation:

1. `make test-monitoring-telemetry-foundation-unit`

Add tests that prove:

1. `lane_response_kind` counters are recorded only when a lane exists,
2. `human_friction` rows are derived correctly for `likely_human`, `unknown_interactive`, and combined `interactive`,
3. shadow and enforced execution modes stay separate,
4. adversary-sim rows do not pollute the human-friction summary,
5. rates are zero-safe when denominators are zero.

# Outcome

When this lands, Shuma will still not have the full Monitoring overhaul, but it will have the backend truth needed for it:

1. lane-aware human-friction rates,
2. derived from bounded runtime summaries,
3. without widening raw event retention,
4. and without asking the UI to invent operator meaning from raw counters.
