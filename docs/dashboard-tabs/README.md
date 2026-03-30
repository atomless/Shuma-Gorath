# Dashboard Tabs

Canonical tab reference for the dashboard. These docs are intentionally tab-specific so operator guidance stays aligned with the current UI.

Top-level tabs and routes:

- [`traffic.md`](traffic.md) - `#traffic`
- [`ip-bans.md`](ip-bans.md) - `#ip-bans`
- [`red-team.md`](red-team.md) - `#red-team`
- [`game-loop.md`](game-loop.md) - `#game-loop`
- [`tuning.md`](tuning.md) - `#tuning`
- [`verification.md`](verification.md) - `#verification`
- [`traps.md`](traps.md) - `#traps`
- [`rate-limiting.md`](rate-limiting.md) - `#rate-limiting`
- [`geo.md`](geo.md) - `#geo`
- [`policy.md`](policy.md) - `#policy`
- [`status.md`](status.md) - `#status`
- [`advanced.md`](advanced.md) - `#advanced`
- [`diagnostics.md`](diagnostics.md) - `#diagnostics`

Implementation source of truth:

- Routing and tab shell: [`dashboard/src/routes/+page.svelte`](../../dashboard/src/routes/+page.svelte)
- Tab list constant: [`dashboard/src/lib/domain/dashboard-state.js`](../../dashboard/src/lib/domain/dashboard-state.js)
- Tab refresh orchestration: [`dashboard/src/lib/runtime/dashboard-runtime-refresh.js`](../../dashboard/src/lib/runtime/dashboard-runtime-refresh.js)
