# DEP-ENT-1 Strict Enterprise Ban-Sync Implementation Plan

Date: 2026-03-21
Status: Active implementation plan

Related context:

- [`2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](./2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)
- [`2026-02-13-provider-externalization-design.md`](./2026-02-13-provider-externalization-design.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](./2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../research/2026-03-21-dep-ent-1-strict-enterprise-ban-sync-readiness-review.md`](../research/2026-03-21-dep-ent-1-strict-enterprise-ban-sync-readiness-review.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/security-review.md`](../../todos/security-review.md)
- [`../../src/config/mod.rs`](../../src/config/mod.rs)
- [`../../src/providers/contracts.rs`](../../src/providers/contracts.rs)
- [`../../src/providers/external.rs`](../../src/providers/external.rs)
- [`../../src/providers/registry.rs`](../../src/providers/registry.rs)
- [`../../src/runtime/policy_pipeline.rs`](../../src/runtime/policy_pipeline.rs)
- [`../../src/runtime/effect_intents/intent_executor.rs`](../../src/runtime/effect_intents/intent_executor.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../Makefile`](../../Makefile)

## Goal

Implement strict enterprise distributed ban-sync mode so authoritative multi-instance deployments no longer have a silent local-only divergence path when the external ban backend is unavailable.

## Architecture

Keep the existing provider boundary and enterprise config guardrail model, but make the ban store honest in the same way the distributed rate limiter already is:

1. explicit outage posture,
2. explicit operation result semantics,
3. authoritative enterprise guardrails that require strict posture,
4. and focused verification that proves the contract name matches the real behavior.

## Code-Truth Baseline

Today the ban-store path still has three problems:

1. config has no explicit ban-store outage posture,
2. the external provider silently falls back to internal local state for reads and writes,
3. admin and runtime call sites cannot tell the difference between synced, deferred-local, or failed operations.

`DEP-ENT-1` fixes those three issues without bundling in the later observability, integration, or outage-matrix tranches.

## Core Decisions

### 1. Ban-store outage posture must be explicit

Add one env-only control:

1. `SHUMA_BAN_STORE_OUTAGE_MODE`

Accepted values:

1. `fallback_internal`
2. `fail_open`
3. `fail_closed`

Default:

1. `fallback_internal`

This keeps self-hosted and additive enterprise behavior low-friction by default while allowing authoritative enterprise to require strictness.

### 2. Authoritative enterprise must require `fail_closed`

When all of these are true:

1. `SHUMA_ENTERPRISE_MULTI_INSTANCE=true`
2. `provider_backends.ban_store=external`
3. `edge_integration_mode=authoritative`

then Shuma must reject startup or deployment validation unless:

1. `SHUMA_BAN_STORE_OUTAGE_MODE=fail_closed`

This keeps authoritative enterprise honest: it must not silently downgrade to per-instance local state when the shared ban backend degrades.

### 3. Provider operations need truthful result types

The ban-store provider contract should expose:

1. whether a lookup was available versus degraded,
2. whether a write was synced, deferred-local, or failed,
3. and whether admin list reads are authoritative or unavailable.

This is required so the runtime and admin surfaces stop collapsing all degraded states into normal success paths.

### 4. Runtime and admin behavior should be strict, not speculative

Under strict outage posture:

1. no local fallback write is allowed,
2. manual ban and unban must return failure instead of quietly mutating local-only state,
3. admin ban-list reads must not quietly present local-only state as enterprise truth,
4. runtime ban checks must deny rather than create local-only divergence.

This tranche does not need to invent richer multi-instance retry orchestration. It only needs to eliminate silent split-brain behavior.

### 5. Provider labels and docs must match the new contract

Once outage posture becomes configurable, the implementation label can no longer claim unconditional internal fallback.

Docs and labels should describe the ban store as:

1. Redis-backed distributed sync with explicit outage posture,
2. not Redis with hidden internal fallback.

## Execution Slices

### DEP-ENT-1-1: Ban-store outage posture config contract

Scope:

1. add `BanStoreOutageMode` parsing, defaults, validation, and runtime exposure,
2. add config tests for valid values and enterprise authoritative guardrails,
3. wire deployment/setup env handling and help text,
4. update provider implementation labels if they currently imply unconditional fallback.

Acceptance:

1. `SHUMA_BAN_STORE_OUTAGE_MODE` exists in defaults, setup, env validation, deployment preflight, and runtime export surfaces,
2. authoritative enterprise plus `ban_store=external` requires `fail_closed`,
3. focused tests fail if the guardrail is removed or the accepted values drift.

### DEP-ENT-1-2: Strict provider semantics for reads and writes

Scope:

1. extend the ban-store provider contract with truthful lookup and write outcomes,
2. stop unconditional internal fallback in the external provider,
3. preserve internal fallback only when `fallback_internal` is explicitly selected,
4. return explicit `failed` or `unavailable` outcomes for strict outage modes.

Acceptance:

1. backend failure in `fallback_internal` mode still uses local fallback,
2. backend failure in `fail_open` or `fail_closed` mode does not write local-only state,
3. tests prove synced versus deferred versus failed write results,
4. tests prove lookup/list behavior changes with outage posture rather than always collapsing to internal.

### DEP-ENT-1-3: Runtime and admin call-site truthfulness

Scope:

1. make `POST /admin/ban` and `POST /admin/unban` fail truthfully when strict sync cannot be maintained,
2. make `GET /admin/ban` fail or degrade truthfully instead of silently reading local-only state under strict outage modes,
3. make runtime ban checks deny under `fail_closed` rather than creating local-only divergence,
4. keep the slice local to ban-store behavior and do not broaden into rate-limiter or global request-flow redesign.

Acceptance:

1. admin manual ban/unban no longer report success after a strict-mode backend failure,
2. runtime no longer silently converts external backend failure into local-only enterprise truth,
3. tests cover admin and runtime boundaries where the hidden regression could otherwise survive.

### DEP-ENT-1-3A: Provider-aware operator ban-read surfaces

Scope:

1. make operator-visible ban-read surfaces use provider-aware active-ban semantics instead of unconditional local scans,
2. cover `/admin/ip-bans/delta` and `/admin/ip-bans/stream` active-ban snapshots plus monitoring and analytics ban-count summaries that still read local state directly,
3. keep the slice read-only and local to operator/status surfaces rather than redesigning the event log or the provider contract again.

Acceptance:

1. operator ban-read surfaces no longer present local-only active-ban state as authoritative enterprise truth under strict outage posture,
2. monitoring and analytics ban counts degrade truthfully when authoritative active-ban state is unavailable,
3. focused tests cover the operator-visible boundaries where this drift could otherwise remain hidden.

### DEP-ENT-1-4: Focused verification and docs

Scope:

1. add a truthful focused Makefile target for the strict enterprise ban-store contract,
2. refresh configuration, deployment, and module-boundary docs,
3. update TODO/completed history and write the tranche review.

Acceptance:

1. the make target name matches the real verification scope,
2. docs explicitly state that authoritative enterprise requires external ban store plus `fail_closed`,
3. docs no longer describe unconditional internal fallback as the enterprise contract.

## Verification Strategy

Add or refine focused Makefile coverage before relying on ad hoc commands.

Expected tranche verification:

1. `make test-enterprise-ban-store-contract`
2. `git diff --check`

The target should cover:

1. config parsing and guardrail tests,
2. external provider outage-semantics tests,
3. admin ban-path truthfulness tests,
4. runtime ban-check strictness tests if a focused selector exists,
5. operator-visible ban-read truthfulness tests once `DEP-ENT-1-3A` lands.

## Completion Criteria

`DEP-ENT-1` is complete when:

1. authoritative enterprise requires `ban_store=external` plus `SHUMA_BAN_STORE_OUTAGE_MODE=fail_closed`,
2. the external ban store no longer silently writes or reads local-only state under strict outage modes,
3. admin surfaces fail truthfully when strict sync is unavailable,
4. focused make verification proves the contract end to end,
5. and docs no longer describe unconditional internal fallback as the enterprise target.
