Date: 2026-03-25
Status: Active

# `RSI-GAME-HO-1A` Human-Only Private Loop Readiness Review

## Context

`SIM-SCR-FULL-1` is now satisfied, so the next strict-baseline tranche is no longer about attacker capability breadth. It is about making the existing machine-first loop actually run against the intended `human_only_private` reference stance instead of the older mixed default posture matrix.

The user also clarified the product requirement:

1. `human_only_private` must be the first real operating stance for the loop,
2. verified non-human identities must still be denied under that strict baseline,
3. and later relaxed stances must stay blocked until repeated retained improvement is proved first under the strict baseline.

## Findings

1. The first-working-loop proof was still seeding the older balanced default objectives.
   - `src/admin/api.rs` used `crate::test_support::seed_canary_only_objectives(&store)` inside `post_sim_oversight_route_can_apply_improve_and_archive_first_working_game_loop`.
   - That helper seeded `default_operator_objectives(...)`, so the route-level loop proof never actually exercised `human_only_private`.

2. The proof did not assert the active preset or verified-identity mode.
   - The route-level test only proved apply and archive behavior.
   - It did not prove that `operator_snapshot_v1.non_human_stance_presets.active_preset_id` and `effective_non_human_policy.active_preset_id` had resolved to `human_only_private`, nor that verified identities were still denied.

3. The shared-host verifier only checked operator-snapshot schema presence.
   - `scripts/tests/live_feedback_loop_remote.py` fetched `/admin/operator-snapshot` and only recorded `schema_version`.
   - That meant a live or emulated loop could drift back to a looser stance without failing the focused proof.

## Conclusion

`RSI-GAME-HO-1A` should:

1. seed the first-working-loop proof with strict `human_only_private` objectives,
2. assert both snapshot preset resolution and `verified_identities_denied`,
3. make the shared-host feedback-loop verifier fail closed unless the same strict baseline is visible there too,
4. and update the paper trail so `RSI-GAME-HO-1B` becomes the next strict-baseline slice.
