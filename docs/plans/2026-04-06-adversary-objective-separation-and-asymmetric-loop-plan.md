Date: 2026-04-06
Status: Proposed

Related context:

- [`../research/2026-03-20-adversary-evolution-loop-role-synthesis.md`](../research/2026-03-20-adversary-evolution-loop-role-synthesis.md)
- [`../research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](../research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
- [`../research/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](../research/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md)
- [`../research/2026-03-24-rsi-roles-1-triadic-role-contract-post-implementation-review.md`](../research/2026-03-24-rsi-roles-1-triadic-role-contract-post-implementation-review.md)
- [`../research/2026-03-25-scrapling-full-attacker-capability-principle-review.md`](../research/2026-03-25-scrapling-full-attacker-capability-principle-review.md)
- [`../research/2026-03-27-human-friction-calibration-ring-review.md`](../research/2026-03-27-human-friction-calibration-ring-review.md)
- [`../research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](../research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`2026-03-24-llm-player-role-decomposition-plan.md`](2026-03-24-llm-player-role-decomposition-plan.md)
- [`2026-03-27-human-friction-calibration-ring-plan.md`](2026-03-27-human-friction-calibration-ring-plan.md)

# Objective

Freeze the methodological answer to a now-central Shuma design question:

1. should the red side be optimized against a defender-authored "malevolence" target,
2. or should it optimize against attacker utility under attacker constraints,
3. while the judge, defender loop, and human-friction calibration remain separate concerns?

This note settles that question before further attacker-runtime, judge, diagnoser, or human-calibration work drifts toward a less stable dual-loop architecture.

# Core Decisions

## 1. The attacker must not optimize against Shuma-authored "malevolence"

The red side must not be trained to satisfy Shuma's own idea of what maliciousness looks like.

That would make the defender the author of the adversary's objective and would invite Goodhart behavior:

1. the attacker would learn to look malicious according to Shuma,
2. instead of becoming effective at attacker goals that matter in the wild.

The correct rule is:

1. Shuma may judge whether an attack is admissible, realistic, and important,
2. but Shuma must not define the red side's fitness as "be more malevolent according to Shuma."

## 2. The attacker should optimize attacker utility under persona-specific goals

The red side should optimize against externally legible attacker objectives under bounded scope, budget, and safety contracts.

Examples:

1. scraper persona
   - maximize content extracted,
   - maximize public-surface reach,
   - maximize persistence before suppression,
   - while minimizing attacker cost and visibility where relevant;
2. stealth or low-and-slow persona
   - maximize sustained extraction or persistence while minimizing detectable friction triggers;
3. browser or agentic persona
   - maximize task completion through challenge, maze, verification, and other defended surfaces;
4. disruptive persona
   - maximize origin cost, bypass depth, downstream burden, or repeated expensive work.

There should not be one universal attacker utility for every lane.
Shuma should prefer a small portfolio of persona-specific attacker utilities over one collapsed global reward.

## 3. Realism envelopes belong to the judge, not to the attacker fitness function

Realism envelopes remain necessary, but their role is narrower and more disciplined than "teach the attacker to be more evil."

They should answer:

1. whether the emitted behavior is attacker-faithful,
2. whether it stayed within the allowed hostile persona model,
3. whether it used only admissible knowledge and tools,
4. and whether the resulting episode is strong enough for promotion, replay, or defender tuning.

This means:

1. realism envelopes are admissibility and judge contracts,
2. not the red side's optimization target,
3. and not a substitute for attacker utility.

## 4. Shuma should remain triadic, not dual-agent and not self-judging

The repo's attacker / defender / judge split remains the correct architecture:

1. attacker
   - searches for high-utility attacks under black-box attacker constraints;
2. defender
   - proposes bounded config changes under fixed legal moves and fixed objectives;
3. judge
   - remains machine-first, independent, and sacred.

Monitoring or Game Loop remains only the human-readable projection of the judge.

Neither player may own or redefine:

1. the scorecard,
2. realism admissibility,
3. held-out evaluation boundaries,
4. or the later promotion rules.

## 5. The red and blue loops must be asymmetric in cadence and authority

Shuma should not run two equally empowered continuously co-evolving loops at the same cadence.

That architecture is likely to be:

1. noisy,
2. hard to attribute,
3. vulnerable to oscillation,
4. and difficult to interpret when regressions appear.

The stronger contract is asymmetric:

1. the red loop searches aggressively,
2. the blue loop moves more slowly inside a bounded config ring,
3. and the judge remains fixed while both players are evaluated against it.

## 6. Human friction remains a separate calibration ring

Human burden must not be inferred from adversary-sim traffic.

The earlier human-friction doctrine remains binding:

1. human-friction evidence must come from real human traversal or an explicitly human-operated calibration workflow,
2. adversary-sim traffic must never count as human-friction evidence,
3. journey success and burden must remain separate,
4. and the human ring must not dilute or pollute the strict adversary proof ring.

This means the attacker loop may optimize exclusion pressure and hostile success,
while the human ring separately asks:

1. whether a human still reached the intended content,
2. what challenge burden was imposed,
3. what delay and retries were added,
4. and whether abandonment increased.

# Architecture Consequence

Shuma should now be described as four coordinated but non-equivalent rings:

1. red search loop
   - persona-specific attacker optimization under black-box attacker constraints;
2. blue tuning loop
   - bounded config tuning against fixed objectives and fixed legal moves;
3. fixed judge
   - machine-first scorecard, admissibility, archive, and escalation authority;
4. human-friction calibration ring
   - real-human burden measurement over the defended state.

This is not "one loop" and it is not "two equal optimization loops."
It is one triadic competitive system plus a separate calibration ring.

# Recommended Episode Cadence

The preferred cadence is staged rather than fully simultaneous:

1. freeze the current defense config,
2. let the red loop search for high-fitness attacks against it,
3. preserve strong attacks as episode evidence and candidate replay material,
4. promote the strongest stable findings into a replay or archive layer,
5. freeze that attacker archive for the next blue phase,
6. let the blue loop tune bounded configuration changes against:
   1. the promoted attacker archive,
   2. current live-traffic evidence,
   3. and fixed operator objectives,
7. evaluate the result through the fixed judge,
8. separately run human-friction calibration against retained candidates,
9. then reopen the next red search phase only after the blue result is judged and archived.

This cadence preserves:

1. attacker realism,
2. defender accountability,
3. judge independence,
4. and human-friction separation.

# Implications For Existing Plan Families

## 1. `SIM-LLM-*` and Scrapling realism work

Attacker-runtime planning should stop speaking as though the red side's job is to satisfy Shuma-defined maliciousness criteria.

Instead:

1. red runtimes should consume persona-specific attacker objectives,
2. black-box constraints,
3. episode budgets,
4. and admissibility boundaries enforced by the judge.

## 2. `RSI-SCORE-*` and judge work

Judge work should continue to own:

1. realism admissibility,
2. exploit progress,
3. evidence quality,
4. breach locality,
5. archive or replay promotion,
6. and config-vs-code escalation decisions.

Judge work must not drift into authoring attacker fitness functions.

## 3. `OVR-AGENT-*` defender work

Defender work should remain bounded to:

1. fixed operator objectives,
2. fixed legal move rings,
3. fixed judge outputs,
4. and the smallest credible repair strategy.

It should not co-train continuously against a moving attacker objective inside the same episode.

## 4. `HUM-FRIC-*` work

Human-friction calibration remains separate and blocked until:

1. the attacker picture is strong enough to calibrate a real defended state,
2. the strict loop is trustworthy,
3. and the human traversal workflow is explicit and safe.

# Execution Shape

## Task 1: Freeze the attacker-objective contract

**Files:**

- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`
- Modify: `docs/plans/2026-03-24-llm-player-role-decomposition-plan.md`

**Work:**

1. State explicitly that attacker lanes optimize attacker utility, not defender-authored malevolence.
2. Introduce persona-specific attacker objective families.
3. Keep black-box attacker constraints explicit and Shuma-blind.

**Acceptance criteria:**

1. attacker planning no longer frames realism as the red side satisfying Shuma's notion of maliciousness,
2. persona-specific attacker objectives are named explicitly,
3. and attacker contracts remain black-box and host-root-first.

## Task 2: Freeze judge ownership over realism and admissibility

**Files:**

- Modify: `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`
- Modify: related `RSI-SCORE-*` planning where needed

**Work:**

1. State explicitly that realism envelopes belong to judge admissibility, not attacker fitness.
2. Keep archive or replay promotion under the judge.
3. Keep held-out score semantics and code-escalation authority judge-owned.

**Acceptance criteria:**

1. the judge's realism role is explicit,
2. attacker planning no longer owns realism scoring,
3. and archive promotion remains judge-owned.

## Task 3: Freeze staged asymmetric cadence instead of same-cadence dual optimization

**Files:**

- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`
- Modify: `docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`

**Work:**

1. Define the staged red-search then blue-tuning cadence explicitly.
2. State that Shuma does not run equal-authority same-cadence co-evolution by default.
3. Preserve archive, replay, and baseline freeze points between red and blue phases.

**Acceptance criteria:**

1. the repo has one canonical answer to "how do the red and blue loops interact?",
2. the answer is asymmetric and staged,
3. and frozen archive or baseline handoff points are explicit.

## Task 4: Keep the human-friction ring separate from adversary optimization

**Files:**

- Modify: `docs/plans/2026-03-27-human-friction-calibration-ring-plan.md`
- Modify: adjacent Game Loop planning where wording still blurs the rings

**Work:**

1. Restate that human burden is not an attacker-objective constraint.
2. Keep human calibration as a separate defended-state evaluation ring.
3. Preserve journey success and burden as distinct human metrics.

**Acceptance criteria:**

1. human-friction planning remains separate from sim-only proof,
2. attacker optimization does not absorb human-friction metrics,
3. and the human ring remains tied to config revision and defended-state lineage.

# Definition Of Done

This planning tranche is complete when:

1. the repo has one canonical plan note that rejects defender-authored malevolence as the attacker objective,
2. the attacker / defender / judge split remains explicit and asymmetrical,
3. the human-friction ring remains separate from attacker optimization,
4. and downstream planning can inherit one stable answer to:
   1. what the attacker optimizes,
   2. what the judge owns,
   3. how the defender moves,
   4. and where human calibration belongs.
