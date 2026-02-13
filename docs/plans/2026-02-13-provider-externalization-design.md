# Provider Externalization Design

Date: 2026-02-13
Status: Proposed (direction agreed)

## Context

Shuma-Gorath must remain easy to run on self-hosted minimal infrastructure while offering deeper enterprise integration where external infrastructure has material advantages.

The current provider seams exist in:

- `src/providers/contracts.rs`
- `src/providers/registry.rs`

Backends are config-selectable (`internal`, `external`) but external selections are still placeholder behavior in runtime mapping.

## Goals

- Keep all defence capabilities internally available, toggleable, and tweakable.
- Minimize energy usage by default through low-cost local checks and selective escalation.
- Preserve Shuma as the policy/orchestration brain.
- Support enterprise edge integration with strongest focus on Akamai.
- Keep Fermyon/Spin deployment ergonomics as a primary runtime path.

## Non-goals

- Building a universal cross-vendor adapter that normalizes every provider feature.
- Outsourcing core decision policy (botness scoring, escalation routing, defence mode composition).
- Externalizing maze/tarpit until a stable and meaningful external API target exists.

## Deployment Personas

1. `self_hosted_minimal` (default)
   - All defences run fully internal.
   - No managed-edge dependency required.
   - Lowest configuration overhead and predictable local behavior.
2. `enterprise_akamai` (target integration)
   - Shuma runs as app-aware policy layer.
   - Akamai edge/Bot Manager outputs can be consumed as external signals and optional authorities.

## Externalization Rubric

Offer external provider swaps when at least one is true:

- External system has authoritative vantage Shuma cannot natively observe (edge/TLS/global telemetry).
- External system provides materially better multi-instance/global state consistency.
- Externalization reduces Shuma-side resource consumption without losing explainability.
- Capability has stable enough semantics to define a durable contract.

Keep internal when any is true:

- Capability is core product differentiation.
- Provider feature semantics vary too much for a safe common adapter.
- External dependency would add high operational coupling with marginal security gain.

## Capability Decision Matrix

| Capability | Internal Default | External Option | Priority | Notes |
| --- | --- | --- | --- | --- |
| `fingerprint_signal` | Yes | Yes (Akamai-focused first) | P1 | Edge has stronger transport/bot telemetry; ingest as normalized signals. |
| `rate_limiter` | Yes | Yes (distributed backend path) | P1 | Externalized state can solve multi-instance atomicity/correctness gaps. |
| `ban_store` | Yes | Yes (distributed sync path) | P1 | Externalized sync narrows edge consistency drift. |
| `challenge_engine` | Yes | Partial | P2 | External attestation can help; policy routing remains internal. |
| `maze_tarpit` | Yes | No (for now) | P3 | Keep Shuma-native; no stable cross-provider abstraction target today. |
| policy composition (`botness`, routing, modes) | Yes | No | P0 internal | Core Shuma behavior; do not externalize. |

## Akamai Integration Modes

- `off`:
  ignore external edge outcomes.
- `advisory` (default for enterprise profile):
  edge outcomes become normalized inputs to Shuma policy.
- `authoritative` (optional):
  selected edge outcomes can short-circuit local routes/actions.

Guardrails:

- Safety-critical local controls stay enforceable (admin protections, trusted-origin gates).
- Effective mode and provider/backend selection must be observable in metrics/logs.
- Fallback behavior must degrade to internal logic on external unavailability.

## Sequenced Implementation Plan

1. Document persona + precedence model in runtime/config/deployment docs.
2. Add explicit `External*` stubs with clear unsupported responses for non-targeted capabilities.
3. Implement Akamai-first `fingerprint_signal` adapter path with normalized signal mapping.
4. Implement distributed `rate_limiter` and `ban_store` external backends with outage posture controls.
5. Add provider contract suites asserting semantic parity and explicit unavailability behavior.
6. Add provider/mode observability labels to metrics and botness decision logs.

## Testing and Verification Expectations

- Unit tests:
  provider contract behavior and fallback semantics.
- Integration tests:
  advisory vs authoritative mode precedence and downgrade behavior.
- Make-based verification path:
  `make test` (with mandatory integration and dashboard e2e expectations per repo workflow).

