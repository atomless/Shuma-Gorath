Date: 2026-03-21
Status: Completed

Related context:

- [`../plans/2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-implementation-plan.md`](../plans/2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-implementation-plan.md)
- [`2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-readiness-review.md`](2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-readiness-review.md)
- [`2026-03-21-wb-2-2-directory-discovery-cache-post-implementation-review.md`](2026-03-21-wb-2-2-directory-discovery-cache-post-implementation-review.md)

# Scope reviewed

`WB-2.3` was the trust-semantics tranche for verified identity. Its job was to make Shuma's gateway and edge handling of signature-relevant headers explicit and regression-protected without changing authorization policy, native/provider verification contracts, or dashboard/operator control surfaces.

# What landed

1. `src/bot_identity/native_http_message_signatures.rs` now has focused regression coverage proving that native HTTP Message Signatures verification:
   - accepts `https`-derived signature components when shared-host forwarded proto trust is established through `SHUMA_FORWARDED_IP_SECRET` plus `X-Shuma-Forwarded-Secret`,
   - rejects the same `https`-derived signature when forwarded proto is present but untrusted,
   - and accepts edge HTTPS context through `spin-full-url=https://...` in the Fermyon edge deployment profile.
2. `src/runtime/upstream_proxy.rs` now has focused gateway coverage proving that the gateway:
   - preserves client `Signature`, `Signature-Input`, and `Signature-Agent` headers,
   - strips `x-shuma-*` trust headers including provider edge verified-identity assertions,
   - and continues to own/regenerate forwarding provenance.
3. `Makefile` now exposes `test-verified-identity-proxy-trust` as the focused regression gate for this trust-boundary contract.
4. Operator/deployer truth is now explicit:
   - `docs/configuration.md` explains the ownership split between client `Signature*` headers and proxy-owned `x-shuma-*` headers, the gateway rewrite behavior, and the difference between shared-host forwarded-proto trust and edge `spin-full-url` HTTPS context.
   - `docs/security-hardening.md` now documents the same ownership model as a deployment hardening rule, including the fact that Shuma strips its own trust headers before forwarding to the upstream origin.
5. The runtime code change itself stayed intentionally minimal:
   - the gateway already implemented the correct strip/pass-through behavior,
   - so the production-path edit was a clarifying comment while the real enforcement gain came from focused tests and documented operator rules.

# Verification

1. `make test-verified-identity-proxy-trust`
2. `make test-verified-identity-native`
3. `make test-verified-identity-provider`
4. `make test-gateway-harness`
5. `make test-runtime-preflight-unit`
6. `git diff --check`

# Review against the plan

1. The tranche meets the `WB-2.3` acceptance criteria:
   - proxy/header mutation risks are now explicit in operator docs,
   - and tests cover trusted forwarding plus header-preservation expectations.
2. The implementation stayed within the agreed scope:
   - no authorization or policy routing changes,
   - no new provider trust path,
   - no dashboard or admin control-surface work.
3. The trust model remains aligned with Shuma's earlier verified-identity work:
   - provider edge assertions still require the forwarded-secret gate,
   - native signature inputs still do not create trust without cryptographic verification,
   - and the gateway still treats `x-shuma-*` headers as proxy-owned internal state rather than upstream-facing request headers.
4. The target naming is truthful:
   - `test-verified-identity-proxy-trust` is a focused contract gate for the proxy/edge semantics added in this tranche, not a broader identity suite replacement.

# Shortfall check

No tranche-local shortfall was found against `WB-2.3`.

# Final shortfall status

No follow-up TODO was required during the `WB-2.3` closeout review.

The next planned work remains:

1. `WB-3.1` named identity policy registry
2. `WB-3.2` downgrade and violation handling
3. later trusted-directory/operator policy surfaces
