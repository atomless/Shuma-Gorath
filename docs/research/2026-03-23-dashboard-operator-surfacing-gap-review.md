# Dashboard Operator Surfacing Gap Review

Date: 2026-03-23
Status: active research

Related context:

- [`2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md`](./2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md)
- [`2026-03-23-host-impact-cost-proxy-and-benchmark-review.md`](./2026-03-23-host-impact-cost-proxy-and-benchmark-review.md)
- [`../plans/2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md`](../plans/2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../dashboard/src/lib/components/dashboard/VerificationTab.svelte`](../../dashboard/src/lib/components/dashboard/VerificationTab.svelte)
- [`../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte`](../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte)
- [`../../dashboard/src/lib/components/dashboard/StatusTab.svelte`](../../dashboard/src/lib/components/dashboard/StatusTab.svelte)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../src/observability/operator_snapshot_verified_identity.rs`](../../src/observability/operator_snapshot_verified_identity.rs)
- [`../../src/admin/operator_objectives_api.rs`](../../src/admin/operator_objectives_api.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)

# Purpose

Decide which recently landed machine-first and control-plane capabilities should become first-class operator surfaces before or during the Monitoring overhaul, and which tab should own each one.

# Executive Summary

Shuma's backend and machine-first contracts have moved ahead of the dashboard.

That is mostly good: it means Monitoring and Tuning can project proven semantics later instead of inventing them early.

But the gap is now wide enough that some operator-important capabilities should no longer remain buried in Advanced JSON or backend-only read paths.

The clean split is:

1. a small number of local, already-settled controls should be surfaced before `MON-OVERHAUL-1`,
2. machine-first benchmark, snapshot, and oversight read contracts should be projected through `MON-OVERHAUL-1`,
3. and operator-objective editing should be treated as part of `TUNE-SURFACE-1`, not bolted into Status or Monitoring prematurely.

# Current Gap Findings

## 1. Verified identity is backend-real but tab-invisible

The dashboard Verification tab currently exposes only:

1. `js_required_enforced`,
2. `cdp_detection_enabled`,
3. `cdp_auto_ban`,
4. `cdp_detection_threshold`,
5. `pow_enabled`,
6. `challenge_puzzle_enabled`,
7. and `not_a_bot_*` controls.

Evidence:

1. [`../../dashboard/src/lib/components/dashboard/VerificationTab.svelte`](../../dashboard/src/lib/components/dashboard/VerificationTab.svelte)
2. [`../dashboard-tabs/verification.md`](../dashboard-tabs/verification.md)

At the same time, the writable config surface already includes a substantial verified-identity contract:

1. `verified_identity.enabled`,
2. `verified_identity.native_web_bot_auth_enabled`,
3. `verified_identity.provider_assertions_enabled`,
4. `verified_identity.non_human_traffic_stance`,
5. `verified_identity.replay_window_seconds`,
6. `verified_identity.clock_skew_seconds`,
7. `verified_identity.directory_cache_ttl_seconds`,
8. `verified_identity.directory_freshness_requirement_seconds`,
9. `verified_identity.named_policies`,
10. `verified_identity.category_defaults`,
11. `verified_identity.service_profiles`.

Evidence:

1. [`../../dashboard/src/lib/domain/config-schema.js`](../../dashboard/src/lib/domain/config-schema.js)
2. [`../../src/admin/api.rs`](../../src/admin/api.rs)

The machine-first snapshot also already exposes a typed verified-identity summary with attempts, success or failure counts, top schemes, top categories, provenance, and policy-tranche volume.

Evidence:

1. [`../../src/observability/operator_snapshot_verified_identity.rs`](../../src/observability/operator_snapshot_verified_identity.rs)

Conclusion:

Verified identity now has enough product weight to deserve first-class operator surfacing.

However, the correct surface is not a narrow `Web Bot Auth` pane. It should be a broader `Verified Identity` pane inside `Verification`, because the backend contract already spans:

1. native Web Bot Auth,
2. provider assertions,
3. replay and freshness controls,
4. and verified-identity policy posture.

## 2. Red Team status truth improved, but the new truth seam is still hidden

`ADV-DIAG-1` introduced status-truth recovery from persisted event evidence, including:

1. `generation_truth_basis`,
2. `lane_diagnostics_truth_basis`,
3. and bounded `persisted_event_evidence`.

That is operationally important because it tells the operator whether the status view is coming from mutable control counters or from recovered event truth after a completed run.

Evidence:

