# TEST-TIER-1 Post-Implementation Review

Date: 2026-03-23

## What Landed

- `Makefile` test-target descriptions now distinguish the canonical local/CI pre-merge suite from live operational proof targets.
- stale target wording was corrected, including the integration scenario count.
- [`docs/testing.md`](../testing.md) now defines the canonical automated test tiers explicitly and states that manual dashboard checks are not a proof tier.

## Verification

- `make help`
- `git diff --check`

## Outcome

`TEST-TIER-1` achieved its narrow goal: contributors now have a clearer map of what each major test tier is for, which targets are live operational proofs, and which targets belong to the routine pre-merge suite.

## Remaining Follow-On

- `TEST-HYGIENE-6` remains the next test-architecture cleanup tranche.
- The dashboard-heavy archaeology replacement work should follow the first Diagnostics-focused Monitoring slice so the rendered proof lands against the settled tab contracts rather than a moving target.
