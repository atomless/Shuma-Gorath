# WB-2.3 Proxy And Edge Trust Semantics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Freeze and verify Shuma's proxy and edge trust contract for verified-identity headers and HTTPS-derived signature inputs.

**Architecture:** Keep the current trust model and make it explicit. Reuse the existing forwarded-secret gate, edge `spin-full-url` HTTPS detection, and gateway request canonicalization path. Add focused tests and small helper/doc updates rather than building a second verifier or a parallel trust path.

**Tech Stack:** Rust, existing native Web Bot Auth verifier tests, gateway forwarding harness in `src/runtime/upstream_proxy.rs`, Makefile-focused regression targets, Markdown operator docs.

---

### Task 1: Capture the focused trust-regression gate

**Files:**
- Modify: `todos/todo.md`
- Modify: `Makefile`
- Test: `src/bot_identity/native_http_message_signatures.rs`
- Test: `src/runtime/upstream_proxy.rs`

**Step 1: Write the failing tests**

Add coverage that proves:
- a native request signed over `https`-derived components verifies when shared-host forwarded proto trust is established,
- the same request fails when the forwarded proto is present but untrusted,
- edge `spin-full-url=https://...` can satisfy the native HTTPS-derived component path without forwarded-secret headers,
- the gateway forwards `Signature`, `Signature-Input`, and `Signature-Agent` while still stripping `x-shuma-*` assertion headers.

**Step 2: Run test to verify it fails**

Run: `make test-verified-identity-proxy-trust`
Expected: FAIL because the focused target does not exist yet and the new assertions are not wired.

**Step 3: Add the focused make target**

Add `test-verified-identity-proxy-trust` that runs only the native trust-semantics and gateway-forwarding coverage relevant to `WB-2.3`.

**Step 4: Run test to verify the failing tests are wired**

Run: `make test-verified-identity-proxy-trust`
Expected: FAIL on the new trust-contract assertions, not on missing target wiring.

### Task 2: Lock the runtime contract with minimal code changes

**Files:**
- Modify: `src/bot_identity/native_http_message_signatures.rs`
- Modify: `src/runtime/upstream_proxy.rs`
- Modify: `src/runtime/upstream_canonicalization.rs`
- Test: `src/bot_identity/native_http_message_signatures.rs`
- Test: `src/runtime/upstream_proxy.rs`

**Step 1: Write the next failing test**

Extend the gateway test so it proves one mixed request shape:
- client signature headers survive the gateway hop,
- `X-Shuma-Forwarded-Secret` is stripped,
- `X-Shuma-Edge-Verified-Identity-*` headers are stripped,
- and the gateway still rewrites forwarding provenance itself.

**Step 2: Run test to verify it fails**

Run: `make test-verified-identity-proxy-trust`
Expected: FAIL because the mixed pass-through/strip contract is not yet explicit enough in tests or helper code.

**Step 3: Write minimal implementation**

Make the smallest production changes needed to keep the contract obvious and stable:
- reuse existing header canonicalization helpers,
- add any small helper/constants or comments needed so signature-header pass-through and Shuma-header stripping are intentional rather than accidental,
- keep provider assertion trust behind the forwarded-secret gate,
- keep native HTTPS trust tied to trusted forwarded proto or edge `spin-full-url`.

**Step 4: Run test to verify it passes**

Run: `make test-verified-identity-proxy-trust`
Expected: PASS.

### Task 3: Make the trust and mutation rules operator-visible

**Files:**
- Modify: `docs/configuration.md`
- Modify: `docs/security-hardening.md`
- Modify: `docs/research/README.md`

**Step 1: Write the failing/coverage-extending checks**

If doc-adjacent regression tests are needed, extend the focused trust target before editing docs.

**Step 2: Run current focused verification**

Run: `make test-verified-identity-proxy-trust`
Expected: PASS before the doc pass.

**Step 3: Write minimal docs**

Document that:
- trusted edge verified-identity assertions require the forwarded-secret trust gate,
- native signature headers are client pass-through inputs rather than Shuma-owned privileged headers,
- the gateway rewrites host/forwarding provenance and strips all `x-shuma-*` headers,
- edge HTTPS trust can come from `spin-full-url` in the edge deployment profile.

**Step 4: Run focused verification again**

Run: `make test-verified-identity-proxy-trust`
Expected: PASS.

### Task 4: Close the tranche honestly

**Files:**
- Add: `docs/research/2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-post-implementation-review.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Run tranche verification**

Run:
- `make test-verified-identity-proxy-trust`
- `make test-verified-identity-native`
- `make test-verified-identity-provider`
- `make test-gateway-harness`
- `make test-runtime-preflight-unit`
- `git diff --check`

Expected: PASS.

**Step 2: Perform code and architecture review**

Compare the implementation against:
- the `WB-2.3` acceptance criteria,
- the readiness review,
- the verified-identity design/implementation plan,
- and Shuma's existing forwarded-header and gateway trust boundaries.

**Step 3: Write minimal follow-up if review finds a shortfall**

If the review finds a tranche-local gap, record it as a new TODO and execute it immediately before claiming completion.

**Step 4: Record completion**

Move the `WB-2.3` TODO to `todos/completed-todo-history.md`, write the post-implementation review doc, commit, and push.
