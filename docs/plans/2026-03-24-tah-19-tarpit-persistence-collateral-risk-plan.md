Date: 2026-03-24
Status: Proposed

Related context:

- [`../research/2026-03-24-tah-19-tarpit-persistence-collateral-risk-review.md`](../research/2026-03-24-tah-19-tarpit-persistence-collateral-risk-review.md)
- [`2026-02-22-http-tarpit-cost-shift-implementation-plan.md`](2026-02-22-http-tarpit-cost-shift-implementation-plan.md)
- [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Objective

Close `TAH-19` by reducing collateral risk in tarpit persistence escalation without weakening the rest of tarpit's bounded-cost architecture or introducing dashboard churn.

# Decisions

1. Bucket-level persistence remains valid for operator-facing offender visibility.
2. Punitive persistence escalation must no longer be driven by bucket counts.
3. Exact-principal escalation state must stay bounded and fail open for unseen principals if the bounded tracker is full.
4. Re-evaluate defaults only after the escalation basis is corrected; do not change thresholds or durations speculatively first.

# Implementation Shape

## 1. Split persistence tracking into two roles

Keep:

1. bucket-oriented persistence state for monitoring and compact offender summaries

Add:

1. exact-principal persistence state for escalation only

The exact-principal key should not expose raw IP in cleartext if it does not need to; a stable digest-backed principal key is sufficient because operators do not need this store surfaced directly.

## 2. Bound exact-principal escalation memory

The exact-principal escalation tracker must:

1. use a capped catalog,
2. avoid unbounded state growth,
3. and fail open for new unseen principals when the cap is full rather than reusing coarse bucket escalation.

That means:

1. existing tracked principals continue to update,
2. unseen principals beyond the cap should behave as fresh principals with no escalation promotion.

## 3. Keep escalation semantics but change the evidence basis

`persistence_escalation(cfg, persistence_count)` can remain the same for this tranche.

The changed input should be:

1. exact-principal count for punitive escalation

not:

1. bucket count

## 4. Strengthen proof

Focused tests must prove:

1. repeated requests from one IP do escalate,
2. a fresh IP in the same `/24` or `/64` does not inherit that escalation,
3. bounded principal tracking fails open for new unseen principals when saturated.

## 5. Re-evaluate defaults after hardening

For this tranche, explicitly assess:

1. `>= 5` short-ban threshold,
2. `>= 10` block threshold,
3. `ban_durations.tarpit_persistence`

If they remain acceptable once the evidence basis is exact-principal, record that no default change is warranted now.

# Verification

Use a focused make target for the tarpit collateral-risk contract that exercises:

1. tarpit runtime exact-principal counting behavior,
2. provider-side same-bucket no-cross-contamination behavior,
3. and any bounded fail-open behavior for tracker saturation.

# TODO Mapping

This plan closes:

1. `TAH-19 Before launch, tighten collateral-risk controls (especially bucket-based persistence escalation), then re-evaluate tarpit defaults.`
