# CDP Detection Carry-Forward Plan

Date: 2026-02-25
Status: Active (carry-forward)
Supersedes: Historical baseline in [`docs/plans/archive/2026-02-13-cdp-detection-excellence-plan.md`](archive/2026-02-13-cdp-detection-excellence-plan.md)

## Scope

This plan captures only the CDP items that remain after the 2026-02-13 plan was partially delivered.

Delivered baseline already includes:
- Probe-family selection and split rollout controls (`cdp_probe_family`, `cdp_probe_rollout_percent`).
- Internal CDP report handling and tiering.
- Akamai edge outcome ingestion path with internal fallback.
- Basic CDP monitoring/admin visibility (`/admin/cdp`, `/admin/cdp/events`, `bot_defence_cdp_detections_total`).

## Remaining Work

1. CDP-2: Bind CDP reports to a signed request context token.
   - Require a signed, short-lived operation context for CDP report acceptance.
   - Reject detached reports that do not map to an active JS verification flow.

2. CDP-3: Add explicit report expiry/replay controls.
   - Enforce single-use operation IDs for CDP report submissions.
   - Expire stale report contexts deterministically and record taxonomy reasons.

3. CDP-4: Add first-class correlation rules with other evidence.
   - Require CDP evidence to be fused with at least one independent signal family (for example fingerprint consistency or sequence/rate evidence) for high-impact outcomes.

4. CDP-5: Require corroboration before direct ban actions.
   - Standalone strong CDP should escalate to a stronger challenge or maze by default.
   - Direct ban remains available only when corroboration policy is satisfied.

5. CDP-6: Complete false-positive review workflow.
   - Extend `/admin/cdp/events` with explicit review outcomes (confirmed attack, likely false positive, needs follow-up) and operator notes metadata.

6. CDP-7: Add per-probe-family effectiveness metrics.
   - Emit low-cardinality probe-family counters for served, triggered, corroborated, and reviewed-false-positive outcomes.

7. CDP-8: Add synthetic evasive regression harness.
   - Add deterministic test scenarios for spoofed reports, suppressed reports, and bypass attempts.

8. CDP-10: Publish probe rotation and rollback runbook.
   - Define promotion/rollback gates for probe families and edge-mode interaction.

## Definition of Done

- CDP report acceptance requires signed context plus replay/expiry enforcement.
- Corroboration gates are enforced before direct CDP-driven bans.
- Probe-family metrics are visible in monitoring and operator docs.
- Evasive regression tests run in canonical `make test` pathways.
- Runbook includes clear rollout, rollback, and false-positive response steps.
