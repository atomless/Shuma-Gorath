Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-plan.md`](../plans/2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-plan.md)
- [`../research/2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-post-implementation-review.md)
- [`../plans/2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-plan.md`](../plans/2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-plan.md)
- [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
- [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../src/observability/operator_snapshot_live_traffic.rs`](../../src/observability/operator_snapshot_live_traffic.rs)

# SIM-SCR-CHALLENGE-2D Review

## Goal

Close the remaining truth gap after `2B`: prove, with bounded machine-readable receipts, which Scrapling-owned defense surfaces were actually exercised and whether those surfaces behaved in the way the owned-surface contract says a real attacker should observe.

## Current Gap

`2B` made the live worker attacker-faithful at the request level, but the repo still lacks a first-class receipt surface for owned defense-surface coverage.

Today Shuma can prove:

1. the worker plan names the owned surfaces and route hints,
2. the Python worker attempts those routes,
3. and recent sim runs expose only coarse summaries such as:
   - observed fulfillment modes,
   - observed category ids,
   - defense delta count,
   - ban outcome count.

That is not enough to answer the exact question `2D` is supposed to close:

- did Scrapling touch every owned surface?
- which owned surfaces were actually satisfied?
- which ones are still missing or underpowered?
- and does any remaining gap really require browser or stealth Scrapling?

## Why the current summaries are insufficient

`defense_delta_count` and generic event-log reason parsing are too coarse.

They can suggest that some defense activity happened, but they cannot truthfully encode:

1. which owned surface a specific malicious request was intended to exercise,
2. whether the observed outcome satisfied the owned success contract,
3. or whether a missing surface is a real gap versus just an unstructured absence of evidence.

If `2D` tried to infer all of that from generic event reasons alone, it would overclaim.

## Cleanest implementation shape

The authoritative receipt should come from the worker result itself.

The worker already knows:

1. which request or probe it is issuing,
2. which owned surface or surfaces that request is meant to exercise,
3. what HTTP outcome it observed,
4. and whether that outcome satisfies the owned success contract for that surface.

So the cleanest shape is:

1. extend the bounded Scrapling worker result contract with per-surface coverage receipts,
2. persist those bounded receipts into the normal sim-tagged telemetry path,
3. aggregate them into recent sim run summaries and operator snapshot surfaces,
4. and compute per-run owned-surface closure from the owned-surface matrix plus the observed fulfillment modes.

## What the receipts should say

Each bounded surface receipt should identify:

1. `surface_id`
2. `success_contract`
3. `coverage_status`
   - for example:
     - `pass_observed`
     - `fail_observed`
     - `transport_error`
4. whether that receipt satisfies the owned success contract
5. bounded sample evidence
   - request method
   - request path
   - last HTTP status when present

This is enough to stay auditable without persisting bodies, privileged context, or unbounded trace detail.

## How closure should be computed

Closure should be calculated against the surfaces required by the fulfillment modes actually observed in the run, not against every Scrapling-owned surface unconditionally.

That means:

1. a crawler-only run should not fail closure on `pow_verify_abuse`,
2. a run that includes `http_agent` should be expected to satisfy the full request-native abuse surface set for that mode,
3. and a multi-tick Scrapling run should merge receipts across its observed modes before deciding whether a surface remains blocking.

## Consequence for `2C`

`2D` is the right decision point for whether `2C` is really needed.

If the new bounded coverage receipts show:

1. all currently owned request-native surfaces are satisfied,
   then `2C` remains blocked;
2. a remaining owned surface is still unsatisfied despite the widened request-native worker,
   then `2C` reopens with a receipt-backed justification.

That is much better than reopening browser or stealth Scrapling on intuition alone.

## Decision

`SIM-SCR-CHALLENGE-2D` should:

1. add bounded per-surface coverage receipts to the Scrapling worker result contract,
2. persist those receipts into Shuma's existing telemetry path,
3. aggregate them into recent sim run and operator snapshot truth,
4. compute per-run owned-surface closure against the owned-surface matrix,
5. and use that closure to decide whether any browser or stealth follow-on is still justified.
