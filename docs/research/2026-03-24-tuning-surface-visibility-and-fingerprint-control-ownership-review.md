Date: 2026-03-24
Status: Proposed

Related context:

- [`../../dashboard/src/lib/components/dashboard/TuningTab.svelte`](../../dashboard/src/lib/components/dashboard/TuningTab.svelte)
- [`../../dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`](../../dashboard/src/lib/components/dashboard/FingerprintingTab.svelte)
- [`../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)
- [`../plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](../plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)
- [`../research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
- [`../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md`](../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md)

# Objective

Settle the clean ownership split between `Tuning` and `Fingerprinting` now that:

1. `Tuning` is being promoted into the operator-owned enforcement surface,
2. the controller mutability policy is being ratified,
3. and the current `Fingerprinting` tab still presents an overlapping "read-only" botness view that is not a truthful ownership boundary.

# Findings

## 1. `Tuning` already owns active botness tuning, but it is too narrow and visually secondary

The current `Tuning` tab already edits:

1. `not_a_bot_risk_threshold`,
2. `challenge_puzzle_risk_threshold`,
3. `botness_maze_threshold`,
4. `botness_weights.js_required`,
5. `botness_weights.geo_risk`,
6. `botness_weights.rate_medium`,
7. `botness_weights.rate_high`.

That makes `Tuning` the beginning of the operator-owned scoring and routing surface already, even though the tab still reads like a small threshold editor rather than the main home for defense posture and controller-tunable scoring controls.

## 2. The `Fingerprinting` tab's "read-only" botness list is a runtime projection, not a real mutability boundary

The current `Fingerprinting` pane labeled `Botness Scoring Signals` is built from runtime `botness_signal_definitions`.

That list currently includes:

1. corroboration inputs already edited in `Tuning`, such as `js_verification_required`, `geo_risk`, `rate_pressure_medium`, and `rate_pressure_high`,
2. plus `fp_*` fingerprint-specific scoring signals,
3. while excluding the dedicated Akamai additive signal, which is surfaced separately.

So "read-only" is truthful only in the narrow sense that the panel is a read model. It is not truthful as an ownership signal, because some of the underlying inputs are already operator-editable elsewhere and more may become controller-tunable once `CTRL-SURFACE-1..3` lands.

## 3. Provider topology and edge-trust posture should stay in `Fingerprinting`

The Akamai controls in `Fingerprinting` edit:

1. `provider_backends.fingerprint_signal`,
2. `edge_integration_mode`.

Those are provider-topology and trust-boundary settings. The mutability review already classifies them as hard-never for the optimizer loop. They still belong in a dedicated operator tab, but they are not part of the controller-tunable scoring surface and should not be moved into `Tuning`.

## 4. Fingerprint-specific scoring knobs are good candidates for `Tuning`, but only after the mutability policy is ratified

The controller-mutability review identified the likely in-bounds fingerprint sensitivity surface as:

1. `fingerprint_signal_enabled`,
2. `fingerprint_state_ttl_seconds`,
3. `fingerprint_flow_window_seconds`,
4. `fingerprint_flow_violation_threshold`,
5. `fingerprint_entropy_budget`,
6. the `fingerprint_family_cap_*` fields.

Those fields are much more naturally grouped with botness thresholds and weights in `Tuning` than with provider-source posture in `Fingerprinting`.

But they should not be surfaced as tuning controls until the repo completes:

1. `CTRL-SURFACE-1` to ratify which of them are really `controller_tunable`,
2. `CTRL-SURFACE-2` to align the action catalog and patch proposer,
3. `CTRL-SURFACE-3` to make the resulting mutability truth canonical and operator-visible.

## 5. The clean split is ownership by intent, not by data source

The right line is:

1. `Tuning` owns operator intent for enforcement and controller-tunable scoring,
2. `Fingerprinting` owns signal-source posture, provider topology, and read-only runtime scoring diagnostics.

That means `Fingerprinting` should keep:

1. Akamai enablement and influence mode,
2. bounded explanations of what the current source posture means,
3. a read-only effective scoring view.

And `Tuning` should become the place where editable botness and fingerprint sensitivity controls live, once the mutability policy says those controls are in-bounds.

# Recommended Direction

## 1. Make `Tuning` visibly primary for operator-owned tuning work

The first visual contract for `Tuning` should be:

1. `Non-Human Traffic Posture` matrix first,
2. then the editable botness and fingerprint control surface,
3. then later budgets and controller-explanation material.

That gives the tab a coherent progression:

1. what the operator wants,
2. what the loop may tune to pursue that intent,
3. how the controller understands and applies those knobs.

## 2. Keep `Fingerprinting` as provider posture plus diagnostics

`Fingerprinting` should not remain a second quasi-tuning tab.

Its settled job should be:

1. provider-source and edge-trust posture,
2. effective scoring or runtime-signal diagnostics,
3. bounded read-only explanations of active fingerprint contributions.

The current `Botness Scoring Signals` panel should be renamed in that direction, for example to:

1. `Effective Scoring Signals`, or
2. `Runtime Scoring Definition`.

That avoids implying the underlying knobs are permanently immutable.

## 3. Sequence the work so ownership changes follow mutability truth, not the other way around

The clean execution order is:

1. `MON-OVERHAUL-1A..1C`
2. `CTRL-SURFACE-1..3`
3. `TUNE-SURFACE-1A` taxonomy posture matrix and visibility uplift
4. `TUNE-SURFACE-1B` consolidate ratified controller-tunable botness and fingerprint controls into `Tuning`
5. `TUNE-SURFACE-1C` later objective-budget and controller-explanation expansion

`Fingerprinting` ownership cleanup should be implemented inside `TUNE-SURFACE-1B`, not as an independent disconnected refactor.

# Conclusion

The repo should not treat the current `Fingerprinting` read model as proof that its signal bars belong there permanently.

The truthful long-term split is:

1. `Tuning` for editable posture and ratified tuning controls,
2. `Fingerprinting` for provider/source posture and effective scoring diagnostics.

The only reason not to move more of that surface immediately is sequencing discipline: the mutability policy must settle which fingerprint knobs are genuinely in-bounds before the dashboard promises that they are part of the tuning surface.
