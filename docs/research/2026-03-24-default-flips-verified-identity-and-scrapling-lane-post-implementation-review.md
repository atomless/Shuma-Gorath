# Default Flips Post-Implementation Review

Date: 2026-03-24
Status: Closed

Related context:

- [`../../config/defaults.env`](../../config/defaults.env)
- [`../../src/admin/adversary_sim_state.rs`](../../src/admin/adversary_sim_state.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)
- [`../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte`](../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/runtime/dashboard-adversary-sim.js`](../../dashboard/src/lib/runtime/dashboard-adversary-sim.js)

# Scope Reviewed

This closeout reviewed two default-behavior changes:

1. verified identity enabled by default,
2. Scrapling as the default adversary-sim lane instead of Synthetic.

# What Landed

1. [`config/defaults.env`](../../config/defaults.env) now seeds `SHUMA_VERIFIED_IDENTITY_ENABLED="true"`.
2. The runtime adversary-sim control state now defaults its desired lane to `scrapling_traffic`.
3. Newly started adversary-sim runs now start on the desired lane rather than hard-coding `synthetic_traffic` as the initial active lane.
4. Dashboard adversary-sim fallbacks now treat `scrapling_traffic` as the default desired lane in:
   - the Red Team component default,
   - dashboard API-client normalization,
   - dashboard runtime normalization,
   - and the focused smoke helper used by dashboard e2e coverage.
5. Config documentation now reflects the verified-identity default flip.

# Review Result

The shipped change matches the intended product stance:

1. verified identity now participates by default without changing the configured non-human authorization stance, because the default stance remains `deny_all_non_human`.
2. Scrapling is now the truthful default sim lane for normal operator usage, while Synthetic remains explicitly selectable for bounded deterministic harness checks.
3. The adversary-sim runtime no longer leaks the old Synthetic default through `active_lane` fallback or start-state initialization when the desired lane is Scrapling.

# Shortfalls Found

Two tranche-local shortfalls surfaced during implementation and were corrected in the same slice:

1. several lifecycle tests were implicitly relying on the old Synthetic default when what they really wanted was explicit Synthetic-lane behavior; those tests now declare `lane: synthetic_traffic` directly where that behavior is the point of the test.
2. `effective_active_lane()` still fell back to `synthetic_traffic` for running states without an explicit `active_lane`, which would have leaked the old default even after the top-level state change; it now falls back to the desired lane instead.

No further tranche-local shortfall remains open.

# Verification

- `make test-verified-identity-config`
- `make test-adversary-sim-domain-contract`
- `make test-dashboard-adversary-sim-lane-contract`
- `git diff --check`

# Operational Note

This slice changes only the default posture:

- verified identity policy remains deny-first until operators configure explicit allowances,
- Synthetic traffic remains available as an explicit sim lane,
- and no controller-mutability or later recursive-improvement boundaries changed in this tranche.
