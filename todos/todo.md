# TODO Roadmap

Last updated: 2026-03-30

This is the active execution-ready work queue.
Blocked or contingent work lives in `todos/blocked-todo.md`.
Completed work lives in `todos/completed-todo-history.md`.
Security finding validity and closure status live in `todos/security-review.md`.
Keep durable workflow and policy guidance in `docs/project-principles.md`, `CONTRIBUTING.md`, and `AGENTS.md`, not in this file.

## P0 Monitoring and Config Lifecycle Stabilization

Reference context:
- [`docs/plans/2026-02-26-adversarial-simulation-v2-plan.md`](../docs/plans/2026-02-26-adversarial-simulation-v2-plan.md)
- [`docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md`](../docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md)
- [`docs/plans/2026-03-11-telemetry-storage-query-efficiency-excellence-plan.md`](../docs/plans/2026-03-11-telemetry-storage-query-efficiency-excellence-plan.md)
- [`docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](../docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md)
- [`docs/plans/2026-03-13-compact-event-telemetry-implementation-plan.md`](../docs/plans/2026-03-13-compact-event-telemetry-implementation-plan.md)
- [`docs/plans/2026-03-14-telemetry-retention-rebaseline-implementation-plan.md`](../docs/plans/2026-03-14-telemetry-retention-rebaseline-implementation-plan.md)
- [`docs/plans/2026-03-12-shadow-mode-telemetry-monitoring-truthfulness-plan.md`](../docs/plans/2026-03-12-shadow-mode-telemetry-monitoring-truthfulness-plan.md)
- [`docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`](../docs/research/2026-03-14-compact-event-telemetry-live-evidence.md)
- [`docs/configuration.md`](../docs/configuration.md)
- [`docs/testing.md`](../docs/testing.md)

### SIM2-R4-4: Config Seeding Lifecycle and Shadow-Mode Semantics

## P0 Monitoring Telemetry Foundations

Reference context:
- [`docs/research/2026-03-17-operator-decision-support-telemetry-audit.md`](../docs/research/2026-03-17-operator-decision-support-telemetry-audit.md)
- [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`docs/research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../docs/research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](../docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`docs/plans/2026-03-18-monitoring-traffic-lane-and-denominator-contract.md`](../docs/plans/2026-03-18-monitoring-traffic-lane-and-denominator-contract.md)
- [`docs/plans/2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](../docs/plans/2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md)
- [`docs/plans/2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md`](../docs/plans/2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md)
- [`docs/plans/2026-03-19-monitoring-human-friction-denominator-plan.md`](../docs/plans/2026-03-19-monitoring-human-friction-denominator-plan.md)
- [`docs/plans/2026-03-19-monitoring-defence-funnel-plan.md`](../docs/plans/2026-03-19-monitoring-defence-funnel-plan.md)
- [`docs/plans/2026-03-18-monitoring-operator-summary-exactness-contract.md`](../docs/plans/2026-03-18-monitoring-operator-summary-exactness-contract.md)
- [`docs/plans/2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md`](../docs/plans/2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md)
- [`docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](../docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md)
- [`docs/plans/2026-03-13-compact-event-telemetry-implementation-plan.md`](../docs/plans/2026-03-13-compact-event-telemetry-implementation-plan.md)
- [`docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`](../docs/research/2026-03-14-compact-event-telemetry-live-evidence.md)

### MON-TEL-1: Cost-Aware Operator Telemetry Foundation
- Prerequisite foundation landed on 2026-03-18:
  - reuse the settled contracts in [`docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](../docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md) and its linked lane, outcome, exactness, and ownership notes while implementing the counter, summary, and read-surface parts of this tranche.
- Controller-readiness review addendum (2026-03-19):
  - treat the remaining telemetry work below as first-order foundation, not later polish, because future operator benchmarks and bounded inside-controller loops will need truthful byte attribution, richer bounded backend summaries, and fuller control/fail-path outcome coverage before the Monitoring UI overhaul should start.
  - reference: [`docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md)

## P0 Machine-First Operator Snapshot Foundations

