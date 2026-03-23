# ADV-DIAG-1 Adversary-Sim Status Truth Post-Implementation Review

Date: 2026-03-23
Status: complete

## References

- Review and design reference: [`2026-03-23-adv-diag-1-adversary-sim-status-truth-review.md`](2026-03-23-adv-diag-1-adversary-sim-status-truth-review.md)
- Execution plan: [`../plans/2026-03-23-adv-diag-1-adversary-sim-status-truth-implementation-plan.md`](../plans/2026-03-23-adv-diag-1-adversary-sim-status-truth-implementation-plan.md)
- Broader sequence plan: [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- Prior live-loop proof: [`2026-03-22-live-linode-feedback-loop-proof.md`](2026-03-22-live-linode-feedback-loop-proof.md)
- Fresh local receipt: [`../../.spin/live_feedback_loop_remote.json`](../../.spin/live_feedback_loop_remote.json)

## Delivered

`ADV-DIAG-1` now makes `/admin/adversary-sim/status` align with immutable shared-host simulation event truth instead of trusting mutable control-state counters alone.

The delivered runtime now:

1. projects recent simulation-tagged run evidence back into a cloned status state for the active or last completed `sim_run_id`,
2. recovers lower-bound generation counters when persisted monitoring evidence proves traffic but mutable counters under-report,
3. recovers lane diagnostics for the selected runtime lane instead of leaving every lane counter at zero,
4. exposes `truth_basis` so operators and tests can distinguish pure control-state truth from persisted-event lower-bound recovery,
5. and surfaces the matching persisted run evidence directly in the status payload.

## Verification Result

Local focused verification passed:

1. `make test-adversary-sim-diagnostics-truth`
2. `make test-adversary-sim-domain-contract`
3. `make test-live-feedback-loop-remote-unit`
4. `make test-oversight-post-sim-trigger`
5. `git diff --check`

Live shared-host verification also passed after deploying `f1149c89bfbf5a22abfb2be6e785bb071b70187e` to the active Linode:

1. `make remote-update`
2. `make test-live-feedback-loop-remote`

The fresh live proof receipt now shows the previously impossible completed-run mismatch is gone:

- completed `generation_truth_basis=persisted_event_lower_bound`
- completed `lane_diagnostics_truth_basis=persisted_event_lower_bound`
- completed `tick_count=1`
- completed `request_count=235`
- completed `persisted_event_run_id=simrun-1774255065-b11a6bc4e7214fdc`

## Review Findings

### 1. Fixed: completed status no longer contradicts persisted event evidence

The primary tranche objective is met. The live receipt now proves that a completed shared-host run can surface lower-bound recovered counters and lane diagnostics instead of the old `tick_count=0`, `request_count=0`, and all-zero lane view while persisted event telemetry proves traffic for the same run.

### 2. No further tranche-local shortfall found

One nuance remains intentionally allowed and is not treated as a shortfall: a newly running sim can still briefly report zero generation counters before either mutable counters or persisted event evidence exist for that run. That is truthful warm-up behavior, not the contradiction this tranche set out to remove.

## Conclusion

`ADV-DIAG-1` is complete.

The first bounded shared-host closed loop is now backed by a status surface that no longer contradicts the immutable telemetry the controller already trusts, so the Monitoring overhaul can proceed without freezing the old adversary-sim diagnostics mismatch into the next operator-facing projection layer.
