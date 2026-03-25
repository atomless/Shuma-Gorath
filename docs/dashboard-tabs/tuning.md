# Dashboard Tab: Tuning

Route: `#tuning`  
Component: [`dashboard/src/lib/components/dashboard/TuningTab.svelte`](../../dashboard/src/lib/components/dashboard/TuningTab.svelte)

Purpose:

- Tune botness thresholds and signal weights.
- Present only the operator-owned surfaces that are inside the canonical controller mutability policy's `controller_tunable` ring, leaving `never` and `manual_only` surfaces out of the main tuning affordance.

Panels:

- `Botness Scoring`:
  - thresholds: `not_a_bot_risk_threshold`, `challenge_puzzle_risk_threshold`, `botness_maze_threshold`.
  - signal weights: `botness_weights.js_required`, `botness_weights.geo_risk`, `botness_weights.rate_medium`, `botness_weights.rate_high`.

Validation:

- Threshold ordering is validated.
- Weights are clamped by explicit numeric ranges.

Mutability note:

- `Tuning` must stay aligned to the canonical `never` / `manual_only` / `controller_tunable` classification rather than inferring eligibility from broader admin writability.
