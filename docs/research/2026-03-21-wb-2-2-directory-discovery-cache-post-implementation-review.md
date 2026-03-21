Date: 2026-03-21
Status: Completed

Related context:

- [`../plans/2026-03-21-wb-2-2-directory-discovery-cache-implementation-plan.md`](../plans/2026-03-21-wb-2-2-directory-discovery-cache-implementation-plan.md)
- [`2026-03-21-wb-2-2-directory-discovery-cache-readiness-review.md`](2026-03-21-wb-2-2-directory-discovery-cache-readiness-review.md)
- [`2026-03-21-wb-2-1-native-http-message-signature-post-implementation-review.md`](2026-03-21-wb-2-1-native-http-message-signature-post-implementation-review.md)

# Scope reviewed

`WB-2.2` added bounded external directory and key discovery/cache behavior to Shuma's native HTTP Message Signatures verifier without changing authorization policy, dashboard surfaces, or proxy/header trust semantics.

# What landed

1. `src/bot_identity/native_http_message_signatures.rs` now:
   - resolves signed external `https://` `Signature-Agent` directory URLs,
   - fetches and parses bounded JWKS directory material through a native fetcher seam,
   - caches successful directory material in the shared key-value store with explicit timestamps,
   - refreshes cached material after `verified_identity.directory_cache_ttl_seconds`,
   - falls back to still-fresh cached material when refresh fails,
   - and fails as `directory_stale` once cached material exceeds `verified_identity.directory_freshness_requirement_seconds`.
2. External native identities now normalize minimally and deterministically as:
   - `stable_identity` = normalized directory URL,
   - `operator` = normalized directory authority/host,
   - `category` = `other`,
   - `end_user_controlled` = `false`,
   while preserving native provenance and directory-source metadata.
3. Directory cache growth is now bounded by a site-local cache index and oldest-entry eviction, with recovery logic that can rebuild the index from cached records if the index is missing or malformed.
4. `Makefile` now exposes `test-verified-identity-directory-discovery` as the focused regression gate for this tranche.
5. Operator/deployer truth was updated:
   - `docs/configuration.md` now explains the directory cache/freshness knobs and the outbound-allowlist dependency,
   - `spin.toml` now documents that the default outbound posture remains closed and that explicit host approval is required for native external discovery.

# Verification

1. `make test-verified-identity-directory-discovery`
2. `make test-verified-identity-native`
3. `make test-verified-identity-provider`
4. `make test-verified-identity-telemetry`
5. `make test-verified-identity-annotations`
6. `git diff --check`

# Review against the plan

1. The tranche meets the `WB-2.2` acceptance criteria:
   - directory fetch and retained cache size are bounded in Shuma-owned logic,
   - stale and failed discovery outcomes are surfaced explicitly,
   - and authentication still does not imply authorization.
2. The implementation stayed inside the agreed scope:
   - no proxy/header trust changes were bundled in,
   - no local identity policy registry was added,
   - no dashboard or admin control surface was introduced.
3. The request-path architecture remains aligned with the larger verified-identity plan:
   - provider assertions and native verification still normalize into the same verified-identity contract,
   - native failure telemetry still preserves provenance,
   - and the default deployment posture remains outbound-closed unless the operator explicitly allows directory hosts.

# Shortfall found and executed

One tranche-local shortfall was found during closeout review:

1. the first `WB-2.2` implementation bounded cache growth only while the explicit cache index remained intact; if the index went missing or malformed, new writes could orphan old directory records and weaken the promised bounded-cache guarantee.

Follow-up task executed immediately:

1. `WB-2.2-REVIEW-1` now rebuilds the cache index from cached directory records when the index is missing or malformed, evicts oldest entries during rebuild, and deletes newly written records if index persistence fails so unindexed growth does not accumulate.

# Final shortfall status

No remaining tranche-local shortfall was found against `WB-2.2` after `WB-2.2-REVIEW-1` landed.

The next planned work remains:

1. `WB-2.3` proxy and edge trust semantics
2. `WB-3.1` named identity policy registry
3. `WB-3.2` downgrade and violation handling
