# TAH-11 Tarpit Observability Post-Implementation Review

Date: 2026-03-24

## Scope

Close out `TAH-11` by expanding tarpit observability beyond the earlier coarse activation and fallback counters.

## Delivered

1. Prometheus now exposes explicit tarpit proof outcomes, chain-violation reasons, and budget-exhaustion reasons in addition to the earlier activation, progression, fallback, escalation, duration, and bytes families.
2. Runtime and provider handling now records:
   - proof required, passed, and failed outcomes,
   - chain violations for step order, missing parent chain, and replay,
   - detailed budget exhaustion reasons for entry concurrency caps and progression egress caps.
3. `/admin/monitoring` `details.tarpit` now projects:
   - progression admissions and denials,
   - proof outcomes,
   - chain-violation totals and reasons,
   - detailed budget exhaustion reasons,
   - fallback actions,
   - offender buckets sourced from capped persistence catalogs.
4. Tarpit offender buckets now use an explicit cardinality guardrail via a capped catalog rather than an unbounded scan path.
5. Focused Makefile verification now exists as `make test-tarpit-observability-contract`.

## Verification

- `make test-tarpit-observability-contract`
- `git diff --check`

## Follow-on

`TAH-12` remains for the operator-facing follow-through:

1. dashboard visibility for the expanded tarpit metrics,
2. and explicit safe-tuning/operator-guidance material.
