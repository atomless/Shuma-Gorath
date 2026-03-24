Date: 2026-03-23
Status: Proposed

Related context:

- [`../research/2026-03-23-dashboard-operator-surfacing-gap-review.md`](../research/2026-03-23-dashboard-operator-surfacing-gap-review.md)
- [`2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md`](2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)
- [`2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md`](2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md)
- [`2026-03-24-identification-tab-rename-and-taxonomy-distinction-plan.md`](2026-03-24-identification-tab-rename-and-taxonomy-distinction-plan.md)
- [`../../dashboard/src/lib/components/dashboard/VerificationTab.svelte`](../../dashboard/src/lib/components/dashboard/VerificationTab.svelte)
- [`../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte`](../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)

# Objective

Queue the missing operator-facing surfaces in the cleanest ownership buckets so the dashboard catches up with recent backend progress without undermining the Monitoring and Tuning redesigns.

# Core Decisions

1. Add only local, already-settled tab surfaces before `MON-OVERHAUL-1`.
2. Keep machine-first read-model projection work inside `MON-OVERHAUL-1`.
3. Keep operator-objective editing inside `TUNE-SURFACE-1`.
4. Do not add raw JSON panes as a substitute for proper operator surfaces.
5. Treat verified identity as a broader product surface than native Web Bot Auth alone.
6. Keep operator-facing product stance surfaces distinct from the later recursive-improvement development reference stance.

# Tranche Plan

## `UI-VID-1`: Verified Identity Pane In Verification Tab

### Goal

Give operators a first-class `Verified Identity` pane in the `Verification` tab so native Web Bot Auth and related verified-identity controls are no longer hidden inside Advanced JSON.

### Files

- Modify:
  - `dashboard/src/lib/components/dashboard/VerificationTab.svelte`
  - `dashboard/src/lib/domain/api-client.js`
  - `dashboard/src/lib/domain/config-runtime.js`
  - `dashboard/src/lib/domain/config-schema.js`
  - `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
  - `Makefile`
- Update docs:
  - `docs/dashboard-tabs/verification.md`
  - `docs/dashboard.md`
  - `docs/testing.md`

### Required behavior

1. Surface verified-identity config controls that already exist and are stable enough for first-class UI:
   - `verified_identity.enabled`
   - `verified_identity.native_web_bot_auth_enabled`
   - `verified_identity.provider_assertions_enabled`
   - `verified_identity.replay_window_seconds`
   - `verified_identity.clock_skew_seconds`
   - `verified_identity.directory_cache_ttl_seconds`
   - `verified_identity.directory_freshness_requirement_seconds`
2. Show a bounded read-only health summary sourced from already-materialized config or monitoring truth:
   - attempts
   - verified
   - failed
   - top failure reasons
   - top schemes
   - top categories
3. Keep named policies, category defaults, and service profiles out of this first pane if the local control shape would be too blunt; those can remain Advanced JSON until a better editor exists.
4. Use existing dashboard components and design tokens only.

### Verification target

Add and use a focused make target such as `make test-dashboard-verified-identity-pane`.

It should prove:

1. the pane renders in `Verification`,
2. config writes round-trip through the existing config save flow,
3. the health summary renders bounded backend truth,
4. and Advanced JSON parity remains intact.

## `UI-RED-1`: Red Team Truth-Basis Diagnostics

### Goal

Expose the recently landed adversary-sim status truth seam so operators can see whether a run summary is coming directly from mutable counters or from recovered persisted event evidence.

### Files

- Modify:
  - `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
  - `dashboard/src/lib/domain/api-client.js`
  - `dashboard/src/lib/domain/adversary-sim.js`
  - `dashboard/src/lib/runtime/dashboard-red-team-controller.js`
  - `Makefile`
- Update docs:
  - `docs/dashboard-tabs/red-team.md`
  - `docs/dashboard.md`
  - `docs/testing.md`

### Required behavior

1. Render the backend `truth_basis` for:
   - generation counters
   - lane diagnostics
2. Render bounded `persisted_event_evidence` when present.
3. Keep the new details subordinate to the existing lifecycle and recent-runs surfaces; do not turn Red Team into a mini Monitoring tab.
4. Use warning or info messaging that clearly distinguishes:
   - direct runtime counters,
   - recovered lower-bound evidence,
   - and unavailable truth.

### Verification target

Add and use a focused make target such as `make test-dashboard-red-team-truth-basis`.

It should prove:

