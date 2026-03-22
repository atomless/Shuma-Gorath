# OVR-APPLY-1 Canary Apply And Rollback Post-Implementation Review

Date: 2026-03-22
Status: complete

## References

- Plan reference: [`../plans/2026-03-22-ovr-apply-1-canary-apply-and-rollback-implementation-plan.md`](../plans/2026-03-22-ovr-apply-1-canary-apply-and-rollback-implementation-plan.md)
- Broader sequence plan: [`../plans/2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md`](../plans/2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md)
- Live proof receipt summary: [`2026-03-22-live-linode-feedback-loop-proof.md`](2026-03-22-live-linode-feedback-loop-proof.md)

## Delivered

`OVR-APPLY-1` now exists as the first bounded shared-host config loop:

1. reconcile can remain manual preview only for direct admin reads,
2. the shared-host agent can progress an eligible recommendation into one bounded canary apply,
3. the controller preserves exact pre-canary config plus baseline comparison state,
4. later cycles judge candidate-versus-baseline improvement over a watch window,
5. and the controller either keeps or rolls back the canary fail-closed when evidence is missing, stale, contradictory, or regressed.

The live shared-host proof now passes against the active Linode deployment at commit `58d07fd07fcb9539fbdeac4fda3f455825f60618`, including:

- periodic run `ovragent-1774210234-dfacee03da991dfe`
- sim run `simrun-1774210234-e94acd912c1ca8ee`
- linked post-sim run `ovragent-1774210415-c9a81c892ab70e13`
- persisted simulated recent-event evidence count `100`

## Review Findings

### 1. Fixed: post-sim trigger originally relied on unstable terminal counters

The first live proof attempt showed a real runtime shortfall: `/admin/events` exposed fresh simulated traffic for the completed `sim_run_id`, but the final adversary-sim control state exposed zero generation counters. The original post-sim trigger therefore missed real completed runs when it depended only on terminal control-state generation fields.

This is now fixed in the delivered runtime:

- state-transition evidence still works when counters are preserved,
- and the controller now falls back to persisted observed simulation events keyed by `sim_run_id` when the control-state counters are absent.

That correction keeps the controller aligned with the repository rule that telemetry is the authoritative map of runtime reality.

### 2. Fixed: live verifier loopback control writes were failing admin trust-boundary checks

The first live verifier attempt also surfaced a tooling shortfall: SSH loopback control writes to `/admin/adversary-sim/control` were missing the public-origin context expected by the admin trust-boundary contract, so the remote proof failed with `403` trust-boundary violations.

The verifier now sends the public `Host` plus `Origin` and `Referer` alongside the forwarded-secret and idempotency contract, and the focused verifier unit tests cover that path.

### 3. Queued follow-up: adversary-sim status counters are still less truthful than event telemetry on shared-host

The passing live receipt still shows a separate diagnostics gap:

- the active or completed sim run can expose `generation.tick_count=0` and `generation.request_count=0`,
- while persisted recent events clearly show live traffic for the same `sim_run_id`.

This no longer blocks `OVR-APPLY-1`, because the controller and live proof now key off persisted observed evidence instead of those unstable counters. But it is still an operator-facing diagnostics truth gap, so it has been queued as follow-up work (`ADV-DIAG-1`) before `MON-OVERHAUL-1`.

## Verification

Commands executed for the final tranche closeout:

1. `make test-oversight-apply`
2. `make test-oversight-post-sim-trigger`
3. `make test-oversight-agent`
4. `make test-live-feedback-loop-remote-unit`
5. `make remote-update`
6. `make test-live-feedback-loop-remote`
7. `git diff --check`

## Conclusion

`OVR-APPLY-1` is complete.

The first bounded shared-host config loop now exists in code, survives live Linode proof, and has no remaining tranche-local controller gap after the two review fixes above. One adjacent diagnostics-truth follow-up is queued separately so Monitoring overhaul does not project stale adversary-sim generation counters as if they were authoritative.
