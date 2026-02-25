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
  - per-trigger duration tuples for `honeypot`, `rate_limit`, `browser`, `cdp`, `admin`.

Validation:

- Threshold ordering is validated.
- Weights are clamped by explicit numeric ranges.
- Duration tuples are validated against bounded minimum/maximum seconds.
