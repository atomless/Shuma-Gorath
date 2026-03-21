Date: 2026-03-21
Status: Completed

Related context:

- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)
- [`2026-03-21-wb-1-1-verified-identity-provider-seam-post-implementation-review.md`](2026-03-21-wb-1-1-verified-identity-provider-seam-post-implementation-review.md)

# Scope reviewed

`WB-1.2` required observe-only telemetry for verified-identity attempts, success-versus-failure classes, freshness outcomes, and native-versus-provider provenance without changing request routing.

# What landed

1. `src/bot_identity/telemetry.rs` now projects canonical `IdentityVerificationResult` values into telemetry records only for attempted verification outcomes and preserves identity metadata needed for monitoring.
2. `src/runtime/request_flow.rs` now records attempted verified-identity observations on the request path through a dedicated monitoring intent without changing allow/deny/challenge behavior.
3. `src/observability/monitoring.rs` now aggregates verified-identity attempts, outcomes, failures, freshness classes, provenance, schemes, and top verified identities into the monitoring summary contract.
4. `src/observability/metrics.rs` now exports matching Prometheus families, and `src/observability/hot_read_documents.rs` bumps the monitoring-summary schema version to reflect the expanded summary payload.
5. `src/admin/api.rs` now proves the `verified_identity` summary contract is present and returns the expected counts and top-identity shape through `/admin/monitoring`.
6. `Makefile` now exposes `test-verified-identity-telemetry` as the focused verification path for this observe-only telemetry tranche.

# Verification

1. `make test-verified-identity-telemetry`
2. `git diff --check`

# Review against the plan

1. The slice matches the `WB-1.2` acceptance criteria:
   - telemetry is observe-only and does not alter routing,
   - monitoring can answer which identities are appearing and how they are verifying,
   - provenance and freshness are explicit so later policy phases can distinguish verified-but-still-restricted traffic from future explicit allow rules.
2. The implementation stayed within the planned boundary:
   - no allow/deny/challenge behavior changed,
   - no verified-identity policy exceptions were introduced,
   - no operator control-surface or dashboard policy UI work was bundled into this slice.
3. The telemetry contract remains reusable for later native Web Bot Auth work because the request-path projection is result-shaped rather than provider-header-shaped.

# Shortfall check

No tranche-local shortfall was found against `WB-1.2`.

The next required work remains:

1. `WB-1.3` request-path identity annotations without routing change
