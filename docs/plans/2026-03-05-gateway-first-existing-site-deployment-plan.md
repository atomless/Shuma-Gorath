# Gateway-Only Existing-Site Deployment Plan

Date: 2026-03-05  
Status: Proposed (implementation-ready)

Related context:
- [`docs/deployment.md`](../deployment.md)
- [`docs/security-hardening.md`](../security-hardening.md)
- [`docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)
- [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`docs/research/2026-03-05-gateway-only-spin-architecture-research-synthesis.md`](../research/2026-03-05-gateway-only-spin-architecture-research-synthesis.md)

## Objectives

1. Make existing-site gateway deployment the primary and only production deployment mode for Shuma.
2. Remove front-door/native allow-response mode before launch (no backward-compatibility path).
3. Preserve one policy engine and one admin/monitoring surface.
4. Deliver low-friction operator setup that avoids requiring site rewrites in Rust.
5. Enforce fail-closed trust boundaries so existing-origin integration does not weaken security posture.

## Non-goals

1. Requiring users to port their existing application into Shuma runtime code.
2. Keeping dual deployment modes (`native/front-door` and `gateway`) in pre-launch runtime.
3. Supporting arbitrary multi-origin fanout routing in the first gateway tranche.
4. Shipping permissive fallback behavior that silently bypasses Shuma when origin forwarding fails.

## Current State and Gap

Current runtime behavior is request-terminating. Public routes are handled directly by Shuma and allow-path responses are local runtime responses, not upstream forwards. The default outbound policy is closed (`allowed_outbound_hosts=[]`), so existing-origin pass-through is not currently implemented.

This is useful for local reference behavior but does not match expected production adoption for operators with existing sites.

## Spin/Fermyon Platform Constraints (Non-Negotiable)

1. Outbound network access is capability-gated by `allowed_outbound_hosts` and is deny-by-default when unset/empty.
2. Gateway must not rely on manual outbound `Host` header rewrites; upstream authority must be explicit and canonical.
3. Outbound pressure budgets must be explicit (`outbound_http.max_concurrent_requests` and connection pooling policy).
4. Reserved Spin runtime routes (`/.well-known/spin/...`) remain runtime-owned and must be reflected in route assumptions/tests.
5. Fermyon paths must not assume variable-templated `allowed_outbound_hosts` portability.

## Target Architecture

### One Policy Core, One Delivery Adapter

Keep one canonical request pipeline (signals, scoring, enforcement transitions, metrics, event taxonomy). After policy evaluation:

1. **Enforcement outcomes** (`block`, `challenge`, `maze`, `tarpit`) remain Shuma-served responses.
2. **Allow outcome** always uses one transport adapter: `upstream_proxy`.

There is no retained `native_responder` allow-path mode.

### Gateway Contract (Single Upstream, Fail-Closed)

Gateway v1 forwards allow-path traffic to a single configured upstream origin with strict constraints:

1. Explicit deployment-profiled upstream contract:
   - edge/Fermyon profile: `https://host[:port]` required.
   - shared-server profile: `https://host[:port]` preferred; insecure `http://` allowed only for explicitly acknowledged local/private IP-literal origin cases.
   - insecure shared-server exceptions are fail-closed by default for metadata/link-local/special-purpose ranges (`169.254.0.0/16`, `fe80::/10`, multicast, unspecified, equivalent classes) unless explicitly allowlisted by guarded config.
   - insecure upstream on DNS hostnames or public-routable targets is rejected at startup.
2. Explicit outbound host allowlist enforcement in Spin manifest/deploy guardrails (no wildcard production posture).
3. Request/response canonicalization policy with denylist for privileged/internal headers.
4. Failure semantics are explicit:
   - transport/proxy failures are fail-closed;
   - origin HTTP status responses (including 4xx/5xx) are deterministic pass-through outcomes.
5. Origin lock guidance is mandatory so direct-to-origin bypass is not possible.
6. Route namespace collision preflight is mandatory before cutover (Shuma-owned paths vs origin public surface).
7. Redirect confinement is fail-closed:
   - redirect follow is allowed only when scheme + authority remain in-scope,
   - cross-origin redirect targets are denied with explicit `redirect_policy_denied` telemetry,
   - redirect hop budget is bounded and enforced.
8. Upstream HTTPS trust is strict:
   - certificate validation + hostname/SNI verification are mandatory,
   - no production skip-verify path exists.

### Security and Trust Boundaries

1. Preserve existing forwarded-header trust gate (`SHUMA_FORWARDED_IP_SECRET` + `X-Shuma-Forwarded-Secret`) and require edge/proxy sanitization.
2. Introduce strict upstream request construction policy:
   - drop privileged inbound headers from client before origin forward,
   - strip client-provided `Forwarded`/`X-Forwarded-*` provenance headers before forwarding,
   - regenerate deterministic forwarded/provenance headers from trusted runtime context only,
   - attach explicit provenance headers only when configured,
   - prevent header smuggling/double-hop trust confusion.
3. Require operator runbook steps for origin lock-down:
   - firewall/mTLS/allowlist so origin accepts only Shuma path.
4. Origin-auth contract must be explicit and testable:
   - credentials are injected only at Shuma proxy boundary,
   - credential rotation has overlap-safe rollout and max-age policy,
   - stale/invalid origin credentials fail closed and emit dedicated diagnostics.

## Codebase Impact Map and Pain Points

Primary touch points from current runtime architecture:

1. `src/runtime/request_flow.rs`
   - multiple local allow exits currently return `200 OK` bodies (static bypass, allowlist exits, terminal allow).
2. `src/runtime/effect_intents/plan_builder.rs` + `src/runtime/effect_intents/response_renderer.rs`
   - `ResponseIntent::OkBody` currently materializes local allow-like responses.
3. `src/runtime/request_router.rs`
   - control-plane and policy-owned routes that must remain local (`/admin`, `/internal`, `/health`, `/metrics`, challenge/maze routes).
4. `src/config/mod.rs`
   - canonical env/runtime guardrail location for gateway contract and fail-fast posture.
5. `spin.toml`
   - currently deny-by-default outbound policy (`allowed_outbound_hosts=[]`) and must become explicit/safe for gateway.

Debt risks if refactor is not structured:

1. partial gateway conversion leaving hidden local allow exits,
2. transport behavior spread across policy/orchestration layers,
3. permissive fallback under upstream failure,
4. platform mismatch from assumptions invalid under Spin runtime constraints.

## Workstream 0: Evidence and Harness First

### Implementation slices

1. GW-0.1: Add forwarding telemetry taxonomy and diagnostics before behavior rewiring.
2. GW-0.2: Add deterministic upstream fixture harness for integration tests.
3. GW-0.3: Add failure-injection harness (timeout/transport/upstream-class).
4. GW-0.4: Add parser/canonicalization adversarial regression cases.

### Exit criteria

- Forwarding behavior has measurable baseline before transport refactor.
- Regression harness can deterministically reproduce forwarding failure classes.

## Workstream A: Architecture Contract and Guardrails

### Implementation slices

1. GW-A1: Write a concise ADR/addendum defining:
   - gateway-only product posture,
   - one-policy/one-adapter design,
   - explicit removal of native/front-door allow mode,
   - failure semantics and security invariants.
2. GW-A2: Define upstream origin config schema and validation:
   - required single-origin target,
   - profile-aware insecure-local exceptions with explicit special-range denies,
   - strict TLS trust requirements for HTTPS upstreams,
   - timeout/body-size/retry bounds,
   - deterministic startup failure semantics when missing/invalid.
3. GW-A3: Extend deploy/runtime validation to fail when outbound/host policy is unsafe or incomplete for configured upstream.
4. GW-A4: Add loop-prevention and route-collision guardrails:
   - canonical authority comparison to reject self-forward configuration;
   - deterministic loop marker + hop budget policy;
   - reserved-route collision preflight report for existing-site cutover.

### Exit criteria

- Gateway-only constraints are documented and validated before forwarding code lands.
- No ambiguous fallback path exists for missing/invalid upstream config.
- Loop-prevention and reserved-route collision guardrails are validated before transport rollout.

## Workstream B: Upstream Forward Adapter and Front-Door Removal

### Implementation slices

1. GW-B1: Add an upstream forward module behind a clear capability boundary.
2. GW-B2: Route allow-path policy outcomes to the upstream adapter unconditionally.
3. GW-B3: Remove native/front-door allow-response path and associated dead runtime branches/config/docs/tests.
4. GW-B4: Keep enforcement outcomes local and unchanged (no upstream challenge/block rendering).
5. GW-B5: Add request/response mapping contract:
   - method/path/query/body forwarding policy,
   - required dropped/preserved headers list,
   - deterministic strip-and-rebuild policy for `Forwarded`/`X-Forwarded-*`,
   - streaming/body-size policy,
   - redirect confinement (in-scope only) and hop-budget policy,
   - gateway v1 protocol support matrix (supported vs explicitly unsupported behaviors).
6. GW-B6: Add explicit failure taxonomy and telemetry fields for origin-forward attempts.

### Exit criteria

- Allowed requests reach existing upstream origin.
- Block/challenge/maze/tarpit behavior remains Shuma-owned and unchanged.
- Native/front-door allow-response code path is removed.
- Origin-forward failures are observable and do not silently degrade security.

## Workstream C: Operator Deployment Paths (Linode + Fermyon)

### Implementation slices

1. GW-C1: Add gateway-only deployment docs/runbook for existing-site adoption.
2. GW-C2: Extend Linode one-shot workflow with gateway bootstrap mode (Shuma VM in front of existing origin).
3. GW-C3: Extend Akamai/Fermyon skill and ops docs with explicit gateway cutover checklist:
   - DNS/edge route to Shuma,
   - origin lock verification,
   - origin-auth credential injection point and overlap-safe rotation procedure,
   - staged rollout and rollback.
4. GW-C4: Add Makefile smoke target for gateway mode:
   - origin reachability,
   - enforced blocked path behavior,
   - bypass-check failure test.

### Exit criteria

- Existing-site operators can deploy with minimal origin app changes.
- Cutover and rollback are explicit and testable in operator workflow.

## Workstream D: Site Surface Discovery Onboarding

### Implementation slices

1. GW-D1: Reuse shared-host discovery outputs (`robots.txt`, `sitemap.xml`, bounded crawl) as onboarding input to gateway policy tuning.
2. GW-D2: Define host-surface catalog handoff contract from discovery artifacts to deterministic synthetic probes.
3. GW-D3: Add operator docs for discovery-first onboarding on existing sites (before strict enforcement).

### Exit criteria

- Surface discovery directly supports gateway onboarding and tuning loops.
- Discovery and gateway policy tuning remain one coherent operator workflow.

## Workstream E: Product Positioning and Decommission Cleanup

### Implementation slices

1. GW-E1: Update docs/help text to present gateway deployment as the only production mode.
2. GW-E2: Remove front-door/native deployment guidance from operator-facing documentation.
3. GW-E3: Remove stale Makefile/docs terminology implying dual-mode production support.

### Exit criteria

- Operator docs consistently present one production adoption path (gateway-only).
- No conflicting guidance suggests full-site rewrite into Shuma runtime.

## Verification Matrix

1. Unit:
   - upstream config validation and guardrail checks.
   - header canonicalization/drop-preserve rules.
   - strict TLS and insecure-profile validation (including special-range denials).
2. Integration:
   - allow-path forwarded to upstream origin.
   - enforcement outcomes served locally.
   - origin-down/transport-failure behavior and telemetry.
   - origin 4xx/5xx pass-through behavior.
   - redirect confinement and hop-budget behavior (`redirect_policy_denied` coverage).
   - no local allow-response path remains.
3. Security integration:
   - direct-origin bypass attempt detection.
   - forwarded-header spoof rejection.
   - forwarded-header strip/rebuild invariants at proxy boundary.
   - privileged-header stripping on forwarded requests.
   - stale/invalid origin-auth credential fail-closed behavior.
   - TLS hostname/cert validation fail-closed behavior.
   - loop detection and fail-closed behavior.
   - reserved-route collision preflight failure behavior.
4. E2E:
   - staged cutover test (monitor -> enforce).
   - rollback test to previous route/origin posture.
   - mandatory Make targets: `make test-gateway-profile-shared-server`, `make test-gateway-profile-edge`, `make smoke-gateway-mode`.

## Sequence and Priority

1. 0 (`GW-0.1..0.4`): evidence/harness first.
2. A (`GW-A1..A4`): contract + guardrails.
3. B (`GW-B1..B6`): forwarding implementation + front-door removal.
4. C (`GW-C1..C4`): deployment UX and operator paths.
5. D (`GW-D1..D3`): discovery onboarding integration.
6. E (`GW-E1..E3`): positioning and decommission cleanup.

## Verification Cadence (Targeted During Build, Full at End)

1. Use targeted verification during implementation slices:
   - run the smallest relevant Make targets for each change (`make test-unit`, `make test-integration`, profile-specific gateway tests, or gateway smoke as appropriate to the modified behavior).
2. Prevent redundant full-suite churn:
   - follow the verification receipt rule in `AGENTS.md` and reuse a valid `.spin/last-full-test-pass.json` when `HEAD` and worktree fingerprint are unchanged.
3. Preserve docs-only efficiency:
   - docs-only slices may skip tests with explicit docs-only verification rationale.
4. Enforce full rigor before tranche sign-off:
   - run `make test` and `make build` before final completion;
   - do not mark the tranche complete unless full-suite, profile-specific, and smoke verification gates are green.

## Risks and Mitigations

1. Risk: origin bypass leaves Shuma ineffective.
   - Mitigation: mandatory origin lock runbook + verification checks.
2. Risk: proxy/header normalization bugs create security regressions.
   - Mitigation: strict mapping contract and adversarial integration tests.
3. Risk: mandatory gateway introduces onboarding friction for local/dev setups.
   - Mitigation: provide deterministic local upstream fixture workflow and clear quickstart commands.
4. Risk: operator complexity during cutover.
   - Mitigation: explicit staged rollout path and single-command smoke checks.

## Exit Criteria (Plan-Level)

1. Existing-site deployment is first-class and documented as the only production mode.
2. Gateway mode is secure-by-default with fail-closed guardrails.
3. Front-door/native allow-response mode is removed from runtime/docs/tests before launch.
4. Verification demonstrates consistent policy behavior with upstream forwarding under normal and failure paths.
