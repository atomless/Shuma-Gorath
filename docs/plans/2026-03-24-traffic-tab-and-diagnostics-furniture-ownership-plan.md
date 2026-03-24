# Traffic Tab And Diagnostics Furniture Ownership Plan

Date: 2026-03-24

## Goal

Introduce a dedicated `Traffic` tab and refactor the current Diagnostics-owned traffic-facing surface into it, so the three dashboard tabs have clear ownership:

1. `Monitoring` proves the self-improving loop is operational.
2. `Traffic` proves traffic telemetry collection is operational and shows what traffic is hitting Shuma and the host.
3. `Diagnostics` proves Shuma's telemetry and defence furniture is operational.

## Principles

1. Do not force charts into Monitoring unless they directly serve loop-accountability.
2. Reuse existing Diagnostics traffic components where truthful; move them rather than rebuilding them.
3. Keep Diagnostics focused on furniture-operational proof and subsystem investigation.
4. Make `Traffic` the primary home for live and recent traffic visibility.
5. Keep the split product-facing and truthful: loop accountability is not the same question as traffic visibility or diagnostics.

## Target ownership

### Monitoring

Owns:

1. current loop status,
2. recent benchmark-family progress,
3. controller action history,
4. outcome frontier in loop-accountability terms,
5. category breakdown and trust/actionability follow-ons.

Does not own by default:

1. generic traffic totals,
2. top IP charts,
3. recent traffic event browsing,
4. generic defense trend breakdowns unless directly tied to loop judgment.

### Traffic

Owns:

1. bounded traffic overview cards,
2. primary traffic charts,
3. defense breakdown as traffic-handling evidence,
4. recent external traffic browsing and filters,
5. manual refresh plus bounded auto-refresh.

Expected first-wave migrated surfaces:

1. `Traffic Overview`
2. `Defense Breakdown`
3. `Recent External Traffic`

### Diagnostics

Owns:

1. telemetry freshness and read-path diagnostics,
2. defence-specific diagnostic sections,
3. raw feed and contributor-style investigation surfaces,
4. export/helper material proving telemetry furniture is operational.

Expected retained surfaces:

1. `Defense-Specific Diagnostics`
2. `Telemetry Diagnostics`
3. `External Monitoring`

## Execution slices

### `TRAFFIC-TAB-1`

Objective:

Introduce the new `Traffic` tab and move the current traffic-facing Diagnostics sections there.

Scope:

1. add a first-class `Traffic` tab to the canonical dashboard tab order,
2. place it after `Monitoring` and before `Diagnostics`,
3. move the current traffic-oriented sections from Diagnostics into `Traffic`,
4. reuse the existing traffic-oriented components and supporting view-model code where possible,
5. give `Traffic` manual refresh and bounded auto-refresh behavior appropriate for a live traffic picture,
6. keep the tab clearly about traffic visibility rather than subsystem internals.

Verification:

1. focused dashboard tab IA proof for the new tab ordering and section ownership,
2. focused rendered proof that the traffic sections now live in `Traffic`,
3. focused refresh-behavior proof for manual and bounded auto-refresh,
4. `git diff --check`.

### `DIAG-CLEANUP-1`

Objective:

Narrow Diagnostics to furniture-operational proof after the traffic split lands.

Scope:

1. remove the migrated traffic-facing sections from Diagnostics,
2. tighten copy and section names so Diagnostics reads as furniture-operational and subsystem-investigation-first,
3. retain only the sections that prove telemetry and defence furniture is operational,
4. clean up helper/view-model ownership that only existed because Diagnostics temporarily hosted the traffic dashboard.

Verification:

1. focused rendered proof for retained Diagnostics sections,
2. focused proof that migrated traffic sections no longer appear in Diagnostics,
3. `git diff --check`.

### `MON-OVERHAUL-1C`

Objective:

Continue the Monitoring overhaul only after the Traffic/Diagnostics ownership boundary is settled.

Sequencing refinement:

1. `MON-OVERHAUL-1C` should follow `TRAFFIC-TAB-1` and `DIAG-CLEANUP-1`,
2. because category breakdown and trust/actionability should land against the cleaned three-way ownership model, not the older two-way transitional split.

## Sequence

1. `TRAFFIC-TAB-1`
2. `DIAG-CLEANUP-1`
3. `MON-OVERHAUL-1C`
4. `CTRL-SURFACE-1`
5. `CTRL-SURFACE-2`
6. `CTRL-SURFACE-3`
7. `TUNE-SURFACE-1A`

## Notes

1. This plan supersedes the idea that the current Diagnostics traffic charts must automatically move into Monitoring.
2. The reuse-first principle still applies, but the rightful reuse target is now `Traffic`.
3. If later Monitoring needs a specific chart, it should adopt only the chart that directly serves loop accountability rather than inheriting a generic traffic dashboard.
