Date: 2026-03-21
Status: Completed

Related context:

- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)
- [`2026-03-21-wb-1-2-verified-identity-observe-only-telemetry-post-implementation-review.md`](2026-03-21-wb-1-2-verified-identity-observe-only-telemetry-post-implementation-review.md)

# Scope reviewed

`WB-1.3` required the normalized verified identity to reach policy evaluation inputs and monitoring context without changing request routing or granting authorization effects.

# What landed

1. `src/runtime/request_facts.rs` now carries optional canonical `VerifiedIdentityEvidence` so later policy phases can match on authenticated identity directly.
2. `src/runtime/policy_pipeline.rs` now threads verified identity into both policy tranches while keeping current policy decisions unchanged.
3. `src/runtime/traffic_classification.rs` now maps canonical verified identities onto exact observed `verified_bot` and `signed_agent` traffic lanes using the existing shared monitoring taxonomy.
4. `src/runtime/request_outcome.rs` and `src/runtime/request_flow.rs` now let verified identity override the monitoring lane for rendered outcomes, so operators can see recognized identities even when those requests are still challenged or otherwise restricted.
5. `Makefile` now exposes `test-verified-identity-annotations` as the focused verification path for this request-path annotation tranche.

# Verification

1. `make test-verified-identity-annotations`
2. `make test-verified-identity-telemetry`
3. `git diff --check`

# Review against the plan

1. The slice matches the `WB-1.3` acceptance criteria:
   - no allow/deny/challenge behavior changed,
   - verified identity is available to later policy phases through `RequestFacts`,
   - monitoring context can now distinguish recognized signed agents and verified bots from generic suspicious or interactive traffic.
2. The implementation stayed within the planned boundary:
   - no identity policy enforcement landed,
   - no provider-only parsing path leaked into policy evaluation,
   - no dashboard or operator control-surface work was bundled into this slice.
3. The request-outcome override keeps authentication and authorization separate: Shuma may still challenge or block a verified identity, but monitoring now records that the restricted request was recognized.

# Shortfall check

No tranche-local shortfall was found against `WB-1.3`.

The next required work is the tranche-level review of the full observe-only verified-identity chain before moving to native verification and policy work.
