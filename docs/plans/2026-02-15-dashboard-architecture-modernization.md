# Dashboard Architecture Modernization Baseline and Decision Gate

Date: 2026-02-15  
Scope: `DSH-R1`, `DSH-R2`, `DSH-R3`  
Target: Tabbed SPA modernization with frameworkless-first delivery.

## 1. Baseline Snapshot (Current State)

### 1.1 File/size/line profile

| Asset | Lines | Bytes |
|---|---:|---:|
| `dashboard/dashboard.js` | 1941 | 70345 |
| `dashboard/modules/config-controls.js` | 769 | 28039 |
| `dashboard/modules/status.js` | 215 | 9701 |
| `dashboard/modules/charts.js` | 214 | 6701 |
| `dashboard/modules/admin-session.js` | 177 | 5276 |
| `dashboard/style.css` | 1406 | 26514 |
| `dashboard/index.html` | 580 | 33503 |

Observations:
- The architectural hotspot remains `dashboard/dashboard.js` (large orchestration + DOM/state concerns).
- The module split exists but orchestration/state remains centralized and imperative.

### 1.2 Runtime/network baseline

- Auto-refresh cadence: every 30 seconds (`setInterval(..., 30000)`).
- Per refresh cycle requests:
  - `GET /admin/analytics`
  - `GET /admin/events?hours=24`
  - `GET /admin/ban`
  - `GET /admin/maze`
  - `GET /admin/cdp`
  - `GET /admin/cdp/events?...`
  - `GET /admin/config`
- Additional chart-range requests are issued when time-range controls are changed.

### 1.3 Dependency/provenance baseline

- Charting is currently loaded from CDN in `dashboard/index.html`:
  - `https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js`
  - `https://cdn.jsdelivr.net/npm/chartjs-plugin-colorschemes@0.5.0/dist/chartjs-plugin-colorschemes.min.js`
- This creates external runtime dependency and provenance variability at render time.

### 1.4 Current test coverage baseline

- Dashboard smoke e2e coverage file: `e2e/dashboard.smoke.spec.js` (6 tests prior to tabbed SPA changes).
- Coverage focus before this modernization:
  - initial load/session
  - ban form validation
  - config dirty-state behavior
  - chart range interactions
  - sticky table headers
  - logout redirect

### 1.5 Performance baseline note

- Recent smoke run output shows first dashboard test completion around ~1.5s and full suite in ~6.9s.
- Direct memory profiling for browser heap is not yet automated in this environment.

## 2. Option Evaluation (Frameworkless vs Lit)

### Option A: Frameworkless modular SPA + JSDoc typing

Pros:
- No new runtime dependency or framework lifecycle model.
- Lowest migration risk from current code.
- Smallest immediate bundle/runtime impact.
- Aligns with existing architecture and contributor familiarity.

Cons:
- Requires stronger discipline for state/lifecycle boundaries.
- Continued manual DOM orchestration risk if module boundaries stay weak.

### Option B: Lit-based UI components

Pros:
- Stronger component lifecycle and template ergonomics.
- Better long-term composability for growing UI surface.
- Clean path for future TS-first frontend structure.

Cons:
- New dependency and contributor surface area.
- Migration complexity increases before architecture seams are stabilized.
- Risk of mixed paradigms during incremental port.

## 3. Decision

Proceed with **frameworkless-first** modernization, then reevaluate after structural slices land.

Rationale:
- Current risk is architectural coupling, not framework absence.
- Tab shell + routing + state/API separation can be delivered with minimal runtime weight.
- Lit remains a viable fallback if objective maintainability gates are not met.

## 4. Framework-Adoption Gate (`DSH-R3`)

Adopt Lit only if one or more conditions persist after `DSH-1` through `DSH-14`:

1. `dashboard/dashboard.js` remains a structural hotspot (>1200 lines with cross-tab orchestration responsibilities still concentrated there).
2. Two consecutive dashboard slices require broad cross-module edits (>3 modules and >250 LOC each) for isolated feature changes.
3. UI lifecycle regressions (tab/view state, stale DOM writes, hidden-panel side effects) repeat across two iterations despite module boundary hardening.
4. E2E stability degrades due imperative lifecycle coupling and cannot be recovered with frameworkless refactor patterns alone.

If triggered:
- run `DSH-G1` as constrained Lit pilot on `Monitoring` tab only,
- compare measured outcomes (bundle/runtime, defect rate, change lead time),
- decide on full migration only after pilot evidence.

## 5. Initial Execution Order

1. Tab shell + hash routing (`DSH-1`, `DSH-2`).
2. Controller/state boundary split (`DSH-3`, `DSH-4`, `DSH-5`).
3. Runtime hardening and provenance cleanup (`DSH-6` to `DSH-10`).
4. Verification/docs/rollback (`DSH-11` to `DSH-14`).

