# MZ-T2 Live Maze Browser E2E Review

Date: 2026-03-24
Status: Accepted

## Why this tranche exists

`MZ-T1` now proves the live Spin-path maze contract over opaque entry, tokenized traversal, checkpoint acceptance, hidden-link issuance, and persisted fallback events. It still does not prove the browser-managed parts of that contract:

- JavaScript-enabled checkpoint submission and hidden-link worker issuance,
- JavaScript-disabled bounded traversal and deterministic fallback,
- deep-tier micro-PoW under a real browser execution path,
- replay rejection and repeated high-confidence escalation outcomes under real browser/session behavior.

That leaves a meaningful gap between the runtime maze contract and the canonical verification surface.

## Existing truth we should reuse

- [`scripts/tests/maze_live_traversal.py`](../../scripts/tests/maze_live_traversal.py) already owns the correct live-gate shape for:
  - config snapshot and exact restore of only mutated keys,
  - loopback-only admin/health reads,
  - fresh external-looking forwarded IP buckets for public traversal,
  - admin preview discovery of the opaque public maze prefix,
  - persisted-event assertions against `/admin/monitoring`.
- [`scripts/tests/adversarial_browser_driver.mjs`](../../scripts/tests/adversarial_browser_driver.mjs) is already the canonical browser-realistic driver:
  - Chromium launch, request/response lineage, DOM-path evidence, timeout/retry error shaping, and header normalization already live there.
- [`scripts/tests/playwright_runtime.py`](../../scripts/tests/playwright_runtime.py) is already the repo-local Playwright bootstrap contract and should remain the browser-runtime owner.
- [`src/maze/runtime.rs`](../../src/maze/runtime.rs) already exposes the exact maze browser behaviors we need to prove:
  - checkpoint requirement after bounded depth,
  - `POST <maze_path_prefix>checkpoint`,
  - `POST <maze_path_prefix>issue-links`,
  - optional deep-tier micro-PoW via `data-pow-difficulty`,
  - high-confidence escalation from repeated replay/checkpoint/proof violations.

## Review conclusion

The cleanest implementation is:

1. keep the live gate orchestration in Python next to `maze_live_traversal.py`,
2. extend the existing Playwright adversarial driver with dedicated maze browser actions instead of adding a second browser runner,
3. expose the new coverage through focused maze-specific `make` targets.

`MZ-T2` should prove four browser contracts:

1. **JS-enabled traversal**
   - browser follows the opaque public entry,
   - browser-managed checkpoint POST happens,
   - hidden links are issued into the DOM,
   - progression can continue through an issued hidden link,
   - replayed traversal token eventually blocks and is evidenced in recent events.
2. **JS-disabled traversal**
   - public traversal works up to the bounded no-JS allowance,
   - deeper progress without checkpoint falls back deterministically,
   - persisted `maze_checkpoint_missing action=challenge` evidence appears.
3. **Micro-PoW**
   - a browser-visible maze link advertises `data-pow-difficulty`,
   - the browser worker solves or attaches the required nonce,
   - traversal succeeds through the protected link instead of falling back.
4. **Repeated high-confidence escalation**
   - repeated checkpoint-missing browser attempts from the same bucket escalate from challenge to block,
   - persisted `maze_checkpoint_missing action=block` evidence appears.

## Guardrails

- Reuse the existing browser driver and Playwright runtime bootstrap; do not create a second browser harness.
- Keep admin/health reads loopback-only and keep public traversal on forwarded external-looking identities.
- Restore only the exact config keys mutated by the live browser gate.
- Assert both rendered browser outcomes and persisted event evidence where the runtime contract is about action/reason, not only page copy.
