# Live Linode Feedback-Loop Post-Verification Review

Date: 2026-03-22
Status: closed

## Review scope

Review `OVR-LIVE-1` against the readiness contract in:

- `docs/research/2026-03-22-live-linode-feedback-loop-verification-readiness-review.md`
- `docs/plans/2026-03-22-live-linode-feedback-loop-verification-plan.md`

## What the tranche proved

The tranche now proves all required live conditions:

1. the active Linode receipt points at current `HEAD` `12671c6ef8c153c5af79a308f3c7f663d9474911`,
2. the runtime still satisfies the shared-host wrapper contract on the live host,
3. `GET /admin/oversight/agent/status` is live and truthful,
4. the internal shared-host-only periodic trigger executed on-host,
5. a completed adversary-sim run generated traffic and produced a linked post-sim agent run,
6. and the resulting state is durable in `.spin/live_feedback_loop_remote.json` plus the public oversight history/status surfaces.

## Shortfalls found during execution

Two real shortfalls surfaced during live verification and were executed immediately:

1. Remote deploy loopback health was too optimistic for this Linode host.
   - Symptom: `make remote-update` rolled back while Spin was still preparing Wasm modules, even though the new service bound successfully moments later.
   - Fix executed: widened the remote loopback health budget and added focused remote-target contract coverage in commit `e146b55` (`fix: widen remote update loopback health budget`).
   - Evidence: `make test-remote-target-contract`
2. The live-proof tool checked the wrong wrapper surface.
   - Symptom: `make test-live-feedback-loop-remote` failed even though the live service was correctly using `scripts/run_with_oversight_supervisor.sh` below `make prod-start`.
   - Fix executed: changed the verifier to inspect the live service process tree, not only the systemd top-level `ExecStart`, and added focused verifier unit coverage in commit `12671c6` (`fix: align live feedback loop proof with systemd wrapper chain`).
   - Evidence: `make test-live-feedback-loop-remote-unit`

## Remaining shortfall assessment

No tranche-local shortfall remains open for `OVR-LIVE-1`.

The live proof is now strong enough to unblock the next stage:

1. the first shared-host recommend-only feedback loop is no longer only locally proved,
2. Monitoring overhaul can now follow a real machine-first live loop instead of an inferred one,
3. and the active deployment/tooling contract now has focused regression gates for the two live-only issues this tranche exposed.

## Residual note

CI status was not checked from this verification tranche, so repository CI remains unverified in this closeout even though the live Linode proof itself passed.
