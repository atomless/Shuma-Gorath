Date: 2026-03-21
Status: Proposed

Related context:

- [`../research/2026-03-21-feedback-loop-and-architecture-debt-review.md`](../research/2026-03-21-feedback-loop-and-architecture-debt-review.md)
- [`../research/2026-03-21-agent-first-feedback-loop-sequencing-review.md`](../research/2026-03-21-agent-first-feedback-loop-sequencing-review.md)
- [`../research/2026-03-21-loop-closure-execution-readiness-review.md`](../research/2026-03-21-loop-closure-execution-readiness-review.md)
- [`../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`../research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md`](../research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md)
- [`2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md`](2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md)
- [`2026-03-21-agent-first-loop-truth-completion-implementation-plan.md`](2026-03-21-agent-first-loop-truth-completion-implementation-plan.md)
- [`2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md`](2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md)
- [`2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md`](2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md)
- [`2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md`](2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md)
- [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`2026-03-20-benchmark-suite-v1-design.md`](2026-03-20-benchmark-suite-v1-design.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`2026-03-15-agentic-era-oversight-design.md`](2026-03-15-agentic-era-oversight-design.md)
- [`2026-03-15-agentic-era-oversight-implementation-plan.md`](2026-03-15-agentic-era-oversight-implementation-plan.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Close the first real Shuma feedback loop and structurally decompose the control-plane hotspots before additional loop logic lands in already oversized files.

# Core Decisions

1. The highest-priority gap is loop closure, not more feature breadth.
2. Structural decomposition must happen in behavior-preserving slices before more benchmark, operator-snapshot, or oversight logic is added to the current hotspot files.
3. The first controller remains recommend-only. It must not apply or schedule changes until benchmark comparison, objectives, and decision lineage are truthful.
4. Monitoring and Tuning remain projections and control surfaces over machine-first contracts, not separate semantic systems.
5. Operator objectives and hard-never config rings must remain outside the controller move set; admin writability must never be treated as controller eligibility by default.
6. The first shared-host agent tweaker loop must precede `MON-OVERHAUL-1` so the human surfaces project proven backend semantics rather than invent them ahead of the loop.
7. Replay-promotion lineage must move from sidecar test tooling into backend contracts before later scheduled-agent planning is reopened.
8. The next coding tranche should start from the detailed 2026-03-21 execution-ready implementation plans, not only from this high-level sequencing note.
9. The first truly closed autonomous tuning loop must not use `synthetic_traffic` as tuning evidence; it must depend on protected Scrapling runtime evidence plus replay-promoted or equivalently confirmed frontier or LLM lineage.
10. Monitoring overhaul should follow the first proven closed loop, not merely the first recommend-only loop, so human surfaces reflect the final protected-evidence and rollback semantics.
11. The representativeness contract for Scrapling and frontier or LLM lanes must be judged against Shuma's canonical non-human taxonomy, not lane-local assumptions.
12. Category classification confidence must land before lane representativeness is considered trustworthy enough for autonomous tuning.
13. The taxonomy comes before attackers: Shuma must define the categories it intends to model before it has enough observed adversary traffic to learn them site-locally.
14. The initial taxonomy should stay stable enough for the first closed loop; what should evolve first is the fingerprinting and classification quality within it. Taxonomy expansion is a later contingency only if important non-human traffic persistently falls outside the existing categories.
15. Taxonomy entries must carry stable machine and human-facing metadata because operator objectives and later tuning surfaces will bind posture directly to those categories.
16. The next LLM adversary step should be bounded category-fulfillment modes behind a pluggable containerized backend contract; the full first-class LLM runtime actor remains later.
17. The first genuinely closed loop ends at bounded config tuning and rollback; the later LLM diagnosis harness and later LLM code loop remain downstream phases rather than part of the first closure slice.

# Target Architecture

## 1. Control Contracts

The first real loop should converge on one backend contract chain:

1. persisted `operator_objectives_v1`,
2. materialized `operator_snapshot_v1`,
3. materialized `benchmark_results_v1` with baseline and candidate comparison semantics,
4. durable decision-evidence ledger,
5. bounded `allowed_actions_v1`,
6. replay-promotion lineage contract,
7. recommend-only reconcile engine,
8. first shared-host agent tweaker harness,
9. thin Monitoring and Tuning projections.

## 2. Structural Goal

The control plane should stop concentrating unrelated responsibilities in single files.

The target state is:

1. `src/admin/api.rs` becomes a thin auth/rate-limit/router shell that delegates to domain modules.
2. `src/admin/adversary_sim.rs` becomes focused on shared state and orchestration composition, not every lane/runtime/diagnostic detail.
3. `src/observability/operator_snapshot.rs` becomes a top-level builder over focused objective, runtime-posture, recent-change, and verified-identity summary modules.
4. `src/observability/benchmark_results.rs` becomes a top-level comparator over focused family evaluators and history/comparison helpers.
5. `src/config/controller_action_surface.rs` becomes a thin derived surface over a smaller catalog and policy helper structure.
6. `scripts/tests/adversarial_simulation_runner.py` becomes an orchestrator over focused modules for contract loading, execution, evidence shaping, discovery scoring, and governance/report checks.

# Phase Plan

## Phase 1: Structural Decomposition Prerequisites

These slices are first because the repo should not continue to land control-loop behavior into the current hotspot files.

### `ARCH-API-1`

Split `src/admin/api.rs` into domain-routed modules without changing endpoint contracts.

Acceptance:

1. auth, rate-limit, and top-level routing remain centralized,
2. operator snapshot, benchmark, monitoring, config, adversary-sim, and diagnostics handlers each move behind dedicated modules,
3. endpoint behavior and focused `make` verification remain unchanged.

### `ARCH-OBS-1`

Split operator snapshot and benchmark materialization into focused modules before more loop semantics land there.

Acceptance:

1. objective-profile helpers, recent-change shaping, verified-identity summary shaping, benchmark family evaluators, and history/comparison helpers each have focused homes,
2. the public contract shape remains unchanged until the later semantic tranche,
3. the top-level orchestrator files stop growing as the next features land.

### `ARCH-SIM-1`

Split `src/admin/adversary_sim.rs` into control-state, lane-runtime, diagnostics, and corpus/worker-plan helpers before reconcile integration work.

Acceptance:

1. adversary control-state transitions remain intact,
2. Scrapling and deterministic lane logic stay behavior-identical,
3. bot-red-team placeholder behavior remains explicit rather than hidden.

### `ADV-RUN-ARCH-1`

Execute the existing adversarial runner refactor as part of this phase, not as later platform cleanup.

Acceptance:

1. contract loading, execution, evidence shaping, discovery scoring, and governance/report logic stop cohabiting one 6k+ line file,
2. promotion and frontier lineage semantics remain unchanged,
3. the runner becomes safe to integrate with later backend promotion work.

## Phase 2: Loop Truth Completion

These slices complete the missing truth the controller needs.

### `OPS-BENCH-2`

Materialize real benchmark history and comparator semantics.

Must include:

1. prior-window or explicit baseline persistence,
2. `improvement_status`,
3. representative adversary scenario-family results,
4. beneficial non-human posture metrics,
5. verified-identity-aware capability gating,
6. explicit candidate-vs-current comparison support for later tuning or code-evolution loops.

### `OPS-SNAPSHOT-2`

Replace backend-default and placeholder operator state with typed site-owned contract surfaces.

Must include:

1. persisted writable `operator_objectives_v1`,
2. objective revision/reference in the snapshot,
3. typed verified-identity summary instead of placeholder text,
4. causal decision/watch-window evidence rather than only recent-change summaries,
5. durable evidence references needed for later reconcile and rollback reasoning.

### `ADV-PROMO-1`

Promote emergent finding and deterministic replay lineage into backend contracts.

Must include:

1. typed replay-candidate and promotion-lineage contract,
2. integration point from current promotion tooling into backend-readable state,
3. snapshot or benchmark visibility for promoted or review-pending replay candidates,
4. no uncontrolled mutation of the deterministic corpus.

## Phase 3: Recommend-Only Reconcile Loop

### `OVR-RECON-1`

Land the first backend recommend-only reconciler using the now-truthful contracts.

Must include:

1. pure reconcile engine,
2. patch policy against `allowed_actions_v1`,
3. typed proposal output,
4. decision ledger persistence,
5. adversary-verification requirement for guarded families,
6. explicit fail-closed behavior when evidence is stale, degraded, or contradictory.

This phase must reuse:

1. the existing adversary-sim lease/idempotency pattern,
2. existing config validation seams,
3. and the machine-first snapshot and benchmark contracts.

## Phase 4: First Machine-First Agent Tweaker Loop

### `OVR-AGENT-1`

Land the first shared-host agent tweaker loop over the truthful backend contracts before Monitoring or Tuning projection work.

Must include:

1. one agent invocation path that calls the same internal reconcile contract whether triggered periodically or immediately after a qualifying adversary-sim run,
2. consumption of `operator_snapshot_v1`, `benchmark_results_v1`, replay-promotion lineage, and recent decision evidence,
3. typed recommend-only proposal outputs and durable evidence references rather than prose-only diagnostics,
4. explicit `no_change`, `insufficient_evidence`, `rerun_sim_required`, and equivalent fail-closed outcomes when signal is stale, contradictory, or incomplete,
5. shared-host control-plane execution only, never request-path or edge-gateway execution.

This phase must prove:

1. the backend loop can read sim-cost and benchmark deltas end to end,
2. proposal families and evidence semantics are now real enough for human projection,
3. and later Monitoring/Tuning work can be derived from demonstrated backend behavior instead of speculative UI-first modeling.

## Phase 5: Protected Tuning Evidence And Closed-Loop Safety

Execution-ready plan chain:

1. [`2026-03-22-taxonomy-and-classification-implementation-plan.md`](2026-03-22-taxonomy-and-classification-implementation-plan.md)
2. [`2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md`](2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md)
3. [`2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md`](2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md)

### `TRAFFIC-TAX-1`

Define the canonical non-human traffic taxonomy that later tuning and lane-representativeness work will use.

### `TRAFFIC-TAX-2`

Materialize bounded category-confidence and evidence receipts so Shuma can tell when both simulated and observed traffic categorization are trustworthy enough to use in tuning decisions.

### `SIM-LLM-FIT-1`

Implement the minimum bounded LLM-backed browser or request modes needed for category fulfillment behind a pluggable model-backend contract, with frontier-backed execution as the initial reference path for the highest-capability categories and optional later local-model backends only if evals prove parity.

### `SIM-FULFILL-1`

Implement the category-to-lane fulfillment matrix across Scrapling and frontier or containerized LLM modes before claiming lane representativeness.

### `SIM-PROTECTED-1`

Codify protected tuning evidence eligibility and explicitly exclude `synthetic_traffic` from any future auto-apply evidence basis.

### `SIM-COVER-1`

Define the representativeness matrix and bounded coverage receipts across Scrapling runtime traffic and replay-promoted frontier or LLM lineage for the non-human categories Shuma intends to optimize over, using the canonical taxonomy rather than lane-local labels.

### `SIM-SCR-CHALLENGE-1`

After the request-native Scrapling category tranches land, add a separate blocked evaluation for widened Scrapling defense-surface coverage:

1. which Shuma request-native defenses Scrapling should be able to hit,
2. which missing interactions can stay inside the current fetcher boundary,
3. which truly require browser or stealth fetchers,
4. and how those claims become receipt-backed rather than aspirational.

This follow-on should remain distinct from the broader `automated_browser` ownership question.

### `SIM-LLM-1A..1C`

Keep the later full LLM attacker-agent track explicitly separate from the bounded `SIM-LLM-FIT-1` fulfillment tranche.

When this blocked work is eventually reopened, decompose it as:

1. `SIM-LLM-1A` attacker-agent black-box contract,
2. `SIM-LLM-1B` attacker-agent episode harness and bounded memory contract,
3. `SIM-LLM-1C` full first-class attacker runtime actor.

Treat this later track as the LLM-backed player in the sim harness rather than as a generic future actor.

### `OPS-OBJECTIVES-3`

Extend `operator_objectives_v1` with category-aware non-human intent so the controller can distinguish `allowed`, `tolerated`, `cost_reduced`, `restricted`, and `blocked` posture by category.

### `OPS-BENCH-3`

Extend `benchmark_results_v1` with protected-lane eligibility and category-aware comparison semantics suitable for canary apply and rollback.

### `OVR-APPLY-1`

Only after the above gates are real, add the first bounded canary apply, watch-window, compare, and rollback loop.

Status update (2026-03-22): complete and live-proven on shared-host per [`../research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md`](../research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md).

### `ADV-DIAG-1`

Before Monitoring and Tuning are reopened, reconcile adversary-sim status diagnostics with the persisted event telemetry that the closed loop now correctly treats as authoritative. The live `OVR-APPLY-1` proof showed that `sim_run_id` event evidence can be truthful while shared-host generation counters remain zero.

Execution reference: [`2026-03-23-adv-diag-1-adversary-sim-status-truth-implementation-plan.md`](2026-03-23-adv-diag-1-adversary-sim-status-truth-implementation-plan.md)

Status update (2026-03-23): complete per [`../research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md`](../research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md).

### `SIM-SCR-FIT-1`

Before Monitoring re-projects adversary semantics, freeze Scrapling's truthful near-term category ownership to the request-native non-human categories it can genuinely own on its current shared-host runtime boundary:

1. `indexing_bot`
2. `ai_scraper_bot`
3. `http_agent`

Execution reference: [`2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)

