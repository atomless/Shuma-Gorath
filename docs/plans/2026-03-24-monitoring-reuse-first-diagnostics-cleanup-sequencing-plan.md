# Monitoring Reuse-First Diagnostics Cleanup Sequencing Plan

Date: 2026-03-24

## Goal

Preserve the reuse-first principle while correcting the destination of the shared aggregate traffic surface: it should move into a dedicated `Traffic` tab, not be forced into Monitoring.

Update on 2026-03-24:

This note is now subordinate to [`2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-plan.md`](2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-plan.md). The reuse-first idea still matters, but the current aggregate traffic charts and traffic tables belong in `Traffic` unless a specific piece directly serves loop-accountability.

## Sequencing decision

1. execute `MON-OVERHAUL-1B` first,
2. then execute `TRAFFIC-TAB-1`,
3. then execute `DIAG-CLEANUP-1`,
4. then execute `MON-OVERHAUL-1C`,
5. then later `TUNE-SURFACE-1`.

## `MON-OVERHAUL-1B` addition

`MON-OVERHAUL-1B` should explicitly:

1. keep only the loop-accountability surfaces that genuinely belong in Monitoring,
2. avoid treating generic traffic visibility as if it must be absorbed by Monitoring,
3. leave the traffic-oriented aggregate surfaces available for `TRAFFIC-TAB-1`,
3. avoid deleting transitional aggregate helpers from Diagnostics until Monitoring's ownership is real and rendered.

## `DIAG-CLEANUP-1`

Objective:

Make Diagnostics clearly diagnostics-first after `Traffic` has claimed the traffic visibility surface and Monitoring has kept only the loop-accountability surfaces it genuinely needs.

Scope:

1. remove or demote the migrated traffic-facing sections from Diagnostics once `Traffic` owns them,
2. keep `Defense-Specific Diagnostics`, `Telemetry Diagnostics`, and `External Monitoring` as the core Diagnostics surface,
3. collapse or relocate helper/export material that is not truly diagnostics-first,
4. delete any now-redundant UI and local helper/view-model code that survived only to support the transitional pre-`MON-OVERHAUL-1B` shape,
5. keep the cleanup ownership-focused rather than turning it into a second redesign.

Verification:

1. focused dashboard rendered proof for the retained Diagnostics sections,
2. focused proof that removed traffic-facing sections now appear only where `Traffic` owns them,
3. `git diff --check`.

## Notes

1. This remains a reuse-first sequencing refinement, not a retreat from the diagnostics-first end state.
2. The point is to avoid deleting shared aggregate traffic surfaces before the right destination tab exists.
3. After `TRAFFIC-TAB-1` and `DIAG-CLEANUP-1`, Diagnostics should be much easier to keep narrow and truthful.
