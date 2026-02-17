# Dashboard ESM Behavior Contract Freeze

Date: 2026-02-17  
Scope: Remaining `DSH-ESM-*` refactor slices  
Goal: lock externally visible dashboard behavior so architecture refactors do not change runtime semantics.

## Contract Matrix

### C1 Tab routing and visibility contract
- URL hash routes are canonical and stable: `#monitoring`, `#ip-bans`, `#status`, `#config`, `#tuning`.
- Exactly one tab is selected and visible at a time.
- Keyboard navigation (`ArrowLeft`, `ArrowRight`, `Home`, `End`) updates hash and active panel deterministically.
- Reload preserves active tab route and panel visibility.

Coverage:
- `e2e/dashboard.smoke.spec.js`:
  - `tab hash route persists selected panel across reload`
  - `tab keyboard navigation updates hash and selected state`
  - shared `assertActiveTabPanelVisibility(...)`

### C2 Tab state surface contract
- Each tab uses explicit tab-state surface (`loading`, `empty`, `error`, hidden for data-ready steady state).
- Refresh failures surface tab-local error text and do not crash page runtime.
- Empty-state text remains explicit and human-readable (no blank panels for empty data).

Coverage:
- `e2e/dashboard.smoke.spec.js`:
  - `dashboard clean-state renders explicit empty placeholders`
  - `tab error state is surfaced when tab-scoped fetch fails`
  - added in this slice: full tab-state matrix checks across tabs

### C3 API payload adaptation contract
- Admin API adapters remain defensive and normalize sparse payloads.
- JSON payload parsing remains resilient when `Content-Type` is missing.
- Dashboard runtime does not regress to raw/undefined rendering on sparse payloads.

Coverage:
- `e2e/dashboard.modules.unit.test.js`:
  - `dashboard API adapters normalize sparse payloads safely`
  - `dashboard API client parses JSON payloads when content-type is missing`

### C4 Monitoring rendering contract
- Monitoring cards/tables/charts render seeded operational data and empty states safely.
- Prometheus helper text remains API-driven and copy actions remain functional.
- No runtime script/style fetch failures or browser page errors under normal flow.

Coverage:
- `e2e/dashboard.smoke.spec.js`:
  - `dashboard loads and shows seeded operational data`
  - runtime guard hooks (`pageerror`, failed script/css requests)
- `e2e/dashboard.modules.unit.test.js`:
  - monitoring view adapter/render tests
  - chart runtime tests

### C5 Config semantics contract
- Dirty-state enable/disable behavior for config save controls remains stable.
- Admin config write UX remains bounded by `SHUMA_ADMIN_CONFIG_WRITE_ENABLED`.
- Save semantics keep state coherent (post-save state returns to non-dirty baseline).

Coverage:
- `e2e/dashboard.smoke.spec.js`:
  - `maze and duration save buttons use shared dirty-state behavior`
  - expanded in this slice with save/roundtrip checks for critical controls
- `e2e/dashboard.modules.unit.test.js`:
  - config form utils/schema/draft-store/config-controls tests

### C6 Auth/session contract
- Unauthenticated access redirects to login.
- Session restore/login/logout behavior remains stable.
- API unauthorized responses route user to login state without partial-broken UI.

Coverage:
- `e2e/dashboard.smoke.spec.js`:
  - `logout redirects back to login page`
  - `session survives reload and time-range controls refresh chart data`

### C7 Architecture guard contract
- No `window.ShumaDashboard*` registry usage in dashboard JS.
- No class-based dashboard module architecture.
- Module graph honors layer direction (`core -> services -> features -> main`) with no circular imports.

Coverage:
- `e2e/dashboard.modules.unit.test.js`:
  - `dashboard ESM guardrails forbid legacy global registry and class syntax`
  - added in this slice: import-graph layer/cycle guard

## Refactor Rule

For remaining ESM slices:
1. If a change touches any contract above, corresponding test coverage must be updated in the same commit.
2. If behavior intentionally changes, it must be recorded explicitly in the no-net-behavior audit doc.
3. `make test` (with `make dev` running) is mandatory before marking contract-affecting slice complete.
