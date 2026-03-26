# TODO Roadmap

Last updated: 2026-03-26

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
- Keep the operator-facing product stance distinct from the later recursive-improvement development reference stance: `MON-OVERHAUL-1` and later `TUNE-SURFACE-1` should project and edit the current operator-selected posture, while run-to-homeostasis episodes remain blocked with `OVR-AGENT-2` and `RSI-METH-1`.

## P0 Attacker-Faithful Scrapling And First Game Loop

Reference context:
- [`docs/research/2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md`](../docs/research/2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md)
- [`docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](../docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`docs/research/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md`](../docs/research/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md)
- [`docs/plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md`](../docs/plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md)
- [`docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`docs/research/2026-03-24-rsi-game-mainline-first-working-loop-review.md`](../docs/research/2026-03-24-rsi-game-mainline-first-working-loop-review.md)
- [`docs/plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../docs/plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)

Current note:
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
- Before further mainline completion claims, the repo now requires explicit acceptance-gate discipline. Do not describe `STANCE-MODEL-1`, `SIM-SCR-FULL-1`, `RSI-GAME-HO-1`, or `RSI-GAME-HO-2` as complete from planning progress, baseline capability, or dashboard pressure signals alone.
- The immediate non-LLM mainline is now:
  - `VERIFY-GATE-1`
  - `STANCE-MODEL-1`
  - `RSI-SCORE-2`
  - `RSI-GAME-HO-1`
- After that strict Scrapling-only proof, the next reopening should be the remaining LLM attacker runtime slice `SIM-LLM-1C3`.
- Do not open `humans_plus_verified_only` until a later second strict-baseline proof has shown retained config-change improvement under both Scrapling and LLM attacker pressure.
- `DIAG-CLEANUP-1` and `MON-OVERHAUL-1C` are now both landed, so the deferred Game Loop and Diagnostics follow-on queue is currently clear while later LLM runtime work stays blocked.

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

- [ ] RSI-SCORE-2 Upgrade the judge, diagnoser, and move selector so strict-loop proof is exploit-first, evidence-quality-aware, urgency-aware, and capable of explicit config-exhaustion referral before `RSI-GAME-HO-1`.
  - Reference context:
    - [`docs/research/2026-03-26-game-loop-scoring-and-diagnoser-audit.md`](../docs/research/2026-03-26-game-loop-scoring-and-diagnoser-audit.md)
    - [`docs/research/2026-03-26-ideal-rsi-game-loop-scoring-review.md`](../docs/research/2026-03-26-ideal-rsi-game-loop-scoring-review.md)
    - [`docs/research/2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md`](../docs/research/2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md)
    - [`docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
    - [`docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
  - Closure gate:
    - runtime/benchmark: the scorecard remains lexicographic with `scalarization=forbidden`, and `benchmark_results_v1` gains first-class terrain-local exploit-progress, evidence-quality, and urgency semantics rather than relying on aggregate suspicious-origin suppression plus coarse adversary proxies alone
    - reconcile/controller: controller-grade logic distinguishes judge state from diagnosis and move ranking, low-confidence or poorly localized exploit evidence cannot drive fine-grained config moves, repeated safe config failures can emit an explicit config-exhaustion or code-evolution referral, and chosen moves are tied to named breach loci plus the smallest credible repair surface
    - dashboard/admin: `Game Loop` separates guardrails, exploit progress, evidence quality, urgency, named breach loci, and config-exhaustion or code-referral truth instead of making them read like one attacker-success score
    - proof: focused `make` paths must pass for exploit-progress scoring, evidence-quality gating, urgency and homeostasis-break semantics, reconcile behavior, and rendered Game Loop accountability
    - insufficient: aggregate suspicious-origin leakage rows only, no machine-visible breach loci, low-confidence evidence still driving config recommendations, scattershot multi-family moves without localized causal basis, homeostasis that ignores sudden bypass regressions, or no explicit route from bounded config failure to code-gap escalation
  - [ ] RSI-SCORE-2C Add multiwindow urgency scoring and event-triggered homeostasis break so newly successful bypasses interrupt the loop immediately and re-anchor it to the last accepted safe baseline.
    - Proof required:
      - `make test-rsi-score-urgency-and-homeostasis`
      - `make test-rsi-game-mainline`
      - `make test-oversight-episode-archive`
  - [ ] RSI-SCORE-2D Separate judge, diagnoser, and move selector more sharply, require smallest-effective localized repair selection, and add explicit config-ring exhaustion or code-evolution referral when bounded safe config moves cannot close the exploit gap.
    - Proof required:
      - `make test-rsi-score-move-selection`
      - `make test-oversight-reconcile`
      - `make test-controller-action-surface`
  - [ ] RSI-SCORE-2E Project the richer judge truth in `Game Loop` so operators can distinguish guardrails, exploit progress, evidence quality, urgency, named breach loci, and config-exhaustion or code-referral states at a glance.
    - Proof required:
      - `make test-dashboard-game-loop-accountability`

- [ ] RSI-GAME-HO-1 Fully operationally prove the strict `human_only_private` Scrapling-driven game loop before any later relaxed stance.
  - Reference context:
    - [`docs/research/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md`](../docs/research/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md)
    - [`docs/research/2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md`](../docs/research/2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md)
    - [`docs/research/2026-03-26-game-loop-scoring-and-diagnoser-audit.md`](../docs/research/2026-03-26-game-loop-scoring-and-diagnoser-audit.md)
    - [`docs/research/2026-03-26-ideal-rsi-game-loop-scoring-review.md`](../docs/research/2026-03-26-ideal-rsi-game-loop-scoring-review.md)
    - [`docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
    - [`docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
    - [`docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
    - [`docs/plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../docs/plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
  - Closure gate:
    - runtime/config: the loop runs against `human_only_private`, verified non-human stays denied, the strict sim-only phase treats adversary-sim lanes as `100%` non-human traffic and drives suspicious forwarded leakage toward zero or equivalent fail-closed suppression rather than the seeded mixed-public `10%` budgets, the exploit-first `RSI-SCORE-2` judge is active, bounded config recommendations are applied, later Scrapling runs execute against changed config, and repeated retain/rollback judgments occur rather than one-off plumbing
    - API/snapshot: recent changes, oversight history, and related machine-first surfaces show repeated cycle lineage, applied changes, and retained vs rolled-back outcomes
    - dashboard/admin: `Game Loop` projects the strict stance truthfully, shows repeated change context, shows measured movement toward the strict target rather than only recommendation plumbing or legacy mismatch rows, preserves the richer `RSI-SCORE-2` judge planes, keeps later human-traversal calibration explicit rather than implying that sim traffic already proved likely-human safety, and does not rely on a weakened baseline to manufacture proof
    - proof: focused `make` paths must pass for strict-stance runtime behavior and repeated judged-cycle proof on the local sim-public surface first; later human-traversal calibration and live-host realism stay separate named proof rings
    - insufficient: one successful loop, one canary apply, recommendation generation without retained improvement, current mixed-site default budgets still appearing as the strict target, weakened-baseline proof standing in for strict-loop proof, or unresolved dashboard/runtime mismatch that still contradicts the claimed improvement
  - [ ] RSI-GAME-HO-1A Run the existing machine-first loop against the corrected `human_only_private` stance with verified non-human traffic still denied under that strict baseline.
  - [ ] RSI-GAME-HO-1B Repeat Scrapling-driven cycles until recommendations become bounded config changes, later runs occur against those changed configs, and watch windows judge retain or rollback truthfully many times rather than once.
  - [ ] RSI-GAME-HO-1C Define and satisfy the unlock condition showing retained config changes and measured improvement toward the strict target, not merely one successful end-to-end cycle.

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
