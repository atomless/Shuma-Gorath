# Dashboard Tab: Tuning

Route: `#tuning`  
Component: [`dashboard/src/lib/components/dashboard/TuningTab.svelte`](../../dashboard/src/lib/components/dashboard/TuningTab.svelte)

Purpose:

- Tune botness thresholds and signal weights.
- Own operator-editable tuning posture, but only for surfaces that sit inside the canonical controller mutability rings; Tuning must not infer controller eligibility from admin writability alone.

Panels:

- `Botness Scoring`:
  - thresholds: `not_a_bot_risk_threshold`, `challenge_puzzle_risk_threshold`, `botness_maze_threshold`.
  - signal weights: `botness_weights.js_required`, `botness_weights.geo_risk`, `botness_weights.rate_medium`, `botness_weights.rate_high`.

Validation:

- Threshold ordering is validated.
- Weights are clamped by explicit numeric ranges.

Policy note:

- Later controller explanations in Tuning must consume `operator_snapshot_v1.allowed_actions` for `controller_mutability`, `auto_proposal_status`, and `proposable_patch_paths` rather than inventing a second local mutability model.
