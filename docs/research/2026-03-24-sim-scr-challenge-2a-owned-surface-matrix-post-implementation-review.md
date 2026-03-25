Date: 2026-03-24
Status: Closed

Related context:

- [`2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-review.md`](2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-review.md)
- [`../plans/2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-plan.md`](../plans/2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-plan.md)
- [`../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`](../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md)

# SIM-SCR-CHALLENGE-2A Post-Implementation Review

## Scope Reviewed

This closeout reviewed the first attacker-faithful Scrapling mainline slice:

1. add a canonical machine-readable owned-surface matrix,
2. freeze per-mode surface-target helpers,
3. and expose a focused make gate for the new contract.

## What Landed

1. Added the new canonical contract module in [`src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs).
2. Added the new observability module export in [`src/observability/mod.rs`](../../src/observability/mod.rs).
3. Froze the first explicit Scrapling owned-surface matrix with:
   - owned request-native surfaces,
   - other-lane browser or stealth surfaces,
   - and explicit out-of-scope surfaces.
4. Froze per-mode target helpers for:
   - `crawler`
   - `bulk_scraper`
   - `http_agent`
5. Added the focused contract gate in [`Makefile`](../../Makefile):
   - `make test-adversary-sim-scrapling-owned-surface-contract`
6. Updated [`docs/testing.md`](../testing.md) so the new contract gate is discoverable from the official test guide.

## Review Result

This slice achieved its goal:

1. later Scrapling work no longer has to infer owned defense surfaces from prose or from the deterministic lane,
2. the success semantics are frozen separately from category ownership,
3. and the repo now has an executable contract for deciding what `SIM-SCR-CHALLENGE-2B` must actually implement.

## Residual Follow-On

1. `SIM-SCR-CHALLENGE-2B`
   - the worker still does not perform the newly frozen malicious interactions for `not_a_bot`, puzzle, PoW, or tarpit progress.
2. `SIM-SCR-CHALLENGE-2C`
   - the explicit other-lane surfaces still need later judgment about whether any belong in browser or stealth Scrapling rather than another lane.
3. `SIM-SCR-CHALLENGE-2D`
   - the matrix is now frozen, but receipt-backed coverage against that matrix is not yet landed.

## Verification

- `make test-adversary-sim-scrapling-owned-surface-contract`
- `git diff --check`