Reference context:
- [`docs/research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](../docs/research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
- [`docs/research/2026-03-20-benchmark-suite-v1-research-synthesis.md`](../docs/research/2026-03-20-benchmark-suite-v1-research-synthesis.md)
- [`docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-implementation-plan.md`](../docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-implementation-plan.md)
- [`docs/plans/2026-03-20-benchmark-suite-v1-design.md`](../docs/plans/2026-03-20-benchmark-suite-v1-design.md)
- [`docs/plans/2026-03-20-benchmark-suite-v1-implementation-plan.md`](../docs/plans/2026-03-20-benchmark-suite-v1-implementation-plan.md)
- [`docs/plans/2026-03-15-agentic-era-oversight-design.md`](../docs/plans/2026-03-15-agentic-era-oversight-design.md)
- [`docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

## P0 First Closed Feedback Loop And Control-Plane Restructuring

Reference context:
- [`docs/research/2026-03-21-feedback-loop-and-architecture-debt-review.md`](../docs/research/2026-03-21-feedback-loop-and-architecture-debt-review.md)
- [`docs/research/2026-03-21-loop-closure-execution-readiness-review.md`](../docs/research/2026-03-21-loop-closure-execution-readiness-review.md)
- [`docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`docs/plans/2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md`](../docs/plans/2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md)
- [`docs/plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md`](../docs/plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md)
- [`docs/plans/2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md`](../docs/plans/2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md)
- [`docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`docs/plans/2026-03-20-benchmark-suite-v1-design.md`](../docs/plans/2026-03-20-benchmark-suite-v1-design.md)
- [`docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`docs/plans/2026-03-15-agentic-era-oversight-design.md`](../docs/plans/2026-03-15-agentic-era-oversight-design.md)
- [`docs/plans/2026-03-15-agentic-era-oversight-implementation-plan.md`](../docs/plans/2026-03-15-agentic-era-oversight-implementation-plan.md)

Current stance:
- Observation and adversary generation are now ahead of objective, comparison, attribution, and recommend-only reconcile truth.
- Execute the structural decomposition tranches before landing more controller or operator-loop logic into the current hotspot files.
- Keep the decomposition slices behavior-preserving; semantic expansion begins only after the hotspot modules have focused seams.
- Land the first shared-host agent tweaker loop before `MON-OVERHAUL-1` so Monitoring and Tuning can project proven machine-first semantics instead of inventing them early.
- Treat the verified-identity observability and benchmarking gap as part of this loop-closure track rather than later polish.
- The first bounded shared-host closed config loop is now live-proven, and the adversary-sim status diagnostics follow-up is now delivered.
- Before `MON-OVERHAUL-1`, settle Scrapling's truthful request-native category ownership and proof so Monitoring does not project the older indexing-only lane semantics.
- The verified-identity calibration and no-harm guardrail track is now delivered before `MON-OVERHAUL-1`, so Monitoring can project the settled taxonomy-alignment, conflict-metric, and fail-closed guardrail semantics instead of the earlier flattened Web Bot Auth path.
- The next prerequisite now begins one step earlier: Shuma must first define a canonical non-human taxonomy and shared classification contract before it can truthfully measure lane representativeness.
- The actively evolving near-term layer should be fingerprinting and categorization quality inside that taxonomy; taxonomy expansion is only a later contingency if important non-human traffic persistently falls outside the existing categories.
- The next bridge to a genuine LLM-in-the-loop system is not the full later `SIM-LLM-1` runtime actor; it is a bounded category-fulfillment LLM tranche that sits between the taxonomy/classifier work and the first closed config loop, with the later LLM diagnosis harness and code loop still downstream of `OVR-APPLY-1`.
- The active mainline is now reprioritized: before further operator-surface cleanup or later player-side game-loop execution, make Scrapling attacker-faithful for the defense surfaces it owns, prove that coverage with receipts, and then run the first explicit self-improving loop over that truthful attacker basis.
- Before `MON-OVERHAUL-1`, expose the already-settled local control truths that no longer belong only in Advanced JSON or backend-only payloads. Verified identity in `Verification` and adversary-sim status truth basis in `Red Team` are now delivered, but the immediate mainline now moves first through attacker-faithful Scrapling and the first working game loop rather than dashboard cleanup.
- Keep the operator-facing product stance distinct from the later recursive-improvement development reference stance: Monitoring should project the current operator-selected posture truthfully, while run-to-homeostasis episodes remain blocked with `OVR-AGENT-2` and `RSI-METH-1`. Do not treat the retired March 23-24 Tuning expansion chain as active roadmap.

## P0 Attacker-Faithful Scrapling And First Game Loop

Reference context:
- [`docs/research/2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md`](../docs/research/2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md)
- [`docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](../docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`docs/research/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md`](../docs/research/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md)
- [`docs/plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md`](../docs/plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md)
- [`docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`docs/research/2026-03-24-rsi-game-mainline-first-working-loop-review.md`](../docs/research/2026-03-24-rsi-game-mainline-first-working-loop-review.md)
- [`docs/plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../docs/plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`docs/research/2026-03-30-adversary-lane-traffic-realism-and-cadence-review.md`](../docs/research/2026-03-30-adversary-lane-traffic-realism-and-cadence-review.md)
- [`docs/plans/2026-03-30-adversary-lane-traffic-realism-and-cadence-plan.md`](../docs/plans/2026-03-30-adversary-lane-traffic-realism-and-cadence-plan.md)

Current note:
- The generated contributor-site chain `SIM-PUBSITE-1A` through `SIM-PUBSITE-1D` is now landed.
- The next corrective architecture priority is now `ROUTE-NS-1A..1F`: the landed contributor-generated site and current control-plane routes still assume `/sim/public/*` and top-level `/admin|/dashboard|/health|/metrics`, but the corrected architecture is root-hosted public content plus `/shuma/*` for Shuma-owned control and operational surfaces.
- Do not start new mixed-attacker proof or later tuning-quality work ahead of that resequenced chain unless a higher-severity regression forces an interruption.
- `SIM-SCR-CHALLENGE-2A`, `SIM-SCR-CHALLENGE-2B`, and `SIM-SCR-CHALLENGE-2D` are landed.
- `RSI-GAME-1A`, `RSI-GAME-1B`, `RSI-SCORE-1`, and `RSI-GAME-1C` are landed.
- `RSI-GAME-MAINLINE-1A` and `RSI-GAME-MAINLINE-1B` are landed.
- The current attacker-faithful Scrapling prerequisite for owned request-native surfaces is satisfied through `SIM-SCR-CHALLENGE-2A`, `SIM-SCR-CHALLENGE-2B`, and `SIM-SCR-CHALLENGE-2D`, but that is now the baseline rather than the full maturity target.
- `SIM-SCR-CAP-1` is now landed: it froze the upstream capability matrix and omission ledger, and the request-native attacker-fidelity follow-on `SIM-SCR-RN-1` is now landed too.
- `SIM-SCR-RN-1` is now landed: the request-native Scrapling lane explicitly pins attacker-faithful Chrome impersonation and stealthy header shaping while no longer advertising itself with an internal worker `User-Agent`.
- `SIM-SCR-FULL-1B1` and `SIM-SCR-FULL-1B2` are now landed: Scrapling now owns `automated_browser`, uses dynamic-browser and stealth-browser personas against `maze_navigation`, `js_verification_execution`, and `browser_automation_detection`, and carries optional request and browser proxy plan support with local proof.
- `SIM-SCR-CHALLENGE-2C` remains blocked only for residual post-matrix browser or stealth follow-on if later receipts show the adopted browser or stealth runtime is still insufficient for a Scrapling-owned surface.
- `SIM-LLM-1A` and `SIM-LLM-1B` are now landed.
- The later full attacker runtime `SIM-LLM-1C` is no longer treated as one implicit next step.
- `SIM-LLM-1C1` is now landed: the later attacker has a real live frontier action-generation seam with provider-vs-fallback lineage and Shuma-blind host-hint sanitization.
- `SIM-LLM-1C2` is now landed: the host-side supervisor dispatches the dedicated LLM runtime worker, reuses the existing container black-box runner for request-mode execution, and ingests a typed `adversary-sim-llm-runtime-result.v1` payload instead of overloading the Scrapling worker result contract.
- The current full-spectrum Scrapling baseline and first working Game Loop proof are now treated as baseline capability, not the unlock condition for later stance relaxation or LLM runtime work.
- `SIM-SCR-FULL-1` is now landed: full-spectrum Scrapling capability, receipt-backed category and defense-surface truth, controller-grade surface-contract scoring, and the operator truth audit are complete, so the next mainline moves to `RSI-SCORE-2`.
- `RSI-SCORE-2` is now landed: the judge, diagnoser, move selector, and Game Loop projection now preserve exploit progress, evidence quality, urgency, named breach loci, and config-exhaustion or code-referral truth as separate planes.
- `RSI-GAME-HO-1` is now landed: the strict `human_only_private` loop is proven on the local `/sim/public/*` surface with live strict-stance runtime checks, post-sim oversight lineage, and repeated retained movement toward zero suspicious leakage.
- A March 27 accountability slice is now landed: Game Loop category posture rows render honestly as `Unscored` when exact shared-path evidence is missing, exploit loci now carry host-cost channels plus repair-family hints, localized high-confidence exploit progress can now drive bounded config tuning, and the Game Loop page now projects origin leakage, board-state breach progress, and loop actionability as separate planes.
- The later March 27 architecture clarification now tightens the Game Loop shape further: restriction scoring is the main quest, recognition quality is a separate evaluation quest, simulator-known category labels remain forbidden in runtime and tuning, and abuse-driven confidence escalation is required as the backstop when explicit hostile identity signals stay weak.
- The remaining immediate Game Loop gaps are therefore no longer just "exact category inference later". They are: vague `Loop Actionability` blocker output, vague `Named Breach Loci`, and a deeper architecture problem where snapshot, benchmark, controller, and dashboard layers still re-entangle recognition evaluation, restriction scoring, and board-state truth through the older family-first model.
- `RSI-SCORE-2F2` is now landed: the recognition-evaluation rail truthfully counts current collapse to `unknown_non_human` as a Shuma inference outcome and no longer lets harness-only `projected_recent_sim_run` placeholders masquerade as degraded category matches.
- `RSI-GAME-ARCH-1B` is now landed: category posture is no longer a primary optimization target or main-loop overall-status trigger for undeclared hostile traffic, and the Game Loop now projects it as `Recognition Evaluation` rather than as the main restriction scoreboard.
- `RSI-SCORE-2F3` is now landed: the Game Loop and benchmark contract explicitly surface `Restriction Confidence` and `Abuse Backstop`, and the Make-driven proof path now executes dedicated urgency tests for that model.
- `RSI-GAME-ARCH-1C`, `RSI-GAME-ARCH-1D`, `RSI-GAME-BOARD-1F`, and `RSI-GAME-BOARD-1G` are now landed: controller diagnosis, recognition evaluation, and move selection are explicit sibling contracts, `Loop Actionability` now groups root causes, controller outcomes, and next-fix surfaces, and breach loci now preserve materialization truth instead of fabricating `0 attempts` or generic missing-data wording.

- A fresh live audit on 2026-03-27 first exposed, and the repo has now fixed, the live protected-evidence blocker: `RSI-GAME-ARCH-1G` is now landed, so strong live Scrapling runtime pressure can count as protected tuning evidence without replay-lineage materialization.
- A same-day follow-on live audit exposed one remaining controller inconsistency after `RSI-GAME-ARCH-1G`: the benchmark and patch-policy layers now accept `live_scrapling_runtime` as protected tuning evidence, but reconcile and canary-apply still fail closed on stale `replay_promotion` metadata because the stale-input guard still reflects the older replay-only architecture.
- A second same-day live audit exposed the next blocker after that stale-guard repair: the live local strict Scrapling loop now reaches `recommend_patch`, but seeded rollout guardrails still stop at `apply.stage=refused` because runtime-dev defaults remain `automated_apply_status=manual_only`.
- A third same-day live audit exposed the next blocker after that rollout-mode repair: the local strict Scrapling loop could apply a bounded canary and truthfully enter `watch_window_open`, but the operator-owned profile still declared `window_hours=24`, so live retain-vs-rollback judgment was still blocked by cadence rather than by controller correctness.
- `RSI-GAME-ARCH-1J` is now landed: runtime-dev has an explicit effective watch-window cadence override, machine-first surfaces expose effective versus declared cadence truth, and the local strict Scrapling loop can now reach real terminal judgment without waiting a full day.
- `RSI-GAME-ARCH-1K` is now landed: canary apply persists explicit candidate-window lifecycle state, adversary-sim supervisor auto-materializes exactly one protected post-change Scrapling follow-on run, periodic judgment can consume that evidence instead of fail-closing as `candidate_window_not_materialized`, and runtime-dev uses the shortest meaningful `30s` follow-on window for that local proof path.
- `RSI-GAME-ARCH-1L` is now landed: terminal `improved` and `rollback_applied` judgments that remain outside budget persist one fresh bounded Scrapling continuation rerun request, adversary-sim supervisor auto-materializes that rerun, only the later post-rerun oversight judgment may open the next bounded canary, and the shared-host loop now advances as `judge -> rerun -> judge -> next bounded move` until an explicit stop condition is reached.
- The board-state doctrine now has explicit follow-on planning for a later frontier-LLM code-evolution ring and a later real-human friction calibration ring, but both remain blocked from execution.
- Before further mainline completion claims, the repo now requires explicit acceptance-gate discipline. Do not describe `STANCE-MODEL-1`, `SIM-SCR-FULL-1`, `RSI-GAME-HO-1`, or `RSI-GAME-HO-2` as complete from planning progress, baseline capability, or dashboard pressure signals alone.
- `SIM-LLM-1C3` is now landed: the later LLM attacker no longer disappears after runtime ingest, and recent-run or operator surfaces now project truthful additive `bot_red_team` runtime lineage without enabling the lane in controls.
- `SIM-REALISM-1` is now the next adversary-sim maturity chain: both Scrapling and Agentic Traffic still need profile-driven cadence, burst, dwell, and identity or session realism before mixed-attacker or tuning claims are called representative.
- `SIM-REALISM-1A` is now landed: the shared versioned realism-profile contract is frozen across Rust planners, Python workers, and Make-driven proof.
- `SIM-REALISM-1B` is now landed: Scrapling personas now execute profile-driven pacing and dwell behavior, emit per-tick `realism_receipt` payloads, and preserve the latest Scrapling realism receipt in recent-run monitoring history. The next active execution priority is now the contributor-generated site chain because realism should continue against a richer public terrain than the current thin dummy surface.
- `SIM-REALISM-1C` is now landed: Agentic Traffic request-mode now executes profile-driven focused micro-bursts and between-burst pauses, the container black-box worker emits typed request-mode `realism_receipt` payloads with focused-page-set and stop-reason truth, and recent-run monitoring history preserves the latest Agentic realism receipt alongside additive LLM runtime lineage. The next active execution priority is now `SIM-REALISM-1D`.
- `SIM-REALISM-1D` is now landed: Agentic Traffic browser-mode now emits a real Playwright-driven black-box session, follows public hints from root through `robots.txt` and sitemap discovery instead of using hidden route knowledge, emits browser-shaped `realism_receipt` payloads with stable session and dwell truth, and preserves that browser receipt in recent-run monitoring history. The next active execution priority is now `SIM-REALISM-2A`.
- `SIM-REALISM-2A` is now landed: request-native Scrapling personas and Agentic Traffic request-mode now consume explicit per-profile pressure envelopes instead of collapsing to the old flat `8 requests / 2 seconds` ceiling, Agentic request-mode now executes bounded concurrent bursts rather than serializing every micro-burst into one file of requests, and both Rust and Python receipt paths preserve peak concurrency and effective cadence truth.
- `SIM-REALISM-2B` is now landed: Scrapling and Agentic Traffic now carry a bounded identity-envelope contract, planners can emit pool-backed request or browser identities, and observer-only realism receipts now distinguish `pool_backed`, `fixed_proxy`, and `degraded_local` identity posture without leaking simulator provenance into Shuma defence truth.
- `SIM-REALISM-2C` is now landed: the shared realism contract now carries a bounded `transport_envelope`, Scrapling and Agentic request-mode emit coherent persona and geo-aligned Accept-Language or user-agent posture instead of pinning everything to one local default, Agentic browser-mode now carries explicit locale and browser-client posture into the Playwright session, and both worker receipt paths preserve the applied transport or locale envelope as observer-only truth.
- `SIM-REALISM-2D` is now landed: Scrapling browser personas now preserve compact XHR-backed secondary-traffic counts, Agentic browser-mode now preserves compact same-origin request-event secondary-traffic counts, and recent-run plus operator-snapshot read models now distinguish top-level action truth from background or subresource browser activity without bloating hot reads into raw traces. The next active execution priority is now `SIM-REALISM-2E`.
- `SIM-REALISM-2E` is now landed: adversary lanes now carry bounded recurrence envelopes, planner and worker receipts preserve explicit dormancy and re-entry truth, supervisor dispatch honors recurrence dormancy without misreporting healthy idle windows as stalled generation, and the next active execution priority is now `SIM-REALISM-2F`.
- `ROUTE-NS-1` is now the immediate corrective architecture chain ahead of the remaining realism work: the generated public-content site must move from `/sim/public/*` to the protected host root, while Shuma-owned control and operational routes move under `/shuma/*`, with no pre-launch compatibility aliases unless explicitly requested.
- `SIM-REALISM-2` is now the explicit follow-on chain after `SIM-REALISM-1D`: the lanes still fall short on trusted-ingress client-IP realism, per-persona exploration depth and frontier realism, compact traversal receipts, richer public discoverability of the dummy surface, explicit degraded identity labeling, and long-horizon recurrence even after the pressure, identity-envelope, transport-envelope, and browser-secondary-traffic tranches landed.
- Do not treat mere lane execution or recent-run visibility as characteristic attacker pressure while the lanes still lack trusted-ingress identity realism, deeper exploration/frontier truth, root-host public-terrain correctness, and long-horizon recurrence behavior even after browser-mode session execution is real.
- Do not open `humans_plus_verified_only` until a later second strict-baseline proof has shown retained config-change improvement under both Scrapling and LLM attacker pressure.
- `DIAG-CLEANUP-1`, `MON-OVERHAUL-1C`, and `SIM-LLM-1C3` are now all landed, so the later combined-attacker strict-baseline proof is no longer blocked by missing LLM runtime visibility.
- The real `RSI-GAME-HO-2` blocker is now architectural and explicit:
  - the mixed-attacker restriction score spine is now landed and controller-grade restriction scoring is no longer effectively Scrapling-only,
  - `bot_red_team` runtime receipts now contribute restriction-grade board loci,
  - and the next remaining blocker is projection truth: operator/admin and dashboard surfaces still need to distinguish judged mixed-attacker episodes from mere lane visibility.
- Do not claim mixed-attacker strict-baseline proof from the new score spine alone; mixed-attacker proof projection and repeated retained improvement under mixed pressure still remain required.

- [ ] SIM-REALISM-2F Replace flat discovery caps with per-persona exploration envelopes.
  - Reference context:
    - [`docs/research/2026-03-30-adversary-lane-wild-traffic-gap-review.md`](../docs/research/2026-03-30-adversary-lane-wild-traffic-gap-review.md)
    - [`docs/plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../docs/plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
    - [`docs/plans/2026-03-20-shared-host-seed-contract.md`](../docs/plans/2026-03-20-shared-host-seed-contract.md)
  - Closure gate:
    - exploration truth: request, depth, byte, and time envelopes must vary by persona or mode instead of every lane collapsing to one flat discovery cap
    - boundary truth: deeper traversal must still derive only from public discovery and accepted hint documents rather than internal route catalogs or simulator-only hints
    - receipt truth: realism receipts must preserve the effective exploration envelope actually applied so shallow traversal can be distinguished from exhausted site frontier
    - proof: add and pass `make test-adversary-sim-exploration-envelope-realism`, and keep retained Scrapling and Agentic worker targets green
    - insufficient: globally raising `max_requests` or `max_depth`, or widening host knowledge to make traversal look deeper

- [ ] SIM-REALISM-2G Add compact exploration receipts and observer truth for traversal depth and frontier state.
  - Reference context:
    - [`docs/research/2026-03-30-adversary-lane-wild-traffic-gap-review.md`](../docs/research/2026-03-30-adversary-lane-wild-traffic-gap-review.md)
    - [`docs/plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../docs/plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
    - [`docs/plans/2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md`](../docs/plans/2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md)
  - Closure gate:
    - receipt truth: persist compact traversal facts including `visited_url_count`, `discovered_url_count`, `deepest_depth_reached`, `sitemap_documents_seen`, `frontier_remaining_count`, and `canonical_public_pages_reached`
    - observer truth: operator and dashboard surfaces must be able to tell whether the lanes stopped early or simply exhausted the reachable public frontier, without bloating hot reads into raw traces
    - boundary truth: the new receipts must stay observer-only and must not become defence truth or simulator-label leakage
    - proof: add and pass `make test-adversary-sim-exploration-receipts`, and keep relevant machine-contract and dashboard accountability targets green if observer rendering changes
    - insufficient: sample-path anecdotes without frontier counts, or dashboard guesswork about traversal progress

- [ ] SIM-REALISM-2H Make the dummy site more richly publicly discoverable without choreography.
  - Reference context:
    - [`docs/research/2026-03-30-contributor-generated-public-content-sim-site-review.md`](../docs/research/2026-03-30-contributor-generated-public-content-sim-site-review.md)
    - [`docs/plans/2026-03-30-contributor-generated-public-content-sim-site-plan.md`](../docs/plans/2026-03-30-contributor-generated-public-content-sim-site-plan.md)
    - [`docs/plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../docs/plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
    - [`docs/plans/2026-03-20-shared-host-seed-contract.md`](../docs/plans/2026-03-20-shared-host-seed-contract.md)
    - [`docs/plans/2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md`](../docs/plans/2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md)
  - Closure gate:
    - surface truth: broader dummy-site surfaces must become reachable through root links, realistic navigation, `robots.txt` sitemap entries, and sitemap documents rather than ad hoc worker hints
    - implementation vehicle: the richer surface should land as a generated contributor content site on the protected host root, not as a runtime repo walker or a slightly expanded hard-coded fake blog
    - contributor truth: contributors running `make dev` must be able to browse the site even when adversary sim is idle or disabled, and that local surface must be usable for human-friction assessment against the active config while runtime-only flows must not silently generate or expose the contributor site
    - cleanup truth: the old five-page hard-coded dummy site must be removed once the generated site path lands
    - discipline: keep hidden or internal route catalogs out of the workers and out of the seed contract
    - proof: add and pass `make test-sim-public-discoverability-contract`, and keep shared-host seed-contract proof green
    - insufficient: private worker-only route hints, simulator convenience endpoints, or sitemap/catalog artifacts treated as discovery truth

- [ ] SIM-REALISM-2I Add trusted-ingress client-IP realism without granting attacker-plane header privileges.
  - Reference context:
    - [`docs/research/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md`](../docs/research/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md)
    - [`docs/research/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md`](../docs/research/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md)
    - [`docs/plans/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md`](../docs/plans/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md)
    - [`docs/plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../docs/plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
    - [`docs/deployment.md`](../docs/deployment.md)
    - [`docs/security-hardening.md`](../docs/security-hardening.md)
  - Closure gate:
    - trust-boundary truth: realistic client-IP restoration for sim traffic must come only through a Shuma-owned trusted ingress or proxy path, not by teaching Scrapling or Agentic workers to emit `x-shuma-forwarded-secret` or any equivalent privileged header
    - runtime truth: when trusted sim ingress is configured, Shuma must observe real parseable client IPs instead of mostly `unknown`; when it is not configured, receipts must explicitly mark identity realism as degraded
    - parity truth: the same trust gate used for external forwarded headers must remain the only authority for client-IP restoration, with no sim-only bypass
    - proof: add and pass `make test-adversary-sim-trusted-ingress-ip-realism`, and keep attacker-plane contract checks and forwarded-header security tests green
    - insufficient: forging `x-forwarded-for` plus `x-shuma-forwarded-secret` from the workers, sim-only IP extraction shortcuts, or claiming realistic IPs from local process churn alone

- [ ] SIM-REALISM-2J Add explicit identity-realism receipts and operator wording for degraded IP truth.
  - Reference context:
    - [`docs/research/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md`](../docs/research/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md)
    - [`docs/plans/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md`](../docs/plans/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md)
    - [`docs/plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../docs/plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
    - [`docs/plans/2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md`](../docs/plans/2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md)
    - [`docs/dashboard-tabs/game-loop.md`](../docs/dashboard-tabs/game-loop.md)
  - Closure gate:
    - observer truth: recent-run and operator surfaces must distinguish `real client IP observed`, `bucketed IP`, and `identity realism degraded` instead of rendering hashed buckets like `h382` as if they were meaningful addresses
    - receipt truth: machine-facing receipts must preserve whether identity realism was pool-backed, trusted-ingress-backed, or degraded
    - wording truth: UI and docs must describe hashed or bucketed identity values as buckets, not as source IPs
    - proof: add and pass `make test-adversary-sim-identity-observer-truth`, and keep relevant dashboard accountability and machine-contract targets green if rendered wording changes
    - insufficient: continuing to show `unknown` or `h*` values without provenance labeling, or UI copy that implies realistic network identity when only degraded local identity exists

- [ ] RSI-GAME-HO-2A3 Project mixed-attacker judged-episode truth through operator/admin and Game Loop surfaces.
  - Reference context:
    - [`docs/research/2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md`](../docs/research/2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md)
    - [`docs/research/2026-03-28-rsi-game-ho-2a2-mixed-attacker-restriction-score-spine-post-implementation-review.md`](../docs/research/2026-03-28-rsi-game-ho-2a2-mixed-attacker-restriction-score-spine-post-implementation-review.md)
    - [`docs/plans/2026-03-28-rsi-game-ho-2-combined-attacker-orchestration-plan.md`](../docs/plans/2026-03-28-rsi-game-ho-2-combined-attacker-orchestration-plan.md)
  - Closure gate:
    - snapshot/admin truth: operator snapshot, oversight history, and `/admin/oversight/agent/status` must preserve which lanes contributed to the judged mixed-attacker episode
    - UI truth: `Red Team` and `Game Loop` must distinguish controller-grade mixed judgment from mere recent lane visibility
    - wording discipline: rendered copy must not overstate maturity from lane presence or ordered follow-on sequencing alone
    - proof: focused API or route tests, `make test-dashboard-red-team-truth-basis`, and `make test-dashboard-game-loop-accountability`
    - insufficient: dashboard visibility of both lanes without judged mixed-episode lineage, or status payloads that still require operators to infer mixed proof from coincidence

- [ ] RSI-GAME-ARCH-1E Retire replaced category-first Game Loop surfaces only after full-path replacement proof.
  - Reference context:
    - [`docs/research/2026-03-27-game-loop-architecture-alignment-gap-review.md`](../docs/research/2026-03-27-game-loop-architecture-alignment-gap-review.md)
    - [`docs/plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../docs/plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
  - Closure gate:
    - retirement discipline: any now-replaced objective, benchmark family, API field, adapter path, UI section, or doc contract from the old category-first model must be removed or explicitly demoted only after end-to-end replacement proof exists
    - doc truth: docs, tests, and dashboard copy must stop advertising the retired architecture once the replacement path is live
    - proof: `make test-benchmark-results-contract`, `make test-dashboard-game-loop-accountability`, `make test`, plus cited full-path evidence for each retired surface
    - insufficient: calling legacy category-first paths defunct without full-path verification, or leaving replaced architecture active and operator-visible after the new restriction-first rails are shipped

- [ ] RSI-GAME-BOARD-1 Refactor Game Loop around board-state doctrine and shared-path sim truth.
  - Reference context:
    - [`docs/research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](../docs/research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
    - [`docs/research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](../docs/research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
    - [`docs/research/2026-03-27-game-loop-shared-path-locality-and-actionability-post-implementation-review.md`](../docs/research/2026-03-27-game-loop-shared-path-locality-and-actionability-post-implementation-review.md)
    - [`docs/plans/2026-03-27-game-loop-board-state-refactor-plan.md`](../docs/plans/2026-03-27-game-loop-board-state-refactor-plan.md)
    - [`docs/plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](../docs/plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
    - [`docs/plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md`](../docs/plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md)
    - [`docs/research/2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md`](../docs/research/2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md)
    - [`docs/research/2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md`](../docs/research/2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md)
    - [`docs/plans/2026-03-24-llm-player-role-decomposition-plan.md`](../docs/plans/2026-03-24-llm-player-role-decomposition-plan.md)
    - [`docs/plans/2026-03-27-ovr-code-1-frontier-llm-code-evolution-ring-plan.md`](../docs/plans/2026-03-27-ovr-code-1-frontier-llm-code-evolution-ring-plan.md)
    - [`docs/plans/2026-03-27-human-friction-calibration-ring-plan.md`](../docs/plans/2026-03-27-human-friction-calibration-ring-plan.md)
  - Closure gate:
    - doctrine: Game Loop docs, scoring, and controller semantics must explicitly treat the host site as the board, Shuma defenses as the movable pieces, and adversary-sim traffic as traffic that shares the same judge path as real traffic
    - truth path: sim metadata may support harness control or audit only and must not become category truth, exploit truth, or tuning truth
    - scoring: the loop must preserve distinct planes for origin leakage and human-cost guardrails, terrain breach progress and host cost, surface-contract or tuning-readiness state, and the split between restriction scoring and recognition evaluation
    - confidence: the loop must treat Shuma confidence as something that accumulates through defense layers and can also rise through short-window abuse pressure when explicit identity signals remain weak
    - config loop: the controller must preserve failed bounded moves, rollback lineage, and anti-repeat memory tied to named breach loci
    - later rings: the planning chain for frontier-LLM code suggestions and real-human friction calibration must be explicit and bounded rather than implied
    - proof: follow the named substep proofs in `docs/plans/2026-03-27-game-loop-board-state-refactor-plan.md`; documentation alignment is sufficient for the planning-only substeps, while runtime or dashboard substeps must name focused `make` proofs
    - insufficient: aggregate-only pressure readouts with no named breach locus, simulator-side convenience labels in scoring, category posture still acting as the main undeclared-traffic restriction score, repeated near-equivalent failed config moves with no explicit memory, or vague later references to code evolution or human friction with no bounded contract

## P1 Verified Bot Identity And Web Bot Auth Foundation

Reference context:
- [`docs/research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](../docs/research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md)
- [`docs/research/2026-03-21-verified-identity-execution-readiness-refresh.md`](../docs/research/2026-03-21-verified-identity-execution-readiness-refresh.md)
- [`docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)
- [`docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

Current stance:
- Verified identity now sits after the delivered machine-first telemetry/snapshot foundations and before mature adversary-sim expansion.
- The first execution slices should stay observe-only: identity contracts, config, provider normalization, telemetry, and request-path annotations without routing change.
- The primary product value is exact non-human restriction and exception management; looser treatment for named verified bots remains an explicit opt-in later policy choice.
- `STANCE-MODEL-1` is now landed: verified identity is no longer a second top-level stance system, and current follow-on work should treat the resolved effective policy contract as the only policy truth.
- Do not bundle authorization policy, low-cost profiles, or dashboard control surfaces into the first tranche.

## P1 Production Adversary-Sim Operating Contract

Reference context:
- [`docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md`](../docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md)
- [`docs/research/2026-03-20-sim-deploy-2-readiness-review.md`](../docs/research/2026-03-20-sim-deploy-2-readiness-review.md)
- [`docs/adversarial-operator-guide.md`](../docs/adversarial-operator-guide.md)
- [`docs/plans/2026-03-20-sim-deploy-2-production-operating-envelope-implementation-plan.md`](../docs/plans/2026-03-20-sim-deploy-2-production-operating-envelope-implementation-plan.md)
- [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)

Current stance:
- Production adversary-sim control is part of Shuma's operating model and must not be runtime-prod-disabled.
- The remaining work is operating-envelope hardening, not approval for production availability.
- Execute this tranche in the order captured by the 2026-03-20 readiness review and implementation plan: verification-target truthfulness, desired-state unification, production posture codification, no-impact verification, then docs/evidence closure.

## P1 Shared-Host Discovery Baseline

Reference plan:
- [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`docs/plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](../docs/plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
- [`docs/plans/2026-03-20-shared-host-scope-fence-contract.md`](../docs/plans/2026-03-20-shared-host-scope-fence-contract.md)
- [`docs/plans/2026-03-20-shared-host-seed-contract.md`](../docs/plans/2026-03-20-shared-host-seed-contract.md)
- [`docs/research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md`](../docs/research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md)

### SIM-SH-SURFACE-1: Minimal Shared-Host Scope And Seed Gate
Scope note:
- `SIM-SCR-LANE-1` should require only the minimal scope-and-seed gate in `SIM-SH-SURFACE-1-1..2` plus `SIM-DEPLOY-2`.
- The observed reachable surface should emerge from traversal telemetry once the emergent lane runs; Shuma should not build a rich precomputed public-surface catalog as the default discovery architecture.

## P1 Privacy and Data-Protection Follow-up

- [ ] SEC-GDPR-2 Enforce deterministic cleanup and expiry for stale fingerprint state keys (`fp:state:*`, `fp:flow:*`, `fp:flow:last_bucket:*`) aligned to configured fingerprint TTL and window controls.
- [ ] SEC-GDPR-3 Add an optional event-log IP minimization mode (raw vs masked or pseudonymized) for privacy-sensitive deployments, with explicit tradeoff documentation.
- [ ] SEC-GDPR-4 Add a deployer-ready privacy and cookie disclosure template in docs (lawful basis, retention table, storage inventory, and rights-handling workflow).

## P1 Fingerprinting and Botness Signal Enrichment

Reference context:
- [`docs/research/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/2026-02-16-fingerprinting-research-synthesis.md)
- [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`docs/research/lessons-from-cloudflare.md`](../docs/research/lessons-from-cloudflare.md)
- [`docs/plans/2026-03-22-taxonomy-and-classification-implementation-plan.md`](../docs/plans/2026-03-22-taxonomy-and-classification-implementation-plan.md)
- [`docs/bot-defence.md`](../docs/bot-defence.md)

Current stance:
- Keep this enrichment chain behind the current `SIM-REALISM-1A..1D` execution priority unless a higher-severity runtime regression interrupts it.
- Additive evidence must remain bounded, explainable, privacy-guarded, and insufficient on its own for hard enforcement.
- New signal work must improve both botness and later category-confidence quality without inventing a second scoring model or bypassing the canonical taxonomy and classification contracts.

- [ ] FP-SIG-1 Add multi-store persistence-abuse signals to the fingerprint and botness pipeline.
  - Reference context:
    - [`docs/research/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/2026-02-16-fingerprinting-research-synthesis.md)
    - [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
  - Closure gate:
    - runtime truth: Shuma must detect bounded multi-store recovery or reconstitution patterns across cookie, local-storage, session-storage, or equivalent short-lived challenge state instead of treating persistence as one cookie-only signal
    - privacy truth: retention, pseudonymization, and TTL behavior must stay explicit and bounded rather than creating long-lived client identity semantics
    - scoring truth: the new persistence-abuse signals must surface stable IDs, runtime definitions, and additive botness contributions without bypassing existing family caps
    - proof: add and pass `make test-fingerprint-persistence-signals`, and keep `make test-traffic-classification-contract` green if classification receipts or readiness change
    - insufficient: another unnamed boolean, long-lived storage linkage, or persistence logic that becomes a hidden hard-block path

- [ ] FP-SIG-2 Add flow-centric request and JavaScript sequence signals to botness and classification evidence.
  - Reference context:
    - [`docs/research/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/2026-02-16-fingerprinting-research-synthesis.md)
    - [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
    - [`docs/research/lessons-from-cloudflare.md`](../docs/research/lessons-from-cloudflare.md)
  - Closure gate:
    - runtime truth: Shuma must add bounded flow or sequence evidence over API family, request ordering, timing windows, or response-aware progression rather than relying only on coarse rate buckets
    - cost truth: the sequence state must stay short-window, low-cardinality, and safe for shared-host resource budgets
    - observability truth: the new sequence signals must appear in runtime definitions and event or outcome evidence with stable identifiers
    - proof: add and pass `make test-fingerprint-flow-signals`, and keep `make test-monitoring-telemetry-foundations` green if request-outcome or event payloads change
    - insufficient: rebranding existing rate pressure as sequence intelligence, or adding unbounded per-session history

- [ ] BOT-SIG-1 Add bounded behavioral evidence signals for traversal depth, friction re-entry, and post-friction persistence.
  - Reference context:
    - [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
    - [`docs/research/lessons-from-cloudflare.md`](../docs/research/lessons-from-cloudflare.md)
  - Closure gate:
    - runtime truth: Shuma must accumulate additive behavioral evidence for deep traversal, repeated re-entry after challenge or ban friction, and other bounded workflow anomalies that are stronger than one-request fingerprint mismatches alone
    - scoring truth: these signals must remain additive and corroborative, not silent replacements for explicit hard detections like honeypot or active ban
    - category truth: if the new behavioral evidence improves category confidence, that improvement must flow through the existing classification-readiness contract rather than through simulator-only shortcuts
    - proof: add and pass `make test-botness-behavioral-signals`, and keep `make test-traffic-classification-contract` green
    - insufficient: counting every deep visit as malicious, or coupling the signals directly to simulator labels or lane metadata

- [ ] BOT-SIG-2 Add optional low-friction challenge-context behavior micro-signals with privacy guardrails.
  - Reference context:
    - [`docs/research/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/2026-02-16-fingerprinting-research-synthesis.md)
    - [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
  - Closure gate:
    - runtime truth: challenge or JS contexts may collect bounded timing, solve-latency, abandonment, or replay or tamper micro-signals, but only where those interactions already exist
    - privacy truth: the implementation must be opt-in, documented, TTL-bounded, and must not become a general behavior-biometric tracking system
    - scoring truth: the micro-signals must remain additive evidence only and must never become the sole hard gate for enforcement
    - proof: add and pass `make test-challenge-behavior-microsignals`, and keep `make test-challenge-verification` green
    - insufficient: always-on cursor biometrics, opaque scoring with no signal IDs, or challenge behavior becoming mandatory for baseline routing

## P1 Category-Labeling Signal Enrichment

Reference context:
- [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`docs/research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md`](../docs/research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md)
- [`docs/research/2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md`](../docs/research/2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md)
- [`docs/plans/2026-03-22-taxonomy-and-classification-implementation-plan.md`](../docs/plans/2026-03-22-taxonomy-and-classification-implementation-plan.md)

Current stance:
- Keep this chain behind the current `SIM-REALISM-1A..1D` priority unless a higher-severity runtime regression interrupts it.
- Exact category claims must stay conservative: when evidence is weak, Shuma must keep failing closed to `unknown_non_human` rather than invent category precision.
- Simulator-known labels remain observer-only and must never become runtime or tuning shortcuts for category assignment.

- [ ] CAT-SIG-1 Add declared-crawler verification signals beyond naive `User-Agent` matching.
  - Reference context:
    - [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
    - [`docs/plans/2026-03-22-taxonomy-and-classification-implementation-plan.md`](../docs/plans/2026-03-22-taxonomy-and-classification-implementation-plan.md)
  - Closure gate:
    - evidence truth: declared-crawler classification must require corroborating signals such as published IP ranges, reverse/forward DNS validation, known crawler tokens, or verified provider detection evidence rather than raw `User-Agent` alone
    - category truth: exact `indexing_bot` assignment must become more trustworthy without widening false-positive risk onto likely humans or generic suspicious automation
    - observability truth: receipts must preserve the specific declared-crawler evidence families that justified the exact category
    - proof: add and pass `make test-category-labeling-signals`, and keep `make test-traffic-classification-contract` green
    - insufficient: one-string allowlists, undocumented crawler exceptions, or exact crawler labels with no corroborating evidence lineage

- [ ] CAT-SIG-2 Add execution-shape signals that distinguish direct HTTP, automated browser, and browser-agent traffic.
  - Reference context:
    - [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
    - [`docs/research/2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md`](../docs/research/2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md)
  - Closure gate:
    - evidence truth: Shuma must incorporate bounded browser-execution, asset-fetch, persistence-marker, and request-shape evidence that helps separate `http_agent`, `automated_browser`, and `browser_agent`
    - category truth: new exact or improved assignments must flow through the shared classification receipt contract with explicit basis, exactness, and degradation status
    - cost truth: the new evidence must stay low-cost and same-path rather than requiring simulator knowledge or privileged host-side route catalogs
    - proof: add and pass `make test-category-labeling-signals`, and keep `make test-traffic-classification-contract` green
    - insufficient: mapping every JS-capable client to `automated_browser`, or every browser session to `browser_agent`

- [ ] CAT-SIG-3 Add crawl and extraction pattern signals that distinguish indexing, scraping, and focused agentic retrieval.
  - Reference context:
    - [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
    - [`docs/research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md`](../docs/research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md)
  - Closure gate:
    - evidence truth: Shuma must add bounded signals around robots/sitemap behavior, traversal breadth, listing/detail harvest patterns, and focused-vs-bulk retrieval that help separate `indexing_bot`, `ai_scraper_bot`, and `http_agent`
    - category truth: the signals must improve classification confidence without depending on content semantics or simulator-side category labels
    - observability truth: classification receipts and operator-facing readiness must reveal which pattern families are actually supporting the richer category claims
    - proof: add and pass `make test-category-labeling-signals`, and keep `make test-traffic-classification-contract` green
    - insufficient: equating any breadth-first crawl with AI scraping, or any direct HTTP fetch pattern with benign preview/service traffic

- [ ] CAT-SIG-4 Add explicit category-confidence fusion over identity, browser/fingerprint, behavioral, and challenge/outcome evidence.
  - Reference context:
    - [`docs/plans/2026-03-22-taxonomy-and-classification-implementation-plan.md`](../docs/plans/2026-03-22-taxonomy-and-classification-implementation-plan.md)
    - [`docs/research/2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md`](../docs/research/2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md)
  - Closure gate:
    - classifier truth: category assignment must be based on explicit bounded fusion of declared identity, browser/fingerprint evidence, behavior, and challenge/outcome evidence rather than one-off local heuristics
    - fail-closed truth: the classifier must still collapse to `unknown_non_human` with honest degraded or insufficient-evidence receipts when exact category confidence is not high enough
    - snapshot truth: operator snapshot and later benchmark/readiness surfaces must show the richer confidence basis without overstating exact inference
    - proof: add and pass `make test-category-labeling-signals`, `make test-traffic-classification-contract`, and `make test-benchmark-results-contract`
    - insufficient: silent heuristic drift, simulator-only confidence boosts, or exact category rows that cannot be explained through receipt evidence families

## P1 Tuning Surface Realignment

Reference context:
- [`docs/dashboard-tabs/tuning.md`](../docs/dashboard-tabs/tuning.md)
- [`docs/dashboard-tabs/fingerprinting.md`](../docs/dashboard-tabs/fingerprinting.md)
- [`docs/research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](../docs/research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
- [`docs/research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`](../docs/research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md)
- [`docs/research/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/2026-02-16-fingerprinting-research-synthesis.md)

Current stance:
- Keep this chain behind the current `SIM-REALISM-1A..1D` execution priority unless the user explicitly reprioritizes it or a higher-severity regression interrupts that work.
- `Tuning` should surface editable cost-shaping controls only; do not turn it into a mixed read-only diagnostics or explanation tab.
- `Tuning` should distinguish bounded loop-tunable controls from any future manual-only cost-shaping controls, but it must not present hard-`never` trust-boundary or provider-topology settings as if they are tuning knobs.
- Future ban-behavior tuning should begin with bounded jitter, repeat-offender, decay, and promotion controls rather than blindly exposing raw `ban_durations.*` as first-class loop moves.

- [ ] TUNE-SURFACE-2 Write a fresh Tuning-tab research and implementation plan around editable cost-shaping controls only.
  - Reference context:
    - [`docs/research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](../docs/research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
    - [`docs/research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`](../docs/research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md)
    - [`docs/research/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/2026-02-16-fingerprinting-research-synthesis.md)
  - Closure gate:
    - contract truth: the new research and plan must define `Tuning` as an editable cost-shaping surface, not a read-only explanation surface
    - ownership truth: the plan must explicitly separate `controller_tunable`, future `manual_only` cost-shaping, and hard-`never` trust-topology or authorization policy
    - horizon truth: the plan must distinguish short-window loop tuning from slower re-entry and recurrence tuning over longer evidence windows
    - proof: add new research and plan docs, update any affected roadmap or dashboard-tab docs, and keep `git diff --check` clean
    - insufficient: reviving the retired March 23-24 chain verbatim, or treating raw sanction-policy values as tuning without fresh mutability justification

- [ ] TUNE-SURFACE-2A Expand `Tuning` to the full currently ratified editable controller-tunable ring.
  - Reference context:
    - [`docs/research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](../docs/research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
    - [`docs/dashboard-tabs/tuning.md`](../docs/dashboard-tabs/tuning.md)
  - Closure gate:
    - surface truth: `Tuning` must expose editable controls for the current bounded tuning ring rather than only the narrow botness-threshold subset
    - scope truth: the surfaced controls must cover the ratified tuning families that materially shape hostile cost or human friction, including `js_required`, `proof_of_work`, `challenge`, `not_a_bot`, `maze` rollout, `cdp_detection`, current botness thresholds and weights, and the in-bounds fingerprint sensitivity controls
    - UI truth: the tab must remain editable-only and must not embed read-only signal inventories or runtime-definition furniture inside the main tuning surface
    - proof: add and pass focused dashboard coverage through `make test-dashboard-policy-pane-ownership`, keep `make test-dashboard-config-surface-contract`, `make test-controller-mutability-policy`, and `make test-controller-action-surface` green, and update `docs/dashboard-tabs/tuning.md`
    - insufficient: moving labels without wiring save semantics, or expanding the tab with controls that are still outside the ratified bounded ring

- [ ] TUNE-SURFACE-2B Move editable fingerprint sensitivity controls into `Tuning` and narrow `Fingerprinting` to source or provider posture only.
  - Reference context:
    - [`docs/dashboard-tabs/fingerprinting.md`](../docs/dashboard-tabs/fingerprinting.md)
    - [`docs/research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](../docs/research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
    - [`docs/research/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/2026-02-16-fingerprinting-research-synthesis.md)
  - Closure gate:
    - ownership truth: editable fingerprint sensitivity knobs must live with the other bounded cost-shaping controls in `Tuning`
    - boundary truth: provider-topology and edge-trust controls such as `provider_backends.fingerprint_signal` and `edge_integration_mode` must stay out of `Tuning`
    - UI truth: `Fingerprinting` must stop presenting a misleading read-only ownership boundary for knobs that are actually editable elsewhere
    - proof: add and pass focused dashboard coverage through `make test-dashboard-policy-pane-ownership`, keep `make test-dashboard-config-surface-contract` and `make test-controller-hard-boundaries` green, and update `docs/dashboard-tabs/tuning.md` plus `docs/dashboard-tabs/fingerprinting.md`
    - insufficient: duplicating the same fingerprint knobs across tabs, or moving trust-topology controls into `Tuning`

- [ ] TUNE-BAN-1 Design bounded ban-behavior tuning primitives for recurrence, re-entry, and longer-window cost shaping.
  - Reference context:
    - [`docs/research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`](../docs/research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md)
    - [`docs/research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](../docs/research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
  - Closure gate:
    - design truth: the research and plan must evaluate bounded jitter bands, repeat-offender ladders, decay windows, and promotion thresholds as candidate tuning primitives for recurring non-human traffic
    - boundary truth: the design must explicitly justify whether any part of ban behavior should move from hard-`never` to `manual_only` or `controller_tunable`, and it must not assume raw `ban_durations.*` become loop knobs by default
    - horizon truth: the work must define which ban-behavior knobs require longer evidence windows than the current short loop runs and how those windows remain observable and rollback-safe
    - proof: add new research and plan docs, update roadmap or TODO sequencing if the execution order changes, and keep `git diff --check` clean
    - insufficient: equating "ban durations matter" with "all ban durations should move into `Tuning`", or leaving recurrence tuning undefined beyond vague longer-window language

- [ ] TUNE-BAN-2 If bounded ban-behavior tuning is ratified, surface those controls in `Tuning` with explicit horizon and mutability labeling.
  - Reference context:
    - [`docs/research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`](../docs/research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md)
    - the future plan produced by `TUNE-BAN-1`
  - Closure gate:
    - surface truth: only the ratified bounded ban-behavior knobs, such as jitter width, repeat-offender multipliers, decay windows, or promotion thresholds, may appear in `Tuning`
    - labeling truth: the UI must clearly distinguish short-window loop-tunable controls from longer-window or manual-review controls so operators do not mistake time-horizon semantics
    - safety truth: any new ban-behavior controls must preserve auditability, rollback explainability, and explicit protection against synchronized expiry or punitive overreach
    - proof: add and pass focused dashboard coverage through `make test-dashboard-policy-pane-ownership`, keep `make test-controller-mutability-policy`, `make test-controller-action-surface`, and the focused ban-behavior verification target added by the tranche green, and update the relevant dashboard-tab docs
    - insufficient: exposing raw family base durations as the first shipped loop knobs, or shipping longer-window controls without proof of observability and rollback semantics

## P2 Hardening and Coverage

Architecture alignment reference:
- [`docs/plans/2026-02-23-maze-tarpit-architecture-alignment-plan.md`](../docs/plans/2026-02-23-maze-tarpit-architecture-alignment-plan.md)

- [ ] TEST-HYGIENE-3 Replace the remaining dashboard source-contract archaeology checks with rendered-behavior coverage where practical, especially for tab-surface ownership and monitoring section composition, so tests prove operator-visible behavior instead of string-level absence of old implementations.
- [ ] TEST-HYGIENE-4 Add a focused dashboard behavior test proving two distinct adversary-simulation `sim_run_id` values render as two `Recent Red Team Runs` rows when both runs are still present in the bounded monitoring window.
- [ ] TEST-HYGIENE-5 Add dashboard coverage that proves Monitoring headline charts remain enforced-only while shadow-mode labeling stays explicit in the raw/recent-event surfaces, so shadow truthfulness is verified at the rendered UI level instead of inferred from source structure.
- [ ] TAH-12 Add dashboard visibility for the expanded tarpit progression and egress metrics plus operator guidance for safe tuning.

## P2 Later Product Work

- [ ] INSPECT-1: Ephemeral Admin Defence Inspection Mode
  - Reference context:
    - [`docs/challenge-verification.md`](../docs/challenge-verification.md)
    - [`docs/tarpit.md`](../docs/tarpit.md)
    - [`docs/dashboard-tabs/tuning.md`](../docs/dashboard-tabs/tuning.md)
    - [`src/runtime/policy_graph.rs`](../src/runtime/policy_graph.rs)
    - [`src/runtime/request_flow.rs`](../src/runtime/request_flow.rs)
  - [ ] INSPECT-1-1 Write a short design note that defines the exact contract: admin-only inspection controls in the Tuning tab, current authed admin IP derived server-side, ephemeral state with explicit expiry, no freeform IP entry, and no persisted `always_challenge_ips` config surface in normal config or Advanced JSON.
  - [ ] INSPECT-1-2 Add runtime state and admin API primitives for activate/deactivate/status so inspection binds to the currently authed admin IP under trusted admin auth, survives page refreshes for a bounded TTL, and expires/cleans up deterministically.
  - [ ] INSPECT-1-3 Model inspection as an explicit next-request entry point rather than a fake botness score or threshold override: the operator arms one inspection target (`Not-a-Bot`, `Puzzle`, `Maze`, or `Challenge-abuse escalation`) for the next eligible request, and the system then behaves normally from that point onward.
  - [ ] INSPECT-1-4 Keep the implementation at the policy/response boundary so existing defence modules remain truthful and unchanged internally; do not add walkthrough-specific scoring or success/failure branches inside Not-a-Bot, Puzzle, Maze, or Tarpit modules.
  - [ ] INSPECT-1-5 Keep direct trap rendering out of inspection mode: Maze and Tarpit previews remain the direct surfaces for previewing those traps. For tarpit-related inspection, add a truthful operator path that exercises the confirmed challenge-abuse routing logic that would escalate to tarpit, without requiring the operator to hand-craft replay/tamper abuse manually.
  - [ ] INSPECT-1-6 Add the Tuning-tab control surface as simple arm/disarm inspection actions with status/expiry copy, no operator IP input field, and clear explanation of which actions inspect human-path routing versus challenge-abuse escalation routing.
  - [ ] INSPECT-1-7 Add unit, integration, and dashboard end-to-end coverage proving activation, expiry, trusted admin IP rebinding, non-admin isolation, consumption of armed entry points, correct normal post-entry behaviour, challenge-abuse escalation inspection, and cleanup without persistent bans/collateral state leakage.
  - [ ] INSPECT-1-8 Update operator docs and verification guidance so admins know exactly what each inspection entry point exercises, what still follows normal runtime semantics after entry, and how inspection differs from the existing direct Maze/Tarpit preview links on local and deployed environments.
- [ ] SIM-DET-L1 Add optional deterministic seed input for runtime-toggle runs to support exact tune-confirm-repeat replay when desired; keep default behavior non-seeded.
- [ ] NAB-12 Evaluate optional PAT-style private attestation signal ingestion as additive evidence only (non-blocking).
- [ ] NAB-13 Execute short Not-a-Bot hardening sprint per [`docs/plans/2026-02-21-not-a-bot-hardening-sprint.md`](../docs/plans/2026-02-21-not-a-bot-hardening-sprint.md).
- [ ] Add ASN and network dimensions in GEO policy logic, not just country list. (`src/signals/geo/mod.rs`, `src/config/mod.rs`, `src/admin/api.rs`)
- [ ] Add GEO and ASN observability and alerting (metrics, dashboard panels, docs). (`src/observability/metrics.rs`, dashboard, docs)

## P3 Platform and Configuration Clarity

- [ ] Centralize dashboard tab metadata (tab ids, loading copy, refresh defaults, and invalidation scopes) into one shared registry consumed by route, route-controller, refresh-runtime, and native-runtime code so stale fallback tabs/messages cannot drift.
- [ ] ADV-JSON-1 Audit the Dashboard Advanced tab runtime-variable inventory and Advanced JSON seed for full parity and organization.
  - Reference context:
    - [`docs/dashboard-tabs/advanced.md`](../docs/dashboard-tabs/advanced.md)
    - [`dashboard/src/lib/domain/config-schema.js`](../dashboard/src/lib/domain/config-schema.js)
    - [`dashboard/src/lib/components/dashboard/AdvancedTab.svelte`](../dashboard/src/lib/components/dashboard/AdvancedTab.svelte)
    - [`src/admin/api.rs`](../src/admin/api.rs)
    - [`docs/configuration.md`](../docs/configuration.md)
  - Acceptance criteria:
    - every admin-writable KV config var accepted by `POST /admin/config` appears in Advanced JSON,
    - runtime-visible read-only vars surfaced in Advanced are truthfully classified,
    - stale, missing, or misleading entries are corrected,
    - variables are logically grouped and ordered so the Advanced surface remains navigable as the config surface grows,
    - docs and parity tests are updated so drift fails fast.
- [ ] Resolve the `ip_range_suggestions_*` classification exception so the documented config model stays honest: either make those runtime-visible KV knobs admin-writable with Advanced JSON parity, or move them out of the persisted read-only exception path and document the chosen contract.
- [ ] Write objective criteria for future repository splits (API stability, release cadence, ownership, operational coupling).
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving the shared-host-first control plane and a later thin edge-gateway adapter path.
- [ ] Evaluate renaming `SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_PUZZLE_THRESHOLD` to reflect botness semantics.
- [ ] Standardize terminology across code, UI, and docs so `honeypot` and `maze` are used consistently instead of interchangeably.
- [ ] Long-term option: integrate upstream identity or proxy auth (OIDC/SAML) for dashboard and admin instead of app-level key login.

## Final Pre-Launch Gate

- [ ] PERF-LAUNCH-1 Execute a final pre-launch performance and optimization pass (dashboard bundle-size budgets in strict mode, runtime latency/<abbr title="Central Processing Unit">CPU</abbr>/memory envelopes, and high-cost request-path profiling), then lock release thresholds and acceptance criteria.
