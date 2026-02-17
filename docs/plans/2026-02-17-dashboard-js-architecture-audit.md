# Dashboard JS Architecture Audit and Refactor Plan

Date: 2026-02-17
Scope: `dashboard/dashboard.js`, `dashboard/modules/*`, dashboard monitoring UI glue
Intent: improve modularity, shared utils, functional style boundaries, and maintainability while preserving behavior.

## Current Architectural Findings

1. Single-file orchestration hotspot
- `dashboard/dashboard.js` currently mixes:
  - pure parsing/normalization helpers,
  - DOM querying/render logic,
  - network orchestration,
  - config dirty-state logic,
  - tab refresh scheduling,
  - chart-instance lifecycle.
- This concentration increases edit blast radius and makes isolated testing harder.

2. Mixed concerns inside monitoring/render path
- Monitoring summary rendering, trend chart management, and Prometheus helper rendering are embedded in `dashboard/dashboard.js`.
- The file contains both domain transforms and imperative view writes in adjacent logic blocks.

3. Repetition and contract drift risk
- Prometheus helper strings were repeated across API payload, HTML defaults, and JS fallback text.
- Repetition in different layers makes drift likely and reviews harder.

4. Excessive direct DOM access from feature logic
- Repeated `document.getElementById` calls occur in domain logic paths.
- Lack of local feature-level view adapter boundaries increases incidental coupling.

5. Existing good seams to leverage
- Current modules (`api-client.js`, `dashboard-state.js`, `tab-lifecycle.js`, `config-controls.js`, `status.js`, `charts.js`) already establish a frameworkless module pattern via global exports.
- This supports low-risk incremental extraction without introducing framework/runtime dependencies.

## Target Architecture Principles (Frameworkless-First)

1. Orchestrator remains thin
- `dashboard/dashboard.js` should coordinate tab refresh and cross-module interactions, not own deep rendering details.

2. Feature modules own rendering + local state
- High-cohesion features (for example monitoring panel) should own:
  - data normalization for view purposes,
  - DOM updates,
  - local chart instance state,
  - feature-local helper interactions.

3. API payload as canonical contract for helper content
- UI examples/help content should flow from API payload to avoid copy drift.

4. Functional-style helper boundaries
- Prefer pure helpers (`normalize`, `format`, `buildRows`) separated from side-effectful DOM write functions.

5. Safe incremental extraction
- Extract one cohesive feature at a time.
- Keep IDs and payload shapes stable to avoid behavior regressions.

## Slice Plan

### Slice A (completed)
- Extract monitoring summary + Prometheus helper rendering to `dashboard/modules/monitoring-view.js`.
- Keep chart updates and copy-button behavior inside that module.
- Reduce `dashboard/dashboard.js` responsibilities to orchestration and data fetch.
- Keep existing API payload contract unchanged.

### Slice B (completed)
- Extract shared config parsing/normalization helpers into `dashboard/modules/config-form-utils.js`.
- Extract shared config path inventory into `dashboard/modules/config-schema.js`.
- Keep behavior compatible while reducing duplication across dashboard modules.

### Slice C (completed)
- Extract ban/event/CDP table renderers and related formatting into `dashboard/modules/tables-view.js`.
- Keep quick row actions wired through module callbacks to preserve existing behavior.

### Slice D (completed)
- Introduce feature-local DOM access adapters to reduce repeated global querying and centralize element contract checks.
- Applied shared DOM cache helpers in hot paths for `dashboard/dashboard.js` and `dashboard/modules/config-controls.js`, replacing direct uncached `getElementById` calls.

### Slice E (completed)
- Add shared core utility modules:
  - `dashboard/modules/core/format.js` for escaping/formatting/equality helpers.
  - `dashboard/modules/core/dom.js` for cached selectors, safe setters, and write scheduling.
- Add `dashboard/modules/config-draft-store.js` and route config dirty-state tracking through one store baseline.
- Add no-op chart redraw guards and coalesced DOM refresh writes for monitoring refresh cycles.
- Keep frameworkless architecture while reducing render churn and callback surface complexity.

## Verification Requirements

- `make test-unit`
- `make test-dashboard-e2e`
- Preserve existing behavior and endpoint usage.
