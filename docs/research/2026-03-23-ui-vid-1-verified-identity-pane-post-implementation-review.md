# UI-VID-1 Post-Implementation Review

Date: 2026-03-23
Status: Closed

Related context:

- [`2026-03-23-dashboard-operator-surfacing-gap-review.md`](2026-03-23-dashboard-operator-surfacing-gap-review.md)
- [`../plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](../plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)
- [`../../dashboard/src/lib/components/dashboard/VerificationTab.svelte`](../../dashboard/src/lib/components/dashboard/VerificationTab.svelte)
- [`../../dashboard/src/lib/runtime/dashboard-runtime-refresh.js`](../../dashboard/src/lib/runtime/dashboard-runtime-refresh.js)
- [`../../dashboard/src/routes/+page.svelte`](../../dashboard/src/routes/+page.svelte)

# Scope Reviewed

This closeout reviewed the delivered `UI-VID-1` slice:

1. first-class verified-identity mechanics in the `Verification` tab,
2. bounded verified-identity health projection sourced from `operator_snapshot_v1`,
3. and the local refresh lifecycle needed to hydrate that summary truthfully on tab activation.

# What Landed

1. `Verification` now surfaces:
   - verified-identity enablement,
   - native Web Bot Auth toggle,
   - provider assertions toggle,
   - replay window,
   - clock skew,
   - directory cache TTL,
   - directory freshness requirement.
2. The tab also now renders a bounded health summary from `operator_snapshot_v1`:
   - availability,
   - attempts,
   - verified,
   - failed,
   - unique identities,
   - named-policy and service-profile counts,
   - top failure reasons,
   - top schemes,
   - top categories.
3. The dashboard client now exposes `GET /admin/operator-snapshot` and stores it as a bounded dashboard snapshot.
4. `Verification` refresh now hydrates operator snapshot truth in addition to shared config.
5. Route activation now treats `Verification` as incomplete until its operator-snapshot summary is present locally.

# Review Result

The delivered slice matches the plan's intended ownership split:

1. verified-identity mechanics and health now live in `Verification`,
2. richer read-model projection still remains reserved for `MON-OVERHAUL-1`,
3. and verified-identity posture editing still remains outside this first pane.

The implementation also stayed aligned with the existing dashboard architecture:

- shared config still owns the save path,
- the new summary is read-only operator-snapshot truth,
- and the UI reuses existing config panel, toggle, numeric-input, and status-list patterns rather than inventing new dashboard idioms.

# Shortfalls Found

One tranche-local shortfall appeared during implementation:

1. `Verification` originally never fetched `operator_snapshot_v1` on tab activation if shared config had already been loaded earlier in the session.

This was caused by the existing `shouldRefreshOnActivate` gate treating config presence as sufficient hydration for the `Verification` tab. The slice now corrects that by treating missing `operatorSnapshot.verified_identity` truth as incomplete, so the summary hydrates on first activation and does not silently remain absent.

No further tranche-local shortfall remains open.

# Verification

- `make test-dashboard-verified-identity-pane`
- `git diff --check`

# Operational Note

This pane intentionally stops at stable mechanics and bounded health:

- named-policy editing,
- category-default editing,
- broader benchmark or oversight projection,
- and richer verified-identity calibration storytelling

remain downstream work and should continue to land in the planned Monitoring and Tuning tranches rather than accreting into this local pre-Monitoring surface.
