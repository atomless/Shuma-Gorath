# RSI-GAME-HO-1 Strict Human-Only Operational Proof Post-Implementation Review

Date: 2026-03-26
Status: Closed

Related context:

- [`../../Makefile`](../../Makefile)
- [`../../scripts/tests/adversary_runtime_toggle_surface_gate.py`](../../scripts/tests/adversary_runtime_toggle_surface_gate.py)
- [`../../scripts/tests/test_adversary_runtime_toggle_surface_gate.py`](../../scripts/tests/test_adversary_runtime_toggle_surface_gate.py)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../docs/testing.md`](../../docs/testing.md)
- [`../../docs/dashboard-tabs/game-loop.md`](../../docs/dashboard-tabs/game-loop.md)
- [`../../docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../../docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Scope Reviewed

This closeout reviewed `RSI-GAME-HO-1`: fully operationally prove the strict `human_only_private` Scrapling-driven game loop before any later relaxed stance or LLM-runtime reopening.

# What Landed

1. Shuma now has a focused strict-stance proof target, `make test-rsi-game-human-only-strict`, that proves:
   - the local live runtime runs the loop under `human_only_private`,
   - verified non-human stays suppressed through `strict_human_only`,
   - suspicious forwarded request, byte, and latency targets are `0.0` for the strict sim-only phase,
   - the local `/sim/public/*` surface is exercised by a real Scrapling run,
   - and a matching `post_adversary_sim` oversight run materializes for that sim run.
2. Shuma now has a focused repeated-proof target, `make test-rsi-game-human-only-proof`, that extends the strict runtime proof with deterministic repeated cycle evidence:
   - ten retained improving cycles,
   - bounded config changes applied across multiple repair families,
   - measured movement from non-zero suspicious leakage to `0.0`,
   - and archive-level homeostasis lineage that remains `improving` rather than reading as one lucky canary.
3. Route-level archive proof now freezes the strict stance context in machine-first history:
   - archived judged rows preserve `evaluation_context.profile_id = human_only_private`,
   - strict verified-identity override lineage remains explicit,
   - and retained strict-baseline episodes stay distinguishable in `oversight_history_v1`.
4. The runtime surface gate now checks the strict loop truth directly instead of only general adversary activity:
   - latest Scrapling coverage must report `human_only_private`,
   - strict zero suspicious-forwarded targets must be present,
   - strict verified-identity override mode must be present,
   - and the latest sim run must be matched by a completed `post_adversary_sim` oversight episode.

# Acceptance Review

`RSI-GAME-HO-1` required:

1. strict `human_only_private` runtime truth,
2. repeated judged config-change cycles rather than one-off plumbing,
3. machine-first lineage for applied changes and retain or rollback outcomes,
4. truthful operator projection of the strict stance without mixed-site default leakage budgets standing in for the strict target,
5. and proof that does not weaken the baseline merely to manufacture a breach signal.

Those criteria are now satisfied:

1. The live local runtime proof shows the active strict profile is `human_only_private`, verified non-human remains fail-closed under `strict_human_only`, and suspicious forwarded request, byte, and latency targets are all `0.0` for the adversary-sim-only phase.
2. The repeated-cycle proof now goes beyond one route-level apply and one rollback:
   - it records ten retained improving cycles,
   - it proves later cycles run against changed config,
   - and it ends at measured zero suspicious leakage rather than only proving loop plumbing.
3. Machine-first archive and status surfaces preserve strict-baseline lineage, retained outcomes, and repeated judged-cycle context instead of collapsing back to one latest recommendation.
4. The operator-facing Game Loop truth was already corrected in `RSI-SCORE-2E`, and this tranche now supplies the missing strict-baseline proof underneath that projection rather than relying on mixed-site defaults or recommendation-only visibility.
5. The proof ring stays on the unchanged strict baseline. It does not weaken `human_only_private` to create a positive-control result, and it keeps later human traversal calibration as a separate follow-on proof ring.

In practical terms, the repo now has a truthful local strict-loop proof on the first canonical surface:

1. live local `/sim/public/*` runtime under the strict stance,
2. post-sim oversight trigger and strict archive lineage,
3. repeated retained bounded config moves,
4. and measured movement toward the strict zero-leakage target.

# Shortfalls Found

This closure does not yet make the later LLM attacker a live recent-run participant.

The next active mainline is therefore:

1. `SIM-LLM-1C3` remaining LLM runtime proof closure,
2. then blocked `RSI-GAME-HO-2` for the second strict-baseline proof under combined Scrapling plus LLM pressure,
3. and only after that the later `humans_plus_verified_only` sweep.

Human traversal calibration also remains explicitly separate:

1. the current tranche proves the strict sim-only loop,
2. not the eventual friction borne by real human visitors under the discovered strict config.

# Verification

- `make test-rsi-game-human-only-strict`
- `make test-rsi-game-human-only-proof`