1. persisted-event lower-bound truth renders when present,
2. the empty and direct-counter states remain explicit,
3. and recent-run rendering is unchanged.

## `MON-OVERHAUL-1`: Monitoring Projection Of Machine-First Read Models

### Goal

Keep the Monitoring overhaul responsible for projecting the backend machine-first read models rather than sneaking them into local tabs first, and frame that projection as loop-accountability first rather than generic operator chrome.

### Monitoring-owned projection scope

1. `GET /admin/operator-snapshot`
2. `GET /admin/benchmark-results`
3. `GET /admin/oversight/history`
4. `GET /admin/oversight/agent/status`
5. any reused or extracted shared aggregate chart/view-model surface currently living in the transitional Diagnostics implementation

### Required operator stories

1. what the loop currently believes,
2. whether the current window is improving, stable, or regressing against the prior window,
3. what the last reconcile or apply cycle decided,
4. whether recent changes improved or regressed the benchmark envelope across multiple recent completed loops rather than only the latest decision,
5. where the remaining problem sits in the non-human taxonomy,
6. whether tuning is eligible,
7. and whether verified or tolerated non-human traffic is being harmed.
8. and all of that should be expressed against the current operator-selected product stance rather than against the later development reference stance.

### Sequencing note

`MON-OVERHAUL-1B` should land before a ruthless Diagnostics cleanup so Monitoring can first reuse or extract the current shared aggregate chart and view-model surface. A focused `DIAG-CLEANUP-1` tranche should then remove the remaining aggregate leftovers from Diagnostics once Monitoring owns the pieces it still needs.

## `TUNE-SURFACE-1`: Operator Objectives And Category Posture Editor

### Goal

Keep primary control-plane editing of site intent in Tuning rather than scattering it across Status or Monitoring.

As of 2026-03-23, the first concrete shape for this work is now settled:

1. the non-human category posture editor should live in `Tuning`, not `Policy`,
2. because it is active defense posture and controller intent rather than passive `robots.txt`-style declaration,
3. and the first UI should be a taxonomy posture matrix over the stable canonical categories, optionally seeded from a small set of stance archetypes.
4. Those archetypes are operator-facing product presets and must not be silently reinterpreted as the later recursive-improvement reference-stance mechanism.

### Tuning-owned control scope

1. `GET /admin/operator-objectives`
2. `POST /admin/operator-objectives`
3. category posture editing over the stable taxonomy
4. later patch-family and controller posture explanation derived from the canonical controller mutability policy, not from raw admin-config writability

First concrete UI contract:

1. one row per canonical non-human category,
2. one column per posture scale value (`allowed`, `tolerated`, `cost_reduced`, `restricted`, `blocked`),
3. row labels and descriptions taken directly from the taxonomy contract,
4. a small optional preset selector above the matrix that seeds row values only and falls back to `custom` after manual edits,
5. no silent cross-tab writes into `Policy` or `Verification` from this first tuning slice.
6. this first slice should read as visually primary inside `Tuning`; the matrix must not be buried under lower-level threshold controls.

Follow-on ownership after `TUNE-SURFACE-1A`:

1. `Tuning` should become the canonical editable home for ratified controller-tunable botness and fingerprint controls,
2. the current `Fingerprinting` tab should be renamed to `Identification` and retain provider-topology and signal-source posture plus a read-only effective scoring diagnostic view,
3. `Identification` should also explain how the available signals distinguish the canonical non-human taxonomy categories,
4. and that consolidation should execute only after `CTRL-SURFACE-1..3` ratifies which fingerprint controls are genuinely in-bounds.

# Sequence

1. Finish the remaining verified-identity backend truth work.
2. Land `UI-VID-1`.
3. Land `UI-RED-1`.
4. Then execute `MON-OVERHAUL-1`.
5. Then ratify the controller mutability policy and hard-no-touch boundary (`CTRL-SURFACE-1..3`).
6. Then execute `TUNE-SURFACE-1A`.
7. Then execute `TUNE-SURFACE-1B`.
8. Then execute `TUNE-SURFACE-1C`.

# Definition Of Done

This plan is satisfied when:

1. verified identity is no longer Advanced-only for basic operator use,
2. Red Team exposes its status truth basis explicitly,
3. Monitoring owns the human projection of snapshot, benchmark, and oversight read models,
4. Tuning owns operator-objective and category-posture editing,
5. and the dashboard no longer lags the backend in these operator-critical areas.
