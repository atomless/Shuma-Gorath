Date: 2026-03-22
Status: Proposed

Related context:

- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md`](../plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md)
- [`../plans/2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md`](../plans/2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../plans/2026-03-20-benchmark-suite-v1-design.md`](../plans/2026-03-20-benchmark-suite-v1-design.md)
- [`../research/2026-03-20-adversary-evolution-loop-role-synthesis.md`](../research/2026-03-20-adversary-evolution-loop-role-synthesis.md)
- [`../adversarial-operator-guide.md`](../adversarial-operator-guide.md)
- [`../testing.md`](../testing.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Freeze the safety rules for the first truly closed autonomous tuning loop so Shuma does not graduate from recommend-only diagnosis to auto-apply using unrepresentative or category-blind adversary evidence.

# Current Ground Truth

## 1. The control plane still exposes `synthetic_traffic` as a first-class selectable lane

Current lane surfaces explicitly include `synthetic_traffic`, `scrapling_traffic`, and `bot_red_team`:

1. [`../../src/admin/adversary_sim_state.rs`](../../src/admin/adversary_sim_state.rs)
2. [`../api.md`](../api.md)
3. [`../testing.md`](../testing.md)

That is acceptable for harness, contract, and fallback verification, but it is not acceptable as the evidence basis for autonomous tuning.

## 2. The frontier or LLM lane is still discovery input, not a protected tuning oracle

Current operator and testing guidance already say:

1. frontier attempt remains advisory or non-blocking,
2. deterministic replay remains the blocking regression oracle,
3. degraded frontier status does not override deterministic blocking gates.

References:

1. [`../adversarial-operator-guide.md`](../adversarial-operator-guide.md)
2. [`../testing.md`](../testing.md)

This is the correct current safety posture. It means raw frontier or LLM discoveries should not directly drive auto-apply decisions.

## 3. Shuma already distinguishes real runtime evidence from synthetic-only verification

The repo already records that:

1. realtime SIM2 summary marked `synthetic_benchmark` does not prove runtime production behavior,
2. telemetry emission must be runtime-generated,
3. no synthetic monitoring injection should stand in for real adversary signal.

References:

1. [`../sim2-real-adversary-traffic-contract.md`](../sim2-real-adversary-traffic-contract.md)
2. [`../adversarial-operator-guide.md`](../adversarial-operator-guide.md)
3. [`../../scripts/tests/adversarial/sim2_realtime_bench_summary.md`](../../scripts/tests/adversarial/sim2_realtime_bench_summary.md)

# Findings

## 1. Synthetic traffic must be tuning-ineligible

Google's canary guidance is explicit that artificial load can miss important state and organic traffic effects, especially in mutable systems with caches, cookies, affinity, or stateful side effects. It recommends real traffic or representative traffic copy when synthetic load is not representative.

Sources:

1. [Google SRE Workbook: Canarying Releases](https://sre.google/workbook/canarying-releases/)

Implication for Shuma:

1. `synthetic_traffic` is useful for contract sanity and low-cost harness proof,
2. it must not count as protected tuning evidence for autonomous apply,
3. and any benchmark or reconcile path that can later gate auto-apply must make that ineligibility explicit in its contract.

## 2. Closed-loop tuning needs protected baseline-versus-candidate comparison, not one opaque score

Both Google SRE canary guidance and Spinnaker's automated canary analysis model compare new behavior against a protected baseline under comparable traffic and require rollback or abort on significant degradation.

Sources:

1. [Google SRE Workbook: Canarying Releases](https://sre.google/workbook/canarying-releases/)
2. [Continuous Delivery with Spinnaker](https://spinnaker.io/docs/concepts/ebook/ContinuousDeliveryWithSpinnaker.pdf)

Implication for Shuma:

1. the current recommend-only loop is the right intermediate stage,
2. the future closed loop must be `protected evidence -> canary apply -> watch window -> compare -> continue or rollback`,
3. and that comparison must be category-aware rather than a single blended non-human score.

## 3. The operator objective function must encode category intent before autonomous tuning

IBM's autonomic computing architecture is explicit that self-managing systems should optimize against high-level human-defined objectives rather than local heuristics alone.

Source:

1. [IBM Research: Autonomic computing: Architectural approach and prototype](https://research.ibm.com/publications/autonomic-computing-architectural-approach-and-prototype)

Implication for Shuma:

1. the operator must be able to declare which non-human categories are desired, tolerated, restricted, or disallowed,
2. autonomous tuning must optimize against that explicit objective function,
3. and category-blind “reduce bot cost” tuning is unsafe because it can suppress beneficial or allowed automated traffic while chasing one attacker class.

## 4. Shuma needs a representativeness matrix before auto-apply is allowed

The first closed loop should require a bounded coverage proof over the non-human categories Shuma intends to distinguish, at minimum:

1. indexing bots,
2. automated browsers,
3. AI scraper bots,
4. agents acting on behalf of human users.

For each category, Shuma needs evidence about the traffic properties that materially affect defensive behavior, for example:

1. request cadence and concurrency shape,
2. header and navigation behavior,
3. cookie, cache, and session behavior,
4. JavaScript and browser-automation behavior,
5. robots or crawl semantics,
6. authenticated or verified-identity behavior where relevant,
7. replay-promotion or deterministic confirmation status,
8. whether the category is represented by Scrapling runtime traffic, replay-promoted frontier lineage, or both.

Without that matrix, the loop can only say “some adversary traffic improved or worsened,” not “the categories the operator actually cares about remained protected while the target attacker cost increased.”

## 5. Raw frontier or LLM output should remain advisory until promoted into protected evidence

This review does not require Shuma to wait for the full containerized LLM runtime actor before all later tuning work can continue.

The safer intermediate rule is:

1. Scrapling live runtime traffic is protected tuning evidence once coverage proof exists,
2. raw frontier or LLM discoveries remain advisory,
3. frontier or LLM findings become protected tuning evidence only after replay promotion or equivalent deterministic confirmation records the lineage into backend contracts.

That aligns with the current repo direction that deterministic replay is the blocking oracle while emergent lanes feed discovery and promotion.

# Decisions

1. `synthetic_traffic` remains a valid harness and contract lane, but it is tuning-ineligible for any future autonomous apply loop.
2. The first closed loop must use protected evidence from Scrapling runtime traffic and replay-promoted or equivalently confirmed frontier or LLM lineage, not raw synthetic traffic and not raw one-off frontier attempts.
3. Autonomous tuning is blocked until Shuma has a versioned representativeness matrix and coverage receipts for the non-human categories it intends to optimize over.
4. `operator_objectives_v1` must grow category-aware intent so autonomous tuning can distinguish desired, tolerated, and unwanted non-human traffic.
5. `benchmark_results_v1` must grow protected-lane eligibility and category-aware comparison semantics before it can authorize auto-apply.
6. `MON-OVERHAUL-1` should follow the proven closed tuning loop rather than define its semantics in advance.

# Required Follow-On Work

1. `SIM-PROTECTED-1` to codify protected tuning evidence eligibility and explicitly exclude `synthetic_traffic` from autonomous tuning evidence.
2. `SIM-COVER-1` to define the representativeness matrix and bounded coverage receipts across Scrapling and replay-promoted frontier or LLM lineage.
3. `OPS-OBJECTIVES-3` to add category-aware operator objectives and exclusion policy.
4. `OPS-BENCH-3` to make benchmark eligibility and improvement semantics category-aware and protected-lane-aware.
5. `OVR-APPLY-1` to add the later canary apply, watch-window, and rollback loop only after the above gates are delivered.

# Result

The priority remains the closed autonomous tuning loop, but the path is now explicit:

1. do not promote the current recommend-only loop directly into auto-apply,
2. first make the evidence basis protected and category-aware,
3. then add canary apply and rollback,
4. and only after that let Monitoring project the settled truth of the loop.
