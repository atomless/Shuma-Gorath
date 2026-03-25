Date: 2026-03-24

Related research:

- [`../research/2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-review.md`](../research/2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-review.md)
- [`2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-plan.md`](2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-plan.md)
- [`2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-plan.md`](2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-plan.md)

# SIM-SCR-CHALLENGE-2D Plan

## Objective

Turn attacker-faithful Scrapling surface coverage into a bounded receipt-backed truth surface, so the repo can prove which owned defense surfaces are satisfied per run and whether any remaining gap really requires browser or stealth Scrapling.

## Tasks

1. Extend the bounded worker-result contract.
   - Add bounded per-surface coverage receipts to `ScraplingWorkerResult`.
   - Keep the receipt shape compact and auditable:
     - `surface_id`
     - `success_contract`
     - `coverage_status`
     - `satisfied`
     - bounded sample request evidence

2. Teach the real worker to emit those receipts.
   - Track surface coverage inside the crawler, bulk-scraper, and http-agent personas.
   - Record the surfaces a request is intended to exercise and the resulting observed outcome.
   - Keep request bodies and other secret-like material out of the receipt surface.

3. Persist the receipts into the normal telemetry path.
   - Persist bounded surface receipts as sim-tagged event evidence when a worker result is accepted.
   - Reuse the existing recent sim run aggregation path instead of inventing a second ad hoc history surface.

4. Aggregate per-run owned-surface closure.
   - Extend recent sim run summaries and operator snapshot recent-run rows with:
     - bounded surface receipts
     - overall owned-surface closure status
     - blocking owned surface ids
   - Compute closure against the owned surfaces required by the fulfillment modes observed in that run.

5. Add focused proof.
   - Add focused worker tests proving the emitted surface receipts.
   - Add focused Rust tests proving recent sim run aggregation and operator snapshot projection.
   - Add a narrow make target for Scrapling surface-coverage receipts and update `docs/testing.md`.

6. Close the paper trail.
   - Update indexes and closeout notes.
   - Move `SIM-SCR-CHALLENGE-2D` into completed history after verification.
   - If the receipts show all current owned surfaces are satisfied, keep `SIM-SCR-CHALLENGE-2C` blocked explicitly.

## Verification

- `make test-adversary-sim-scrapling-coverage-receipts`
- `git diff --check`