### `SIM-SCR-FIT-2`

Implement bounded Scrapling request personas for those request-native categories without widening the current worker into a browser-agent runtime.

Execution reference: [`2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)

### `SIM-SCR-COVER-2`

Prove the expanded Scrapling ownership through the canonical classification and coverage receipts so later Monitoring projects receipt-backed category truth rather than the older indexing-only lane story.

Execution reference: [`2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)

### `VID-TAX-1`

Before Monitoring is redesigned, replace the current flattened verified-identity category collapse with a tested crosswalk from verified-identity categories into the canonical non-human taxonomy.

Execution reference: [`2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)

### `VID-TAX-2`

Add explicit verified-identity versus taxonomy alignment receipts so later machine-first contracts can show whether high-confidence verified traffic is being categorized faithfully or through degraded fallback.

Execution reference: [`2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)

### `VID-BOT-1`

Add benchmark and snapshot metrics that quantify verified-identity versus botness conflicts so Shuma can detect calibration drift rather than tuning through it blindly.

Execution reference: [`2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)

### `VID-GUARD-1`

Make diagnosis and bounded tuning fail closed when verified-identity friction mismatch or unresolved botness conflicts show likely harm against configured tolerated or allowed verified traffic.

Execution reference: [`2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)

### `HOST-COST-1`

