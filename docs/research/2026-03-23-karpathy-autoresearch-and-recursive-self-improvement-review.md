# Karpathy Autoresearch And Recursive Self-Improvement Review

Date: 2026-03-23
Status: Proposed planning driver

Related context:

- [`2026-03-21-feedback-loop-and-architecture-debt-review.md`](2026-03-21-feedback-loop-and-architecture-debt-review.md)
- [`2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-review.md`](2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-review.md)
- [`2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-review.md`](2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-review.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

Primary external sources inspected:

- [karpathy/autoresearch](https://github.com/karpathy/autoresearch)
- [aiming-lab/AutoResearchClaw](https://github.com/aiming-lab/AutoResearchClaw)
- [wanshuiyin/Auto-claude-code-research-in-sleep](https://github.com/wanshuiyin/Auto-claude-code-research-in-sleep)
- [davebcn87/pi-autoresearch](https://github.com/davebcn87/pi-autoresearch)
- [uditgoenka/autoresearch](https://github.com/uditgoenka/autoresearch)
- [trevin-creator/autoresearch-mlx](https://github.com/trevin-creator/autoresearch-mlx)
- [Techris93/secops-autoresearch](https://github.com/Techris93/secops-autoresearch)

# Purpose

Study what Karpathy's `autoresearch` and its stronger descendants can teach Shuma about building a recursive self-improving system, and separate the genuinely useful methodology from the broader autonomous-research packaging that is less relevant to Shuma's bounded defense loop.

# Executive Summary

Karpathy's `autoresearch` is useful to Shuma not because it proves "AI should rewrite itself", but because it proves a much narrower and more important point:

1. small mutable surface,
2. fixed evaluation harness,
3. one metric,
4. one experiment at a time,
5. mechanical keep-or-revert,
6. git as durable memory,
7. repeat.

That is the methodological core Shuma should learn from.

The downstream projects are useful because they show which extensions seem to survive contact with real usage:

1. domain-agnostic benchmarking and guard commands,
2. persistent experiment logs and resumable state,
3. noise-aware confidence scoring,
4. separate executor and critic roles,
5. bounded cross-run lessons,
6. self-healing around failed experiments and invalid outputs.

The main caution is just as important:

1. the farther descendants move away from a tiny mutable target and a fixed mechanical evaluator, the more they become "large autonomous workflow systems" rather than honest recursive-improvement loops.
2. for Shuma, the first recursive-improvement loop should therefore stay bounded to protected-evidence diagnosis, config changes, watch-window judgment, and rollback.
3. broader LLM diagnosis and later code-evolution should remain downstream phases, exactly as the current roadmap already intends.

# What Karpathy's `autoresearch` Actually Contributes

The upstream repository is deliberately tiny:

1. one fixed prep/eval file,
2. one mutable training file,
3. one human-authored `program.md`,
4. one fixed 5-minute evaluation budget,
5. one metric: `val_bpb`,
6. keep improvement, revert regression.

That matters more than the specific ML workload.

The real contribution is a pattern:

1. **Constrain the mutable surface hard.**
   - Upstream only lets the agent edit `train.py`.
   - This keeps diffs reviewable and makes attribution of wins and losses tractable.

2. **Hold the evaluator fixed.**
   - `prepare.py` and the evaluation function are off-limits.
   - The agent cannot redefine success.

3. **Use a fixed-time budget instead of an open-ended run.**
   - This makes experiments comparable even when architecture or workload shape changes.

4. **Make selection mechanical.**
   - Better metric: keep.
   - Worse or equal metric: revert.

5. **Use version control as the experiment ledger.**
   - The branch advances only when the metric justifies it.

6. **Put the "research organization" outside the target code.**
   - The human evolves `program.md`.
   - The agent evolves `train.py`.

That split is especially important for Shuma. It suggests that later recursive self-improvement should distinguish between:

1. the system under optimization,
2. the evaluator and safety envelope,
3. and the "org code" or policy code that tells the agent how to search.

# What The Stronger Descendants Add

## 1. `pi-autoresearch`: domain-general loop infrastructure

`pi-autoresearch` is the most directly useful descendant for Shuma's future loop design.

It keeps the core upstream pattern but generalizes it into:

1. a domain-agnostic experiment runner,
2. persistent session files,
3. append-only result logs,
4. optional correctness backpressure checks,
5. and a confidence score based on Median Absolute Deviation.

Useful lessons:

1. **Separate infrastructure from domain knowledge.**
   - The extension handles run/log/keep-revert mechanics.
   - The skill carries domain-specific objective, metric, and scope knowledge.
   - That maps well to Shuma's likely future split between controller mechanics and domain-specific benchmark families.

2. **Persist the run context outside model context.**
   - `autoresearch.md` and `autoresearch.jsonl` survive resets and new agent sessions.
   - Shuma already has the beginnings of this pattern in the decision ledger, snapshot contracts, and replay/promotion lineage. The lesson is to keep investing there instead of relying on transient agent context.

3. **Add noise-aware confidence instead of trusting single-run wins.**
   - The confidence score is a strong methodological improvement over raw keep/revert.
   - For Shuma, this is directly relevant to watch-window judgment, especially once `OVR-AGENT-2` or `OVR-CODE-1` begin comparing candidate changes against noisy traffic outcomes.

4. **Use guard commands as correctness backpressure.**
   - `autoresearch.checks.sh` blocks "wins" that break correctness.
   - Shuma's equivalent is stronger: protected evidence, verified-identity no-harm guardrails, config validation, benchmark comparison, and rollback. The lesson is to continue treating these as first-class keep/revert gates, not optional polish.

## 2. `uditgoenka/autoresearch`: generalized autonomous improvement discipline

This project reframes `autoresearch` as a general-purpose improvement engine for any domain with:

1. a goal,
2. a metric,
3. a verify command,
4. and a loop.

Useful lessons:

1. **One focused change per iteration** is worth preserving explicitly.
   - This is already philosophically aligned with Shuma's bounded patch-family approach.
   - It argues against later mixed config-plus-code-plus-policy mega-patches.

2. **Mechanical verification only** remains the right doctrine.
   - This reinforces Shuma's machine-first benchmark contracts and Monitoring-as-accountability direction.

3. **Guard commands are a useful abstraction.**
   - In Shuma's case, the guard stack is not a shell command but a bundle of invariants:
     - verified-identity no-harm,
     - tuning eligibility,
     - protected evidence,
     - watch-window comparison,
     - rollback triggers.

This repo is less interesting for novel implementation details than for confirming that the upstream methodology survives generalization if the loop stays metric-driven and atomic.

## 3. `wanshuiyin/Auto-claude-code-research-in-sleep` (ARIS): cross-model review and methodology portability

ARIS is broader than `autoresearch`, but one part is highly relevant: it pushes hard on the idea that self-review by the same model falls into local minima and that cross-model critique improves outcomes.

Useful lessons:

1. **Separate proposer and critic roles.**
   - For Shuma, this is highly relevant to the later `OVR-AGENT-2` and especially `OVR-CODE-1` phases.
   - A future diagnosis or code-evolution loop should not trust the same model both to propose and to validate its own changes.

2. **Treat methodology as portable and file-based.**
   - ARIS emphasizes workflow over platform.
   - That is a good fit for Shuma's repo preference for explicit contracts, ledgers, and docs over opaque orchestration magic.

3. **Use review loops to kill weak claims, not just generate new ones.**
   - This maps directly onto Shuma's need for rollback, refusal, and "observe longer" outcomes.
   - Recursive improvement is not just about more changes; it is also about disciplined non-adoption.

The caution is that ARIS is much broader than the first recursive-improvement loop Shuma needs. Its workflow richness is useful later, but it is not the right template for the next bounded control tranche.

## 4. `AutoResearchClaw`: self-healing pipelines and lesson extraction

`AutoResearchClaw` takes the autonomous research idea much further into a multi-stage paper-generation and experimentation platform. That makes it less directly usable as a template for Shuma's first loop, but it still contributes a few useful ideas.

Useful lessons:

1. **Self-healing around failed runs is valuable.**
   - The pipeline explicitly diagnoses and repairs failed stages.
   - Shuma should take the bounded version of this idea: better handling of experiment failure, degraded evidence, and retryable controller states.

2. **Cross-run lesson extraction can help, if bounded.**
   - The MetaClaw integration stores structured lessons with time decay.
   - This is a useful direction for Shuma's later controller expansion:
     - retain structured lessons,
     - give them explicit expiry or decay,
     - and never let them silently override benchmark truth.

3. **Anti-fabrication is a first-class concern.**
   - Their explicit anti-fabrication systems mirror a core Shuma concern: the agent must not be able to redefine truth.
   - For Shuma this means independent benchmark, telemetry, and replay evidence must remain authoritative over agent narrative.

The caution is stronger here:

1. `AutoResearchClaw` is a large autonomous workflow system with many moving parts.
2. Shuma should resist importing that complexity into the first recursive-improvement loop.
3. The useful pieces are lesson-memory, failure recovery, and independent evidence checks, not the full 23-stage pipeline shape.

## 5. `trevin-creator/autoresearch-mlx`: local-platform-specific optima

The MLX port preserves the upstream pattern almost exactly, but the reported results emphasize that different hardware found different winning stacks.

That matters for Shuma because it reinforces:

1. **recursive improvement is local to the operating envelope,**
2. **there may not be one globally best configuration,**
3. **the loop should optimize for the site it is protecting, not for an abstract universal benchmark.**

That fits Shuma's current architecture very well:

1. site-owned operator objectives,
2. local traffic evidence,
3. local cost/friction budgets,
4. and a shared-host-first deployment story.

## 6. `secops-autoresearch`: direct security adaptation and the synthetic-data caution

This repo is useful mostly as a cautionary mirror.

It applies the same pattern to detection rules and thresholds, but its default loop optimizes against deterministic synthetic labeled attacks.

That is exactly where Shuma has already chosen a stricter position:

1. the first closed loop must not auto-tune on synthetic traffic alone,
2. protected evidence and representative adversary coverage must gate tuning,
3. and observed traffic plus replay-promoted lineage must stay central.

The lesson is not that security autoresearch is a bad idea. The lesson is:

1. **security loops become misleading very quickly if the benchmark corpus is too synthetic or too self-referential.**

That reinforces Shuma's existing representativeness and protected-evidence discipline rather than weakening it.

# What Shuma Should Learn

## 1. Keep the mutable surface tiny for as long as possible

The first recursive-improvement loop in Shuma should stay narrower than many of the descendants:

1. config changes first,
2. bounded patch families,
3. fixed evaluation and rollback semantics,
4. no broad code rewriting yet.

This validates the current roadmap where:

1. `OVR-APPLY-1` closed the first config loop,
2. `OVR-AGENT-2` remains later,
3. `OVR-CODE-1` remains later still.

## 2. Keep the evaluator more sacred than the mutator

Karpathy's biggest design strength is evaluator immutability.

For Shuma, the equivalent sacred layer is:

1. traffic telemetry,
2. operator snapshot materialization,
3. benchmark families,
4. verified-identity guardrails,
5. representativeness and protected-evidence gates,
6. watch-window comparison.

The later agent must not be allowed to loosen those contracts as part of the same loop it is trying to optimize through.

## 3. Separate "org code" from "target code"

The upstream `program.md` pattern is surprisingly important.

For Shuma, later recursive self-improvement likely needs three layers:

1. **target layer**
   - config first, later bounded code surfaces
2. **truth layer**
   - telemetry, benchmarks, replay evidence, category coverage, guardrails
3. **search-policy layer**
   - prompts, heuristics, lesson memory, proposal ranking, retry strategy

The search-policy layer should be where most of the early "self-improvement" happens, not by immediately letting the model rewrite the detection core.

## 4. Make keep-or-revert selection more noise-aware than upstream

Karpathy's keep/revert rule is elegant, but downstream projects like `pi-autoresearch` improve it by treating noise explicitly.

Shuma should copy that lesson.

The later loop should prefer:

1. candidate vs baseline comparison,
2. bounded watch windows,
3. repeated or confidence-shaped acceptance,
4. and explicit "observe longer" or "insufficient evidence" outcomes.

This is already partly true in Shuma today. The lesson is to deepen it, not relax it.

## 5. Use separate proposer and critic roles for later LLM phases

ARIS is persuasive on one point: self-review by the same model is weak.

If Shuma later introduces:

1. LLM diagnosis/recommendation over the machine-first contracts,
2. or benchmark-driven code-evolution planning,

then it should strongly prefer:

1. a proposer model or phase,
2. a critic model or phase,
3. independent benchmark truth outside both.

That is a cleaner route to recursive improvement than one monolithic "smart agent".

## 6. Treat lesson memory as bounded, explicit, and decayable

The more ambitious descendants reinforce that run-to-run memory is useful, but dangerous when uncontrolled.

For Shuma, future lesson memory should be:

1. structured,
2. scoped to a site or environment,
3. tied to benchmarked outcomes,
4. explicitly expiring or decaying,
5. advisory until re-proved by current evidence.

That would fit later `OVR-AGENT-2` better than freeform long-context recollection.

## 7. Preserve site-local optimization

The MLX and general-domain descendants reinforce a simple point:

1. the best loop does not chase universal truth first,
2. it finds what works under a specific local operating envelope.

For Shuma, this means:

1. per-site operator objectives remain correct,
2. per-site category posture remains correct,
3. site-local traffic and cost truth remain correct,
4. and later fleet-level learning should be advisory, not a silent override.

# What Shuma Should Not Learn

## 1. Do not copy the "never stop" doctrine literally

Karpathy's loop tells the agent to never stop until interrupted.

That is fine for an overnight research toy.
It is not the right operational doctrine for Shuma.

Shuma needs:

1. explicit budgets,
2. bounded apply counts,
3. bounded watch windows,
4. refusal states,
5. and operator-visible accountability.

## 2. Do not let synthetic-only evaluation become tuning truth

The direct security descendants make this risk concrete.

Shuma is right to insist that:

1. synthetic traffic can still be useful as a harness,
2. but it must not become the sole evidence basis for autonomous tuning.

## 3. Do not widen the mutable surface too early

Many descendants broaden quickly into:

1. multi-stage pipelines,
2. many files,
3. many agents,
4. many outputs.

That is interesting, but it weakens attribution and rollback.

Shuma should keep the first recursive-improvement loop small enough that:

1. a regression has a legible cause,
2. a rollback has a bounded blast radius,
3. and Monitoring can explain what happened.

## 4. Do not let the agent narrate success without independent proof

The stronger descendants add self-healing and self-explanation, but the underlying lesson is that the agent narrative is never enough.

For Shuma:

1. telemetry is the map,
2. benchmarks are the judgment contract,
3. Monitoring is the human-readable accountability layer,
4. agent prose remains secondary.

# Recommended Direction For Shuma's Later Recursive Self-Improvement

When `OVR-AGENT-2` and later `OVR-CODE-1` reopen, the clean direction is:

1. keep the current machine-first benchmark and guardrail stack authoritative,
2. add a bounded search-policy layer that learns from prior decision outcomes,
3. use proposer/critic separation for LLM reasoning,
4. keep changes atomic and family-bounded,
5. accept or reject them only through independent benchmark comparison,
6. and keep site-local evidence primary over generic prior lessons.

In other words:

1. Shuma should borrow `autoresearch`'s methodology,
2. not its exact workload,
3. not the maximalist "autonomous everything" posture,
4. and not synthetic-only optimization shortcuts.

# Conclusion

Karpathy's `autoresearch` is best understood as a proof that recursive self-improvement becomes tractable when the loop is narrow, metric-driven, and brutally mechanical.

That is exactly the lesson Shuma should keep.

The descendants show three worthwhile extensions:

1. noise-aware and resumable experimentation,
2. bounded memory of prior lessons,
3. separate execution and critique roles.

They also show the main failure mode:

1. once the loop gets too broad, too self-referential, or too synthetic, it stops being an honest recursive-improvement system and starts becoming an opaque autonomous workflow.

Shuma's current roadmap is therefore directionally right:

1. prove the bounded closed config loop first,
2. keep Monitoring as the accountability surface for that loop,
3. and only then reopen the later LLM diagnosis and code-evolution phases with tighter methodology borrowed from the `autoresearch` lineage.
