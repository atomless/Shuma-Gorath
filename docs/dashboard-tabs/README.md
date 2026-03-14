# Dashboard Tabs

Canonical tab reference for the dashboard. These docs are intentionally tab-specific so operator guidance stays aligned with the current UI.

Top-level tabs and routes:

- [`monitoring.md`](monitoring.md) - `#monitoring`
- [`ip-bans.md`](ip-bans.md) - `#ip-bans`
- [`status.md`](status.md) - `#status`
- [`red-team.md`](red-team.md) - `#red-team`
- [`verification.md`](verification.md) - `#verification`
- [`traps.md`](traps.md) - `#traps`
- [`rate-limiting.md`](rate-limiting.md) - `#rate-limiting`
- [`geo.md`](geo.md) - `#geo`
- [`fingerprinting.md`](fingerprinting.md) - `#fingerprinting`
- [`robots.md`](robots.md) - `#robots`
- [`tuning.md`](tuning.md) - `#tuning`
- [`advanced.md`](advanced.md) - `#advanced`

Implementation source of truth:

- Routing and tab shell: [`dashboard/src/routes/+page.svelte`](../../dashboard/src/routes/+page.svelte)
- Tab list constant: [`dashboard/src/lib/domain/dashboard-state.js`](../../dashboard/src/lib/domain/dashboard-state.js)
- Tab refresh orchestration: [`dashboard/src/lib/runtime/dashboard-runtime-refresh.js`](../../dashboard/src/lib/runtime/dashboard-runtime-refresh.js)