Add bounded forwarded-latency telemetry to the machine-first request-outcome path so Shuma can measure host-impact proxies that go beyond suspicious request and byte rates.

Execution reference: [`2026-03-23-host-impact-cost-proxy-and-benchmark-implementation-plan.md`](2026-03-23-host-impact-cost-proxy-and-benchmark-implementation-plan.md)

### `HOST-COST-2`

Thread the new host-impact proxy through `operator_snapshot_v1`, `operator_objectives_v1`, and `benchmark_results_v1` so reconcile and later Monitoring can consume a truthful suspicious host-impact metric.

Execution reference: [`2026-03-23-host-impact-cost-proxy-and-benchmark-implementation-plan.md`](2026-03-23-host-impact-cost-proxy-and-benchmark-implementation-plan.md)

## Phase 6: Human Operator Projection

### `UI-VID-1`

Before the larger Monitoring rewrite, add a first-class `Verified Identity` pane to `Verification` so native Web Bot Auth and other verified-identity basics are not Advanced-only.

Execution reference: [`2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)

### `UI-RED-1`

Before the larger Monitoring rewrite, surface adversary-sim status `truth_basis` and persisted-event recovery details in `Red Team` so operators can tell when counters are direct versus recovered lower-bound truth.

Execution reference: [`2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)

