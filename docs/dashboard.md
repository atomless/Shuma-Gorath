# 🐙 Dashboard Documentation

## 🐙 Overview

The dashboard is a tabbed SvelteKit admin UI for monitoring and runtime configuration of Shuma-Gorath.

Canonical tab behavior and controls are documented in [`docs/dashboard-tabs/README.md`](dashboard-tabs/README.md).

Related terminology and architecture docs:

- [`fingerprinting-terminology.md`](fingerprinting-terminology.md)
- [`fingerprinting-signal-planes.md`](fingerprinting-signal-planes.md)

## 🐙 Tab Routes

The dashboard uses URL hash routes:

- `#monitoring` - [`dashboard-tabs/monitoring.md`](dashboard-tabs/monitoring.md)
- `#ip-bans` - [`dashboard-tabs/ip-bans.md`](dashboard-tabs/ip-bans.md)
- `#status` - [`dashboard-tabs/status.md`](dashboard-tabs/status.md)
- `#verification` - [`dashboard-tabs/verification.md`](dashboard-tabs/verification.md)
- `#traps` - [`dashboard-tabs/traps.md`](dashboard-tabs/traps.md)
- `#rate-limiting` - [`dashboard-tabs/rate-limiting.md`](dashboard-tabs/rate-limiting.md)
- `#geo` - [`dashboard-tabs/geo.md`](dashboard-tabs/geo.md)
- `#fingerprinting` - [`dashboard-tabs/fingerprinting.md`](dashboard-tabs/fingerprinting.md)
- `#robots` - [`dashboard-tabs/robots.md`](dashboard-tabs/robots.md)
- `#tuning` - [`dashboard-tabs/tuning.md`](dashboard-tabs/tuning.md)
- `#advanced` - [`dashboard-tabs/advanced.md`](dashboard-tabs/advanced.md)

Behavior:

- Selected tab is reflected in the URL hash.
- Reload preserves selected tab.
- Keyboard tab navigation is supported (`Left`, `Right`, `Home`, `End`).
- Each tab exposes explicit loading, empty, and error state messaging.

## 🐙 Refresh Model

- Polling and refresh are scoped to the active tab.
- Auto-refresh is available only on `Monitoring` and `IP Bans`.
- All other tabs refresh on initial load, on explicit refresh events, and after relevant save flows.
- Monitoring and IP-bans snapshots use bounded local cache to reduce repeated admin API load on rapid remount/revisit.

## 🐙 Runtime Architecture

- UI is SvelteKit static output served from `dist/dashboard`.
- Route orchestration lives in [`dashboard/src/routes/+page.svelte`](../dashboard/src/routes/+page.svelte).
- Tab list/state contracts are defined in [`dashboard/src/lib/domain/dashboard-state.js`](../dashboard/src/lib/domain/dashboard-state.js).
- Refresh orchestration is in [`dashboard/src/lib/runtime/dashboard-runtime-refresh.js`](../dashboard/src/lib/runtime/dashboard-runtime-refresh.js).
- Runtime session/config mutation boundary is [`dashboard/src/lib/runtime/dashboard-native-runtime.js`](../dashboard/src/lib/runtime/dashboard-native-runtime.js).

## 🐙 Access

Development:

- `http://127.0.0.1:3000/dashboard/index.html`
- `http://127.0.0.1:3000/dashboard` (redirects to `/dashboard/index.html`)

Notes:

- Login page: `/dashboard/login.html`
- Admin session uses same-origin cookie + CSRF header for state-changing calls.
- Config panes are editable only when `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=true`.

## 🐙 Admin API Endpoints Used by Dashboard

- `GET /admin/session`
- `POST /admin/logout`
- `GET /admin/monitoring?hours=24&limit=10`
- `GET /admin/events?hours=N`
- `GET /admin/ban`
- `POST /admin/ban`
- `POST /admin/unban`
- `GET /admin/cdp`
- `GET /admin/config`
- `POST /admin/config`
- `POST /admin/config/validate`
- `GET /admin/robots`
- `POST /admin/robots/preview`

Preview links surfaced in tab UI:

- `GET /admin/maze/preview`
- `GET /admin/tarpit/preview`
- `GET /robots.txt`

## 🐙 Local Asset Provenance

Chart runtime is vendored locally:

- Asset: `dashboard/static/assets/vendor/chart-lite-1.0.0.min.js`
- Version: `chart-lite-1.0.0`
- SHA-256: `5eec3d4b98e9ddc1fb88c44e0953b8bded137779a4d930c6ab2647a431308388`

## 🐙 Rollback

Pre-launch cutover model: no legacy dashboard runtime branch is retained in-tree.

Rollback method:

1. Revert offending commit(s).
2. Run `make test`.
3. Run `make build`.
4. Redeploy.

