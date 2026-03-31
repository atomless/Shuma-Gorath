# Dashboard Tab: Policy

Route: `#policy`  
Component: [`dashboard/src/lib/components/dashboard/RobotsTab.svelte`](../../dashboard/src/lib/components/dashboard/RobotsTab.svelte)

Purpose:

- Configure served `robots.txt` policy, ban durations, browser policy, and trusted path bypass rules.

Panels and controls:

- `Robots and AI policy`:
  - `Serve a Robots.txt Specifying Bot Policy` toggle (`robots_enabled`),
  - crawl delay input (`robots_crawl_delay`, range `0..60`),
  - AI policy toggles:
  - opt out AI training (`ai_policy_block_training`),
  - opt out AI search (`ai_policy_block_search`),
  - restrict search engines (`ai_policy_allow_search_engines`, inverse of UI toggle).
- preview controls:
  - direct link to `/robots.txt` for current saved policy,
  - show or hide preview fed by `POST /shuma/admin/robots/preview` with unsaved state.
- `Ban Durations`:
  - per-trigger duration tuples for `honeypot`, `ip_range_honeypot`, `maze_crawler`, `rate_limit`, `cdp`, `edge_fingerprint`, `tarpit_persistence`, `not_a_bot_abuse`, `challenge_puzzle_abuse`, and `admin`.
- `Browser Policy`:
  - toggle (`browser_policy_enabled`),
  - minimum-version signal rules (`browser_block`).
- `Path Allowlist`:
  - toggle (`path_allowlist_enabled`) to enable or disable path-bypass matching,
  - trusted bypass paths (`path_allowlist`) for webhook and integration endpoints,
  - supports exact paths (for example `/webhook/stripe`) and prefix wildcards (for example `/api/integrations/*`),
  - entries are preserved while disabled and take effect when re-enabled.

Preview behavior:

- When preview is open, it auto-refreshes on control changes with debounce.

Validation:

- Crawl delay must be between `0` and `60` seconds.
- Duration tuples are validated against bounded minimum and maximum seconds.
- Browser rules must be valid `BrowserName,min_major` lines.
