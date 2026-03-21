Date: 2026-03-21
Status: Ready for execution

Related context:

- [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md)
- [`2026-03-21-verified-identity-execution-readiness-refresh.md`](2026-03-21-verified-identity-execution-readiness-refresh.md)
- [`2026-03-21-wb-2-1-native-http-message-signature-post-implementation-review.md`](2026-03-21-wb-2-1-native-http-message-signature-post-implementation-review.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)

# Scope reviewed

`WB-2.2` is the next native verified-identity tranche after `WB-2.1`. Its job is to turn the current inline-only resolver into a bounded external directory and key discovery/cache layer without crossing into proxy trust semantics, authorization policy, or dashboard/operator control work.

# Findings

1. The current native verifier in `src/bot_identity/native_http_message_signatures.rs` already cleanly separates parsing, freshness, replay, and directory resolution, so `WB-2.2` can extend the resolver layer without changing the verified-identity contract or request-path enforcement semantics.
2. The shared request-path storage seam in `src/challenge/mod.rs` intentionally has no TTL primitive. That means directory caching should follow Shuma's existing pattern of storing serialized state with explicit timestamps and enforcing expiry on read, rather than widening the cross-cutting storage interface for this tranche.
3. The config placeholders needed for `WB-2.2` already exist:
   - `verified_identity.directory_cache_ttl_seconds`
   - `verified_identity.directory_freshness_requirement_seconds`
   Their default relationship (`3600` cache TTL, `86400` freshness requirement) implies a two-threshold model:
   - refresh cached material after the cache TTL,
   - but continue using still-fresh cached material if a refresh attempt fails and the material has not aged past the freshness requirement.
4. Spin outbound HTTP is deny-by-default and this repository currently keeps `spin.toml` on `allowed_outbound_hosts = []` for the bot-defence component. `WB-2.2` therefore must not broaden the repo to wildcard outbound access. External directory fetches should succeed only when the deployment explicitly allows those hosts, and transport or capability failures should surface as explicit verification failures rather than silent success.
5. The current native identity contract has no richer external directory metadata source for operator name, category, or end-user control flags. For `WB-2.2`, the clean minimal normalization is:
   - `stable_identity` = normalized external directory URL,
   - `operator` = normalized directory authority/host,
   - `category` = `other`,
   - `end_user_controlled` = `false`,
   until later policy/metadata phases add stronger operator-owned mapping.
6. Cache size must be bounded, not merely time-limited. A TTL alone does not stop an attacker from filling the keyspace with many unique external directory URLs. `WB-2.2` therefore needs a bounded per-site cache index and deterministic eviction of the oldest cached entries.
7. Request-path fetch cost must also be bounded. The resolver should cap:
   - how many external `Signature-Agent` links it will try per request,
   - the maximum response body size accepted for a directory document,
   - and the number of cache entries retained per site.
8. This tranche should stay strictly inside authentication and discovery. The following remain out of scope:
   - proxy/header preservation and gateway semantics (`WB-2.3`),
   - named identity authorization policy (`WB-3.*`),
   - trusted-directory operator UI or admin control surfaces (`WB-5.*`).

# Decisions

1. `WB-2.2` will resolve only signed external `Signature-Agent` URLs over `https://`. Insecure `http://` links and any unsupported or failed outbound transport will be treated as explicit `directory_unavailable` verification failures.
2. Directory cache semantics will be:
   - cached material younger than both the cache TTL and the freshness requirement is used directly,
   - cached material older than the cache TTL triggers a refresh attempt,
   - if refresh fails but cached material is still within the freshness requirement, Shuma keeps using the cached material,
   - once cached material exceeds the freshness requirement, verification fails as `directory_stale`.
3. Cache records will be stored in site-scoped key-value entries with explicit timestamps and a bounded site-local index so the resolver can evict the oldest cached entries deterministically.
4. Malformed cache records will be treated as unusable local state: delete them, attempt a fresh fetch, and fail closed if a fresh fetch cannot produce valid directory material.
5. `WB-2.2` will add a focused Makefile gate for native directory discovery/cache behavior rather than relying only on the broader native verifier target.

# Why this is the right next slice

1. It closes the exact tranche-local gap intentionally left open by `WB-2.1`.
2. It keeps authentication deterministic and observable without granting any new authorization effect.
3. It respects Shuma's existing secure outbound posture instead of smuggling in wildcard egress under the banner of discovery.
4. It sets up `WB-2.3` cleanly by making directory resolution explicit before tackling proxy and header-preservation semantics.
