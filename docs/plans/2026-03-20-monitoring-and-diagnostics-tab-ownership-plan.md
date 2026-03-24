# Monitoring And Diagnostics Tab Ownership Plan

Date: 2026-03-20

## Context

The backend telemetry foundation is now complete enough for the Monitoring overhaul to begin, but the current `Monitoring` tab is still the legacy subsystem-by-subsystem diagnostic surface. That legacy surface is valuable during the transition, but it is the wrong starting point for the operator decision surface Shuma needs next.

The existing research and telemetry plans already establish the target direction:

- `docs/research/2026-03-17-operator-decision-support-telemetry-audit.md`
- `docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`
- `docs/research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`
- `docs/research/2026-03-19-pre-monitoring-overhaul-telemetry-foundation-closeout-review.md`

This note settles the immediate UI ownership question before `MON-OVERHAUL-1` begins.

## Decision

Shuma is now committed to a three-way split:

1. `Monitoring`
   - Human-readable accountability surface for the closed loop.
   - Must not inherit generic traffic-dashboard or subsystem-by-subsystem ownership by default.

2. `Traffic`
   - Live and recent traffic visibility surface.
   - Owns the traffic picture and traffic-handling overview that does not belong in loop accountability or furniture diagnostics.

3. `Diagnostics`
   - Contributor and deep-inspection diagnostics surface.
   - Owns furniture-operational proof and subsystem investigation.

The older two-way `Monitoring`/`Diagnostics` transitional split was a useful bridge, but it is no longer the final target.

## Ownership Contract

### Monitoring

`Monitoring` now owns only the future operator-facing contract:

- attacker-effectiveness versus human-friction visibility,
- loop verdict and benchmark status,
- bounded recent benchmark progress over completed loops,
- change judgment and watch-window result,
- recent controller action history tied to loop outcomes,
- category-aware non-human outcome breakdowns,
- trust and actionability blockers,
- enforced versus shadow storytelling,
- bounded benchmark-grade summaries that later tuning and oversight loops can trust.

Until the remaining overhaul lands, the tab should remain accountability-first and truthful about its status.

### Traffic

`Traffic` owns the traffic visibility surface:

- bounded traffic totals and request mix,
- traffic charts and time series,
- traffic-handling breakdowns,
- recent external traffic browsing,
- manual and bounded auto-refresh for the live traffic picture.

### Diagnostics

`Diagnostics` owns the furniture-operational and subsystem investigation surface:

- CDP, maze, tarpit, honeypot, challenge, PoW, rate, GEO, and IP-range detail sections,
- raw feed and low-level telemetry diagnostics,
- Prometheus helper and contributor-oriented deep inspection.

As the new `Traffic` tab lands, Diagnostics should stop acting as the main traffic dashboard and narrow to proving that Shuma's furniture is operational.

### Status

`Status` remains the operator health and control-plane truth surface:

- dashboard connectivity,
- telemetry delivery health,
- retention health,
- runtime performance telemetry.

It should not absorb the full legacy Monitoring surface.

### IP Bans

`IP Bans` remains the active-ban and ban-operations surface. It should continue to avoid low-level diagnostics in the main body.

## Refresh And Runtime Contract

During the transition:

- `Traffic` should own the live traffic refresh behavior.
- `Diagnostics` should remain contributor-oriented and need only the refresh behavior required for furniture and subsystem investigation.
- auto-refresh remains available only on `IP Bans` and `Red Team`.
- `Monitoring` no longer needs auto-refresh for the placeholder stage.
- the backend monitoring data source remains shared; the split is surface ownership, not a telemetry fork.
- dashboard request/runtime telemetry should describe `Traffic` and `Diagnostics` truthfully once the split is complete.

## Why This Is Better Than Wiping

- It preserves valuable diagnostic depth while the new Monitoring surface is designed carefully.
- It avoids a messy hybrid tab that mixes operator outcomes with contributor mechanics.
- It gives the Monitoring overhaul a genuinely clean slate.
- It keeps the section-ownership boundary explicit for later roadmap work and future test cleanup.

## Implementation Notes

1. Keep `Monitoring` loop-accountability-first.
2. Introduce `Traffic` as the home for the traffic-oriented transitional sections.
3. Narrow `Diagnostics` to furniture-operational proof.
4. Route `Traffic` and `Diagnostics` through the existing bounded monitoring snapshot/delta flow with truthful ownership.
5. Update docs and tests so traffic expectations point at `Traffic`, not `Diagnostics`, and subsystem/furniture expectations point at `Diagnostics`.

## Out Of Scope

- The substantive Monitoring operator redesign itself.
- New telemetry summaries or dashboard semantics beyond the ownership split.
- Reworking the legacy diagnostics surface beyond light copy cleanup needed for truthfulness.
