Date: 2026-03-24
Status: Defunct on 2026-03-29

Related context:

- [`../research/2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md`](../research/2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md)
- [`2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md`](2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md)
- [`2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)
- [`../../dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`](../../dashboard/src/lib/components/dashboard/FingerprintingTab.svelte)
- [`../../src/runtime/non_human_taxonomy.rs`](../../src/runtime/non_human_taxonomy.rs)

Retirement note (2026-03-29):

1. This plan is retained for audit only.
2. The March 23-24 Tuning expansion chain is now defunct, so this rename and remit change is no longer active roadmap.
3. The dashboard should not treat this document as executable sequencing for the current `Fingerprinting` tab.

# Objective

Make the future `Fingerprinting` tab truthfully reflect its remit by renaming it to `Identification` and expanding the planned read surface so it explains both:

1. the signals used to identify non-human traffic,
2. and how those signals distinguish categories in the canonical non-human taxonomy.

# Core Decisions

1. The future tab name should be `Identification`, not `Fingerprinting`.
2. The tab is an explanatory and diagnostic surface for identification logic, not the main editable tuning surface.
3. The category-distinction story is part of the tab’s first-class remit, not optional later garnish.
4. The tab should distinguish between available, active, and influential evidence where the backend truth supports that distinction.
5. This work should execute inside the later `TUNE-SURFACE-1B` ownership cleanup rather than as a detached rename-only slice.
6. The first version of `Category Distinction` should omit explicit `not useful` entries and show only meaningful signal families per category.
7. When the rename lands, `Identification` should move to sit immediately after `Tuning` in the dashboard tab ordering.

# Required Behavior

## Tab name and framing

1. Rename the tab label and route key from the old fingerprinting concept to `Identification`.
2. Frame the tab as “how Shuma identifies non-human traffic” rather than “browser or transport fingerprinting.”
3. Move the tab so `Identification` appears immediately after `Tuning` in the top-level dashboard ordering.

## Section ownership

The tab should aim to contain:

1. `Signal Sources`
   - provider-source posture
   - passive signal families
   - bounded explanatory copy about what can be observed
2. `Effective Identification Signals`
   - the active runtime scoring or classification signals
   - read-only diagnostic projection
3. `Category Distinction`
   - a matrix showing how signal families distinguish canonical taxonomy categories
4. later, if supported by truthful backend data:
   - `Identification Limits`

## Category-distinction contract

The `Category Distinction` section should:

1. use canonical taxonomy labels and descriptions,
2. show only meaningful signal families for each category rather than rendering a full noise-heavy matrix of absences,
3. group those visible signal families with bounded explanatory roles such as `primary`, `supporting`, or `disambiguating`,
4. leave irrelevant or currently non-helpful combinations unrendered in the first version,
5. stay bounded and explanatory rather than pretending the classification system is more deterministic than it really is.

## First-wave presentation preference

The first implementation should prefer a sparse per-category presentation over a dense full matrix.

That means:

1. each category row should list only the evidence families that materially help distinguish it,
2. the UI should not render repeated `not useful` or empty cells by default,
3. and later broader matrix views should be added only if operators genuinely need cross-category comparison at that density.

## Editing boundary

1. Keep provider-topology controls that belong to source posture in `Identification`.
2. Keep ratified editable botness and fingerprint knobs in `Tuning`.
3. Do not let the rename reopen the earlier ownership split.

# Sequencing

1. `MON-OVERHAUL-1A..1C`
2. `CTRL-SURFACE-1..3`
3. `TUNE-SURFACE-1A`
4. `TUNE-SURFACE-1B`
   - rename `Fingerprinting` to `Identification`
   - move `Identification` to sit immediately after `Tuning`
   - move ratified editable controls into `Tuning`
   - reframe the identification tab around signal sources, effective signals, and taxonomy distinction
5. `TUNE-SURFACE-1C`

# Backlog Refinement

`TUNE-SURFACE-1B` should now explicitly include:

1. the tab rename from `Fingerprinting` to `Identification`,
2. the identification remit clarification,
3. the tab-order move so `Identification` sits directly after `Tuning`,
4. and the first category-distinction read surface over the canonical taxonomy.

# Definition Of Done

This plan is satisfied when:

1. the tab is renamed to `Identification`,
2. its remit clearly explains how Shuma identifies non-human traffic,
3. it shows how the taxonomy categories are made distinct through the available meaningful signals without cluttering the surface with explicit non-useful entries,
4. it appears immediately after `Tuning` in the dashboard tab order,
5. and the rename does not blur the editing boundary that keeps tuning controls in `Tuning`.
