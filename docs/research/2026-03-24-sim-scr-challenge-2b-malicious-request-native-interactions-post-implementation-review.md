Date: 2026-03-24
Status: Completed

Related plan:

- [`../plans/2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-plan.md`](../plans/2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-plan.md)

# SIM-SCR-CHALLENGE-2B Post-Implementation Review

## What Landed

`SIM-SCR-CHALLENGE-2B` now makes the live request-native Scrapling worker meaningfully attacker-faithful for the owned request-native surfaces.

Delivered work:

1. The Rust worker plan now carries the bounded route and surface contract the Python worker is allowed to act on.
2. The Python worker now validates and consumes `surface_targets` and `runtime_paths` before executing any persona behavior.
3. The real request-native personas now mix ordinary success traffic with malicious submits:
   - `bulk_scraper` keeps catalog/detail traversal and adds malicious `not_a_bot` and puzzle submits.
   - `http_agent` keeps method mix and redirect follow-up and adds malicious `not_a_bot`, puzzle, PoW verify, and tarpit progress abuse.
   - `crawler` adds a light public-search challenge-routing probe without widening into submit-abuse surfaces it does not own.
4. The repo now has a focused proof path for that widened request-native behavior:
   - `make test-adversary-sim-scrapling-malicious-request-native`

## What The Proof Now Shows

The repo now proves that:

- the beat payload exposes bounded owned-surface and runtime-path hints to the worker,
- `bulk_scraper` hits `not_a_bot` and puzzle submit surfaces while still succeeding on ordinary public traffic,
- `http_agent` hits `not_a_bot`, puzzle, PoW verify, and tarpit progress abuse surfaces while still succeeding on ordinary request-native traffic,
- and the existing hosted-worker beat/result contract still holds.

Verification used:

- `make test-adversary-sim-scrapling-malicious-request-native`
- `make test-adversary-sim-scrapling-worker`
- `make test-adversary-sim-scrapling-category-fit`
- `git diff --check`

## Assessment Of `SIM-SCR-CHALLENGE-2C`

`2C` should not stay in the active mainline right now.

After `2B`, the current owned-surface matrix is satisfied by request-native Scrapling behavior. The repo does not currently have a proven owned surface that requires browser or stealth Scrapling to stay truthful.

So the correct next step is:

1. `SIM-SCR-CHALLENGE-2D`
2. then `CTRL-SURFACE-1..3`
3. then the judge-side game-contract work

`SIM-SCR-CHALLENGE-2C` should remain blocked and reopen only if:

- `2D` proves a remaining uncovered owned surface,
- or a future owned-surface review explicitly reassigns a surface to browser/stealth Scrapling.

## Remaining Gap

`2B` proves malicious interaction behavior locally and at the worker/beat contract seam, but it does not yet produce final coverage receipts for every owned surface in the canonical observability path.

That remains the exact remit of `SIM-SCR-CHALLENGE-2D`.
