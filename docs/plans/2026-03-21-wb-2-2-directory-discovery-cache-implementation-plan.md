# WB-2.2 Directory Discovery Cache Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add bounded external HTTP Message Signatures directory discovery and caching to the native verified-identity path.

**Architecture:** Extend the native resolver in `src/bot_identity/native_http_message_signatures.rs` rather than creating a parallel verifier. Keep the shared verified-identity contract unchanged, store cache records with explicit timestamps in the existing key-value seam, and bound request-path cost with caps on external links, response bytes, and retained cache entries.

**Tech Stack:** Rust, `spin-sdk` outbound HTTP on wasm32, existing `KeyValueStore` seam, `web-bot-auth` JWKS/keyring types, Makefile-focused test targets.

---

### Task 1: Capture the execution boundary and focused test gate

**Files:**
- Modify: `todos/todo.md`
- Modify: `Makefile`
- Test: `src/bot_identity/native_http_message_signatures.rs`

**Step 1: Write the failing tests**

Add focused native-directory tests that prove:
- a signed request with an external `Signature-Agent` can verify after fetching directory material,
- a stale cached directory fails as `directory_stale`,
- a refresh failure falls back to still-fresh cached material,
- cache growth is bounded by deterministic eviction.

**Step 2: Run test to verify it fails**

Run: `make test-verified-identity-directory-discovery`
Expected: FAIL because the native resolver still returns `directory_unavailable` for external directories and the focused make target does not exist yet.

**Step 3: Add the focused make target**

Add `test-verified-identity-directory-discovery` that runs only the directory-discovery/cache coverage in `src/bot_identity/native_http_message_signatures.rs`.

**Step 4: Run test to verify the failing tests are now wired**

Run: `make test-verified-identity-directory-discovery`
Expected: FAIL on the new external-directory behavior assertions, not on missing target wiring.

### Task 2: Implement bounded external directory resolution and cache storage

**Files:**
- Modify: `src/bot_identity/native_http_message_signatures.rs`
- Test: `src/bot_identity/native_http_message_signatures.rs`

**Step 1: Write the next failing test**

Add a test proving the resolver stores fetched external directory material and reuses it without a second fetch while the cache remains within the direct-use window.

**Step 2: Run test to verify it fails**

Run: `make test-verified-identity-directory-discovery`
Expected: FAIL because there is still no external fetch/cache path.

**Step 3: Write minimal implementation**

Implement:
- a bounded external directory fetcher abstraction,
- HTTPS-only external resolution,
- serialized cache records with explicit timestamps,
- bounded site-local cache index and oldest-entry eviction,
- fallback from refresh failure to still-fresh cached material,
- `directory_stale` once cached material ages past the freshness requirement.

Keep the normalization minimal:
- `stable_identity` from the normalized directory URL,
- `operator` from the directory authority,
- `category=other`,
- `end_user_controlled=false`.

**Step 4: Run test to verify it passes**

Run: `make test-verified-identity-directory-discovery`
Expected: PASS with the new directory-resolution behavior.

### Task 3: Re-prove the wider native verifier and operator-facing docs

**Files:**
- Modify: `docs/configuration.md`
- Modify: `docs/research/README.md`
- Test: `src/runtime/request_flow.rs`

**Step 1: Write the failing/coverage-extending doc-and-regression checks**

Extend native tests if needed so failed directory states still preserve native provenance and existing replay/freshness behavior does not regress.

**Step 2: Run test to verify current wider behavior before docs**

Run: `make test-verified-identity-native`
Expected: PASS or reveal any native-verifier regressions introduced by the new resolver.

**Step 3: Write minimal implementation/docs**

Document that external native directory discovery:
- remains authentication-only,
- requires explicitly approved outbound hosts at deployment time,
- and uses the verified-identity directory cache/freshness controls already exposed in config.

**Step 4: Run focused verification**

Run:
- `make test-verified-identity-directory-discovery`
- `make test-verified-identity-native`

Expected: PASS.

### Task 4: Close the tranche honestly

**Files:**
- Modify: `docs/research/2026-03-21-wb-2-2-directory-discovery-cache-readiness-review.md`
- Add: `docs/research/2026-03-21-wb-2-2-directory-discovery-cache-post-implementation-review.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Run tranche verification**

Run:
- `make test-verified-identity-directory-discovery`
- `make test-verified-identity-native`
- `make test-verified-identity-provider`
- `make test-verified-identity-telemetry`
- `make test-verified-identity-annotations`
- `git diff --check`

Expected: PASS.

**Step 2: Perform code and architecture review**

Compare the implementation against:
- the `WB-2.2` acceptance criteria,
- the readiness review,
- the existing verified-identity design/implementation plan,
- and Shuma's outbound/trust-boundary rules.

**Step 3: Write minimal follow-up if review finds a shortfall**

If the review finds a tranche-local gap, record it as a new TODO and execute it immediately before claiming completion.

**Step 4: Record completion**

Move the `WB-2.2` TODO to `todos/completed-todo-history.md`, write the post-implementation review doc, commit, and push.
