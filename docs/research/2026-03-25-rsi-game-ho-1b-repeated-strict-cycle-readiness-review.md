Date: 2026-03-25
Status: Active

# `RSI-GAME-HO-1B` Repeated Strict-Cycle Readiness Review

## Context

`RSI-GAME-HO-1A` is now complete: the first-working loop and the shared-host verifier both fail closed unless the active stance is `human_only_private` and verified non-human traffic remains denied.

That closes the baseline-selection gap, but it does not yet prove the stronger operational claim the user asked for:

1. recommendations become real bounded config changes,
2. later Scrapling runs occur against those changed configs,
3. watch windows keep judging retain or rollback truthfully,
4. and the loop can iterate through multiple cycles rather than one successful demo.

## Findings

1. The current mainline proof still only demonstrates one retained cycle.
   - `src/admin/api.rs::post_sim_oversight_route_can_apply_improve_and_archive_first_working_game_loop` proves one post-sim canary apply and one later retained judgment.
   - That is still the first-working-loop milestone, not repeated-cycle proof.

2. Existing lower-level agent tests already prove both retain and rollback, but only in isolated single-cycle unit tests.
   - `src/admin/oversight_agent.rs` has separate tests for:
     - watch-window-open,
     - rollback on regression,
     - and retain on improvement.
   - None of those prove a later cycle runs against config retained from an earlier cycle.

3. The current snapshot seeding path hardcodes only one candidate family.
   - `src/test_support.rs::seed_candidate_snapshot(...)` always seeds `candidate_action_families = ["fingerprint_signal"]`.
   - That makes the current route-level proof unable to demonstrate a truthful second bounded move after fingerprinting is already enabled.

4. The active mainline bundle still names only the first-working-loop proof.
   - `make test-rsi-game-mainline` is still about the first explicit working loop.
   - `make test-scrapling-game-loop-mainline` bundles that proof, but not yet a repeated-cycle strict-baseline gate.

## Conclusion

`RSI-GAME-HO-1B` should:

1. extend the route-level strict-baseline proof into at least two real cycles,
2. drive a second bounded move from a different legal family after the first retained config change,
3. prove both retained and rolled-back outcomes across repeated strict-baseline cycles,
4. and add a dedicated focused Make target for this repeated-cycle contract rather than overloading the first-working-loop target name.
