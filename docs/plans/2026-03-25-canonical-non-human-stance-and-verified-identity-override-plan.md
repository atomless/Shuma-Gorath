Date: 2026-03-25
Status: Proposed

Related context:

- [`../research/2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md`](../research/2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md)
- [`../research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](../research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md)
- [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../src/bot_identity/policy.rs`](../../src/bot_identity/policy.rs)
- [`../../src/runtime/policy_graph.rs`](../../src/runtime/policy_graph.rs)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Replace the current dual-stance model with one canonical non-human stance system, make verified identity an evidence and explicit-override layer inside that system, and align Game Loop methodology with a strict `human_only_private` baseline before later relaxed verified-identity sweeps.

# Core Decisions

1. Shuma should have exactly one canonical non-human stance authority over the full canonical taxonomy.
2. Verified identity should not own an independent top-level non-human stance.
3. Verified identity should remain useful as:
   1. authenticated evidence,
   2. named local exception matching,
   3. optional service-profile selection,
   4. and telemetry or calibration truth.
4. `human_only_private` should deny or equivalently suppress verified non-human traffic during the strict development baseline.
5. `humans_plus_verified_only` should be the first later relaxed verified-identity sweep candidate after the strict baseline proves useful under both Scrapling and later LLM attacker pressure.
6. Runtime, benchmark, Game Loop, and Tuning should all consume one resolved effective policy contract rather than parallel interpretations.
7. Because Shuma is pre-launch, this should be implemented as a clean architectural correction rather than a compatibility layer.

# Execution Shape

## `STANCE-MODEL-1`: Canonical non-human stance and verified-identity override redesign

This tranche should land before later recursive-improvement methodology execution and before any broader permissive verified-identity product stance work.

It should produce:

1. one canonical stance model over the full taxonomy,
2. one resolved effective policy projection,
3. and one explicit verified-identity override contract that no longer competes with the canonical stance.

## `STANCE-MODEL-1A`: Define canonical stance presets and the resolved effective policy contract

**Likely files:**

1. `src/observability/operator_snapshot_objectives.rs`
2. `src/runtime/non_human_taxonomy.rs`
3. `src/admin/api.rs`
4. `docs/configuration.md`

**Work:**

1. Define canonical stance presets over the full taxonomy, with at least:
   1. `human_only_private`
   2. `humans_plus_verified_only`
2. Keep per-category postures as the persisted source of truth or the fully derived stance output, but not both as competing policy models.
3. Add a machine-first `effective_non_human_policy_v1` style projection that records:
   1. base category posture,
   2. verified-identity override or named exception when present,
   3. effective posture,
   4. and source-of-authority lineage.
4. Make clear that `human_only_private` denies verified non-human traffic too.

**Acceptance:**

1. Shuma has one authoritative non-human posture model.
2. Strict and relaxed stance presets are explicit and machine-readable.
3. The strict baseline no longer hides verified-identity permissiveness.

## `STANCE-MODEL-1B`: Rebase verified identity onto evidence and explicit-override semantics

**Likely files:**

1. `src/bot_identity/policy.rs`
2. `src/config/mod.rs`
3. `src/runtime/policy_graph.rs`
4. `docs/configuration.md`

**Work:**

1. Remove the independent `verified_identity.non_human_traffic_stance` authority from the cleaned model.
2. Preserve:
   1. `verified_identity.enabled`
   2. `native_web_bot_auth_enabled`
   3. `provider_assertions_enabled`
   4. named identity policies
   5. service profiles
3. Make verified-identity named policies and optional category-specific allowances operate as explicit overrides within the canonical stance model instead of replacing it.
4. Preserve the current rule that authenticated identity never implies automatic allow.

**Acceptance:**

1. Verified identity stops acting like a second policy regime.
2. Runtime authorization becomes easier to reason about.
3. Web Bot Auth and trusted provider assertions still work as verification paths, but no longer carry a competing top-level stance.

## `STANCE-MODEL-1C`: Align benchmark, Game Loop, and Tuning to the resolved policy contract

**Likely files:**

1. `src/observability/benchmark_non_human_categories.rs`
2. `src/observability/benchmark_beneficial_non_human.rs`
3. `src/observability/operator_snapshot_verified_identity.rs`
4. `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`
5. `dashboard/src/lib/components/dashboard/TuningTab.svelte`

**Work:**

1. Make benchmark families and Game Loop rows score against the resolved effective policy rather than the old split model.
2. Show when verified identity is being denied by the current stance versus when an explicit verified override applies.
3. Keep operator-facing product stance editing separate from later development reference stance execution.
4. Ensure Game Loop’s first reference stance is `human_only_private`, with later relaxed sweeps including `humans_plus_verified_only`.

**Acceptance:**

1. Game Loop output is interpretable against the policy runtime actually enforces.
2. Benchmark rows no longer overstate disagreement created purely by policy-model mismatch.
3. Tuning continues to edit operator-facing posture without inventing a second stance system.

## `STANCE-MODEL-1D`: Sequence the later methodology around the corrected stance model

**Likely files:**

1. `docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`
2. `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`
3. `todos/blocked-todo.md`

**Work:**

1. Make later recursive-improvement methodology explicitly consume the corrected stance model.
2. Keep `human_only_private` as the first development reference stance.
3. Add `humans_plus_verified_only` as the first relaxed verified-identity sweep candidate only after the strict baseline and the later combined Scrapling-plus-LLM proof.
4. Prevent later controller or code-evolution work from scoring against the old dual-stance semantics.

**Acceptance:**

1. Later recursive-improvement execution is pinned to the corrected policy model.
2. Strict-baseline and later verified-identity sweeps are no longer semantically muddled.

# Backlog Integration

1. Add `STANCE-MODEL-1` as the active policy-model redesign tranche.
2. Make `RSI-METH-1` consume `STANCE-MODEL-1` before later run-to-homeostasis execution.
3. Update the verified-identity planning chain so future work no longer assumes an independent verified-identity stance system.
4. Keep the operator-facing Tuning presets distinct from later development reference stance execution, even when they share labels.

# Definition Of Done

This tranche is satisfied when:

1. Shuma has one canonical non-human stance model,
2. verified identity is evidence plus explicit override rather than a competing stance source,
3. `human_only_private` denies verified non-human traffic during the strict baseline,
4. `humans_plus_verified_only` is defined as a later relaxed verified-identity stance,
5. and runtime, benchmark, Game Loop, and Tuning all consume the same resolved effective policy contract.
