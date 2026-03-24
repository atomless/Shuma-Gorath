Date: 2026-03-23
Status: Proposed

Related context:

- [`../research/2026-03-23-tuning-tab-taxonomy-posture-matrix-and-policy-archetypes-review.md`](../research/2026-03-23-tuning-tab-taxonomy-posture-matrix-and-policy-archetypes-review.md)
- [`2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md`](2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md)
- [`2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)
- [`2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md`](2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../src/runtime/non_human_taxonomy.rs`](../../src/runtime/non_human_taxonomy.rs)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../dashboard/src/lib/components/dashboard/TuningTab.svelte`](../../dashboard/src/lib/components/dashboard/TuningTab.svelte)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/domain/config-schema.js`](../../dashboard/src/lib/domain/config-schema.js)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Define the first implementation-ready UI contract for `TUNE-SURFACE-1` so the Tuning tab becomes the operator-owned editor for non-human category posture over the stable taxonomy, without creating a second policy model or cross-tab side-effect surface.

# Core Decisions

1. `Tuning` owns category posture because it is active defense posture and controller intent, not passive crawl-policy declaration.
2. The primary editor is a taxonomy posture matrix, not a stack of unrelated controls.
3. The persisted source of truth remains `operator_objectives_v1.category_postures`.
4. Presets are optional seed actions only; they do not become persisted first-class policy objects.
5. The first version must not silently modify `Policy`-tab or `Verification`-tab settings.
6. The preset set is an operator-facing product contract and must remain distinct from the later recursive-improvement development reference stance methodology, even if one label overlaps.
7. Later controller-patch-family explanation in `Tuning` must derive from the canonical controller mutability policy rather than from raw admin-config writability.

# Scope

## In scope

1. Fetching and saving operator objectives through the existing admin objective endpoints.
2. Rendering the canonical taxonomy as the main tuning editor.
3. Rendering and editing one posture selection per category.
4. Seeding the matrix from a small set of local starter archetypes.
5. Explicit dirty/custom state when the matrix diverges from a named preset.

## Out of scope

1. Broader threshold and weight editing redesign.
2. Cross-tab policy bundles that also mutate verified-identity or `robots.txt` state.
3. Full operator-objective budget editing.
4. Patch-family, rollout, or apply-loop history controls.

# Required UI Contract

## Section ownership

The Tuning tab should gain a first-class section titled:

1. `Non-Human Traffic Posture`

This section should visually dominate the early Tuning contract and should not be buried under threshold micro-controls.

## Preset control

Render a small preset selector above the matrix.

Initial preset set:

1. `Balanced default`
2. `Human-only / private`
3. `Search-visible, AI-restricted`
4. `Agent-friendly, scraper-hostile`
5. `Open access / low friction`

Required semantics:

1. Choosing a preset seeds local unsaved matrix values only.
2. Preset selection must be reversible before save.
3. Manual row edits after preset application must change the visible preset state to `Custom`.
4. Persisted save payload remains raw `category_postures` rows; do not persist a separate preset id.
5. Do not couple preset choice to later recursive-improvement reference-stance logic; this surface edits the live operator stance only.

## Matrix layout

Render:

1. one row per canonical category from the taxonomy contract,
2. one column per posture scale value,
3. one selected value per row.

Each row must show:

1. category label,
2. short description,
3. one bounded single-choice selector across the five posture values.

Use a matrix layout rather than independent fieldsets so operators can compare row posture quickly.

## Semantic help

Render a short legend explaining the posture scale:

1. `Allowed`
2. `Tolerated`
3. `Cost reduced`
4. `Restricted`
5. `Blocked`

This legend must explain intent in operator language rather than backend wording only.

# Data Flow

1. Load canonical taxonomy from the existing backend-readable source already surfaced through snapshot or objective-related payloads.
2. Load current operator objectives from `GET /admin/operator-objectives`.
3. Normalize the saved `category_postures` rows into a matrix-friendly local model.
4. Apply preset seeds only in local component state.
5. Save back through `POST /admin/operator-objectives` using the canonical `category_postures` row structure.
6. Preserve revision-aware objective behavior and existing validation semantics.

# Files

Modify:

1. `dashboard/src/lib/components/dashboard/TuningTab.svelte`
2. `dashboard/src/lib/domain/api-client.js`
3. `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
4. `dashboard/src/lib/state/dashboard-store.js`
5. `Makefile`

Likely add:

1. a shared category-posture matrix component in `dashboard/src/lib/components/dashboard/tuning/` or a similarly canonical dashboard subdirectory if the existing component surface does not already cover it.

Update docs:

1. `docs/dashboard-tabs/tuning.md`
2. `docs/dashboard.md`
3. `docs/testing.md`

# Verification

Add a focused make target such as:

1. `make test-dashboard-tuning-category-posture-matrix`

It should prove:

1. the matrix renders every canonical category,
2. only one posture is selectable per row,
3. preset selection seeds the rows,
4. manual edits switch the UI to `Custom`,
5. save payloads round-trip through `operator_objectives_v1`,
6. and the Tuning tab does not silently mutate unrelated `Policy` or `Verification` controls.

# Backlog Refinement

`TUNE-SURFACE-1` should now be understood as starting with three sub-slices:

1. `TUNE-SURFACE-1A`
   - taxonomy posture matrix, preset seeding, and a visibly primary section layout
2. `TUNE-SURFACE-1B`
   - controller-tunable botness and fingerprint control consolidation into `Tuning`, plus the matching `Fingerprinting` ownership cleanup, after the canonical controller mutability policy is ratified
3. `TUNE-SURFACE-1C`
   - later objective-budget and controller-explanation expansion, after `1B` settles the operator-owned editing surface

The first UI slice should land `1A` only.

# Definition Of Done

This plan is satisfied when:

1. the Tuning tab exposes the full canonical taxonomy as an operator-editable posture matrix,
2. presets exist only as optional matrix seeds,
3. the save path persists only canonical `category_postures`,
4. the section is clearly framed as active defense posture rather than `robots.txt`-style site policy,
5. and the implementation reuses the existing dashboard design language and objective contracts without creating a second posture model.
