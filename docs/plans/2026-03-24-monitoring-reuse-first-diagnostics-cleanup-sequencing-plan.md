# Monitoring Reuse-First Diagnostics Cleanup Sequencing Plan

Date: 2026-03-24

## Goal

Refine the Monitoring/Diagnostics sequence so Monitoring can first reuse or extract the remaining shared aggregate chart and view-model surface, and only then can Diagnostics be cleaned up more aggressively.

## Sequencing decision

1. execute `MON-OVERHAUL-1B` first,
2. then `TEST-HYGIENE-6`,
3. then `DIAG-CLEANUP-1`,
4. then `MON-OVERHAUL-1C`,
5. then later `TUNE-SURFACE-1`.

## `MON-OVERHAUL-1B` addition

`MON-OVERHAUL-1B` should explicitly:

1. reuse existing shared aggregate chart or view-model components from the current Diagnostics-owned transitional surface where that reuse is truthful,
2. extract shared pieces when Monitoring and Diagnostics both still need them,
3. avoid deleting transitional aggregate helpers from Diagnostics until Monitoring's ownership is real and rendered.

## `DIAG-CLEANUP-1`

Objective:

Make Diagnostics clearly diagnostics-first after Monitoring has claimed any aggregate loop-accountability UI it genuinely needs.

Scope:

1. remove or demote aggregate Monitoring leftovers from Diagnostics once Monitoring owns their accountable projection,
2. keep `Recent External Traffic`, `Defense-Specific Diagnostics`, and `Telemetry Diagnostics` as the core Diagnostics surface,
3. collapse or relocate helper/export material that is not truly diagnostics-first,
4. delete any now-redundant UI and local helper/view-model code that survived only to support the transitional pre-`MON-OVERHAUL-1B` shape,
5. keep the cleanup ownership-focused rather than turning it into a second redesign.

Verification:

1. focused dashboard rendered proof for the retained Diagnostics sections,
2. focused proof that removed aggregate sections now appear only where Monitoring owns them,
3. `git diff --check`.

## Notes

1. This is a reuse-first sequencing refinement, not a retreat from the diagnostics-first end state.
2. The point is to avoid deleting shared aggregate surfaces before Monitoring has had one clean chance to adopt or factor them.
3. After `DIAG-CLEANUP-1`, Diagnostics should be much easier to keep narrow and truthful.
