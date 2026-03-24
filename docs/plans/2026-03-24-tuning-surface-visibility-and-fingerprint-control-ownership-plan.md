Date: 2026-03-24
Status: Proposed

Related context:

- [`../research/2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-review.md`](../research/2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-review.md)
- [`2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)
- [`2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md`](2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md)
- [`2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)
- [`../../dashboard/src/lib/components/dashboard/TuningTab.svelte`](../../dashboard/src/lib/components/dashboard/TuningTab.svelte)
- [`../../dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`](../../dashboard/src/lib/components/dashboard/FingerprintingTab.svelte)

# Objective

Define the clean execution sequence that makes the `Tuning` tab more visible and consolidates ratified botness and fingerprint tuning controls there, while reducing `Fingerprinting` to truthful provider-source posture and read-only effective scoring diagnostics.

# Core Decisions

1. `Tuning` is the operator-owned enforcement and tuning surface, not merely a small threshold editor.
2. `Fingerprinting` keeps provider-topology and signal-source posture, not controller-tunable scoring ownership.
3. The current `Fingerprinting` signal bars remain read-only diagnostics, but they must no longer imply that the underlying controls are permanently immutable or owned by that tab.
4. No editable fingerprint control should move into `Tuning` until `CTRL-SURFACE-1..3` ratifies that it is genuinely inside the bounded controller-tunable ring.
5. The first Tuning work remains the taxonomy posture matrix; botness and fingerprint control consolidation comes immediately after that, not before.

# Planned Tranches

## `TUNE-SURFACE-1A`

### Goal

Land the first visibly dominant `Tuning` contract:

1. the `Non-Human Traffic Posture` matrix,
2. preset seeding,
3. and a layout where category posture is clearly the primary section of the tab rather than a side panel below threshold micro-controls.

### Notes

This tranche is already specified in [`2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md). The additional settled requirement is that the section must feel visually primary so the tab reads as a real tuning surface from first render.

## `TUNE-SURFACE-1B`

### Goal

Move the ratified controller-tunable botness and fingerprint controls into `Tuning`, and narrow `Fingerprinting` to provider-source posture plus effective scoring diagnostics.

### Required behavior

1. Keep `provider_backends.fingerprint_signal` and `edge_integration_mode` in `Fingerprinting`.
2. Keep the read-only runtime scoring-definition panel in `Fingerprinting`, but rename and document it as a diagnostic projection rather than a mutability claim.
3. Move or add into `Tuning` the botness and fingerprint knobs that `CTRL-SURFACE-1..3` ratifies as controller-tunable.
4. Keep the moved controls grouped by operator meaning, for example:
   1. botness thresholds,
   2. cross-signal weights,
   3. fingerprint sensitivity and caps.
5. Make the Tuning tab the canonical editable home for those controls; do not leave the same writable control split across tabs.

### Files

Modify:

1. `dashboard/src/lib/components/dashboard/TuningTab.svelte`
2. `dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`
3. `dashboard/src/lib/domain/api-client.js`
4. `dashboard/src/lib/domain/config-schema.js`
5. `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
6. `dashboard/src/lib/state/dashboard-store.js`
7. `Makefile`

Update docs:

1. `docs/dashboard-tabs/tuning.md`
2. `docs/dashboard-tabs/fingerprinting.md`
3. `docs/dashboard.md`
4. `docs/testing.md`

### Verification targets

Add focused make targets such as:

1. `make test-dashboard-tuning-control-ownership`
2. `make test-dashboard-fingerprinting-diagnostics-ownership`

They should prove:

1. the ratified editable botness and fingerprint controls render only in `Tuning`,
2. `Fingerprinting` still renders provider-topology controls and effective scoring diagnostics,
3. the read-only scoring panel does not regress into a second writable editor,
4. and save payloads remain truthful to the canonical config or objective contracts.

## `TUNE-SURFACE-1C`

### Goal

Add the later objective-budget editing and controller-explanation layer after `TUNE-SURFACE-1B` finishes the ownership split.

### Required behavior

1. Explain which families are `controller_tunable`, `manual_only`, or `never` using the canonical mutability policy.
2. Add later budget editing only after the operator-facing posture and tuning-control ownership is settled.
3. Keep explanation surfaces derived from the canonical mutability policy rather than from ad hoc local tab logic.

# Sequencing

1. Finish `MON-OVERHAUL-1A..1C`.
2. Finish `CTRL-SURFACE-1..3`.
3. Execute `TUNE-SURFACE-1A`.
4. Execute `TUNE-SURFACE-1B`.
5. Execute `TUNE-SURFACE-1C`.

This work should remain blocked until both Monitoring and controller mutability are settled, because:

1. Monitoring must define the operator-facing accountability pattern first,
2. the controller mutability policy must define the editable fingerprint surface truthfully before the dashboard claims ownership of it.

# Definition Of Done

This plan is satisfied when:

1. `Tuning` clearly reads as the operator-owned tuning surface from first render,
2. the taxonomy posture matrix is the primary section of that tab,
3. ratified botness and fingerprint tuning controls live in `Tuning`,
4. `Fingerprinting` keeps only provider-source posture plus effective scoring diagnostics,
5. and the read-only scoring panel in `Fingerprinting` is framed as a diagnostic projection rather than as a hard mutability boundary.