### `MON-OVERHAUL-1`

Rebuild Monitoring as the thin human projection over the machine-first contracts after the backend truth, first working closed loop, `ADV-DIAG-1` diagnostics-truth follow-up, the Scrapling request-native category follow-ons, the verified-identity calibration track, the host-impact cost proxy track, and the local `UI-VID-1` and `UI-RED-1` tab surfacing slices are complete.

Monitoring is now explicitly scoped as the human-readable accountability surface for the closed loop rather than the primary manual tuning cockpit: it should lead with loop verdict, outcome frontier, controller judgment, category breakdown, and trust or actionability blockers. Live and recent traffic visibility should now move into a dedicated `Traffic` tab, while Diagnostics becomes more explicitly diagnostics-first and furniture-operational.

It should also show bounded progress over recent completed loops against benchmark families and controller action history, not only the latest loop outcome.

The Monitoring follow-on sequence is now explicitly three-way: after `MON-OVERHAUL-1B` makes loop accountability real, `TRAFFIC-TAB-1` should move the current traffic-facing Diagnostics surface into a dedicated `Traffic` tab, `DIAG-CLEANUP-1` should then narrow Diagnostics to furniture-operational proof, and only then should `MON-OVERHAUL-1C` land against the cleaned ownership boundary. That later Game Loop slice should keep the existing overall top line, make the true numeric objective budgets readable as target-vs-current budget usage, and express taxonomy categories as target-achievement rows rather than inventing fake per-category budgets.

