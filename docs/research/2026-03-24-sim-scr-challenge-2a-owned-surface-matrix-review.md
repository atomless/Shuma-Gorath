Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`](../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)

# SIM-SCR-CHALLENGE-2A Review

## Goal

Freeze a machine-readable owned-surface matrix for the Scrapling lane so the later malicious interaction work has an exact contract for:

1. which defense surfaces Scrapling owns,
2. which transport class those surfaces require,
3. whether Scrapling must touch them,
4. and whether a malicious Scrapling-powered attacker should be able to pass them, should fail them, or should expect mixed outcomes.

## Why The Existing Category Contract Is Not Enough

[`non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs) already freezes category ownership well:

- `crawler` -> `indexing_bot`
- `bulk_scraper` -> `ai_scraper_bot`
- `http_agent` -> `http_agent`

But category ownership does not answer the attacker-faithfulness question the user raised.

A request-native Scrapling lane can still be expected to hit multiple Shuma defense surfaces while representing those categories:

- rate pressure,
- geo or IP policy,
- challenge routing,
- `not_a_bot`,
- puzzle escalation or submission,
- PoW abuse,
- and, if it really owns the full request-native challenge-abuse path, tarpit progress abuse.

Without a separate surface matrix, later work can drift into either:

1. underpowered Scrapling that claims coverage without exercising those surfaces, or
2. uncontrolled scope creep where Scrapling starts owning browser-only or self-reporting surfaces it should not.

## Recommended Contract Shape

Freeze the owned-surface truth in code, not only prose.

The cleanest pattern is a sibling to [`non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs):

- canonical schema summary,
- one row per relevant defense surface,
- exact mode assignments,
- exact interaction requirement,
- exact success expectation,
- explicit notes for surfaces assigned elsewhere.

That gives later tranches a stable contract they can consume without rereading planning docs.

## Recommended Surface Taxonomy

The matrix should distinguish at least:

### Owned request-native surfaces

- `public_path_traversal`
- `challenge_routing`
- `rate_pressure`
- `geo_ip_policy`
- `not_a_bot_submit`
- `puzzle_submit_or_escalation`
- `pow_verify_abuse`
- `tarpit_progress_abuse`

### Explicit non-owned or other-lane surfaces

- `maze_navigation`
- `js_verification_execution`
- `browser_automation_detection`
- `cdp_report_ingestion`
- `verified_identity_attestation`

## Recommended Success Semantics

Use two separate fields rather than one overloaded label:

1. `interaction_requirement`
   - `must_touch`
   - `must_not_touch`

2. `success_contract`
   - `should_pass_some`
   - `mixed_outcomes`
   - `should_fail`
   - `outside_scrapling_scope`

This keeps the matrix truthful for surfaces like rate or challenge routing where a malicious attacker should encounter the surface but not deterministically always pass or always fail it.

## Decision

`SIM-SCR-CHALLENGE-2A` should land as a machine-readable repo contract plus focused verification, not as prose only.
