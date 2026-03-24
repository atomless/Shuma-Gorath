# 2026-03-24 Dashboard DOM De-Shelling Post-Implementation Review

## Scope

This tranche corrected needless wrapper shells and duplicate section chrome across the `Traffic`, `Game Loop`, and `Diagnostics` tabs after user review identified sloppy DOM construction, repeated divider lines, and structurally empty top-level containers.

The purpose of this slice was narrow:

- remove needless section-inside-section ownership shells,
- preserve meaningful content and existing shared component patterns,
- avoid inventing new visual language while cleaning up the rendered DOM,
- and add proof that the rendered tabs no longer accumulate nested section chrome.

This tranche did **not** attempt to complete the broader `DIAG-CLEANUP-1` ownership/content cleanup. That larger content and IA cleanup still remains separate.

## Delivered

### Shared primitive cleanup

- `SectionBlock` now forwards rest props so tabs can assign ownership markers directly to the actual section-owning node.
- `DisclosureSection` now forwards rest props for the same reason.
- `TabStateMessage` now conditionally renders pane notices instead of always emitting a hidden empty notice shell.

### Traffic

- Removed the redundant outer `section.section` wrapper around `RecentEventsTable`.
- Removed the nested `SectionBlock` shell from `PrimaryCharts` and replaced it with plain shared copy markup (`section-copy-block`) so the charts no longer render section chrome inside a parent section.

### Diagnostics

- Removed redundant outer section wrappers around:
  - `Defense Breakdown`
  - `Telemetry Diagnostics`
  - `External Monitoring`
- Preserved `Defense Breakdown` content itself rather than collapsing the panel.
- Kept the broader diagnostics furniture content intact while eliminating stacked chrome.

### Game Loop

- Kept the existing section ownership model intact.
- Reused shared section-copy styling for `Game Loop` inner copy so the tab no longer carries a one-off wrapper class just to present section copy.

### Proof hardening

- Added source-contract assertions proving the outer redundant section shells are gone.
- Added rendered Playwright assertions proving:
  - `Traffic` has zero `.section .section` descendants,
  - `Diagnostics` has zero `.section .section` descendants,
  - `Game Loop` has zero `.section .section` descendants.

## Why this fix was necessary

The earlier dashboard slices had drifted into local subtree patching without enough whole-tab DOM review. That created exactly the failure mode the user called out:

- meaningful content surviving inside the wrong shells,
- repeated divider lines from stacked `.section` wrappers,
- and empty or border-only containers that added chrome without owning content.

The corrected approach in this tranche was:

1. inspect the full rendered DOM across the affected tabs,
2. identify which node truly owns each section,
3. forward markers/attrs to that owner,
4. then remove only the redundant outer shell.

## Verification

Focused verification passed via canonical `make` paths:

- `make test-dashboard-tab-information-architecture`
- `make test-dashboard-traffic-pane`
- `make test-dashboard-game-loop-accountability`
- `git diff --check`

## Remaining follow-on

This tranche fixes the structural DOM-shell problem, but it does not replace the planned content/ownership cleanup still queued under the broader diagnostics follow-on work. That later cleanup should now happen on top of a cleaner, shallower DOM instead of a stacked-shell structure.
