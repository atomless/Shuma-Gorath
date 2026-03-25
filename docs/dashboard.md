# 🐙 Dashboard Documentation

## 🐙 Overview

The dashboard is a tabbed SvelteKit admin UI for traffic visibility, closed-loop accountability, and runtime configuration of Shuma-Gorath.

Canonical tab behavior and controls are documented in [`docs/dashboard-tabs/README.md`](dashboard-tabs/README.md).

Related terminology and architecture docs:

- [`fingerprinting-terminology.md`](fingerprinting-terminology.md)
- [`fingerprinting-signal-planes.md`](fingerprinting-signal-planes.md)

## 🐙 Tab Routes

The dashboard uses URL hash routes:

- `#traffic` - [`dashboard-tabs/traffic.md`](dashboard-tabs/traffic.md)
- `#ip-bans` - [`dashboard-tabs/ip-bans.md`](dashboard-tabs/ip-bans.md)
- `#red-team` - [`dashboard-tabs/red-team.md`](dashboard-tabs/red-team.md)
- `#game-loop` - [`dashboard-tabs/game-loop.md`](dashboard-tabs/game-loop.md)
- `#tuning` - [`dashboard-tabs/tuning.md`](dashboard-tabs/tuning.md)
- `#verification` - [`dashboard-tabs/verification.md`](dashboard-tabs/verification.md)
- `#traps` - [`dashboard-tabs/traps.md`](dashboard-tabs/traps.md)
- `#rate-limiting` - [`dashboard-tabs/rate-limiting.md`](dashboard-tabs/rate-limiting.md)
- `#geo` - [`dashboard-tabs/geo.md`](dashboard-tabs/geo.md)
- `#fingerprinting` - [`dashboard-tabs/fingerprinting.md`](dashboard-tabs/fingerprinting.md)
- `#policy` - [`dashboard-tabs/policy.md`](dashboard-tabs/policy.md)
- `#status` - [`dashboard-tabs/status.md`](dashboard-tabs/status.md)
- `#advanced` - [`dashboard-tabs/advanced.md`](dashboard-tabs/advanced.md)
- `#diagnostics` - [`dashboard-tabs/diagnostics.md`](dashboard-tabs/diagnostics.md)

Behavior:

- Selected tab is reflected in the URL hash.
- Reload preserves selected tab.
- Keyboard tab navigation is supported (`Left`, `Right`, `Home`, `End`).
- Each tab exposes explicit loading, empty, and error state messaging.
- `Traffic` is now the first visible tab in the canonical tab ordering.
- `Traffic` now owns the live traffic picture.
- `Game Loop` now owns the closed-loop accountability story.
- `Diagnostics` now owns deep subsystem inspection and contributor-focused telemetry detail.

## 🐙 Refresh Model

- `Game Loop` now projects the first real machine-first accountability layer:
  - `operator_snapshot_v1`
  - `benchmark_results_v1`
  - bounded oversight status/history
  - and the bounded machine-first episode archive that now underlies later stepping-stone memory and homeostasis inputs
  - current verdict, multi-loop progress, outcome frontier, controller judgment, and bounded trust/blocker context
- The remaining Game Loop follow-on is `MON-OVERHAUL-1C`, which adds the fuller category-aware pressure and final trust/actionability surface.
- `Traffic` now shares the top-level refresh bar and uses the bounded monitoring refresh path for cost-effective traffic reads.
- Manual refresh is available on `Traffic`, `Diagnostics`, `IP Bans`, and `Red Team`.
- Auto-refresh is available on `Traffic`, `IP Bans`, and `Red Team`.
- Most tabs refresh on initial load, on explicit refresh events, and after relevant save flows.
- `Verification` now refreshes both shared config and the bounded `operator_snapshot_v1` verified-identity summary when the tab activates and that summary is not already present locally.
- The `Red Team` adversary-sim controller is page-scoped rather than tab-scoped:
  - it forces a status refresh on dashboard bootstrap, `Red Team` tab activation, and page-visibility resume,
  - it keeps status polling alive while a control request is submitting/converging and while backend truth reports the sim as `running` or `stopping`,
  - hiding the `Red Team` panel does not pause a running sim or let backend status go stale,
  - and the tab now surfaces whether generation/lane counters come directly from runtime control state or from recovered persisted-event lower-bound evidence.
