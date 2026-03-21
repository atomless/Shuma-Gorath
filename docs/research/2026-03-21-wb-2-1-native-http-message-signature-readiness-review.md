Date: 2026-03-21
Status: Readiness review

Related context:

- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)
- [`2026-03-21-wb-observe-only-tranche-review-and-shortfall-closeout.md`](2026-03-21-wb-observe-only-tranche-review-and-shortfall-closeout.md)

# Purpose

Confirm the smallest clean implementation boundary for `WB-2.1` before touching runtime code.

# Findings

1. The verified-identity observe-only tranche is complete, so the next execution-ready step is the native HTTP Message Signatures verifier path in the internal runtime/backend.
2. The official Cloudflare Rust `web-bot-auth` crate already provides the RFC 9421 parsing and signature-verification core Shuma needs, including `Signature-Agent` parsing and `web-bot-auth` tag filtering.
3. The current Shuma contract split still has one missing runtime seam for native verification:
   - provider-backed verified identity already routes through `VerifiedIdentityProvider`,
   - but the provider contract currently does not receive request-local replay state access.
4. `WB-2.1` does not need to solve bounded remote discovery yet. The implementation plan deliberately reserves directory and key discovery/cache for `WB-2.2`.
5. A clean tranche boundary therefore exists:
   - land deterministic parsing, typed failures, clock-skew evaluation, and replay-window enforcement now,
   - keep real remote directory discovery and cache management for `WB-2.2`,
   - and fail unresolved external `Signature-Agent` directories explicitly as `directory_unavailable` instead of pretending verification succeeded.

# Decision

Implement `WB-2.1` as the internal-runtime native verifier core:

1. extend the verified-identity provider contract to accept request-local store access for replay markers,
2. add a dedicated native HTTP Message Signatures verifier module under `src/bot_identity/`,
3. use the official `web-bot-auth` crate for parsing and cryptographic verification,
4. treat replay defense as a first-class store-backed runtime primitive in this tranche,
5. support self-contained verification inputs now,
6. and return explicit `directory_unavailable` failures for external `Signature-Agent` directory URLs until `WB-2.2` lands the bounded discovery/cache layer.

# Why This Is Safe

1. The slice keeps authentication and authorization separate: native verification still only annotates and observes.
2. It avoids hand-rolled RFC 9421 parsing and signature verification.
3. It does not quietly invent a remote discovery cache ahead of the dedicated bounded-cost tranche.
4. It makes the missing capability honest in telemetry: requests that require remote directory resolution fail explicitly instead of being misclassified as unsigned or trusted.
