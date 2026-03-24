Date: 2026-03-16
Status: Roadmap capture

Related context:

- [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
- [`2026-03-15-agentic-era-oversight-design.md`](2026-03-15-agentic-era-oversight-design.md)
- [`2026-03-15-agentic-era-oversight-implementation-plan.md`](2026-03-15-agentic-era-oversight-implementation-plan.md)
- [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)
- [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md`](2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md)
- [`../research/2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md`](../research/2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md)
- [`../observability.md`](../observability.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Purpose

Capture the major pre-launch work that Shuma still needs, but that is not yet fully planned in execution-ready detail, so implementation can be sequenced deliberately rather than opportunistically.

This note is intentionally a roadmap and sequencing capture, not an implementation-ready plan for every item listed.

# What Is Already Partially Planned

## 1. Adversary-sim maturation is started, but not complete

Already captured:

1. minimal shared-host scope-and-seed gate,
2. Scrapling emergent-lane planning,
3. bounded category-fulfillment LLM lane planning,
4. blocked Scrapling runtime lane,
5. blocked containerized LLM lane,
6. deterministic oracle governance,
7. frontier data-governance work.

Current references:

1. [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
2. [`2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
3. [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md)
4. [`../research/2026-03-23-scrapling-non-human-category-capability-review.md`](../research/2026-03-23-scrapling-non-human-category-capability-review.md)
5. [`2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)
6. `SIM-SH-SURFACE-1`, `SIM-SCR-FIT-1`, `SIM-SCR-FIT-2`, `SIM-SCR-COVER-2`, `VID-TAX-1`, `VID-TAX-2`, `VID-BOT-1`, `VID-GUARD-1`, `UI-VID-1`, `UI-RED-1`, and `SIM-LLM-FIT-1` in [`../../todos/todo.md`](../../todos/todo.md)
7. `SIM-SCR-BROWSER-1` and `SIM-LLM-1` in [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)
8. the execution-ready closed-loop chain in [`2026-03-22-taxonomy-and-classification-implementation-plan.md`](2026-03-22-taxonomy-and-classification-implementation-plan.md), [`2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md`](2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md), and [`2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md`](2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md)

Gap:

1. this was true when first written, but the gap is now closed by [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md), which makes deterministic traffic the oracle and comparator, emergent lanes the primary adaptive inputs, and replay promotion into deterministic memory a named future step.

## 2. Oversight-controller direction is planned, but the operator surfaces it depends on are not all ready

Already captured:

1. bounded oversight controller,
2. budget snapshots,
3. recommend/canary/autonomous rollout model.

Current references:

1. [`2026-03-15-agentic-era-oversight-design.md`](2026-03-15-agentic-era-oversight-design.md)
2. [`2026-03-15-agentic-era-oversight-implementation-plan.md`](2026-03-15-agentic-era-oversight-implementation-plan.md)

Gap:

1. monitoring and tuning surfaces are not yet planned as the operator-grade inputs and outputs of that controller.

## 3. Central intelligence, ban jitter, and local recidive now have design direction

Already captured:

1. banded ban jitter,
2. local repeat-offender ladder,
3. central intelligence classes and controller fit.

Current references:

1. [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)

Gap:

1. storage architecture, governance, and sequencing relative to monitoring, tuning, and oversight are not yet broken out as their own roadmap work.

## 4. Several sequence-critical backlog items already exist, but were not yet reflected in the master order

Already captured in backlog:

1. dashboard connection-state hardening,
2. admin config contract truthfulness cleanup,
3. production adversary-sim operating-envelope hardening,
4. shared-host discovery baseline,
5. privacy and state-minimization follow-up,
6. final pre-launch performance gate.

Current references:

1. `SIM2-R4-CONN-1` in [`../../todos/todo.md`](../../todos/todo.md)
2. config contract truthfulness item in [`../../todos/todo.md`](../../todos/todo.md)
3. `SIM-DEPLOY-2` in [`../../todos/todo.md`](../../todos/todo.md)
4. `SIM-SH-SURFACE-1` in [`../../todos/todo.md`](../../todos/todo.md)
5. `SEC-GDPR-2` and `SEC-GDPR-3` in [`../../todos/todo.md`](../../todos/todo.md) and [`../../todos/security-review.md`](../../todos/security-review.md)
6. `PERF-LAUNCH-1` in [`../../todos/todo.md`](../../todos/todo.md)

Gap:

1. these items materially change sequencing, but they were not yet called out in the roadmap as explicit prerequisites or release gates.

# Major Missing Planning Tracks

## A. Mature Adversary-Sim As An Operating Input, Not Just A Contributor Tool

Shuma still needs a coherent mature adversary-sim roadmap that ends with:

1. deterministic oracle lane,
2. Scrapling crawler/scraper lane,
3. containerized frontier-driven adversary lane,
4. credible representation of current-era automation, crawlers, scrapers, and agentic browsing behavior,
5. and clear integration into tuning and oversight loops.

This should explicitly answer:

1. what remains deterministic and release-blocking,
2. what remains emergent and advisory,
3. what telemetry from each lane is trustworthy enough to drive policy changes,
4. and how run results become tuning evidence rather than mere diagnostics.

Status update (2026-03-20):

1. Captured in [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md).
2. Deterministic traffic is now explicitly treated as oracle, comparator, and curated memory rather than the primary adaptive diagnosis lane.
3. Scrapling and later frontier-agent lanes are now explicitly treated as the primary adaptive discovery inputs.
4. Shared-host work is now narrowed to a minimal scope fence and operator-defined seed gate rather than the whole first adaptive milestone.
5. Traversal telemetry is now the intended adversary-reachable surface map; a rich precomputed public-surface catalog is no longer the default execution model.
6. Reviewed promotion from emergent exploit to deterministic replay case is now an explicit roadmap concept.
7. The guiding rule for future sim, benchmark, and replay planning is now explicit: telemetry is the map.

Status update (2026-03-22):

1. The next maturity gate is no longer just “first recommend-only agent loop exists.”
2. Closed autonomous tuning is now explicitly blocked until Shuma can prove protected tuning evidence and category coverage across the non-human categories it intends to optimize.
3. `synthetic_traffic` remains useful for harness and contract verification, but it must not count as tuning-grade evidence.
4. Raw frontier or LLM discoveries remain advisory until replay promotion or equivalent deterministic confirmation makes their lineage part of the protected tuning evidence set.
5. A new prerequisite now sits even earlier in the chain: Shuma must first define its own canonical non-human taxonomy and classification contract before it can truthfully judge whether Scrapling and frontier or LLM lanes are representative.
6. The near-term adaptive layer should be fingerprinting and categorization quality within that canonical taxonomy; taxonomy expansion is only a later contingency if important traffic persistently falls outside the existing categories.
7. The bridge to a genuine LLM-in-the-loop system is now explicit: bounded LLM adversary modes should land first as category-fulfillment slices behind a pluggable containerized backend, the first closed loop should stop at config tuning and rollback, and only after that should the later LLM diagnosis harness and code-evolution loop reopen.

## B. Tuning Surface Completion

Shuma still needs a plan for completing the Tuning tab as the operator surface for:

1. route thresholds,
2. defence thresholds,
3. ban families and duration policy,
4. recidive policy,
5. intelligence influence thresholds,
6. and the future controller-tunable config families.

This is not a simple UI cleanup task. It is a config-governance and operator-contract task.

The plan must define:

1. which thresholds belong in Tuning,
2. which belong elsewhere,
3. which are read-only diagnostics,
4. and which are safe for future autonomous recommendation or bounded auto-apply.

Status update (2026-03-24):

1. The first concrete Tuning shape is now the taxonomy posture matrix plus optional stance-archetype seeding in [`2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md).
2. Those stance archetypes are operator-facing product presets, not the later recursive-improvement development reference stance contract.
3. The later development reference stance and run-to-homeostasis methodology are now tracked separately in [`../research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](../research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md) and [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md).

## C. Monitoring Overhaul For Operators, Not Contributors

Shuma still needs a monitoring redesign focused on operator questions:

1. where are attackers being effectively intercepted,
2. where are attackers probably getting through,
3. what is the apparent human-friction cost,
4. when `shadow_mode` is active, what Shuma says it would have enforced,
5. what actually happened while enforcement was active,
6. and how should those two telemetry modes stay clearly separated without implying a paired live counterfactual for each request.

This was the previous expectation, but the sequencing has now tightened: the first closed autonomous tuning loop should precede the Monitoring overhaul. Monitoring still needs operator-grade design, but it should now project the proven protected-evidence, category-aware objective, and rollback semantics of that loop rather than define them in advance.

Status update (2026-03-24):

1. Monitoring is now explicitly being redesigned as a loop-accountability surface over the current operator-selected product stance.
2. The later development reference stance should not leak into the first Monitoring contract except, much later, as clearly labeled separate evaluation context.
3. The next ownership refinement should now introduce a dedicated `Traffic` tab so the current traffic-facing Diagnostics surface does not get forced into Monitoring. After that, `DIAG-CLEANUP-1` can narrow Diagnostics to furniture-operational proof.
4. See [`2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md`](2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md), [`2026-03-24-monitoring-reuse-first-diagnostics-cleanup-sequencing-plan.md`](2026-03-24-monitoring-reuse-first-diagnostics-cleanup-sequencing-plan.md), and [`2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md).

## H. Protected Tuning Evidence And Category Coverage

Shuma now needs an explicit plan and execution track for protected autonomous tuning.

That track must answer:

1. which evidence sources are tuning-eligible and which remain harness-only,
2. how Scrapling runtime traffic and replay-promoted frontier or LLM lineage jointly cover the non-human categories Shuma cares about,
3. how operator objectives express desired, tolerated, and unwanted non-human categories,
4. how benchmark contracts fail closed when category coverage is incomplete,
5. and how the first canary apply plus rollback loop proves itself before Monitoring is redesigned around it.

This is now a priority gate ahead of `MON-OVERHAUL-1`.

## I. Canonical Non-Human Taxonomy Before Lane Representativeness

Shuma now also needs an explicit planning and implementation track for canonical non-human traffic taxonomy.

That track must answer:

1. what categories of non-human traffic Shuma intends to model before attackers have been observed,
2. how it classifies both simulated and observed traffic into those categories,
3. how operator intent attaches to those categories,
4. how Scrapling and frontier or LLM lanes are implemented to fulfill them,
5. and only after that how those lanes jointly represent them well enough for tuning.

This is now a prerequisite for the representativeness contract itself, not only a nice-to-have refinement.

That taxonomy may later need a governed expansion path, but that is not a first-loop priority. The higher-priority work is to improve fingerprinting and categorization quality within the seeded category set.

## D. Adversary-Sim Telemetry Retention And Disposal

Shuma still needs a distinct lifecycle policy for adversary-sim telemetry.

Today, the repo has telemetry retention planning in general, but it does not yet fully settle:

1. whether adversary-sim telemetry should retain on the same timescales as real traffic,
2. when sim telemetry is considered actioned and disposable,
3. whether sim telemetry should have separate hot-read and raw retention windows,
4. and what evidence should remain after cleanup for audit without carrying full sim payload history.

This should be planned as its own operating model because the economics and value profile are different from real-traffic telemetry.

## E. Central-Intelligence Storage And Service Architecture

Shuma still needs a dedicated architecture plan for where central intelligence lives.

Questions still open:

1. repo-linked artifact, separate service, or provider-backed managed store,
2. publish/subscribe or periodic snapshot fetch,
3. what must be signed or authenticated,
4. whether data is site-local, fleet-local, or community-shared,
5. how removal and false-positive governance works,
6. and how Shuma consumes the data without coupling runtime decisions to a fragile external dependency.

This must be planned before implementation because the storage, trust, and governance model will shape every later API and telemetry contract.

## F. Verified Bot Identity And Web Bot Auth Foundation

Shuma now has research and design direction for verified bot identity, but its placement in the execution order must be explicit.

This track should formalize:

1. a dedicated verified-identity lane,
2. native Web Bot Auth and HTTP Message Signatures handling,
3. provider-normalized verified-bot and signed-agent inputs,
4. named local allow, restrict, and deny policy for authenticated bots and agents,
5. and lower-cost service profiles for beneficial authenticated agents.

This track must land before:

1. mature adversary-sim,
2. central intelligence,
3. and the scheduled agent operator loop.

Reason:

1. adversary-sim should be able to exercise both legitimate verified agents and spoofed or replayed signed-agent attempts,
2. central intelligence must remain separate from identity and must not mint it,
3. and the future agent loop must not be asked to tune or reason about trust-boundary controls before they are formalized.

## J. Later Recursive-Improvement Methodology

Shuma now also has an explicit methodology for the later recursive-improvement phases, and that methodology should constrain how remaining open roadmap items are interpreted.

That methodology says:

1. later controller expansion should first consume a canonical recursive-improvement game contract defining immutable rules, sacred evaluator, bounded legal move ring, shortfall-attribution policy, and episode archive,
2. later controller expansion should also consume explicit contracts for the judge scorecard, player protocol schemas, held-out evaluation separation, and canonical audit or provenance lineage rather than leaving those details implicit inside role-specific planning,
3. later controller expansion should then begin from `Human-only / private` as a development reference stance,
4. later runs should continue as bounded run-to-homeostasis episodes rather than as one-shot recommendations,
5. later relaxed preset sweeps should come after strict-reference stabilization,
6. later automation should be modeled as an LLM-backed attacker agent, an LLM-backed defender agent, and an independent judge rather than only two frontier-model roles,
7. and later code evolution should keep the strict reference stance as a regression anchor.

This does **not** change:

1. the current seeded product default,
2. the meaning of operator-facing stance presets in `Tuning`,
3. or the product-facing role of `Monitoring`.

It changes only the methodology for the later blocked recursive-improvement phases such as `SIM-LLM-1`, `OVR-AGENT-2`, `OVR-CODE-1`, and `RSI-METH-1`.

## G. Edge-Instance Ban Sync And Distributed State Correctness

Shuma already has planned work for enterprise multi-instance ban synchronization, and this roadmap should state clearly where it fits.

Status update (2026-03-21):

1. This remains valid architecture context for a later edge gateway posture, but it is no longer on the current pre-launch mainline while shared-host remains the supported full control plane.
2. Keep this track conceptually separate from central intelligence, but defer its execution until the later edge gateway plus shared-host control-plane architecture is planned explicitly.

This track is about:

1. one site or one deployment,
2. exact active ban state,
3. converged enforcement across instances,
4. synchronized unban and expiry behavior,
5. and observable sync lag, outage posture, and drift.

This is not the same thing as central intelligence or a centralized worst-offender record.

It should remain separate because:

1. edge-instance ban sync is site-local operational correctness when that deployment shape is actually in scope,
2. central intelligence is cross-site or fleet memory and enrichment,
3. ban sync mirrors exact current enforcement state,
4. while central intelligence should stay advisory-first or high-confidence-feed-driven and must not become a blind active-ban replication channel across unrelated sites.

## H. Scheduled Agent Analyzer, Recommender, And Reconfigurer

Shuma still needs a full plan for the scheduled agentic operator loop:

1. how it is scheduled,
2. which model/runtime stack it uses,
3. what data it reads,
4. what it is allowed to propose,
5. what it is allowed to apply automatically,
6. and whether code-change suggestions or pull requests are part of the same system or a separate one.

This must clearly separate:

1. config tuning recommendations,
2. config auto-apply,
3. code-change recommendations,
4. and code-change execution or PR creation.

Those are not the same risk class and should not be treated as one automation mode.

## H2. First Closed Feedback Loop Completion And Structural Decomposition

Shuma still needs one explicit near-term planning track for the gap between:

1. today's machine-first read contracts plus adversary control,
2. and the actual first closed feedback loop with baseline comparison, persisted objectives, causal decision evidence, and recommend-only reconcile.

This is now also a structural architecture track because the current hotspot files are already too large to keep absorbing more control-loop behavior cleanly.

That track must explicitly sequence:

1. behavior-preserving decomposition of `src/admin/api.rs`, `src/admin/adversary_sim.rs`, `src/observability/operator_snapshot.rs`, `src/observability/benchmark_results.rs`, `src/config/controller_action_surface.rs`, and `scripts/tests/adversarial_simulation_runner.py`,
2. benchmark-history and comparison materialization,
3. persisted `operator_objectives_v1` plus typed verified-identity and decision-evidence snapshot content,
4. replay-promotion lineage integration into backend contracts,
5. the recommend-only reconcile engine,
6. the first shared-host agent tweaker loop over sim-cost and benchmark feedback,
7. and only then Monitoring/Tuning projection plus later broader scheduled-agent planning.

Status update (2026-03-21):

1. Captured in [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md).
2. The corresponding architecture review is [`../research/2026-03-21-feedback-loop-and-architecture-debt-review.md`](../research/2026-03-21-feedback-loop-and-architecture-debt-review.md).
3. The sequencing correction that moves the first machine-first agent tweaker loop ahead of `MON-OVERHAUL-1` is captured in [`../research/2026-03-21-agent-first-feedback-loop-sequencing-review.md`](../research/2026-03-21-agent-first-feedback-loop-sequencing-review.md).
4. The queue is now execution-ready in detailed form via [`2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md`](2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md), [`2026-03-21-agent-first-loop-truth-completion-implementation-plan.md`](2026-03-21-agent-first-loop-truth-completion-implementation-plan.md), and [`2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md`](2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md).

## I. Privacy, State-Minimization, And Final Launch Gates

Shuma also needs explicit late-stage gates that should not be left implicit:

1. deterministic cleanup for stale fingerprint state,
2. optional event-log IP minimization for privacy-sensitive deployments,
3. and a final performance and optimization gate before launch.

These are not optional polish items because:

1. richer monitoring, verified identity, mature adversary-sim, and any later shared intelligence all increase the importance of retention discipline and privacy posture,
2. and a final performance gate is the point where Shuma proves its defences are effective without imposing unacceptable CPU, memory, or latency cost.

# Recommended Sequencing

## Stage 0: Operator-Surface Truth Prerequisites

1. Dashboard connection-state hardening.
2. Clear heartbeat-owned connection-state contract and diagnostics.
3. Separate `GET /admin/config` operational overlays from the writable `POST /admin/config` contract.

Status update (2026-03-17):

1. Delivered. The dashboard now treats the admin-session heartbeat as the sole global connection-state writer, surfaces heartbeat diagnostics and local-failure counters in `Status`, and keeps non-heartbeat request failures local.
2. `GET /admin/config` now returns a split `{ config, runtime }` envelope so writable KV settings and read-only operational/runtime overlays are no longer presented as one flat contract.
3. This stage is now closed; the next execution focus should move to Stage 1 controller-grade monitoring telemetry foundations.

Reason:

1. operator surfaces should be truthful before Shuma invests in a full monitoring overhaul or treats the Tuning tab as the control-plane contract.
2. otherwise the project risks rebuilding operator UX on top of noisy connection state or misleading config semantics.

## Stage 1: Controller-Grade Monitoring Telemetry Foundations

1. Preserve the settled lane, outcome, exactness, and summary-ownership contracts as the one telemetry foundation.
2. Extend request-outcome counters with forwarded-versus-local byte attribution so suspicious origin cost and likely-human friction cost can be benchmarked truthfully.
3. Materialize bounded operator summaries for `response_kind`, `policy_source`, and `route_action_family` instead of leaving those semantics stranded in raw counters.
4. Close the remaining control-path and fail-path request-outcome coverage gaps, or codify and prove intentional exclusions where a path must remain out of scope.
5. Keep adversary-sim traffic first-class but separate from live operator ingress so future benchmarks can compare live and simulated outcomes without polluting one another.

Reason:

1. the recent telemetry-foundation review showed that Shuma now has the right seam, but not yet the benchmark-grade telemetry needed for agentic-era operators or future inside controllers.
2. without controller-grade byte attribution, richer bounded summary semantics, and fuller terminal-path coverage, Monitoring risks becoming visually better while still failing to expose the truths later tuning loops must optimize against.

Status update (2026-03-19):

1. Delivered. Outcome-attributed bytes, bounded `response_kind` and `policy_source` and `route_action_family` summaries, origin-safe follow-up telemetry, and the terminal-path truth boundary are now in place.
2. The backend telemetry foundation is therefore complete enough for the Monitoring overhaul to begin once the section-ownership plan is written.
3. No further broad telemetry architecture sweep is required before `MON-OVERHAUL-1`; the remaining pre-Monitoring work is now the adversary-side truth follow-on that expands Scrapling's request-native category ownership and receipt-backed coverage, the verified-identity calibration follow-on that turns Web Bot Auth and other verified traffic into faithful taxonomy truth and closed-loop guardrails, the host-impact cost-proxy follow-on that upgrades suspicious-origin cost from request-and-byte-only semantics to a settled latency-shaped host-impact proxy, and the small local dashboard surfacing follow-ons that expose verified identity in `Verification` and adversary-sim truth-basis in `Red Team` before the larger human projection is redesigned around those semantics. The Monitoring redesign itself should now be treated as loop-accountability first, with Diagnostics becoming more intentionally diagnostics-focused.

Status update (2026-03-20):

1. Delivered. The section-ownership plan now explicitly splits the legacy subsystem-by-subsystem surface into a new `Diagnostics` tab and reserves `Monitoring` as a clean slate for the operator decision surface.
2. This means the next implementation work can safely move the legacy Monitoring implementation without muddying the eventual operator-facing Monitoring contract.

## Stage 2: Machine-First Monitoring, Tuning, And Objective Loop Foundations

1. Delivered. Monitoring/Diagnostics ownership split so the legacy diagnostic surface now has a truthful home.
2. Define `operator_objectives_v1`, `operator_snapshot_v1`, and `allowed_actions_v1` as the machine-first control contract for both future agents and later human Monitoring surfaces.
3. Define benchmark contracts that will later tell Shuma whether the codebase itself is improving the bot-cost versus human-friction arms race.
4. Build Monitoring as a thin projection over that snapshot, with explicit live versus shadow versus adversary-sim separation.
5. Complete the Tuning tab against the same objective and action model.

Reason:

1. once the controller-grade telemetry foundation exists, Shuma should not jump straight to chart-first Monitoring work; it should first define the machine-readable contract that a future scheduled controller and a later human dashboard will both consume.
2. with the ownership split now implemented, the next work in this stage is the operator snapshot and objective-loop foundation, followed by benchmark criteria for project evolution and then the human Monitoring projection rather than more transition mechanics.

Status update (2026-03-20):

1. Delivered. The machine-first operator snapshot foundation now includes `operator_objectives_v1`, `operator_snapshot_v1`, the bounded `recent_changes` ledger, and `allowed_actions_v1`.
2. The next Stage 2 work is now the benchmark contract tranche, not a human-chart-first Monitoring build.
3. Monitoring overhaul should be treated as a thin projection over `operator_snapshot_v1` plus benchmark-family truth.
4. Shuma should explicitly preserve two later loops: per-instance config tuning and project-level code evolution.
5. Code and PR generation should remain behind a later benchmark-driven planning gate rather than being folded into the first tuning loop.

Status update (2026-03-20, benchmark addendum):

1. Delivered. The static `benchmark_suite_v1` registry now exists as a machine-first backend contract and read surface.
2. Delivered. The first bounded `benchmark_results_v1` envelope now exists as a machine-first backend read surface derived from `operator_snapshot_v1`.
3. Delivered. The first explicit benchmark-driven escalation boundary now exists, separating `config_tuning_candidate`, `observe_longer`, and `code_evolution_candidate` with review-aware trigger and blocker metadata.
4. Delivered. `benchmark_results_v1` is now projected directly into `operator_snapshot_v1`, and the standalone `/admin/benchmark-results` read path now returns that same materialized current-instance contract rather than inventing a second benchmark summary.
5. The first benchmark families stay intentionally small: suspicious-origin cost, likely-human friction, representative adversary effectiveness, and beneficial non-human posture.
6. Delivered. The later fleet or central-intelligence enrichment contract is now defined as a separate advisory layer for scenario selection, family priority, and bounded weight bias rather than another source of benchmark truth.
7. Monitoring is now discussion-ready from the machine-first base: it should project `operator_snapshot_v1` plus nested `benchmark_results_v1` rather than invent a second human-only notion of success or regression.
8. Later central-intelligence architecture should implement the enrichment layer through a real data plane rather than reopening the local benchmark semantics.
9. This keeps the measuring stick for config tuning, future scheduled controllers, and later code evolution aligned from the start.

Status update (2026-03-21):

1. Shuma now explicitly shelves Fermyon as a near-term full-runtime target and treats the shared-host control plane as the pre-launch mainline.
2. Edge/Fermyon and enterprise distributed-state follow-on work therefore move off the mainline sequence and into a later gateway-only side branch.
3. The next pre-launch stages should optimize for verified identity, shared-host adversary maturity, the first machine-first agent loop, and only then Monitoring/Tuning projection rather than edge cron ownership or multi-instance edge convergence.

## Stage 3: Verified Identity Foundation

1. Canonical verified-identity contract.
2. Native Web Bot Auth and HTTP Message Signatures handling.
3. Provider-normalized verified-bot and signed-agent inputs.
4. Named local allow, restrict, and deny policy for authenticated bots and agents.
5. Verified-agent monitoring and policy truth surfaces.

Reason:

1. Shuma should formalize identity before sim, intelligence, or agentic reconfiguration so authentication, authorization, and reputation remain separate from the outset.
2. This gives later sim lanes a truthful beneficial-agent and spoofed-agent target model to test against.
3. It also keeps trust-boundary controls out of later autonomous tuning work until those controls exist explicitly.

## Stage 4: Adversary-Sim Foundations

1. Production adversary-sim operating envelope hardening.
2. Shared-host discovery baseline.

Reason:

1. Shuma should settle production posture, kill-switches, desired-state truth, and no-impact guarantees before broadening the sim.
2. realistic Scrapling and containerized lanes should start from a truthful discovered public-surface baseline rather than ad hoc targets.

## Stage 5: Mature Adversary-Sim As A Tuning Input

1. Scrapling lane.
2. Containerized frontier lane as a bounded emergent actor.
3. Verified-agent, spoofed-agent, and replay-attempt scenarios against the identity lane.
4. Explicit mapping from each lane's evidence to tuning confidence.

Reason:

1. Shuma needs realistic attacker input before automated tuning can be trusted to optimize against the actual agentic threat landscape.

## Stage 6: First Machine-First Agent Tweaker Loop

1. Build the first shared-host agent tweaker harness over `operator_snapshot_v1`, nested `benchmark_results_v1`, `allowed_actions_v1`, decision evidence, and adversary-sim cost outcomes.
2. Support both periodic execution and immediate post-sim triggering through one internal reconcile or agent contract rather than separate controller implementations.
3. Keep the first agent loop recommend-only and typed: it should emit bounded config proposals, evidence references, and explicit no-change or rerun outcomes rather than ad hoc prose.
4. Use that loop to prove which benchmark deltas, watch outcomes, replay-promotion signals, and rollback references actually matter.

Reason:

1. the system should work for agents first, so the next truthful milestone after reconcile is the backend loop that actually reads sim evidence and produces bounded tuning proposals.
2. once that loop exists, Monitoring and Tuning can be designed around demonstrated machine-first semantics rather than guessed operator desires.

## Stage 7: Monitoring Projection And Tuning Surface Completion

1. Build Monitoring as a thin projection over `operator_snapshot_v1`, nested `benchmark_results_v1`, decision evidence, and the first working agent loop.
2. Complete the Tuning tab as the bounded operator control surface over the same machine-first contract and proven patch families.
3. Keep live, shadow, adversary-sim, benchmark, proposal, and watch semantics visibly separate so the human surfaces reflect the loop that actually exists.

Reason:

1. once the first agent loop is real, Monitoring can surface exactly what the backend uses and Tuning can expose exactly the safe action families that loop has proven useful.
2. this avoids turning either dashboard surface into a speculative semantic model.

## Stage 8: Sim-Telemetry Lifecycle

1. Separate retention and disposal policy for sim telemetry.
2. Clear distinction between actioned and unactioned sim evidence.
3. Audit residue kept minimal but sufficient.

Reason:

1. once emergent and frontier lanes exist, sim telemetry volume and cost will matter much more.

## Stage 9: Privacy And State-Minimization Gate

1. Deterministic cleanup for stale fingerprint state.
2. Optional event-log IP minimization mode and explicit tradeoff docs.
3. Re-check telemetry retention posture against the richer evidence surfaces now in play.

Reason:

1. before central intelligence or later autonomous oversight, Shuma should have a cleaner answer for what state persists, how long it persists, and when raw IP retention is actually necessary.

## Stage 10: Central Intelligence Architecture

1. Storage and service architecture.
2. Governance and false-positive process.
3. Observe-only ingest first.
4. Advisory usage before stronger enforcement.

Reason:

1. external and shared memory should not be wired into runtime policy before its trust and blast-radius model is explicit.
2. this must follow verified identity so Shuma keeps "who is this?" separate from "what does outside reputation say about it?"
3. this must also follow the shared-host-first benchmark and policy loop so exact local active enforcement and bounded benchmark truth are already settled.

## Stage 11: Later Scheduled And Autonomous Agent Expansion

1. Broaden the first agent loop into later always-on scheduling, if still needed after the shared-host-first loop proves out.
2. Add narrow config auto-apply with canary and rollback only after the recommend-only path and human projections are trustworthy.
3. Keep code-change recommendation as a separate path from bounded config tuning.
4. Only later, if ever, allow a PR-generating path with stricter review gates.

Reason:

1. Shuma should first prove the bounded shared-host agent loop and then expand from that proven base rather than plan the broader autonomous system in the abstract.
2. later expansion should still stand on explicit identity policy, mature sim evidence, Monitoring/Tuning projection, and central-intelligence governance.

## Stage 12: Final Pre-Launch Performance Gate

1. Execute the final performance and optimization pass.
2. Enforce bundle, latency, CPU, memory, and high-cost request-path acceptance thresholds.
3. Lock release thresholds and final launch criteria.

Reason:

1. this is the point where Shuma proves the complete system is both effective and efficient enough to launch.

# Recommended Design Calls To Lock Early

1. Keep request-path logic deterministic and Rust-only.
2. Harden dashboard connection-state truth and admin-config truth before relying on those surfaces as operator control planes.
3. Treat controller-grade monitoring telemetry foundations as a prerequisite for both the Monitoring overhaul and any future bounded benchmark/controller loop.
4. Treat the first machine-first agent tweaker loop as a prerequisite for the Monitoring overhaul and any later autonomous tuning expansion.
5. Treat tuning-tab completion as a control-plane contract, not a cosmetic dashboard task.
6. Keep edge-gateway distributed-state work off the shared-host-first pre-launch mainline until a later gateway-only architecture is planned explicitly.
7. Formalize verified bot identity before mature sim, central intelligence, or scheduled agentic reconfiguration so identity, authorization, and reputation are separated cleanly.
8. Settle production adversary-sim posture and the minimal shared-host scope-and-seed gate before expanding emergent lanes.
9. Keep adversary-sim telemetry retention distinct from real-traffic retention.
10. Add a privacy and state-minimization gate before central intelligence so richer telemetry and shared-memory work do not outpace retention discipline.
11. Treat central intelligence as a separate service or data plane concern, not a side effect of the Git repository.
12. Keep config auto-tuning and code-change/PR generation as separate systems with separate permissions and review paths.
13. Treat the final performance gate as a release stage, not a cleanup afterthought.

# Side Branches, Not Mainline Sequence

These items should remain explicitly off the mainline sequence:

1. later edge gateway plus shared-host control-plane split architecture, including any edge-local distributed-state correctness work
   - only after the shared-host-first pre-launch loop is operating cleanly and the later edge architecture is planned explicitly;
2. optional asynchronous mirroring of high-confidence bans to Akamai Network Lists
   - only after a later edge-gateway architecture and enterprise distributed-state baseline are re-committed intentionally;
3. external breach to replayable attack pipeline
   - only after the first emergent lanes are producing stable exploit findings, mature adversary-sim is established, and retention governance is in place.

# Roadmap Outcome

This roadmap suggests that the next pre-launch excellence sequence should be:

1. operator-surface truth prerequisites,
2. controller-grade monitoring telemetry foundations,
3. machine-first monitoring, tuning, and objective-loop foundations,
4. verified bot identity and Web Bot Auth foundation,
5. adversary-sim foundations,
6. mature adversary-sim lanes,
7. first machine-first agent tweaker loop,
8. monitoring projection and tuning surface completion,
9. sim-telemetry retention lifecycle,
10. privacy and state-minimization gate,
11. central-intelligence architecture,
12. later scheduled and autonomous agent expansion,
13. final pre-launch performance gate.

That order makes the future autonomous loop far more likely to be truthful, low-risk, and actually useful.