- Diagnostics and IP-bans snapshots use bounded local cache to reduce repeated admin API load on rapid remount/revisit.

## 🐙 Runtime Architecture

- UI is SvelteKit static output served from `dist/dashboard`.
- Route orchestration lives in [`dashboard/src/routes/+page.svelte`](../dashboard/src/routes/+page.svelte).
- Tab list/state contracts are defined in [`dashboard/src/lib/domain/dashboard-state.js`](../dashboard/src/lib/domain/dashboard-state.js).
- The page-scoped adversary-sim intent controller lives in [`dashboard/src/lib/runtime/dashboard-red-team-controller.js`](../dashboard/src/lib/runtime/dashboard-red-team-controller.js).
- Refresh orchestration is in [`dashboard/src/lib/runtime/dashboard-runtime-refresh.js`](../dashboard/src/lib/runtime/dashboard-runtime-refresh.js).
- Runtime session/config mutation boundary is [`dashboard/src/lib/runtime/dashboard-native-runtime.js`](../dashboard/src/lib/runtime/dashboard-native-runtime.js).

## 🐙 Access

Development:

- `http://127.0.0.1:3000/dashboard/index.html`
- `http://127.0.0.1:3000/dashboard` (redirects to `/dashboard/index.html`)

Notes:

- Login page: `/dashboard/login.html`
- Logged-out navigation to `/dashboard` or `/dashboard/index.html` now keeps the dashboard shell hidden while session truth is restored and leaves only the normal disconnected striped page state visible before redirect.
- Login form uses a native form `POST` to `/admin/login`, a visible readonly `Account` field with `autocomplete="username"`, and `current-password` semantics for the API key field so browsers/password managers can recognize it as a normal sign-in flow and associate the saved key with the dashboard account for this origin.
- When the login page is ready for a new session, it focuses the API key field automatically so operators can paste the key immediately on local and remote dashboards.
- Both dashboard entry routes advertise an explicit dashboard-scoped favicon under `/dashboard/assets/...` so browsers do not fall back to probing `/favicon.ico` at the protected site root.
- When `shadow_mode` is enabled, the dashboard header overlays the Shuma-Gorath image with the dashboard eye marker so operators can tell at a glance that the current session is in logging-only mode.
- Admin session uses same-origin cookie + CSRF header for state-changing calls.
- Config panes are editable only when `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=true`.
- Adversary Sim OFF -> ON toggles show a frontier-key warning when no frontier provider keys are configured:
  - continue without frontier calls, or
  - cancel, add `SHUMA_FRONTIER_*_API_KEY` values to `.env.local`, restart `make dev`, then toggle on again.

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
- `GET /admin/operator-snapshot`
- `GET /admin/benchmark-results`
- `GET /admin/oversight/history`
- `GET /admin/oversight/agent/status`
- `POST /admin/config`
- `POST /admin/config/validate`
- `POST /admin/adversary-sim/control`
- `GET /admin/adversary-sim/status`
- `GET /admin/robots`
- `POST /admin/robots/preview`

Preview links surfaced in tab UI:

- `GET /admin/maze/preview`
- `GET /admin/tarpit/preview`
- `GET /robots.txt`

## 🐙 Chart Runtime Provenance

Dashboard charts use upstream Chart.js via the workspace package manager:

- Package: `chart.js`
- Version: `4.5.1` (pinned in `package.json` and lockfile)
- Loading model: ESM dependency loaded through `dashboard/src/lib/domain/services/chart-runtime-adapter.js`

## 🐙 Rollback

Pre-launch cutover model: no legacy dashboard runtime branch is retained in-tree.

Rollback method:

1. Revert offending commit(s).
2. Run `make test`.
3. Run `make build`.
4. Redeploy.
