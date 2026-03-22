Date: 2026-03-22
Status: Proposed

Related context:

- [`../research/2026-03-22-live-linode-feedback-loop-proof.md`](../research/2026-03-22-live-linode-feedback-loop-proof.md)
- [`../research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md`](../research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md)
- [`../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`../research/2026-03-20-adversary-evolution-loop-role-synthesis.md`](../research/2026-03-20-adversary-evolution-loop-role-synthesis.md)
- [`../research/2026-02-28-sim2-gcr-2-containerized-black-box-capability-orchestration-research.md`](../research/2026-02-28-sim2-gcr-2-containerized-black-box-capability-orchestration-research.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md`](../plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Review the current state of Shuma's feedback loop and freeze the next optimal path from today's live recommend-only Scrapling loop to:

1. a genuinely closed config-tuning loop informed by Scrapling plus LLM-backed adversary evidence,
2. a later LLM-backed diagnosis/config agent over the proven closed loop,
3. and only after that a benchmark-driven LLM code-evolution loop.

# Findings

## 1. Shuma already has a live recommend-only loop, but not a closed loop

The shared-host feedback loop is now real and live-proved:

1. periodic shared-host supervisor execution works,
2. post-sim triggering works,
3. and the bounded recommend-only reconcile contract is durable and inspectable.

But that loop is still recommend-only. It does not yet:

1. apply config changes,
2. watch the result against protected evidence,
3. roll back on degradation,
4. or feed broader LLM diagnosis and code-evolution paths.

So the real architectural gap is no longer "whether a loop exists," but "how to close it safely."

## 2. The first closed loop should end at config tuning, not code changes

The repo's own benchmark and reconcile planning already points in this direction:

1. benchmark comparison exists to support both tuning and later code evolution,
2. but code evolution is intentionally a later loop,
3. and the first autonomous path is supposed to be bounded canary config apply plus rollback.

That is the right shape.

If Shuma jumps straight from today's recommend-only loop to LLM-suggested code changes, it will skip the lower-risk, lower-latency proving ground that the config loop provides.

The first genuinely closed loop should therefore be:

1. adversary evidence,
2. benchmark comparison,
3. bounded config change,
4. watch window,
5. rollback or retain,
6. deterministic replay promotion.

Only after that should broader LLM diagnosis/config scheduling and later code evolution reopen.

## 3. The LLM adversary lane and the LLM diagnosis lane are different roles

The mature adversary-sim roadmap already separates:

1. `frontier_agent` as a later adversary actor,
2. and `diagnosis_and_tuning_agent` as the later controller-side analysis role.

That separation should stay explicit.

The adversary lane exists to generate realistic hostile or high-cost non-human traffic.

The diagnosis lane exists to read bounded machine-first contracts and propose safe changes.

They may later reuse some model infrastructure, but they are not one system and should not be planned as one mega-agent.

## 4. The next LLM adversary step should be bounded category-fulfillment modes, not the full `SIM-LLM-1` actor

The current blocked `SIM-LLM-1` item is a full first-class runtime actor:

1. instruction-driven,
2. containerized,
3. and broad enough to look like a mature standalone subsystem.

That is still too large as the next coding step.

The next useful step is smaller:

1. implement only the minimum LLM-backed browser or request modes needed to fulfill the categories Scrapling cannot cover well,
2. keep those modes behind the existing capability-safe container boundary,
3. and judge them only by category fulfillment and representativeness, not by whether the full later runtime actor already exists.

That turns the near-term LLM work into a direct dependency of `SIM-FULFILL-1` and `SIM-COVER-1`, rather than a separate platform project that could outrun the evidence model.

## 5. The model backend should be pluggable and eval-driven

The repo now needs an architectural answer to "frontier calls or a small local model in a container?"

The cleanest answer is: neither should be baked into the contract.

The contract should be:

1. one containerized LLM adversary boundary,
2. one model-backend abstraction behind that boundary,
3. one representativeness eval surface that judges outputs by category fulfillment and coverage rather than backend identity.

Within that shape, the best near-term default is:

1. frontier-backed execution as the reference backend for the highest-capability browser and agentic categories,
2. because capability and planning quality are the scarcest resource there,
3. while a smaller self-hosted backend remains an optimization candidate later only if evals prove it can fulfill the intended categories with acceptable parity.

That follows the same general engineering rule used elsewhere in the repo:

1. prove the contract with the clearest and most capable backend first,
2. then optimize cost or locality behind a stable interface,
3. and never let backend convenience redefine the benchmark basis.

## 6. Protected evidence still matters more than raw LLM output

Even when the LLM adversary lane exists, raw LLM output should not become tuning truth automatically.

The repo already settled the right boundary:

1. raw frontier or LLM discoveries are advisory,
2. replay-promoted or equivalently confirmed lineage is what becomes protected evidence,
3. and `synthetic_traffic` remains tuning-ineligible.

That means the first LLM adversary step should optimize for:

1. category fulfillment,
2. replayability,
3. and lineage quality,

not for immediate direct authority over tuning.

## 7. The later LLM diagnosis and code-evolution loops should remain blocked, but the bridge to them must be explicit

`OVR-AGENT-2` and `OVR-CODE-1` are still correctly blocked.

But the repo needs a more explicit bridge:

1. close the config loop first,
2. then let a later LLM diagnosis agent operate over the proven machine-first contracts and protected evidence,
3. and only after that let a later benchmark-driven LLM code loop reopen.

Without that bridge, the roadmap leaves too much ambiguity about how today's live Scrapling loop becomes the later LLM-in-the-loop system the product actually wants.

# Decisions

1. The next active priority remains `TRAFFIC-TAX-1`, then `TRAFFIC-TAX-2`.
2. The first genuinely closed loop ends at bounded config tuning and rollback, not code evolution.
3. The LLM adversary lane and the LLM diagnosis lane remain separate roles.
4. The next LLM adversary step should be a bounded category-fulfillment implementation slice, not the full later `SIM-LLM-1` runtime actor.
5. The LLM adversary boundary should be containerized and model-backend-agnostic.
6. Frontier-backed execution should be the initial reference backend for the highest-capability categories; smaller local backends remain optional follow-on candidates only if evals prove category-fulfillment parity and acceptable operational cost.
7. Raw LLM adversary output remains advisory until replay promotion or equivalent deterministic confirmation makes it protected evidence.
8. `OVR-AGENT-2` should be the later LLM-backed diagnosis/config harness over the proven config loop.
9. `OVR-CODE-1` should stay behind the closed config loop and benchmark proof.

# Required Follow-On Work

1. `TRAFFIC-TAX-1` to define the stable operator-facing taxonomy.
2. `TRAFFIC-TAX-2` to define the classifier, confidence, evidence, and abuse-score chain.
3. `SIM-LLM-FIT-1` to implement the minimum bounded LLM-backed browser or request modes needed for category fulfillment behind a pluggable backend contract.
4. `SIM-FULFILL-1` to map categories to Scrapling and LLM-backed modes.
5. `SIM-COVER-1` to prove category coverage and representativeness.
6. `SIM-PROTECTED-1` to admit only protected evidence into closed-loop tuning.
7. `OPS-OBJECTIVES-3` and `OPS-BENCH-3` to make category posture and judgment controller-grade.
8. `OVR-APPLY-1` to close the config loop.
9. `OVR-AGENT-2` to add the later LLM-backed diagnosis/config harness over the proven loop.
10. `OVR-CODE-1` to add the later benchmark-driven code-evolution harness.

# Result

The route from today's system to the target system is now:

1. taxonomy and classifier first,
2. bounded LLM adversary modes second,
3. coverage and protected-evidence gates third,
4. closed config loop fourth,
5. later LLM diagnosis harness fifth,
6. later LLM code loop last.

That keeps Shuma on the shortest path to a genuine closed loop without letting either the LLM adversary lane or the LLM diagnosis/code loops outrun the evidence, safety, and rollback semantics they need.
