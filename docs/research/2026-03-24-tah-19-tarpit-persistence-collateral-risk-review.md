Date: 2026-03-24
Status: Informing implementation

Related context:

- [`../plans/2026-02-22-http-tarpit-cost-shift-implementation-plan.md`](../plans/2026-02-22-http-tarpit-cost-shift-implementation-plan.md)
- [`../plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](../plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)
- [`../tarpit.md`](../tarpit.md)
- [`../../src/providers/internal.rs`](../../src/providers/internal.rs)
- [`../../src/tarpit/runtime.rs`](../../src/tarpit/runtime.rs)
- [`../../todos/todo.md`](../../todos/todo.md)

# Objective

Tighten tarpit persistence escalation so it remains a high-confidence hostile-automation control rather than a bucket-cross-contaminating collateral-risk source.

# Findings

## Current escalation basis is too coarse

Today `maybe_handle_tarpit(...)` increments tarpit persistence through:

1. `src/providers/internal.rs` -> `crate::tarpit::runtime::next_persistence_count(...)`
2. `src/tarpit/runtime.rs` -> persistence key `tarpit:persistence:<site_id>:<ip_bucket>`
3. `src/tarpit/runtime.rs` -> `persistence_escalation(cfg, persistence_count)`

The state key uses `bucket_ip(ip)`, which means:

1. IPv4 is aggregated at `/24`
2. IPv6 is aggregated at `/64`

That makes escalation a function of a coarse neighborhood rather than the exact current offender.

## The collateral-risk shape is real

Because escalation is based on bucket count but the resulting short-ban or block is applied to the exact current IP, a fresh IP can inherit escalation pressure from other actors in the same bucket.

That is especially undesirable for:

1. shared hosting or office NATs,
2. residential ISP prefixes,
3. and any deployment where tarpit is triggered by high-confidence abuse paths but still needs to avoid broad neighborhood compounding.

## The operator-facing bucket view is still useful

The coarse bucket state is still valuable for:

1. compact offender visibility,
2. bounded cardinality in monitoring,
3. and identifying broad abuse neighborhoods.

So the right fix is not "delete bucket tracking entirely."

## The escalation basis should be separated from the telemetry basis

Tarpit persistence needs two different truths:

1. a coarse bucket-oriented truth for operator visibility,
2. and a narrower exact-principal truth for punitive escalation.

That matches the broader recidive research direction in the 2026-03-16 ban-jitter and repeat-offender design: escalation memory should be local, bounded, and careful about false-positive compounding.

## Bounded fail-open is preferable to unbounded exact-principal state

Exact-principal tracking can increase key cardinality if implemented naively.

For pre-launch hardening, the safer bias is:

1. keep exact-principal escalation tracking bounded,
2. and if the bounded tracker is saturated, fail open on new unseen principals rather than silently reintroduce coarse escalation or unbounded storage growth.

## Default thresholds do not necessarily need to change once the basis is fixed

The existing thresholds:

1. short ban at `>= 5`
2. block at `>= 10`

are aggressive when driven by a shared bucket.

They may remain acceptable when driven by a bounded exact-principal counter for already-tarpitted traffic. So the first step should be to fix the confidence basis, then re-evaluate defaults against that corrected behavior rather than lowering thresholds blindly first.
