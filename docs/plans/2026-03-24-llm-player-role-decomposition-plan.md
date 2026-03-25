Date: 2026-03-24
Status: Proposed

Related context:

- [`../research/2026-03-24-llm-player-role-decomposition-review.md`](../research/2026-03-24-llm-player-role-decomposition-review.md)
- [`../research/2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](../research/2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md)
- [`../research/2026-03-24-game-loop-audit-trail-and-github-provenance-review.md`](../research/2026-03-24-game-loop-audit-trail-and-github-provenance-review.md)
- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md)
- [`2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md`](2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md)
- [`2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`](2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Decompose the two later LLM-backed player roles so the recursive-improvement game has:

1. a clear LLM attacker-agent track,
2. a clear LLM defender-agent track,
3. one non-LLM judge,
4. and a backlog that shows which player-side implementation slices must exist before the later autonomous loop is execution-ready.

# Core Decisions

1. `SIM-LLM-1` is the umbrella attacker-agent track and should stay distinct from `SIM-LLM-FIT-1`.
2. `OVR-AGENT-2` is the umbrella defender-agent track and should stay distinct from `OVR-CODE-1`.
3. The attacker-agent track should decompose into:
   1. black-box contract,
   2. episode harness,
   3. later full runtime actor.
4. The defender-agent track should decompose into:
   1. sacred input and bounded output contract,
   2. recommendation-only runtime,
   3. later bounded autonomous episode controller.
5. The judge contract remains in `RSI-GAME-1A..1C` and must not be merged into either player track.
6. Later player-side execution planning must consume:
   1. `RSI-SCORE-1` for the judge scorecard semantics,
   2. `RSI-PROTO-1` for canonical player wire schemas,
   3. `RSI-EVAL-1` for held-out evaluation separation,
   4. and `RSI-AUDIT-1` for shared episode and proposal provenance.
7. The later player-side runtime tracks must not be treated as execution-ready until Scrapling-owned defense surfaces are attacker-faithful and receipt-backed. For the current owned request-native matrix that prerequisite is now satisfied by `SIM-SCR-CHALLENGE-2A`, `SIM-SCR-CHALLENGE-2B`, and `SIM-SCR-CHALLENGE-2D`; where a later owned-surface review proves browser or stealth Scrapling is needed, reopen `SIM-SCR-CHALLENGE-2C` or `SIM-SCR-BROWSER-1` explicitly.

## Task 1: `SIM-LLM-1A`

### LLM attacker-agent black-box contract

**Files:**

- Modify: `docs/plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**

1. Define the full attacker-agent role as an LLM-backed player in the sim harness.
2. Make the black-box boundary explicit:
   1. no admin credentials,
   2. no privileged secrets,
   3. no silent access to the judge or controller state,
   4. only ratified environment observations and tools,
   5. only the public-knowledge position available to an outside attacker,
   6. no Shuma-specific route, defense, implementation, source-code, or documentation knowledge,
   7. and no web or repo lookup path that could reveal Shuma-specific internals outside the attacked host itself.
3. Define the attacker-agent action surface:
   1. request generation,
   2. browser or automation actions where allowed,
   3. tool invocations,
   4. episode and token budgets.
4. Define the initial attacker context:
   1. host-site root entrypoint only,
   2. the non-human category or category family it is meant to fulfill,
   3. malicious-category priming where the category should behave maliciously,
   4. and only bounded public-site hints the attacker could discover from the host itself, such as `robots.txt`, sitemap references, and traversal-visible pages.
4. Define the receipt model:
   1. prompts or instructions lineage where safe,
   2. action trace,
   3. resulting traffic lineage,
   4. replay or protected-evidence implications.

**Acceptance criteria:**

1. the repo has one canonical answer to "what is the later LLM attacker agent allowed to see and do?",
2. the attacker role is clearly subordinate to the independent judge,
3. the black-box boundary is explicit enough to prevent quiet privilege drift,
4. and the attacker is clearly host-root-first and Shuma-blind rather than secretly Shuma-aware.
5. and the attacker is explicitly confined to outside-attacker public knowledge rather than any repo, docs, or internal-product awareness.

Protocol note:

1. `SIM-LLM-1A` must now emit and consume the landed `RSI-PROTO-1` schema families rather than defining a bespoke attacker-only wire format.
2. In particular, attacker observations and actions must live inside the shared envelope revision and use the canonical attacker `message_kind` families defined there.

## Task 2: `SIM-LLM-1B`

### LLM attacker-agent episode harness and bounded memory contract

**Files:**

- Modify: `docs/plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`
- Modify: `docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**

1. Define the attacker-agent episode shape:
   1. initial objective,
   2. environment reset,
   3. bounded action horizon,
   4. completion and failure states.
2. Define how attacker-side memory works:
   1. what prior episode information may be retained,
   2. what league, archive, or curriculum inputs are allowed,
   3. what remains hidden because it belongs to the judge.
3. Define how the attacker-agent transitions from bounded category-fulfillment work into a true adaptive adversary player.

**Acceptance criteria:**

1. the later attacker role is no longer just "a runtime actor" in prose,
2. and the repo has a concrete episode and memory contract for that player.

## Task 3: `SIM-LLM-1C`

### Full LLM attacker-agent runtime actor over the settled black-box and episode contracts

**Files:**

- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `todos/blocked-todo.md`
- Later code targets: containerized adversary runtime modules, orchestration helpers, attack-trace persistence

**Work:**

1. Reframe the old `SIM-LLM-1` item as the later full actor over the already-settled `SIM-LLM-1A` and `SIM-LLM-1B` contracts.
2. Keep it blocked behind:
   1. coverage proof,
   2. protected evidence,
   3. closed config loop proof,
   4. and the judge contract.

**Acceptance criteria:**

1. the full attacker-agent runtime is now clearly later than its contract and episode prerequisites,
2. and no one future TODO is carrying all attacker-side semantics alone.

## Task 4: `OVR-AGENT-2A`

### LLM defender-agent sacred input and bounded output contract

**Files:**

- Modify: `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**

1. Define the defender role as an LLM-backed player over bounded legal moves.
2. Define the sacred input bundle:
   1. operator objectives,
   2. benchmark and snapshot results,
   3. move-selection policy outputs,
   4. recent episode archive,
   5. protected-evidence status.
3. Define the defender output schema:
   1. bounded proposal,
   2. refusal,
   3. need-more-evidence,
   4. code-gap escalation,
   5. later code-evolution referral.
4. Define what the defender must never mutate:
   1. operator rules,
   2. hard-never config rings,
   3. judge criteria.

**Acceptance criteria:**

1. the repo has one canonical answer to "what may the later LLM defender agent read and emit?",
2. the defender role is explicitly bounded by the legal move ring,
3. and the defender cannot silently become a second judge.

Protocol note:

1. `OVR-AGENT-2A` must now emit and consume the landed `RSI-PROTO-1` schema families rather than inventing a defender-local vocabulary.
2. Defender outputs must therefore remain inside the canonical `config_proposal`, `refusal`, `need_more_evidence`, `code_gap_escalation`, and `code_evolution_referral` families.

Evaluation note:

1. later attacker and defender runtimes must now also inherit the landed `RSI-EVAL-1` visibility boundary,
2. so protected evidence may inform player behavior,
3. but held-out judge contexts and raw anchor inventories must remain outside player context windows.

## Task 5: `OVR-AGENT-2B`

### Recommendation-only LLM defender runtime over the proven closed loop

**Files:**

- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**

1. Define the first live defender-agent phase as recommendation-only.
2. Make it explicit that this phase consumes the closed config loop and the judge contract rather than bypassing them.
3. Keep code evolution out of scope.

**Acceptance criteria:**

1. the first defender-agent runtime is clearly narrower than the full later autonomous loop,
2. and it is clearly separate from code generation or PR creation.

## Task 6: `OVR-AGENT-2C`

### Later bounded autonomous defender episode controller

**Files:**

- Modify: `docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`
- Modify: `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**

1. Define the later phase where the defender participates in bounded run-to-homeostasis episodes.
2. Keep this phase explicitly downstream of:
   1. the sacred judge contract,
   2. move-selection policy,
   3. episode archive,
   4. role contract,
   5. and recommendation-only defender proof.
3. Preserve the split between:
   1. bounded config moves,
   2. code-gap escalation,
   3. and later code evolution.

**Acceptance criteria:**

1. the later defender autonomy is no longer an underspecified umbrella,
2. and the progression from recommend-only to bounded autonomous episodes is explicit.

# Sequencing

1. Keep the deferred operator-facing dashboard cleanup behind the current backend mainline; the immediate next backend slice is `SIM-LLM-1A`.
2. Keep the judge decomposition first: `RSI-GAME-1A`, `RSI-GAME-1B`, and `RSI-GAME-1C`.
3. Land `RSI-SCORE-1`, `RSI-PROTO-1`, and `RSI-EVAL-1` before any player role is treated as protocol-complete.
4. Land `RSI-AUDIT-1` before player-side runtimes are treated as operationally auditable.
5. `RSI-ROLES-1` is now landed and fixes the attacker/defender/judge split before reopening the later autonomous LLM lanes.
6. Land `SIM-LLM-1A` and `SIM-LLM-1B` before treating the full attacker runtime as execution-ready.
7. Land `OVR-AGENT-2A` before any defender-agent runtime planning is treated as execution-ready.
8. Land `OVR-AGENT-2B` before `OVR-AGENT-2C`.
9. Keep `OVR-CODE-1` downstream of the settled defender-agent track rather than folding it into defender planning.
10. Do not treat the fuller attacker or defender runtime tracks as execution-ready until the current attacker-faithful Scrapling baseline remains receipt-backed for owned request-native surfaces and, where a later matrix requires browser or stealth Scrapling, `SIM-SCR-CHALLENGE-2C` or `SIM-SCR-BROWSER-1` is also complete.

Current note:

1. `RSI-ROLES-1` and `RSI-PROTO-1` are now landed.
2. `RSI-EVAL-1` is now landed.
3. `RSI-AUDIT-1A` is now landed.
4. The remaining cross-cutting audit work is now the later GitHub-backed code lineage and operator projection layers, not the shared runtime-relevant episode vocabulary.

# Definition Of Done

This plan is complete when:

1. the attacker and defender roles are explicitly named as later LLM-backed players,
2. the judge remains explicitly non-LLM and machine-first,
3. the blocked backlog decomposes both player roles into concrete slices,
4. and the later recursive-improvement planning no longer relies on `SIM-LLM-1` and `OVR-AGENT-2` as umbrella placeholders.
