Date: 2026-03-21
Status: Completed

Related context:

- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)
- [`2026-03-21-wb-1-3-verified-identity-request-path-annotations-post-implementation-review.md`](2026-03-21-wb-1-3-verified-identity-request-path-annotations-post-implementation-review.md)

# Scope reviewed

Full code and architectural review of the first observe-only verified-identity tranche:

1. `WB-0.1` canonical identity domain
2. `WB-0.2` config placeholders and validation
3. `WB-1.1` provider seam normalization
4. `WB-1.2` observe-only telemetry
5. `WB-1.3` request-path annotations without routing change

# Review against the plan

1. The tranche as delivered preserves the plan's core architecture:
   - authentication and authorization remain separate,
   - verified identity flows through one canonical internal contract,
   - and the provider seam does not create a provider-only policy path.
2. The tranche stayed observe-only as intended:
   - no allow/deny/challenge behavior changed,
   - verified identities can still be challenged or denied,
   - and no low-cost profile, named policy, or dashboard control-surface logic was prematurely bundled in.
3. The implementation now reaches all of the planned pre-policy boundaries:
   - policy inputs can see canonical verified identity through `RequestFacts`,
   - monitoring summaries can answer which identities are showing up and how they verify,
   - and request-outcome lanes can distinguish recognized signed agents and verified bots even when they remain restricted.

# Shortfall found

One tranche-level shortfall was found during review:

1. The verified-identity Prometheus families added in `WB-1.2` were not directly regression-tested at the rendering boundary. The admin monitoring summary and request-outcome lane path were tested, but the operator-facing `/metrics` contract still lacked a focused assertion because the renderer only accepted `spin_sdk::key_value::Store`.

# Follow-up task created and executed

Task: `WB-REVIEW-1` Observe-only tranche metrics boundary coverage

Executed immediately:

1. Added a generic internal `render_metrics_with_store(...)` helper over the shared key-value contract.
2. Kept the public `render_metrics(&Store)` boundary unchanged.
3. Added a focused verified-identity metrics regression test.
4. Reattached that test to `make test-verified-identity-telemetry`.

# Verification

1. `make test-verified-identity-telemetry`
2. `make test-verified-identity-annotations`
3. `git diff --check`

# Final shortfall status

No remaining shortfall was found against the observe-only verified-identity tranche after `WB-REVIEW-1` landed.

The next planned work is:

1. `WB-2.1` native HTTP Message Signature verification
2. `WB-2.2` directory and key discovery/cache
3. `WB-3.1` named identity policy registry