1. [`../../src/admin/adversary_sim_status_truth.rs`](../../src/admin/adversary_sim_status_truth.rs)
2. [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)
3. [`../api.md`](../api.md)

But the Red Team tab docs and client surface still only claim lifecycle status and recent runs.

Evidence:

1. [`../dashboard-tabs/red-team.md`](../dashboard-tabs/red-team.md)
2. `rg -n "truth_basis|persisted_event_evidence|generation_truth_basis|lane_diagnostics_truth_basis" ../../dashboard/src`

Conclusion:

The Red Team tab should surface this truth seam directly. This is local tab truth, not Monitoring semantics, so it can land before `MON-OVERHAUL-1`.

## 3. Operator objectives exist as primary state, but have no dashboard control surface

The backend already has a dedicated operator-objectives primary-state endpoint:

1. `GET /admin/operator-objectives`
2. `POST /admin/operator-objectives`

Evidence:

1. [`../../src/admin/operator_objectives_api.rs`](../../src/admin/operator_objectives_api.rs)
2. [`../api.md`](../api.md)

These objectives now include category posture truth and drive the feedback loop.

But the dashboard API client currently does not call those endpoints.

Evidence:

1. [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)

Conclusion:

This absolutely deserves operator surfacing, but it is not a Monitoring concern and should not be improvised into Status. It belongs to the later `TUNE-SURFACE-1` contract because it is a primary control-plane editor for category posture and benchmark intent.

## 4. Oversight status, history, and reconcile preview are backend-real but not yet projected

The backend now exposes:

1. `POST /admin/oversight/reconcile`,
2. `GET /admin/oversight/history`,
3. `GET /admin/oversight/agent/status`.

Evidence:

1. [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
2. [`../api.md`](../api.md)

Those are exactly the kinds of machine-first read contracts the next Monitoring surface should project.

But there is no current dashboard API usage for them.

Evidence:

1. [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)

Conclusion:

These should be surfaced, but as part of `MON-OVERHAUL-1`, not as ad hoc one-off panels ahead of the Monitoring redesign.

## 5. Operator snapshot and benchmark results are also backend-real but not yet projected

The backend already exposes:

1. `GET /admin/operator-snapshot`
2. `GET /admin/benchmark-results`

Evidence:

1. [`../../src/admin/operator_snapshot_api.rs`](../../src/admin/operator_snapshot_api.rs)
2. [`../../src/admin/benchmark_api.rs`](../../src/admin/benchmark_api.rs)
3. [`../api.md`](../api.md)

These are the correct machine-first sources for:

1. benchmark status,
2. tuning eligibility,
3. category posture,
4. allowed actions,
5. recent changes,
6. and the verified-identity and host-impact summaries.

Conclusion:

They should be treated as Monitoring-owned read models, not as raw JSON panes or improvised Status cards.

# Recommended Ownership

## Immediate pre-Monitoring local tab work

These can land before `MON-OVERHAUL-1` without inventing new cross-tab semantics:

1. `UI-VID-1`: add a `Verified Identity` pane to the `Verification` tab
2. `UI-RED-1`: add adversary-sim truth-basis and recovered-evidence rendering to the `Red Team` tab

## Monitoring-owned later projection

These should be held for `MON-OVERHAUL-1`:

1. operator snapshot summary projection,
2. benchmark results projection,
3. oversight status and history projection,
4. later host-impact cost projection,
5. later verified-identity alignment and botness-conflict projection.

The Monitoring overhaul should consume those machine-first contracts and render them for humans. It should not create parallel local semantics first.

## Tuning-owned later control

These should be held for `TUNE-SURFACE-1`:

1. operator-objectives editor,
2. category posture editor,
3. and later category-aware tuning controls that reuse the stable taxonomy and benchmark semantics.

# Recommended Sequence

1. Land `UI-VID-1` after the remaining verified-identity backend truth tranches so the first pane surfaces already-faithful semantics.
2. Land `UI-RED-1` once the local dashboard queue allows it because the backend truth it renders is already shipped.
3. Keep `MON-OVERHAUL-1` responsible for operator-snapshot, benchmark-results, and oversight read-model projection.
4. Keep `TUNE-SURFACE-1` responsible for operator-objectives and category-posture editing.

# Non-Goals

Do not:

1. add a raw `Web Bot Auth` pane that ignores provider assertions and broader verified-identity posture,
2. add raw JSON panels for operator snapshot or benchmark results,
3. move operator objectives into Status,
4. or scatter oversight state across Red Team, Status, and Monitoring without one ownership plan.
