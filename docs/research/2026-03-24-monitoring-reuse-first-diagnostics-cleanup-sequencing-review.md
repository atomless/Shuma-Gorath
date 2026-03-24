# Monitoring Reuse-First Diagnostics Cleanup Sequencing Review

Date: 2026-03-24

Update on 2026-03-24:

This review captured the correct reuse-first instinct, but the later ownership decision refined the destination. The current traffic-facing aggregate surfaces should now move into a dedicated `Traffic` tab rather than being pulled into Monitoring by default. See [`2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-review.md`](2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-review.md) and [`2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-plan.md`](../plans/2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-plan.md).

## Question

Should Diagnostics be cleaned up before the next operator-surface tranche, or should Shuma first give the current shared traffic charts and view-model surface a truthful home elsewhere?

## Current state

The current Diagnostics implementation is intentionally transitional. It already contains material that is not ideal long-term diagnostics ownership, but it also still hosts reusable shared components and view-model helpers that the dashboard is likely to need during the next ownership refactor.

Relevant current shared surface:

1. [`dashboard/src/lib/components/dashboard/monitoring/OverviewStats.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/OverviewStats.svelte)
2. [`dashboard/src/lib/components/dashboard/monitoring/PrimaryCharts.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/PrimaryCharts.svelte)
3. [`dashboard/src/lib/components/dashboard/monitoring/DefenseTrendBlocks.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/DefenseTrendBlocks.svelte)
4. [`dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
5. the still-transitional aggregate sections in [`dashboard/src/lib/components/dashboard/DiagnosticsTab.svelte`](../../dashboard/src/lib/components/dashboard/DiagnosticsTab.svelte):
   - `Traffic Overview`
   - `Defense Breakdown`
   - `External Monitoring`

## Conclusion

Do not clean Diagnostics up ruthlessly before the traffic-facing aggregate surfaces have a truthful new home.

That would create one of two bad outcomes:

1. delete or destabilize reusable chart and view-model code before Monitoring has adopted it, or
2. keep the old machinery around anyway, which means the cleanup tranche would not really simplify ownership.

Instead:

1. let `MON-OVERHAUL-1B` finish the loop-accountability projection,
2. then introduce a dedicated `Traffic` tab that claims the shared traffic-facing aggregate chart and view-model surface,
3. then execute a dedicated Diagnostics cleanup tranche that removes the migrated traffic-facing leftovers and narrows Diagnostics to furniture-operational proof,
4. and only after that finish the Monitoring category/trust surface.

## Practical sequencing decision

The better sequence is:

1. `MON-OVERHAUL-1B`
2. `TRAFFIC-TAB-1`
3. `DIAG-CLEANUP-1`
4. `MON-OVERHAUL-1C`

## Why this is cleaner

1. The current traffic-facing aggregate surface gets a truthful dedicated home instead of being forced into Monitoring.
2. Diagnostics can then be reduced to genuinely diagnostics-first material with much less uncertainty.
3. The cleanup can remove redundant UI and now-unneeded helper code with confidence rather than defensive caution.
4. Later Monitoring work lands against the more truthful final three-way ownership boundary rather than a still-transitional intermediate state.
