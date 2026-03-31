# Dashboard Tab: Traps

Route: `#traps`  
Component: [`dashboard/src/lib/components/dashboard/TrapsTab.svelte`](../../dashboard/src/lib/components/dashboard/TrapsTab.svelte)

Purpose:

- Configure trap-style defenses: maze, tarpit, and honeypot paths.

Panels:

- `Maze`:
  - enable toggle (`maze_enabled`),
  - auto-ban toggle (`maze_auto_ban`),
  - auto-ban threshold (`maze_auto_ban_threshold`).
- `Tarpit`:
  - enable toggle (`tarpit_enabled`).
  - UI warns when maze is disabled because tarpit serving requires maze routing.
- `Honeypot Paths`:
  - enable toggle (`honeypot_enabled`),
  - path list (`honeypots`) with strict path validation.

Preview links:

- Maze preview: `/shuma/admin/maze/preview`.
- Tarpit preview: `/shuma/admin/tarpit/preview`.

Writes:

- `maze_*`, `tarpit_enabled`, `honeypot_*` config keys.
