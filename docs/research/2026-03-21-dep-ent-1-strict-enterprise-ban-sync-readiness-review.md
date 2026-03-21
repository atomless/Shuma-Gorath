Date: 2026-03-21
Status: Active readiness review

Related context:

- [`../plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](../plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)
- [`../plans/2026-02-13-provider-externalization-design.md`](../plans/2026-02-13-provider-externalization-design.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/security-review.md`](../../todos/security-review.md)
- [`../../src/config/mod.rs`](../../src/config/mod.rs)
- [`../../src/config/tests.rs`](../../src/config/tests.rs)
- [`../../src/providers/contracts.rs`](../../src/providers/contracts.rs)
- [`../../src/providers/external.rs`](../../src/providers/external.rs)
- [`../../src/providers/registry.rs`](../../src/providers/registry.rs)
- [`../../src/runtime/policy_pipeline.rs`](../../src/runtime/policy_pipeline.rs)
- [`../../src/runtime/effect_intents/intent_executor.rs`](../../src/runtime/effect_intents/intent_executor.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../docs/configuration.md`](../../docs/configuration.md)
- [`../../docs/deployment.md`](../../docs/deployment.md)
- [`../../docs/module-boundaries.md`](../../docs/module-boundaries.md)

# Purpose

Assess whether `DEP-ENT-1` is execution-ready and identify the smallest clean tranche order for closing the open enterprise multi-instance ban correctness finding.

# Current Code Truth

## 1. Configuration only blocks local-only enterprise posture, not ban-backend outage drift

Shuma already has one important enterprise guardrail:

1. `enterprise_multi_instance=true` plus `edge_integration_mode=authoritative` rejects local-only rate or ban state.
2. additive or off posture can still run temporarily with `SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true`.

That protects against obviously wrong static configuration, but it does not yet define what must happen when the external ban backend becomes unavailable after startup.

## 2. The external ban-store adapter still hardcodes silent local fallback

The current external ban-store implementation still does all four of these on backend failure:

1. `is_banned` falls back to internal local state,
2. `list_active_bans` falls back to internal local state,
3. `ban_ip_with_fingerprint` falls back to a local-only write,
4. `unban_ip` falls back to a local-only delete.

That means Shuma can still split into per-instance active-ban truth during backend outage even when enterprise multi-instance posture is otherwise configured correctly.

## 3. The provider contract is not expressive enough for strict enterprise truth yet

Today the ban-store provider contract assumes reads always collapse to plain values and writes are fire-and-forget:

1. reads return `bool` or `Vec<...>`,
2. writes return `()`,
3. and the separate `sync_ban` / `sync_unban` helpers only report whether a Redis URL exists, not whether the actual operation stayed authoritative.

That is why admin and runtime call sites cannot currently distinguish:

1. fully synced external success,
2. local fallback,
3. or backend-unavailable failure.

## 4. Operator-facing docs still describe unconditional fallback

The current operator docs and module-boundary docs still describe the ban store as:

1. Redis-backed when configured,
2. with fallback to internal ban behavior when unavailable.

That wording is no longer acceptable for the authoritative enterprise target captured in `DEP-ENT-1`.

# Readiness Findings

## 1. `DEP-ENT-1` is execution-ready now

The needed code seams already exist:

1. provider contract and registry boundaries are in place,
2. enterprise config guardrails already exist,
3. deployment preflight already validates enterprise env posture,
4. and there are focused provider/config tests to extend.

This tranche does not need another prerequisite foundation.

## 2. The first clean slice is an explicit ban-store outage posture contract

The external rate limiter already has an explicit outage posture model. The ban store should match that level of truth:

1. fallback behavior must become an explicit configured posture,
2. authoritative enterprise must require `fail_closed`,
3. and the provider contract must stop pretending every path is always available.

## 3. Admin and runtime call sites need truthful degraded semantics in the same tranche

Adding only config parsing would not close the finding.

The same tranche should also ensure:

1. admin ban and unban do not silently succeed via local fallback when strict sync is required,
2. ban-list reads do not silently present local-only state as authoritative enterprise truth,
3. and runtime ban checks do not quietly create per-instance drift under strict posture.

## 4. `DEP-ENT-1` should stop short of observability and two-instance proof

Those belong to `DEP-ENT-2..4`.

This tranche should focus only on:

1. strict operating semantics,
2. truthful config/deploy guardrails,
3. truthful provider labels and docs,
4. and one focused make target that proves the strict contract.

# Recommended Slice Order

1. `DEP-ENT-1-1`: add explicit ban-store outage posture config plus provider contract results.
2. `DEP-ENT-1-2`: remove silent local fallback under strict posture across runtime and admin call paths.
3. `DEP-ENT-1-3`: require `fail_closed` for authoritative enterprise guardrails and deployment validation.
4. `DEP-ENT-1-4`: add focused Makefile verification and refresh docs to describe the settled contract truthfully.

# Outcome

Treat `DEP-ENT-1` as the next active execution tranche.

The correct first move is not enterprise integration breadth. It is tightening the ban-store contract so authoritative enterprise can no longer drift into local-only state when the distributed backend degrades.
