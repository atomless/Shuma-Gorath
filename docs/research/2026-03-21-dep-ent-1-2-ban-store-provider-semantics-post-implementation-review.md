Date: 2026-03-21
Status: Post-implementation review

Related context:

- [`../plans/2026-03-21-dep-ent-1-strict-enterprise-ban-sync-implementation-plan.md`](../plans/2026-03-21-dep-ent-1-strict-enterprise-ban-sync-implementation-plan.md)
- [`../../src/providers/contracts.rs`](../../src/providers/contracts.rs)
- [`../../src/providers/external.rs`](../../src/providers/external.rs)
- [`../../src/providers/internal.rs`](../../src/providers/internal.rs)
- [`../../src/providers/registry.rs`](../../src/providers/registry.rs)
- [`../../src/runtime/policy_pipeline.rs`](../../src/runtime/policy_pipeline.rs)
- [`../../src/runtime/effect_intents/intent_executor.rs`](../../src/runtime/effect_intents/intent_executor.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../Makefile`](../../Makefile)

# Scope Reviewed

`DEP-ENT-1-2`: truthful ban-store provider read and write outcomes under the explicit outage posture contract.

# What Landed

1. Added explicit provider read results:
   - `BanLookupResult::{Banned, NotBanned, Unavailable}`
   - `BanListResult::{Available, Unavailable}`
2. Changed ban-store writes to return `BanSyncResult` directly from `ban_ip_with_fingerprint` and `unban_ip`.
3. Removed the stale `sync_ban` and `sync_unban` helper path so the contract no longer pretends that checking for a Redis URL is equivalent to proving a synced write.
4. Updated the external provider so:
   - backend success returns authoritative results,
   - `fallback_internal` still uses local fallback and reports `Deferred`,
   - `fail_open` and `fail_closed` return `Unavailable` or `Failed` without writing local-only state.
5. Updated the internal provider to project its local state through the new contract so runtime and admin call sites can consume one provider shape while `DEP-ENT-1-3` tightens their degraded-state behavior.
6. Expanded the focused make gate to cover strict read, write, and unban outage semantics.

# Verification Evidence

1. `make test-enterprise-ban-store-contract`
2. `git diff --check`

# Plan Versus Implementation

This slice met the planned `DEP-ENT-1-2` contract:

1. the provider boundary now distinguishes available versus unavailable reads,
2. writes now distinguish synced versus deferred-local versus failed outcomes,
3. and the external adapter no longer silently falls back to local-only state when strict outage posture is configured.

# Shortfalls

No new tranche-local shortfall was found inside `DEP-ENT-1-2`.

The remaining work is the already-planned next tranche:

1. `DEP-ENT-1-3` still needs to stop runtime and admin from collapsing these explicit provider results back into normal success paths under strict outage posture.

# Next Recommended Step

Proceed to `DEP-ENT-1-3`.
