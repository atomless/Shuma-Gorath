Date: 2026-03-21
Status: Post-implementation review

Related context:

- [`../plans/2026-03-21-dep-ent-1-strict-enterprise-ban-sync-implementation-plan.md`](../plans/2026-03-21-dep-ent-1-strict-enterprise-ban-sync-implementation-plan.md)
- [`../../src/runtime/policy_pipeline.rs`](../../src/runtime/policy_pipeline.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../Makefile`](../../Makefile)

# Scope Reviewed

`DEP-ENT-1-3`: runtime and admin call-site truthfulness under strict enterprise ban-store outage posture.

# What Landed

1. Runtime ban checks now interpret provider lookup results through outage posture instead of treating only explicit `Banned` as authoritative:
   - `Unavailable` now maps to an existing ban when `SHUMA_BAN_STORE_OUTAGE_MODE=fail_closed`,
   - and remains non-banning in non-strict modes.
2. Manual `POST /admin/ban` and `POST /admin/unban` now return `503` instead of reporting success when strict sync fails.
3. `GET /admin/ban` now returns `503` instead of silently presenting an empty or local-only list when the provider reports active-ban reads as unavailable.
4. Focused contract coverage now proves the runtime strictness branch and the admin success-versus-failure behavior directly.

# Verification Evidence

1. `make test-enterprise-ban-store-contract`
2. `git diff --check`

# Plan Versus Implementation

This slice met the planned `DEP-ENT-1-3` contract:

1. admin manual ban and unban no longer claim success after a strict backend failure,
2. the primary admin active-ban list no longer hides provider unavailability behind a normal success path,
3. and runtime ban checks now fail closed under the configured strict enterprise posture.

# Shortfalls

One new follow-on shortfall was found during closeout:

1. several operator-visible ban-read surfaces still bypass the provider boundary and read local active-ban state directly, including the `/admin/ip-bans/delta` and `/admin/ip-bans/stream` snapshots plus monitoring and analytics ban counts in `src/admin/api.rs`.

This is not a hidden failure inside the delivered `DEP-ENT-1-3` scope, but it is still truthfulness drift against the broader enterprise authoritative posture. It has therefore been captured as a new immediate follow-on tranche: `DEP-ENT-1-3A`.

# Next Recommended Step

Proceed to `DEP-ENT-1-3A`, then continue to `DEP-ENT-1-4`.
