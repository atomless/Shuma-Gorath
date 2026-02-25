# JS Verification Carry-Forward Plan

Date: 2026-02-25
Status: Active (carry-forward)
Supersedes: Historical baseline in [`docs/plans/archive/2026-02-13-js-verification-excellence-plan.md`](archive/2026-02-13-js-verification-excellence-plan.md)

## Scope

This plan captures remaining JS verification hardening work after the 2026-02-13 plan.

Delivered baseline already includes:
- JS verification interstitial and `js_verified` gate.
- Optional PoW path integration.
- Basic flow sequencing/timing primitives for challenge paths.
- Edge integration mode framework and Akamai signal ingestion.

## Remaining Work

1. JS-1: Replace static `js_verified` cookie token model.
   - Move from IP-only HMAC marker to short-lived signed verification token with issue context (`ip_bucket`, `ua_bucket`, issued/expiry, mode metadata).

2. JS-2: Add nonce/replay protection for JS-only verification completion.
   - JS-only path must post a signed verification completion payload (not just set a static cookie client-side).

3. JS-3: Implement progressive JS verification modes by risk tier.
   - Separate low/medium/high verification depth with explicit policy transitions.

4. JS-4: Apply event-order/timing plausibility checks to JS-only completion flow.
   - Reuse shared operation-envelope primitives where possible.

5. JS-5: Separate JS gate rendering from CDP/PoW orchestration.
   - Refactor so JS gate, CDP reporting, and PoW solve/verify are distinct modules with explicit contracts.

6. JS-6: Define explicit no-JS fallback policy for high-risk requests.
   - Keep normal low-risk no-JS users friction-light while ensuring high-risk no-JS traffic follows deterministic escalation.

7. JS-7: Finalize Akamai interaction contract for JS verification outcomes.
   - Document exactly how Akamai additive/authoritative evidence modifies JS verification routing without replacing local policy ownership.

8. JS-8: Add JS verification mode metrics.
   - Add served/pass/fail/replay/latency counters by verification mode.

9. JS-9: Add integration coverage for token expiry/replay/downgrade paths.

10. JS-10: Publish operator runbook for JS verification rollout and rollback.

## Definition of Done

- JS verification completion is server-validated, signed, short-lived, and replay-safe.
- Risk-tiered mode selection and no-JS fallback are explicit and tested.
- Metrics and runbook are in place for safe rollout and incident rollback.
