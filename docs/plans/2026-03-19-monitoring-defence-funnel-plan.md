Date: 2026-03-19
Status: Active implementation plan

Related context:

- [`../research/2026-03-17-operator-decision-support-telemetry-audit.md`](../research/2026-03-17-operator-decision-support-telemetry-audit.md)
- [`../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`../research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../research/2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](./2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md)
- [`2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md`](./2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md)
- [`2026-03-19-monitoring-human-friction-denominator-plan.md`](./2026-03-19-monitoring-human-friction-denominator-plan.md)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../todos/todo.md`](../../todos/todo.md)

# Purpose

Define the first normalized defence-effectiveness funnel contract for `MON-TEL-1-4`.

This plan is intentionally conservative. It does not try to synthesize perfect funnels for every defence family. It defines one shared row shape, limits the first implementation to the families with enough backend truth today, and uses explicit `null` stage values where Shuma does not yet have trustworthy coverage.

# Why This Is The Next Slice

Shuma now has:

1. request-outcome lane and scope summaries,
2. bounded `response_kind`, `policy_source`, and `route_action_family` breakdowns,
3. human-friction denominators,
4. and existing subsystem summaries for `not_a_bot`, `pow`, and challenge failures.

What is still missing is a comparable operator summary that says, across defence families:

1. what got triggered,
2. what passed,
3. what failed,
4. what escalated,
5. what likely touched human-adjacent traffic.

Without that shared shape, Monitoring will still be forced back into per-widget interpretation even though the telemetry foundation is already rich enough to support something cleaner.

# Design Decision

## 1. Add one normalized `defence_funnel` summary family

Add one top-level summary family:

1. `defence_funnel`

This summary should be:

1. backend-derived,
2. bounded,
3. reused across supported defence families,
4. and honest about unavailable stages.

## 2. Use one shared row shape with optional stage values

Recommended row shape:

```rust
pub(crate) struct DefenceFunnelRow {
    pub execution_mode: String,
    pub family: String,
    pub candidate_requests: Option<u64>,
    pub triggered_requests: Option<u64>,
    pub friction_requests: Option<u64>,
    pub passed_requests: Option<u64>,
    pub failed_requests: Option<u64>,
    pub escalated_requests: Option<u64>,
    pub denied_requests: Option<u64>,
    pub suspicious_forwarded_requests: Option<u64>,
    pub likely_human_affected_requests: Option<u64>,
}
```

Use `Option<u64>`, not `0`, for stages Shuma cannot currently support truthfully.

That keeps the summary safe for later operator and controller use because it distinguishes:

1. zero observed events,
2. from unavailable measurement coverage.

# First-Wave Supported Families

The first implementation should include only the families with enough backend truth today:

1. `not_a_bot`
2. `pow`
3. `challenge`
4. `js_challenge`
5. `maze`

Do not include first-wave rows for:

1. `honeypot`
2. `rate_limit`
3. `geo`
4. `tarpit`
5. `ban`

Those can follow later once the measurement boundary is cleaner.

# Stage Mapping Rules

## 1. `not_a_bot`

Populate:

1. `candidate_requests` = `not_a_bot.served`
2. `triggered_requests` = `not_a_bot.served`
3. `friction_requests` = `not_a_bot.served`
4. `passed_requests` = `not_a_bot.pass`
5. `failed_requests` = `not_a_bot.fail`
6. `escalated_requests` = `not_a_bot.escalate`
7. `likely_human_affected_requests` = `human_friction.likely_human.not_a_bot_requests`

Leave `denied_requests` and `suspicious_forwarded_requests` as `None` in the first wave.

## 2. `pow`

Populate:

1. `candidate_requests` = `pow.total_attempts`
2. `triggered_requests` = `pow.total_attempts`
3. `friction_requests` = `pow.total_attempts`
4. `passed_requests` = `pow.total_successes`
5. `failed_requests` = `pow.total_failures`

Leave the other stages as `None` for now.

## 3. `challenge`

Populate:

1. `candidate_requests` = request-outcome `response_kind = challenge` for live ingress-primary requests in the current execution mode
2. `triggered_requests` = same as candidate for first wave
3. `friction_requests` = same as candidate for first wave
4. `failed_requests` = `challenge.total_failures`
5. `likely_human_affected_requests` = `human_friction.likely_human.challenge_requests`

Leave `passed_requests`, `escalated_requests`, `denied_requests`, and `suspicious_forwarded_requests` as `None`.

## 4. `js_challenge`

Populate:

1. `candidate_requests` = request-outcome `response_kind = js_challenge`
2. `triggered_requests` = same
3. `friction_requests` = same
4. `likely_human_affected_requests` = `human_friction.likely_human.js_challenge_requests`

Leave the other stages as `None`.

## 5. `maze`

Populate:

1. `candidate_requests` = request-outcome `response_kind = maze`
2. `triggered_requests` = same
3. `friction_requests` = same
4. `likely_human_affected_requests` = `human_friction.likely_human.maze_requests`

Leave the other stages as `None` in the first wave.

# Design Guardrails

1. Do not infer pass, deny, or suspicious-forwarded stages unless the backend already has a truthful source.
2. Do not backfill missing stages with `0`; use `None`.
3. Keep the summary bounded and family-level only.
4. Do not add per-path or per-country funnel variants in this tranche.
5. Keep adversary-sim traffic out of the first-wave funnel summary.

# Ownership And Read-Surface Placement

The funnel summary should be treated as a `supporting_summary`, not a required first-paint bootstrap headline.

That means:

1. the summary belongs in the backend monitoring contract,
2. but it should not force bootstrap growth beyond the current bounded summary posture,
3. and it can remain unused by the dashboard until `MON-OVERHAUL-1` consumes it deliberately.

# Verification Plan

Use:

1. `make test-monitoring-telemetry-foundation-unit`

Add tests that prove:

1. each supported family emits one normalized row per execution mode,
2. `None` is used where a stage is intentionally unavailable,
3. `likely_human_affected_requests` comes from the human-friction summary rather than ad hoc widget math,
4. adversary-sim traffic does not populate the funnel rows,
5. the summary survives hot-read refresh into the monitoring summary document and bootstrap payload if it remains bootstrap-resident.

# Outcome

When this lands, Shuma will have one reusable, bounded defence-effectiveness summary contract that Monitoring can later consume without slipping back into one-off subsystem storytelling.
