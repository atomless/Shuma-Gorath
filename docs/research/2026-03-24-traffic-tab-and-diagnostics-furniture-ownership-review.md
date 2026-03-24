# Traffic Tab And Diagnostics Furniture Ownership Review

Date: 2026-03-24

## Goal

Correct the dashboard information architecture so the operator-facing tabs answer three distinct questions cleanly:

1. `Monitoring`: is the closed self-improving loop operational and improving outcomes?
2. `Traffic`: what traffic is Shuma seeing and how is it being handled right now?
3. `Diagnostics`: is Shuma's telemetry and defence furniture operational?

## Current mismatch

The current `Monitoring` redesign and the current `Diagnostics` implementation still leave the aggregate traffic surface in the wrong place.

Observed current state:

1. `Monitoring` now projects loop-accountability contracts, but it does not currently own the live traffic visibility surface.
2. `Diagnostics` still owns traffic-facing aggregate surfaces:
   - `Traffic Overview`
   - `Defense Breakdown`
   - `Recent External Traffic`
   - plus the only traffic-health proof the operator can currently see lives inside `Telemetry Diagnostics`
3. Those sections are not really proving that the closed loop is working, and they are not purely proving that Shuma's internal furniture is operational either.
4. As a result, `Diagnostics` is still doing three jobs at once:
   - traffic picture,
   - subsystem diagnostics,
   - and telemetry-pipeline/freshness proof.

This is why the tab still feels muddled even after the earlier Monitoring/Diagnostics split.

## Decision

Shuma should now explicitly use a three-way split rather than forcing the remaining aggregate traffic surface into either `Monitoring` or `Diagnostics`.

### Monitoring

`Monitoring` should stay loop-accountability-first:

1. benchmark-family movement,
2. recent loop progress,
3. controller judgment,
4. change/apply/rollback/refusal history,
5. trust and blocker state,
6. category-level outcome and guardrail truth.

If a chart or table does not help answer whether the loop is operational and improving outcomes, it does not belong in Monitoring by default.

### Traffic

`Traffic` should become the live and recent traffic visibility surface:

1. bounded traffic totals and request mix,
2. time-series and top-dimension traffic charts,
3. recent external traffic events,
4. a light traffic-telemetry health strip proving that traffic collection is alive,
5. manual refresh and bounded auto-refresh because this is the traffic picture rather than deep diagnostics.

This is the natural home for the current Diagnostics-owned traffic picture, but not for all furniture summary material.

### Diagnostics

`Diagnostics` should narrow to furniture-operational proof:

1. telemetry freshness and read-path diagnostics,
2. defence-system diagnostics and subsystem detail,
3. export/helper material that supports proving telemetry or subsystem furniture is operational,
4. contributor-style investigation when something looks wrong.

The key point is that Diagnostics should prove that Shuma's furniture is working, not act as the primary traffic dashboard.

## Component ownership implication

The current reusable traffic-oriented surfaces that should move to `Traffic` are:

1. `OverviewStats`
2. `PrimaryCharts`
3. `RecentEventsTable`

The current reusable furniture-oriented overview that should stay in `Diagnostics` is:

1. `DefenseTrendBlocks`

The current diagnostics-first surfaces that should stay in `Diagnostics` are:

1. `DiagnosticsSection`
2. `DefenseTrendBlocks`
3. `CdpSection`
4. `MazeSection`
5. `TarpitSection`
6. `HoneypotSection`
7. `ChallengeSection`
8. `PowSection`
9. `RateSection`
10. `GeoSection`
11. `IpRangeSection`
12. `ExternalMonitoringSection`

`Telemetry Diagnostics` should remain in Diagnostics as the full contributor/furniture surface, but `Traffic` may later project a lighter top-level traffic-health summary derived from the same freshness truth.

## Sequencing implication

The previous reuse-first idea remains directionally right, but the destination changes:

1. do not clean up Diagnostics yet,
2. first introduce `Traffic` and move the traffic-oriented surfaces there,
3. then clean Diagnostics down to furniture-operational proof,
4. then continue `MON-OVERHAUL-1C` against the cleaner three-way ownership boundary.

## Why this is better

1. It stops Monitoring from becoming a generic chart dashboard.
2. It stops Diagnostics from remaining an overloaded hybrid.
3. It gives traffic telemetry a truthful first-class home.
4. It preserves reuse of current components without forcing them into the wrong tab merely to satisfy an earlier transitional plan.
