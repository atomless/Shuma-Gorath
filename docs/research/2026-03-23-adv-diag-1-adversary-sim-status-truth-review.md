# ADV-DIAG-1 Adversary-Sim Status Truth Review

Date: 2026-03-23
Status: approved

## Goal

Close the operator-facing truth gap where `/admin/adversary-sim/status` can report zero generation counters and empty lane diagnostics on shared-host even when immutable simulation-tagged monitoring events already prove that the same run generated live traffic.

## References

- Follow-up TODO: [`../../todos/todo.md`](../../todos/todo.md)
- Prior tranche review: [`2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md`](2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md)
- Live proof receipt: [`2026-03-22-live-linode-feedback-loop-proof.md`](2026-03-22-live-linode-feedback-loop-proof.md)
- Control/status path: [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)
- Status contract: [`../../src/admin/adversary_sim_diagnostics.rs`](../../src/admin/adversary_sim_diagnostics.rs)
- Shared-host supervisor/runtime lane path: [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- Immutable event/run summaries: [`../../src/admin/api.rs`](../../src/admin/api.rs), [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)

## Observed Problem

The March 22 live Linode proof showed a real contradiction after a completed shared-host adversary-sim run:

1. `/admin/events` and the recent sim-run summary proved live simulation-tagged traffic for the completed `sim_run_id`,
2. the oversight controller correctly used that persisted evidence for the post-sim trigger,
3. but `/admin/adversary-sim/status` still exposed `generation.tick_count=0` and `generation.request_count=0`.

That leaves the operator-facing diagnostics surface less truthful than the immutable telemetry the controller now treats as authoritative.

## Root-Cause Analysis

The current status path reads only `ControlState`:

- `adversary_sim_status_payload(...)` builds its payload directly from `state.generated_*`, `state.last_generated_at`, and `state.lane_diagnostics`.
- `generation_diagnostics(...)` uses the same mutable control-state fields.

Meanwhile, the shared-host control plane already has a more authoritative source for completed-run evidence:

- simulation-tagged immutable event records,
- plus the derived recent sim-run summaries rebuilt from those records.

The likely mechanism for the contradiction is a shared-host race/timing window in the mutable control-state path. This is an inference from code and the live receipt:

- internal beats mutate generation counters and lane diagnostics before compare-and-save in [`adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs),
- the beat path can discard that freshly mutated in-memory state on save mismatch and re-expose a reloaded state snapshot,
- while the runtime request events already written during the beat remain durably visible in immutable telemetry.

Even if a narrower mutable-state race is later fixed, the architectural issue remains: status currently trusts mutable run-state more than immutable observed telemetry.

## Decision

`ADV-DIAG-1` should make `/admin/adversary-sim/status` event-truth aware instead of relying only on mutable control-state counters.

The chosen shape is:

1. derive recent run evidence for the active or last completed `sim_run_id` from immutable simulation-tagged monitoring summaries,
2. project that evidence back into the status surface as a lower-bound correction when control-state counters or lane diagnostics under-report,
3. keep control-state values when they are already at least as strong as the persisted evidence,
4. expose the truth basis in the payload so operators and tests can distinguish pure control-state counts from persisted-event lower-bound recovery.

## Constraints

- Do not rewrite the controller or beat ownership model inside this tranche.
- Do not weaken the repo rule that telemetry is the map.
- Do not claim exact beat counts from event summaries when only lower-bound evidence exists.
- Keep the fix shared-host safe and behavior-preserving for controller decisions already using persisted event truth.

## Required Proof

`ADV-DIAG-1` is only complete when all of the following are true:

1. a focused unit test proves status recovery when control-state counters are zero but persisted sim-run evidence exists,
2. the shared-host live proof fails if status still reports impossible zero counters after proved sim traffic,
3. the final review confirms Monitoring overhaul can now treat adversary-sim status as aligned with persisted sim evidence rather than contradictory to it.