That operator-surface follow-on is no longer the immediate mainline. The active execution order now moves first through attacker-faithful Scrapling (`SIM-SCR-CHALLENGE-2A..2D`), then the legal move ring (`CTRL-SURFACE-1..3`), then the first judge-side game-contract slices (`RSI-GAME-1A`, `RSI-GAME-1B`, `RSI-SCORE-1`, `RSI-GAME-1C`), and only then the first explicit self-improving loop before returning to deferred dashboard follow-ons. `RSI-GAME-MAINLINE-1A` and `RSI-GAME-MAINLINE-1B` are now landed, so the first working game-loop proof lane is complete.

### `CTRL-SURFACE-1..3`

Before the later operator-control and broader recursive-improvement phases are reopened, codify the controller mutability policy explicitly:

1. `operator_objectives_v1` remains the rule set for the game and is never controller-mutable,
2. hard-never config rings such as provider topology, trust exceptions, privacy posture, and defender safety budgets stay outside the loop,
3. `allowed_actions_v1`, benchmark escalation, and the patch proposer must agree on one bounded controller-tunable surface,
4. and later Monitoring, Tuning, and recursive phases must consume that canonical mutability truth rather than inferring mutability from admin writability.

### `TUNE-SURFACE-1`

Finish the operator control surface once the controller inputs, safe action families, canonical mutability policy, first working agent loop semantics, and adversary-sim diagnostics truth are all aligned, including the operator-objectives and per-category posture editor over the stable operator-facing taxonomy.

The first concrete UI contract for this tranche is now the taxonomy posture matrix in `Tuning`:

1. `Policy` keeps declarative crawl and exemption rules,
2. `Tuning` owns active defense posture over canonical non-human categories,
3. the editor should render one category row against the bounded five-point posture scale,
4. and optional stance archetypes should only seed the matrix rather than becoming a second persisted policy model.
5. this first matrix section should be visually primary so the tab reads as the home of operator-owned tuning rather than as a threshold appendix.

Follow-on ownership after the matrix lands:

1. ratified controller-tunable botness and fingerprint controls should consolidate into `Tuning`,
2. the current `Fingerprinting` tab should be renamed to `Identification` and keep provider-source posture plus effective scoring diagnostics,
3. `Identification` should also explain how the available signals distinguish the canonical non-human taxonomy categories,
4. and the later budget/controller-explanation layer should land only after that ownership split is settled.

These phases should not be started early just because the UI can be edited sooner.

## Phase 7: Later Scheduled-Agent And Code-Evolution Loops

These later items remain intentionally non-execution-ready until the three Phase 5 implementation plans above are complete and live-proved.

### `RSI-GAME-1A..1C` and `RSI-ROLES-1`

Before reopening the later recursive-improvement phases, codify the missing game contract explicitly:

1. immutable rules built from `operator_objectives_v1` plus the canonical controller-mutability policy,
2. sacred evaluator scorecard over benchmark families, safety gates, and regression anchors,
3. bounded legal move ring,
4. explicit shortfall-attribution and move-selection policy rather than only coarse pressure heuristics,
5. tractability boundaries between exact config moves, family-level policy moves, and code or capability gaps,
6. attacker/defender/judge role separation for later recursive-improvement phases,
7. and an episode archive or stepping-stone memory that later run-to-homeostasis episodes can use.

The remaining protocol-level contracts under that game should also stay explicit rather than implicit:

1. `RSI-SCORE-1` for the canonical judge scorecard over optimization targets, hard guardrails, regression anchors, and homeostasis inputs,
2. `RSI-PROTO-1` for canonical attacker and defender observation, action, proposal, refusal, and receipt schemas,
3. `RSI-EVAL-1` for the boundary between player-visible protected evidence and judge-held-out evaluation contexts,
4. and `RSI-AUDIT-1` for canonical config and later GitHub-backed code provenance across recursive-improvement episodes.

Current note:

1. `RSI-GAME-1A`, `RSI-GAME-1B`, `RSI-SCORE-1`, `RSI-GAME-1C`, `RSI-GAME-MAINLINE-1A`, `RSI-GAME-MAINLINE-1B`, and `RSI-ROLES-1` are now landed.
2. The remaining recursive-improvement contract gap is no longer the high-level role split, but the player protocol, held-out evaluation, and audit lineage contracts that sit on top of it.

### `OVR-AGENT-2A..2C`

Reopen the later LLM-backed defender-agent track only after the first shared-host agent loop, the first closed config loop, Monitoring projection, Tuning surface, replay-promotion contract, and central-intelligence architecture all exist.

