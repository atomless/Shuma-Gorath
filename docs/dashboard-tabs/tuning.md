# Dashboard Tab: Tuning

Route: `#tuning`  
Component: [`dashboard/src/lib/components/dashboard/TuningTab.svelte`](../../dashboard/src/lib/components/dashboard/TuningTab.svelte)

Purpose:

- Tune risk thresholds, signal weights, and ban durations.

Panels:

- `Botness Scoring`:
  - thresholds: `not_a_bot_risk_threshold`, `challenge_puzzle_risk_threshold`, `botness_maze_threshold`.
  - signal weights: `botness_weights.js_required`, `botness_weights.geo_risk`, `botness_weights.rate_medium`, `botness_weights.rate_high`.
  - read-only status/default values and scored/terminal signal inventories.
- `Ban Durations`:
  - per-trigger duration tuples for `honeypot`, `rate_limit`, `cdp`, `admin`.
- `Browser Policy`:
  - toggle (`browser_policy_enabled`),
  - minimum-version signal rules (`browser_block`).
- `Path Allowlist`:
  - trusted bypass paths (`path_allowlist`) for webhook/integration endpoints,
  - supports exact paths (for example `/webhook/stripe`) and prefix wildcards (for example `/api/integrations/*`),
  - active only when `bypass_allowlists_enabled` is enabled.

Validation:

- Threshold ordering is validated.
- Weights are clamped by explicit numeric ranges.
- Duration tuples are validated against bounded minimum/maximum seconds.
