# Monitoring Reuse-First Diagnostics Cleanup Sequencing Review

Date: 2026-03-24

## Question

Should Diagnostics be cleaned up before `MON-OVERHAUL-1B`, or should Monitoring first reuse the current shared charts and view-model surface that still lives inside Diagnostics?

## Current state

The current Diagnostics implementation is intentionally transitional. It already contains material that is not ideal long-term diagnostics ownership, but it also still hosts reusable shared components and view-model helpers that Monitoring is likely to need during `MON-OVERHAUL-1B`.

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

Do not clean Diagnostics up ruthlessly before `MON-OVERHAUL-1B`.

That would create one of two bad outcomes:

1. delete or destabilize reusable chart and view-model code before Monitoring has adopted it, or
2. keep the old machinery around anyway, which means the cleanup tranche would not really simplify ownership.

Instead:

1. let `MON-OVERHAUL-1B` reuse or extract the shared aggregate chart and view-model surface it genuinely needs,
2. then run the focused test-hygiene cleanup against the now-settled rendered contracts,
3. then execute a dedicated Diagnostics cleanup tranche that removes the aggregate Monitoring leftovers more aggressively,
4. and only after that finish the Monitoring category/trust surface.

## Practical sequencing decision

The better sequence is:

1. `MON-OVERHAUL-1B`
2. `TEST-HYGIENE-6`
3. `DIAG-CLEANUP-1`
4. `MON-OVERHAUL-1C`

## Why this is cleaner

1. Monitoring gets first claim on the reusable aggregate surface.
2. Diagnostics can then be reduced to genuinely diagnostics-first material with much less uncertainty.
3. The cleanup can remove redundant UI and now-unneeded helper code with confidence rather than defensive caution.
4. The rendered test cleanup lands against the more truthful final ownership boundary rather than a still-transitional intermediate state.
