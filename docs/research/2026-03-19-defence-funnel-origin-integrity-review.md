Date: 2026-03-19
Status: Post-implementation architecture review

Related context:

- [`2026-03-19-controller-readiness-telemetry-foundation-review.md`](./2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`../plans/2026-03-19-monitoring-defence-funnel-plan.md`](../plans/2026-03-19-monitoring-defence-funnel-plan.md)
- [`../plans/2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md`](../plans/2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../src/runtime/sim_telemetry.rs`](../../src/runtime/sim_telemetry.rs)

# Purpose

Record the first post-implementation shortfall discovered while landing `MON-TEL-1-4`.

The defence funnel contract was directionally correct, but the first implementation risked blending live traffic and adversary-sim evidence through older family-specific counters that predate the new origin-aware request-outcome foundation.

# Finding

Shuma's normalized request-outcome summaries now separate `live` and `adversary_sim` traffic correctly.

However, several older family-specific telemetry paths still aggregate without an explicit traffic-origin dimension:

1. `not_a_bot.served`
2. `not_a_bot.submitted`
3. `not_a_bot.outcome`
4. `challenge.total`
5. `pow.total`
6. `pow.success`
7. `pow.outcome`

That creates a truthfulness risk if the first defence-funnel rows consume those counters directly while also claiming to represent the external-traffic operator view.

# Implication

Until those legacy family counters become origin-aware, the funnel must not populate stage values that would quietly blend:

1. live external traffic,
2. and adversary-sim traffic.

So the correct first-wave rule is:

1. populate funnel stages from origin-safe request-outcome and human-friction summaries where available,
2. leave stages as `null` where the only available source is still origin-blended,
3. and defer the richer family-stage population to a follow-on telemetry slice.

# Corrected First-Wave Contract

Safe today:

1. `candidate_requests`
2. `triggered_requests`
3. `friction_requests`
4. `likely_human_affected_requests`

Only for families backed by origin-safe request-outcome rows:

1. `not_a_bot`
2. `challenge`
3. `js_challenge`
4. `maze`

Not safe yet:

1. `not_a_bot.passed_requests`
2. `not_a_bot.failed_requests`
3. `not_a_bot.escalated_requests`
4. `challenge.failed_requests`
5. `pow.*` funnel stages

# Required Follow-On Work

Add one immediate controller-grade follow-on slice before the Monitoring overhaul:

1. make `not_a_bot`, `challenge`, and `pow` follow-up telemetry origin-aware,
2. migrate operator summaries and defence-funnel stages to consume the live-only view by default,
3. keep adversary-sim evidence available separately for Red Team and tuning evidence rather than hiding it.

# Outcome

This review does not change the roadmap direction.

It tightens the bearing:

1. `MON-TEL-1-4` must land as an honest first-wave funnel, not an overconfident one,
2. and the next backend telemetry slice must close the origin-integrity gap for legacy family counters before the Monitoring UI overhaul begins.
