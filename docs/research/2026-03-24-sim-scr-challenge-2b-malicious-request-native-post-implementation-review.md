# SIM-SCR-CHALLENGE-2B Post-Implementation Review

Date: 2026-03-24

## What landed

`SIM-SCR-CHALLENGE-2B` now has a truthful first implementation slice for malicious request-native Scrapling behavior:

- the existing `http_agent` persona now performs hostile request-native interaction against public Scrapling-owned challenge surfaces rather than only benign JSON helper routes,
- those interactions include:
  - challenge-routing pressure via public search,
  - hostile `not_a_bot` submit,
  - hostile puzzle submit,
  - hostile PoW verify submit,
- and the worker result plus persisted lane diagnostics now carry `surface_interactions` receipts so Shuma can inspect which owned surfaces the Scrapling lane actually touched.

## Why this is better

Before this tranche, the request-native Scrapling lane still behaved too politely for the owned challenge surfaces. It could generate request-native traffic and fulfill category mapping, but it did not actually use Scrapling the way a malicious request-native attacker would use it against Shuma's public challenge endpoints.

This slice makes the lane materially more attacker-faithful without widening into browser-runtime adoption or privileged internal knowledge:

- it stays inside the existing `http_agent` persona,
- it keeps the current bounded tick budget,
- and it turns the worker-result contract into a receipt-bearing proof surface rather than leaving challenge-touch claims implied by unit-local request logs only.

## Proof

The focused worker gate now proves:

- bounded hosted-worker beat and result exchange,
- hostile public search and challenge-surface requests from the Scrapling worker,
- persisted `surface_interactions` counters in lane diagnostics,
- and preservation of the stale-result fail-closed contract.

Verification:

- `make test-adversary-sim-scrapling-worker`
- `git diff --check`

## Follow-on judgment

This closes the implementation half of `SIM-SCR-CHALLENGE-2B`.

The next active gap is no longer "can the worker submit hostile request-native challenge traffic?" The next gap is coverage closure:

- prove which owned surfaces are now touched end-to-end,
- make any remaining gaps explicit,
- and reopen browser or stealth Scrapling only if receipt-backed closure proves request-native Scrapling is still insufficient for an owned surface.
