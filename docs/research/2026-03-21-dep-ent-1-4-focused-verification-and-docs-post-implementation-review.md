Date: 2026-03-21
Status: Post-implementation review

Related context:

- [`../plans/2026-03-21-dep-ent-1-strict-enterprise-ban-sync-implementation-plan.md`](../plans/2026-03-21-dep-ent-1-strict-enterprise-ban-sync-implementation-plan.md)
- [`../../Makefile`](../../Makefile)
- [`../../docs/configuration.md`](../../docs/configuration.md)
- [`../../docs/deployment.md`](../../docs/deployment.md)
- [`../../docs/module-boundaries.md`](../../docs/module-boundaries.md)
- [`../../docs/api.md`](../../docs/api.md)
- [`../../docs/quick-reference.md`](../../docs/quick-reference.md)

# Scope Reviewed

`DEP-ENT-1-4`: focused verification and truthful docs for the settled strict enterprise ban-store contract.

# What Landed

1. Re-checked the focused verification target against the now-landed implementation and confirmed `make test-enterprise-ban-store-contract` already truthfully covers the intended contract surface:
   - config parsing and enterprise guardrail tests,
   - provider outage semantics,
   - runtime strict lookup behavior,
   - primary admin ban-path truthfulness,
   - and operator-visible ban-read truthfulness through dashboard preservation tests.
2. Refreshed configuration, deployment, and module-boundary docs so they no longer describe external ban store as an unconditional internal fallback path.
3. Added explicit operator guidance that:
   - `SHUMA_BAN_STORE_OUTAGE_MODE` defaults to `fallback_internal` for permissive self-hosted posture,
   - authoritative enterprise multi-instance ban sync requires `SHUMA_PROVIDER_BAN_STORE=external` plus `SHUMA_BAN_STORE_OUTAGE_MODE=fail_closed`,
   - and strict outage modes do not serve local-only ban-state fallback to admin or operator surfaces.
4. Updated API and quick-reference docs so operator-visible read and write behavior matches the code:
   - `GET /admin/ban` may return `503` when authoritative active-ban reads are unavailable,
   - manual ban and unban writes may return `503` under strict outage posture,
   - `/admin/analytics` exposes `ban_store_status` and `ban_store_message`,
   - `/admin/ip-bans/delta` and `/admin/ip-bans/stream` expose `active_bans_status` and `active_bans_message`,
   - and monitoring docs now state that unavailable ban-state reads propagate as explicit nullable or unavailable markers instead of numeric zero.

# Verification Evidence

1. `make test-enterprise-ban-store-contract`
2. `git diff --check`

# Plan Versus Implementation

This slice met the planned `DEP-ENT-1-4` contract:

1. the focused make target name matches the real verification scope,
2. docs now explicitly state that authoritative enterprise requires external ban store plus `SHUMA_BAN_STORE_OUTAGE_MODE=fail_closed`,
3. docs no longer describe unconditional internal fallback as the enterprise contract,
4. and the operator-facing API references now match the strict read and write truthfulness delivered in `DEP-ENT-1-3` and `DEP-ENT-1-3A`.

One implementation detail evolved slightly from the original expectation: no further Makefile edits were needed in this tranche because the focused target had already been refined during the earlier slices. The closeout work here was to verify that truthfulness and document it explicitly.

# Shortfalls

No new tranche-local shortfall was found inside `DEP-ENT-1-4`.

`DEP-ENT-1` is now complete.

# Next Recommended Step

Proceed to `DEP-ENT-2`.
