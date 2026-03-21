Date: 2026-03-21
Status: Completed

Related context:

- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)
- [`2026-03-21-wb-2-1-native-http-message-signature-readiness-review.md`](2026-03-21-wb-2-1-native-http-message-signature-readiness-review.md)

# Scope reviewed

`WB-2.1` required the first native HTTP Message Signatures verifier path for Shuma's internal runtime/backend, including typed failures, replay/clock-skew enforcement, and explicit `Signature-Agent` binding without yet bundling the bounded remote discovery/cache layer planned for `WB-2.2`.

# What landed

1. `src/bot_identity/native_http_message_signatures.rs` now implements the native verifier core on top of the official Rust `web-bot-auth` crate.
2. The verifier now:
   - detects unsigned `Signature-Agent` claims as failures,
   - validates signed `Signature-Agent` binding,
   - evaluates clock skew and freshness deterministically,
   - and records replay markers through the shared key-value store.
3. `src/providers/contracts.rs`, `src/providers/internal.rs`, `src/providers/external.rs`, `src/providers/registry.rs`, and `src/runtime/request_flow.rs` now carry the request-local store/site context required for replay defense and correct provenance reporting.
4. The internal runtime/backend now runs native verification when enabled; the external backend keeps the existing trusted edge/provider assertion normalizer.
5. The native path currently supports self-contained verification material and returns explicit `directory_unavailable` failures for unresolved external `Signature-Agent` directories until `WB-2.2` lands the bounded discovery/cache layer.
6. `Makefile` now exposes `test-verified-identity-native` as the focused regression gate for this tranche.

# Verification

1. `make test-verified-identity-native`
2. `make test-verified-identity-provider`
3. `make test-verified-identity-telemetry`
4. `make test-verified-identity-annotations`
5. `git diff --check`

# Review against the plan

1. The slice meets the `WB-2.1` acceptance criteria:
   - verification is deterministic,
   - failure reasons are typed and observable,
   - and unsigned `Signature-Agent` style claims do not count as verified identity.
2. The implementation stayed inside the planned boundary:
   - no authorization policy was bundled in,
   - no dashboard control surface was added,
   - and bounded remote directory discovery/cache remains explicitly deferred to `WB-2.2`.
3. The request-path contract remains aligned with the larger architecture:
   - native verification and provider assertions still normalize into the same internal result type,
   - authentication still does not imply authorization,
   - and monitoring can distinguish native versus provider provenance.

# Shortfall found and executed

One tranche-local shortfall was found during the closeout review:

1. replay-state reads and writes in the new native verifier initially failed open on key-value errors, which weakened the promised deterministic replay-window enforcement.

Follow-up task executed immediately:

1. `WB-2.1-REVIEW-1` made replay-state access fail closed by turning malformed or unavailable replay markers and failed replay-marker persistence into explicit `replay_rejected` outcomes.

# Final shortfall status

No remaining tranche-local shortfall was found against `WB-2.1` after `WB-2.1-REVIEW-1` landed.

The next planned work remains:

1. `WB-2.2` bounded directory and key discovery/cache
2. `WB-2.3` proxy and edge trust semantics
3. `WB-3.1` named identity policy registry