When this blocked work is eventually reopened, decompose it as:

1. `OVR-AGENT-2A` sacred input and bounded output contract,
2. `OVR-AGENT-2B` recommendation-only defender runtime,
3. `OVR-AGENT-2C` later bounded autonomous defender episode controller.

Later controller planning should also adopt the recursive-improvement methodology captured in [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md):

1. use `Human-only / private` as the first development reference stance,
2. run bounded optimization episodes until target-not-met and progress-not-flat are both true,
3. define homeostasis over recent completed watch-window cycles,
4. and only later broaden into preset sweeps over relaxed operator stances.

It should also consume the canonical game-contract and move-selection plan captured in [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md) rather than reconstructing its own implicit rules, evaluator, or move set.

It should further consume the later protocol contracts captured in [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md) rather than inventing its own score semantics, player wire formats, or evaluation-visibility rules.

It should also consume the audit and provenance contract captured in [`2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md`](2026-03-24-game-loop-audit-trail-and-github-provenance-plan.md) so later defender episodes leave stable receipt lineage instead of only transient recommendation prose.

When this later phase is designed, treat it as only one player in a larger triad:

1. LLM-backed attacker agent in the sim harness,
2. LLM-backed defender agent in the diagnosis/config loop,
3. and the machine-first benchmark stack plus Monitoring projection as the independent judge.

### `OVR-CODE-1`

Keep the later benchmark-driven LLM code-evolution or PR-generation path behind the bounded config loop, the later diagnosis harness, and benchmark-comparison proof.

When this phase reopens, code-evolution proposals should treat the strict reference stance as a mandatory regression anchor even when optimizing more permissive target stances.

They should also inherit the canonical game contract and move-selection boundaries rather than invent a second notion of objective, legal move scope, or progress.

They should further lean on GitHub as the canonical code-lineage ledger for branch, PR, review, check, merge, revert, and later artifact-attestation provenance wherever feasible, while leaving benchmark and no-harm outcome truth with the machine-first judge.

# Scheduling Rules

1. Execute the phases in order.
2. Do not blend structural decomposition and semantic expansion in the same tranche.
3. Keep the first decomposition slices behavior-preserving and test-focused.
4. Keep one hotspot file as the primary target per refactor tranche wherever practical.
5. Do not reopen `MON-OVERHAUL-1`, `TUNE-SURFACE-1`, or `OVR-AGENT-2` until the blockers listed in this plan are satisfied, including protected tuning evidence, category-coverage proof, the settled Scrapling request-native ownership follow-ons, the verified-identity calibration tranches, the host-impact cost proxy tranches, the controller-mutability tranches, and the ownership split captured in `UI-VID-1` and `UI-RED-1`.
6. Treat periodic scheduling and post-sim triggering as adapter paths over one reconcile or agent contract, not as separate controller implementations.

# File-Length And Separation Guardrails

These are review heuristics, not the product goal themselves:

1. `src/admin/api.rs` should trend toward a thin router shell rather than continue as the home of endpoint implementations.
2. `scripts/tests/adversarial_simulation_runner.py` should become a driver/orchestrator rather than a repository of unrelated execution and governance logic.
3. `src/admin/adversary_sim.rs` should stop co-locating control-state machinery with lane-specific runtime details.
4. `src/observability/operator_snapshot.rs` and `src/observability/benchmark_results.rs` should stop being the only homes for every summary/comparator concern.
5. Any new loop feature that would grow one of these hotspot files without first attempting extraction should be treated as a planning failure.

# Exit Criteria

This plan is satisfied when:

1. the hotspot decompositions are complete enough that new loop work no longer lands into monolithic files by default,
2. the benchmark contract can express improvement or regression against a real baseline,
3. the operator snapshot contains persisted objectives, typed verified-identity summary, and causal decision evidence,
4. the recommend-only reconcile engine exists as a backend contract,
5. the first shared-host agent tweaker loop exists and can exercise the backend contracts against sim-cost and benchmark feedback,
6. replay-promotion lineage is part of the backend control plane rather than sidecar-only tooling,
7. Monitoring and Tuning consume those contracts rather than parallel semantics,
8. the first autonomous tuning loop is blocked until protected evidence and category-aware objective gates are delivered,
9. Monitoring and Tuning consume the proven closed-loop semantics plus faithful verified-identity calibration rather than the earlier recommend-only subset,
10. and only then the later scheduled-agent and code-evolution planning can resume.
