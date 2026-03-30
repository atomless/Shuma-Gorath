# Dashboard Tab: Tuning

Route: `#tuning`  
Component: [`dashboard/src/lib/components/dashboard/TuningTab.svelte`](../../dashboard/src/lib/components/dashboard/TuningTab.svelte)

Purpose:

- Tune botness thresholds and signal weights.
- Present only the operator-owned surfaces that are inside the canonical controller mutability policy's `controller_tunable` ring, leaving `never` and `manual_only` surfaces out of the main tuning affordance.
- Temporarily carry the current read-only botness scoring-definition inventory until the broader `TUNE-SURFACE-2*` realignment lands.

Current scope note:

- The broader March 23-24 Tuning expansion chain around category-posture editing, `Identification` ownership changes, and later objective-budget surfaces is retired as defunct and must not be treated as current roadmap.

Panels:

- `Botness Scoring`:
  - thresholds: `not_a_bot_risk_threshold`, `challenge_puzzle_risk_threshold`, `botness_maze_threshold`.
  - signal weights: `botness_weights.js_required`, `botness_weights.geo_risk`, `botness_weights.rate_medium`, `botness_weights.rate_high`.
- `Current Botness Scoring Signals` (temporary, read-only):
  - current additive signal definitions from `botness_signal_definitions`,
  - excludes the dedicated Akamai additive edge contribution, which now sits with `Verification`.

Validation:

- Threshold ordering is validated.
- Weights are clamped by explicit numeric ranges.

Mutability note:

- `Tuning` must stay aligned to the canonical `never` / `manual_only` / `controller_tunable` classification rather than inferring eligibility from broader admin writability.
- The read-only scoring-definition panel is a temporary bridge only; it does not redefine `Tuning` as a mixed diagnostics tab.
