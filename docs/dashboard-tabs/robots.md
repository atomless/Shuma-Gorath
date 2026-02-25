# Dashboard Tab: Robots.txt

Route: `#robots`  
Component: [`dashboard/src/lib/components/dashboard/RobotsTab.svelte`](../../dashboard/src/lib/components/dashboard/RobotsTab.svelte)

Purpose:

- Configure served `robots.txt` policy and preview current/unsaved output.

Panels and controls:

- `Serve a Robots.txt Specifying Bot Policy` toggle (`robots_enabled`).
- Crawl delay input (`robots_crawl_delay`, range `0..60`).
- AI policy toggles:
  - opt out AI training (`ai_policy_block_training`),
  - opt out AI search (`ai_policy_block_search`),
  - restrict search engines (`ai_policy_allow_search_engines`, inverse of UI toggle).
- Preview controls:
  - direct link to `/robots.txt` for current saved policy,
  - show/hide preview fed by `POST /admin/robots/preview` with unsaved state.

Preview behavior:

- When preview is open, it auto-refreshes on control changes with debounce.
