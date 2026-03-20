# Shared-Host Scope Fence Contract

Date: 2026-03-20
Status: Active implementation plan

Related context:

- [`../research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md`](../research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md)
- [`2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](./2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](./2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](./2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`../../src/crawler_policy/mod.rs`](../../src/crawler_policy/mod.rs)
- [`../../src/runtime/upstream_proxy.rs`](../../src/runtime/upstream_proxy.rs)
- [`../../scripts/site_surface_catalog.py`](../../scripts/site_surface_catalog.py)
- [`../../scripts/tests/adversarial/lane_contract.v1.json`](../../scripts/tests/adversarial/lane_contract.v1.json)

## Purpose

Define the first executable contract for `SIM-SH-SURFACE-1-1`:

1. one narrow shared-host descriptor,
2. one fail-closed URL scope gate,
3. one structured rejection taxonomy,
4. and one clear boundary between deploy-time catalog tooling and emergent-lane discovery truth.

This note exists because the roadmap and discovery design are now clear, but the repo still lacks the concrete contract that future seed tooling and later Scrapling runtime work should implement.

## Why The Older `SIM-SCR-2` Proposal Needs Narrowing

The older Scrapling plan correctly identified the safety requirements, but its first implementation sketch was still too config-heavy for the current stage.

Specifically, the older note still pointed first-wave work at:

1. `config/defaults.env`,
2. KV seeding and admin config write paths,
3. dashboard Advanced JSON parity,
4. and runtime status exposure.

That is now too broad for the first shared-host gate.

At this stage Shuma does not yet have:

1. a Scrapling runtime lane that consumes this policy,
2. an operator workflow that needs persistent dashboard editing for it,
3. or evidence that a new admin-writable config surface is the cleanest first home.

Adding that surface now would create speculative control-plane debt before the lane exists.

## Core Decision

`SIM-SH-SURFACE-1-1` should land first as an executable contract and pre-lane tooling surface, not as a new admin or KV configuration subsystem.

That means:

1. define one versioned shared-host scope descriptor,
2. define one shared normalization and validation algorithm,
3. enforce fail-closed structured rejection reasons,
4. prove the contract through focused tests and a truthful `make` target,
5. and defer admin config, dashboard controls, and runtime status projection until there is an actual lane or operator workflow that needs them.

## Contract Shape

The first descriptor should stay minimal.

Recommended shape:

```json
{
  "schema_version": "shared-host-scope-contract.v1",
  "allowed_hosts": ["example.com", "www.example.com"],
  "denied_path_prefixes": ["/admin", "/internal", "/dashboard", "/session", "/auth", "/login"],
  "require_https": true,
  "deny_ip_literals": true
}
```

## Descriptor Rules

### 1. `allowed_hosts` is required

Rules:

1. entries must be host or authority values only, not full URLs,
2. entries must normalize to lowercase ASCII with trailing dots removed,
3. userinfo, path, query, and fragment content is forbidden,
4. and at least one allowed host must be present.

Rationale:

1. host confinement is the primary safety boundary for shared-host traversal,
2. and this should be explicit rather than inferred from a later crawl result.

### 2. `denied_path_prefixes` is required

Rules:

1. entries must start with `/`,
2. entries must be normalized path prefixes only, not glob patterns,
3. query and fragment syntax is forbidden inside prefix definitions,
4. and the first contract should ship with an internal or privileged default family even if later operators can extend it.

Required baseline deny family:

1. `/admin`
2. `/internal`
3. `/dashboard`
4. `/session`
5. `/auth`
6. `/login`

Additional families may be added later when real hosted-site evidence proves they belong in the default deny set.

### 3. `require_https` defaults to `true`

Rules:

1. non-HTTPS absolute targets fail closed when this is enabled,
2. relative links are resolved against an already accepted base URL,
3. and this remains a scope rule rather than a deploy transport hint.

### 4. `deny_ip_literals` defaults to `true`

Rules:

1. IPv4 and IPv6 literal hosts are rejected,
2. bracketed IPv6 literals are rejected the same way,
3. and disabling this later should require explicit justification rather than silent fallback.

## Gate Semantics

For every absolute target URL accepted as a seed or traversal candidate, the validator must:

1. parse the URL strictly,
2. require a host,
3. require HTTPS when `require_https=true`,
4. reject IP-literal hosts when `deny_ip_literals=true`,
5. require the normalized host to be in `allowed_hosts`,
6. reject any path under a denied prefix,
7. and return a structured allow or reject result rather than a boolean-only answer.

## Redirect Revalidation

Redirect handling must reuse the same fail-closed principle already present in gateway forwarding:

1. relative redirects are resolved against the accepted current URL and then rechecked,
2. scheme-relative redirects must be normalized and rechecked,
3. absolute redirects must pass the same host, scheme, and denied-path validation,
4. and any redirect target that fails the same gate is rejected as out of scope.

This is the same architectural posture already used in [`../../src/runtime/upstream_proxy.rs`](../../src/runtime/upstream_proxy.rs): redirects are not trusted just because the first request was accepted.

## Structured Rejection Taxonomy

The first contract should define explicit reason codes so seed tooling, future runtime code, and operator evidence can stay aligned.

Minimum set:

1. `malformed_url`
2. `missing_host`
3. `non_https`
4. `ip_literal_host`
5. `host_not_allowed`
6. `denied_path_prefix`
7. `redirect_target_out_of_scope`

These reasons should be stable contract values, not freeform strings assembled differently by each consumer.

## Explicit Boundary: Deployment Catalog Is Not Discovery Truth

The repo already has a real deployment helper in [`../../scripts/build_site_surface_catalog.py`](../../scripts/build_site_surface_catalog.py) and [`../../scripts/site_surface_catalog.py`](../../scripts/site_surface_catalog.py).

That tooling still has a valid job:

1. gateway route-collision preflight,
2. deploy-time parity and onboarding evidence,
3. and deterministic public-path receipts for existing-site setup.

It must not become the emergent lane's working surface map.

So this contract explicitly separates:

1. deploy-time catalog artifacts for gateway and setup workflows,
2. from adversary traversal truth, which must emerge from accepted seeds plus observed telemetry.

## What `SIM-SH-SURFACE-1-1` Should Not Do

This slice must not:

1. add dashboard controls,
2. add KV-backed admin config fields,
3. add Advanced JSON parity burden,
4. add a rich path/query-pattern DSL,
5. or revive a catalog-first runtime discovery model.

Those are either later operator-surface concerns or explicitly rejected by the current discovery architecture.

## Implementation Order

### 1. Version the contract first

Add a versioned contract artifact similar to existing adversarial contract files so drift can fail fast.

Recommended first artifact:

1. `scripts/tests/adversarial/shared_host_scope_contract.v1.json`

### 2. Add one shared validator in the pre-lane tooling surface

The first executable consumer should be a small shared Python module because:

1. the next slice (`SIM-SH-SURFACE-1-2`) is expected to be Make target and tooling first,
2. there is not yet a runtime Scrapling lane that justifies dormant Rust control-plane wiring,
3. and a focused tooling-first validator keeps the contract executable without prebuilding unused runtime surface.

Recommended first touchpoints:

1. `scripts/tests/shared_host_scope.py` (new shared normalization and validation helper)
2. `scripts/tests/test_shared_host_scope.py` (unit coverage)
3. `scripts/tests/check_shared_host_scope_contract.py` (contract parity check)
4. `Makefile` (`make test-shared-host-scope-contract`)

### 3. Make `SIM-SH-SURFACE-1-2` consume this contract directly

The minimal seed intake slice should not define a second URL gate.

It should:

1. load the same descriptor,
2. run every primary, robots-derived, and manual seed candidate through the same validator,
3. preserve source provenance,
4. and record rejection reasons without inventing another taxonomy.

### 4. Delay Rust control-plane exposure until a real runtime consumer exists

When Scrapling lane or runtime status work begins, the Rust side should mirror or consume the same contract deliberately.

Until then, do not add:

1. dormant config fields,
2. read-only status payload placeholders,
3. or dashboard-only scaffolding.

## Acceptance Criteria For `SIM-SH-SURFACE-1-1`

This tranche should be considered complete when:

1. one authoritative versioned shared-host scope contract exists,
2. the contract defines the minimal descriptor above and the rejection taxonomy,
3. the executable validator is fail-closed and test-covered,
4. redirect revalidation is part of the contract rather than an implementation detail,
5. and the repo docs no longer imply that first-wave shared-host scope work must begin with admin or KV config plumbing.

## Verification Plan

Add and use:

1. `make test-shared-host-scope-contract`

That target should prove:

1. contract JSON shape is valid,
2. validator and contract stay in parity,
3. malformed, non-HTTPS, IP-literal, off-host, denied-path, and redirect-escape targets are rejected with stable reason codes,
4. and accepted in-scope URLs normalize deterministically.

## Outcome

When this plan is executed, Shuma will have the minimal shared-host safety gate it actually needs:

1. narrow,
2. fail-closed,
3. reusable by the next seed slice,
4. and cleanly separated from both deployment-catalog tooling and later runtime lane control surfaces.
