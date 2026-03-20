Date: 2026-03-16
Status: Roadmap capture

Related context:

- [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`2026-03-15-agentic-era-oversight-design.md`](2026-03-15-agentic-era-oversight-design.md)
- [`2026-03-15-agentic-era-oversight-implementation-plan.md`](2026-03-15-agentic-era-oversight-implementation-plan.md)
- [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)
- [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../observability.md`](../observability.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Purpose

Capture the major pre-launch work that Shuma still needs, but that is not yet fully planned in execution-ready detail, so implementation can be sequenced deliberately rather than opportunistically.

This note is intentionally a roadmap and sequencing capture, not an implementation-ready plan for every item listed.

# What Is Already Partially Planned

## 1. Adversary-sim maturation is started, but not complete

Already captured:

1. shared-host discovery first,
2. Scrapling surface catalog work,
3. blocked Scrapling runtime lane,
4. blocked containerized LLM lane,
5. deterministic oracle governance,
6. frontier data-governance work.

Current references:

1. [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
2. `SIM-SH-SURFACE-1` in [`../../todos/todo.md`](../../todos/todo.md)
3. `SIM-SCR-LANE-1` and `SIM-LLM-1` in [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

Gap:

1. there is not yet one mature end-state roadmap that ties deterministic, Scrapling, and containerized frontier lanes to the future tuning and oversight loop as one operating system.

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

## C. Monitoring Overhaul For Operators, Not Contributors

Shuma still needs a monitoring redesign focused on operator questions:

1. where are attackers being effectively intercepted,
2. where are attackers probably getting through,
3. what is the apparent human-friction cost,
4. when `shadow_mode` is active, what Shuma says it would have enforced,
5. what actually happened while enforcement was active,
6. and how should those two telemetry modes stay clearly separated without implying a paired live counterfactual for each request.

This is a foundational prerequisite for autonomous tuning. If monitoring remains contributor-diagnostic rather than operator-decisional, the controller will lack the right evidence surface.

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

## G. Edge-Instance Ban Sync And Distributed State Correctness

Shuma already has planned work for enterprise multi-instance ban synchronization, and this roadmap should state clearly where it fits.

This track is about:

1. one site or one deployment,
2. exact active ban state,
3. converged enforcement across instances,
4. synchronized unban and expiry behavior,
5. and observable sync lag, outage posture, and drift.

This is not the same thing as central intelligence or a centralized worst-offender record.

It should remain separate because:

1. edge-instance ban sync is site-local operational correctness,
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
3. No further telemetry architecture sweep is required before `MON-OVERHAUL-1`; the next work should move to Monitoring surface design and ownership planning.

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

1. The next Stage 2 work is now explicitly the machine-first operator snapshot foundation, not a human-chart-first Monitoring build.
2. Monitoring overhaul should be treated as a thin projection over `operator_snapshot_v1`.
3. Shuma should explicitly preserve two later loops: per-instance config tuning and project-level code evolution.
4. The scheduled controller planning should remain blocked until that snapshot contract and the later tuning-action contract exist.
5. Code and PR generation should remain behind a later benchmark-driven planning gate rather than being folded into the first tuning loop.

## Stage 3: Edge-Instance Ban Sync And Distributed State Correctness

1. Strict distributed ban-store mode for enterprise authoritative operation.
2. Ban and unban convergence observability and sync-lag truth surfaces.
3. Two-instance and failure-mode verification for convergence and outage posture.

Reason:

1. multi-instance enforcement correctness is a deployment-local prerequisite, not an optional intelligence feature.
2. mature multi-instance adversary-sim should not be trusted until Shuma can prove that locally issued bans converge across instances.
3. this keeps the exact current-enforcement plane separate from later shared-intelligence work.

## Stage 4: Verified Identity Foundation

1. Canonical verified-identity contract.
2. Native Web Bot Auth and HTTP Message Signatures handling.
3. Provider-normalized verified-bot and signed-agent inputs.
4. Named local allow, restrict, and deny policy for authenticated bots and agents.
5. Verified-agent monitoring and policy truth surfaces.

Reason:

1. Shuma should formalize identity before sim, intelligence, or agentic reconfiguration so authentication, authorization, and reputation remain separate from the outset.
2. This gives later sim lanes a truthful beneficial-agent and spoofed-agent target model to test against.
3. It also keeps trust-boundary controls out of later autonomous tuning work until those controls exist explicitly.

## Stage 5: Adversary-Sim Foundations

1. Production adversary-sim operating envelope hardening.
2. Shared-host discovery baseline.

Reason:

1. Shuma should settle production posture, kill-switches, desired-state truth, and no-impact guarantees before broadening the sim.
2. realistic Scrapling and containerized lanes should start from a truthful discovered public-surface baseline rather than ad hoc targets.

## Stage 6: Mature Adversary-Sim As A Tuning Input

1. Scrapling lane.
2. Containerized frontier lane as a bounded emergent actor.
3. Verified-agent, spoofed-agent, and replay-attempt scenarios against the identity lane.
4. Multi-instance convergence scenarios against enterprise ban-sync posture.
5. Explicit mapping from each lane's evidence to tuning confidence.

Reason:

1. Shuma needs realistic attacker input before automated tuning can be trusted to optimize against the actual agentic threat landscape.

## Stage 7: Sim-Telemetry Lifecycle

1. Separate retention and disposal policy for sim telemetry.
2. Clear distinction between actioned and unactioned sim evidence.
3. Audit residue kept minimal but sufficient.

Reason:

1. once emergent and frontier lanes exist, sim telemetry volume and cost will matter much more.

## Stage 8: Privacy And State-Minimization Gate

1. Deterministic cleanup for stale fingerprint state.
2. Optional event-log IP minimization mode and explicit tradeoff docs.
3. Re-check telemetry retention posture against the richer evidence surfaces now in play.

Reason:

1. before central intelligence or later autonomous oversight, Shuma should have a cleaner answer for what state persists, how long it persists, and when raw IP retention is actually necessary.

## Stage 9: Central Intelligence Architecture

1. Storage and service architecture.
2. Governance and false-positive process.
3. Observe-only ingest first.
4. Advisory usage before stronger enforcement.

Reason:

1. external and shared memory should not be wired into runtime policy before its trust and blast-radius model is explicit.
2. this must follow verified identity so Shuma keeps "who is this?" separate from "what does outside reputation say about it?"
3. this must also follow edge-instance ban sync so the exact local active-ban plane is already cleanly separated from cross-site reputation and worst-offender memory.

## Stage 10: Scheduled Agent Operator Loop

1. Recommend-only scheduled agent.
2. Narrow config auto-apply with canary and rollback.
3. Separate code-change recommendation path.
4. Only later, if ever, a PR-generating path with stricter review gates.

Reason:

1. the agent loop should stand on truthful monitoring, explicit identity policy, deployment-local sync correctness, mature sim evidence, tuned config surfaces, and explicit central-intelligence governance.

## Stage 11: Final Pre-Launch Performance Gate

1. Execute the final performance and optimization pass.
2. Enforce bundle, latency, CPU, memory, and high-cost request-path acceptance thresholds.
3. Lock release thresholds and final launch criteria.

Reason:

1. this is the point where Shuma proves the complete system is both effective and efficient enough to launch.

# Recommended Design Calls To Lock Early

1. Keep request-path logic deterministic and Rust-only.
2. Harden dashboard connection-state truth and admin-config truth before relying on those surfaces as operator control planes.
3. Treat controller-grade monitoring telemetry foundations as a prerequisite for both the Monitoring overhaul and any future bounded benchmark/controller loop.
4. Treat monitoring overhaul as a prerequisite for serious autonomous tuning.
5. Treat tuning-tab completion as a control-plane contract, not a cosmetic dashboard task.
6. Treat edge-instance ban sync as deployment-local state correctness and schedule it before mature multi-instance sim or cross-site intelligence work.
7. Formalize verified bot identity before mature sim, central intelligence, or scheduled agentic reconfiguration so identity, authorization, and reputation are separated cleanly.
8. Settle production adversary-sim posture and shared-host discovery before expanding emergent lanes.
9. Keep adversary-sim telemetry retention distinct from real-traffic retention.
10. Add a privacy and state-minimization gate before central intelligence so richer telemetry and shared-memory work do not outpace retention discipline.
11. Treat central intelligence as a separate service or data plane concern, not a side effect of the Git repository.
12. Keep config auto-tuning and code-change/PR generation as separate systems with separate permissions and review paths.
13. Treat the final performance gate as a release stage, not a cleanup afterthought.

# Side Branches, Not Mainline Sequence

These items should remain explicitly off the mainline sequence:

1. optional asynchronous mirroring of high-confidence bans to Akamai Network Lists
   - only after `DEP-ENT-1..5` establish the enterprise distributed-state baseline;
2. external breach to replayable attack pipeline
   - only after shared-host discovery, mature adversary-sim, and retention governance are established.

# Roadmap Outcome

This roadmap suggests that the next pre-launch excellence sequence should be:

1. operator-surface truth prerequisites,
2. controller-grade monitoring telemetry foundations,
3. operator-grade monitoring and tuning surfaces,
4. edge-instance ban sync and distributed state correctness,
5. verified bot identity and Web Bot Auth foundation,
6. adversary-sim foundations,
7. mature adversary-sim lanes,
8. sim-telemetry retention lifecycle,
9. privacy and state-minimization gate,
10. central-intelligence architecture,
11. scheduled agent analyzer and reconfigurer,
12. final pre-launch performance gate.

That order makes the future autonomous loop far more likely to be truthful, low-risk, and actually useful.
