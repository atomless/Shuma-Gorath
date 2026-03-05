# Gateway-Only Architecture Research Synthesis (Spin/Fermyon + Proxy Best Practice)

Date: 2026-03-05
Status: Active guidance for `DEP-GW-1`

## Objective

Define the highest-quality architecture for Shuma gateway-only mode (`client -> shuma -> origin`) using:

1. Official Spin/Fermyon platform constraints.
2. Established reverse-proxy trust-boundary practice.
3. Current WAF evasion research.

## Pass 1: External Research Findings

### 1. Spin outbound networking is capability-gated and deny-by-default

1. Components cannot make outbound HTTP calls unless `allowed_outbound_hosts` is explicitly configured.
2. Host/scheme/port are enforced; out-of-allowlist requests fail.
3. `allowed_outbound_hosts = []` means no outbound access.

Design implication for Shuma:

1. Gateway mode must fail deployment/runtime validation unless upstream origin is explicitly allowlisted.
2. Wildcard outbound permissions must be prohibited in production posture.
3. Variable-templated outbound host entries cannot be assumed portable to Fermyon Cloud.

Sources:

1. [Spin Manifest Reference](https://spinframework.dev/v3/manifest-reference)
2. [Spin HTTP Outbound](https://spinframework.dev/v3/http-outbound)

### 2. Spin cannot set outbound `Host` header manually

Spin outbound requests cannot set the `Host` header directly (Wasmtime limitation).

Design implication for Shuma:

1. Do not design gateway around manual `Host` rewrites.
2. Upstream contract must treat URI authority as canonical origin host/port.
3. Preserve original authority via `X-Forwarded-Host`/`Forwarded` only for application awareness, not transport identity.

Source:

1. [Spin HTTP Outbound](https://spinframework.dev/v3/http-outbound)

### 3. Spin provides outbound runtime throttles that should be used for gateway safety

Spin runtime config exposes:

1. `outbound_http.connection_pooling`
2. `outbound_http.max_concurrent_requests`

Design implication for Shuma:

1. Treat these as mandatory production controls for upstream pressure governance.
2. Include explicit operator defaults and runbook guidance.

Source:

1. [Spin Runtime Configuration](https://spinframework.dev/v3/dynamic-configuration)

### 4. Spin HTTP trigger has reserved runtime routes

Spin reserves `/.well-known/spin/...` and runtime handles them with priority over app routes.

Design implication for Shuma:

1. Gateway tests and docs must account for reserved route behavior.
2. Do not assume catch-all app route fully controls all inbound paths.

Source:

1. [Spin HTTP Trigger](https://spinframework.dev/v3/http-trigger)

### 5. Fermyon runtime forwards Spin request metadata headers

Fermyon exposes `spin-full-url`, `spin-path-info`, `spin-matched-route`, and related metadata to components.

Design implication for Shuma:

1. Upstream mapping logic should use normalized request components and never trust client-supplied pseudo-equivalents.
2. Diagnostics should include normalized route/path provenance.

Source:

1. [Fermyon HTTP Trigger Reference](https://developer.fermyon.com/wasm-functions/http-trigger-reference)

### 6. Mature proxy guidance requires explicit trusted-proxy config for forwarded IP/proto

Envoy and Caddy both emphasize strict trusted-hop/trusted-proxy handling and right-to-left XFF semantics to avoid spoofing.

Design implication for Shuma:

1. Maintain strict forwarded-header trust gate (`X-Shuma-Forwarded-Secret`).
2. Keep forwarded trust parsing deterministic and fail-closed.
3. Add negative tests for spoofed left-most XFF and mixed-hop chains.

Sources:

1. [Envoy Edge Proxy Best Practices](https://www.envoyproxy.io/docs/envoy/latest/configuration/best_practices/edge)
2. [Envoy XFF Original IP Detection](https://www.envoyproxy.io/docs/envoy/latest/api-v3/extensions/http/original_ip_detection/xff/v3/xff.proto)
3. [Caddy Global Options: trusted proxies](https://caddyserver.com/docs/caddyfile/options)

### 7. Reverse-proxy architecture value collapses if origin is directly reachable

Cloudflare architecture docs reiterate reverse proxy posture where traffic must traverse edge/proxy first.

Design implication for Shuma:

1. Origin lock is mandatory (firewall/IP allowlist/mTLS/tokenized private ingress equivalent).
2. Origin-bypass checks must be release gates, not optional docs.

Source:

1. [Cloudflare CDN Reference Architecture](https://developers.cloudflare.com/reference-architecture/architectures/cdn/)

### 8. WAF parsing discrepancy research makes canonicalization non-optional

WAFFLED (2025) demonstrates broad WAF bypass via parser discrepancies across headers/content-types.

Design implication for Shuma:

1. Gateway must normalize/validate forwarded request shape.
2. Reject ambiguous or malformed upstream-bound requests early.
3. Add adversarial tests for parser differential classes.

Source:

1. [WAFFLED paper (arXiv:2503.10846)](https://arxiv.org/abs/2503.10846)

## Pass 1 Resulting Architecture Principles

1. One policy core, one allow transport adapter (`upstream_proxy`) only.
2. Fail-closed startup/runtime if upstream contract is invalid or outbound capability is unsafe.
3. Strict trust-boundary ownership for forwarded headers and provenance.
4. Explicit canonicalization before forwarding.
5. Bounded upstream pressure using Spin outbound runtime controls.
6. Mandatory origin lock and bypass verification in deployment gates.

## Pass 2: Codebase Impact Audit (Current State)

### A. Primary runtime seam

`src/runtime/request_flow.rs` currently terminates allow outcomes locally with `200 OK` bodies:

1. static-bypass path returns local success.
2. path/IP allowlist branches return local success.
3. end-of-flow default returns local success.

This is the highest-impact seam to convert for gateway mode.

### B. Secondary allow-path seam inside effect-intent system

`src/runtime/effect_intents/response_renderer.rs` has `ResponseIntent::OkBody` local rendering.

`src/runtime/effect_intents/plan_builder.rs` uses that for allow-ish policy outcomes (for example emergency allowlist).

Risk:

1. If unmodified, some allow decisions will still bypass origin forwarding even after request-flow refactor.

### C. Early-route ownership must remain explicit and local

`src/runtime/request_router.rs` owns:

1. `/health`, `/metrics`, `/robots.txt`, `/admin/*`, `/internal/*`
2. challenge/not-a-bot flows
3. dashboard redirect and maze assets

Gateway refactor must not accidentally forward these policy/control-plane routes.

### D. Configuration/guardrail integration surface is centralized and suitable

`src/config/mod.rs` already centralizes env-only validation and runtime guardrails.

Good news:

1. This is the right place to add upstream contract validation and fail-fast posture.

### E. Manifest currently blocks outbound networking

`spin.toml` currently sets `allowed_outbound_hosts = []` for `bot-defence`.

Good news:

1. Secure default is already deny-by-default.
2. Gateway enablement can be made explicit and auditable.

### F. Architecture already enforces provider boundary discipline

`docs/module-boundaries.md` and `src/providers/contracts.rs`/`registry.rs` define how cross-cutting capabilities should be introduced.

Implication:

1. Add upstream transport as a first-class runtime/provider seam, not ad-hoc calls spread across request flow.

## Pass 2 Key Pain Points to Avoid Technical Debt

1. Multiple independent local-allow exit points.
2. Mixing policy decisions with transport mechanics.
3. Missing single, typed allow-to-upstream handoff contract.
4. Missing deterministic failure taxonomy for upstream transport.
5. Risk of accidental permissive fallback under origin failure.
6. Risk of platform mismatch if design assumes mutable outbound `Host` header.

## Recommended Clean Implementation Direction

1. Introduce one typed allow transport intent (for example `ForwardAllow`) and eliminate local `OkBody` allow semantics from request path.
2. Keep policy pipeline authoritative for decisions; keep transport adapter authoritative for forwarding.
3. Preserve local rendering ownership only for enforcement/control-plane paths.
4. Enforce startup/deploy guardrails tying upstream origin to Spin outbound allowlist.
5. Add transport taxonomy (`success`, `timeout`, `transport_error`, `upstream_http_error`, `policy_denied`) with metrics/events.
6. Add adversarial parser/forwarding tests and bypass tests before rollout.
7. Explicitly encode Spin constraints (no manual `Host`, no wildcard outbound in production, bounded outbound concurrency).

## Immediate Backlog Guidance

The `DEP-GW-1` TODO tranche should be rewritten to:

1. Put instrumentation and deterministic test harness first.
2. Make gateway-only semantics explicit (no native/front-door allow-mode fallback).
3. Include Spin/Fermyon platform constraints as first-class acceptance criteria.
4. Sequence guardrails before transport implementation.
5. Treat origin-lock verification as release-blocking.
