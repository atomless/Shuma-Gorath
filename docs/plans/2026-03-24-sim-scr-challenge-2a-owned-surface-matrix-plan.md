Date: 2026-03-24

Related research:

- [`../research/2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-review.md`](../research/2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-review.md)
- [`2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)

# SIM-SCR-CHALLENGE-2A Plan

## Objective

Add a canonical machine-readable Scrapling owned-surface matrix and success contract so later Scrapling malicious-interaction and receipt-closure tranches have an exact repo contract to build against.

## Tasks

1. Add a new observability contract module.
   - Create a sibling to `non_human_lane_fulfillment`.
   - Freeze schema version, row shape, owned-status counts, per-mode surface targets, interaction requirement, and success contract.

2. Integrate the contract into focused verification.
   - Add a dedicated `make` target for the new owned-surface contract.
   - Add Rust tests for canonical row content and per-mode target lists.

3. Update the paper trail.
   - Add the new review/plan docs to the indexes.
   - Update `docs/testing.md` with the focused contract target.
   - Move `SIM-SCR-CHALLENGE-2A` into completion history when verified.

## Verification

- `make test-adversary-sim-scrapling-owned-surface-contract`
- `git diff --check`
