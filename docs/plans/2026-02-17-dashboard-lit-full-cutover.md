# Dashboard Lit Full Cutover Plan (All Tabs)

Date: 2026-02-17  
Scope: `DSH-LIT-*` backlog in `todos/todo.md`  
Status: Planned (execution-ready)

## 1. Why this plan now

The framework-adoption gate criteria were defined in:
- `docs/plans/2026-02-15-dashboard-architecture-modernization.md`

The gate condition that objectively triggers migration is currently true:
- `dashboard/dashboard.js` remains >1200 lines with concentrated orchestration responsibilities.

Latest review findings also point to persistent structural coupling:
- large procedural save-handler blob in config controls,
- god-module config UI state,
- implicit god coordinator in dashboard entry,
- broad cross-section dirty-check fan-out and initialization-order guards,
- mixed state/render modules and string-concatenated HTML rendering.

Given pre-launch posture (no backward compatibility requirement), this plan uses a hard cutover and full-tab migration, not a monitoring-only pilot.

## 2. Assumptions and constraints

1. Pre-launch: no backward compatibility layer is required for old dashboard internals.
2. Runtime remains no-build-step from operator perspective (static-served ESM).  
3. Lit is adopted for component rendering/lifecycle; core logic remains functional and data-driven.
4. All tabs move to Lit architecture (Monitoring, IP Bans, Status, Config, Tuning), not a partial hybrid endpoint.
5. Makefile verification remains canonical: `make test` (with Spin running) and `make build`.

## 3. Target architecture

## 3.1 App shell and routing

- Root component: `<shuma-dashboard-app>`.
- Responsibilities:
  - auth/session bootstrap and guard,
  - hash-route tab selection,
  - feature mount/unmount lifecycle,
  - shared refresh scheduling policy.

No module-scope event listener binding is allowed; lifecycle bindings are attached/detached in component lifecycle methods.

## 3.2 State and side effects

- Central store:
  - immutable state object,
  - reducer-based transitions,
  - action creators,
  - memoized selectors.
- Effect layer:
  - network requests via dedicated API client,
  - timer scheduling,
  - clipboard operations,
  - history/hash writes.

Feature components dispatch actions; they do not perform ad-hoc global mutations.

## 3.3 Config architecture

Replace procedural config code with two declarative systems:

1. Save registry:
  - `buttonId`,
  - validator list,
  - patch builder,
  - draft key,
  - success reducer,
  - optional post-save hooks.

2. Config-to-form binding spec:
  - config path,
  - target control id/property,
  - coercion function,
  - draft projection.

This removes copy-paste handler flow and hand-coded section mappers.

## 3.4 Rendering system

- Lit templates replace string `innerHTML` rendering paths.
- Shared primitives for tables/cards/state messages:
  - stat cards,
  - offender cards,
  - table shell,
  - loading/empty/error blocks,
  - standardized form-row controls.
- `unsafeHTML` is disallowed unless explicitly justified and reviewed.

## 3.5 Domain boundaries by tab

- `features/monitoring/*`
- `features/ip-bans/*`
- `features/status/*`
- `features/config/*`
- `features/tuning/*`

Each feature has:
- selectors/view-model derivation,
- Lit component tree,
- action dispatch integration,
- focused tests.

## 4. Review finding to implementation mapping

1. `config-controls.js` procedural blob  
Mapped to:
- `DSH-LIT-CFG1`, `DSH-LIT-CFG2`, `DSH-LIT-CFG4`.

2. `config-ui-state.js` god module  
Mapped to:
- `DSH-LIT-CFG3`, `DSH-LIT-CFG4`, `DSH-LIT-CFG6`.

3. `dashboard.js` implicit god coordinator  
Mapped to:
- `DSH-LIT-APP1`, `DSH-LIT-APP3`, `DSH-LIT-STATE1`, `DSH-LIT-STATE4`, `DSH-LIT-CUT2`.

4. `refreshCoreActionButtonsState()` fan-out and init uncertainty  
Mapped to:
- `DSH-LIT-STATE3`, `DSH-LIT-CFG5`, `DSH-LIT-APP2`.

5. status module mixed state/render  
Mapped to:
- `DSH-LIT-STS1`, `DSH-LIT-STS2`, `DSH-LIT-STS3`.

6. HTML string construction and `innerHTML` churn  
Mapped to:
- `DSH-LIT-UI1`, `DSH-LIT-UI2`, `DSH-LIT-UI3`, `DSH-LIT-CUT3`.

7. module-scope event listeners  
Mapped to:
- `DSH-LIT-APP2`, `DSH-LIT-CUT3`.

## 5. Sequencing (execution order)

1. Decision and runtime foundation:
  - `DSH-LIT-R1`, `DSH-LIT-R2`, `DSH-LIT-DEP1`, `DSH-LIT-APP1`.
2. State/effects architecture:
  - `DSH-LIT-STATE1`..`DSH-LIT-STATE4`.
3. Config architecture replacement:
  - `DSH-LIT-CFG1`..`DSH-LIT-CFG6`.
4. Status and shared rendering primitives:
  - `DSH-LIT-STS1`..`DSH-LIT-STS3`, `DSH-LIT-UI1`..`DSH-LIT-UI3`.
5. Full tab migration:
  - `DSH-LIT-TAB1`..`DSH-LIT-TAB5`.
6. Hard cutover and cleanup:
  - `DSH-LIT-CUT1`..`DSH-LIT-CUT3`.
7. Verification and docs:
  - `DSH-LIT-TEST1`..`DSH-LIT-TEST4`, `DSH-LIT-DOC1`, `DSH-LIT-DOC2`.

## 6. Definition of done

1. No legacy imperative dashboard coordinator remains as active path.
2. All five tabs run through Lit component architecture.
3. Config save + binding logic is declarative (registry/spec based), not duplicated handlers.
4. No module-scope event wiring and no data-bearing `innerHTML` rendering path remains.
5. `make test` and `make build` are green after cutover.
6. Public/contributor docs reflect Lit architecture as canonical dashboard implementation.

