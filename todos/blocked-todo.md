# Blocked TODO Roadmap

Last updated: 2026-03-24

This file holds gated, contingent, or explicitly deferred work that is not execution-ready.
Move an item back into `todos/todo.md` only when its blocking condition is cleared.
Completed work lives in `todos/completed-todo-history.md`.
Security finding validity and closure status live in `todos/security-review.md`.

## P0 Blocked by Shared-Host Discovery and Runtime-Safety Gates

- [ ] SIM-LLM-1 Full LLM-orchestrated, instruction-driven, containerized adversary lane as a first-class runtime actor.
  Blocker: keep blocked until the first bounded category-fulfillment LLM tranche (`SIM-LLM-FIT-1`) plus `TRAFFIC-TAX-1`, `TRAFFIC-TAX-2`, `SIM-FULFILL-1`, `SIM-COVER-1`, `SIM-PROTECTED-1`, and the new Scrapling request-native follow-ons (`SIM-SCR-FIT-1`, `SIM-SCR-FIT-2`, `SIM-SCR-COVER-2`) prove which categories actually need LLM-backed adversary modes and how those modes become protected evidence, and until the first closed config loop (`OVR-APPLY-1`) is proven. The full actor should reuse the existing capability-safe container boundary and a pluggable model-backend contract; frontier-backed execution is the initial reference path for high-capability categories, while smaller local-model backends remain optional later only if category-fulfillment evals prove parity and acceptable operational cost.

- [ ] SIM-SCR-BROWSER-1 Evaluate and, only if proven, adopt Scrapling browser-runtime fulfillment for `automated_browser`.
  Blocker: defer until `SIM-SCR-FIT-1`, `SIM-SCR-FIT-2`, and `SIM-SCR-COVER-2` land, because the current repo-owned Scrapling runtime and shared-host worker path only prove request-fetcher operation. Re-open only when there is a truthful plan for browser dependencies, deploy/runtime cost, test coverage, and coverage receipts that can prove `automated_browser` without blurring into `browser_agent` or `agent_on_behalf_of_human`.

## P1 Blocked by Roadmap Reprioritisation After Deployment Baseline

- [ ] SIM-BREACH-REPLAY-1 External breach to replayable attack pipeline.
  Blocker: defer until the first emergent lanes are producing stable exploit findings, then re-assess replay capture, promotion, retention, and governance against the adopted mature-sim roadmap.

- [ ] SIM-SH-EXPORT-1 Optional export or curation tooling over observed reachable-surface telemetry.
  Blocker: defer unless Scrapling or later frontier telemetry proves a concrete need for bounded export artifacts or deterministic replay-promotion helpers. Any future tooling must be derived from observed traversal telemetry rather than revive a catalog-first discovery architecture.

- [ ] SIM-EDGE-RUNTIME-1 Edge-hosted external-supervisor productization for full adversary-sim runtime.
  Blocker: defer while shared-host remains the supported full Scrapling runtime target and Fermyon/Akamai remains a gateway/edge posture target only. Re-open only when there is a concrete external-supervisor deployment product worth supporting end to end.

## P1 Deferred Edge Gateway And Enterprise Distribution Follow-On

Reference context:
- [`../docs/research/2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md`](../docs/research/2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md)
- [`../docs/plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md`](../docs/plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md)
- [`../docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](../docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)
- [`../docs/plans/2026-03-09-akamai-rate-geo-integration-semantics-note.md`](../docs/plans/2026-03-09-akamai-rate-geo-integration-semantics-note.md)
- [`../docs/plans/2026-03-09-fermyon-akamai-edge-baseline-prerequisite-plan.md`](../docs/plans/2026-03-09-fermyon-akamai-edge-baseline-prerequisite-plan.md)

- [ ] EDGE-GW-ARCH-1 Plan the later thin edge-gateway plus shared-host control-plane split, including state ownership, signed-forwarding contract, day-2 operations model, and which deployment-local distributed-state guarantees still matter once the edge is gateway-only.
  Blocker: defer until the shared-host-first pre-launch loop is operating cleanly and the product intentionally re-commits to a later edge posture.

- [ ] EDGE-GW-ARCH-2 Refactor vendor-shaped edge-runtime assumptions (`edge-fermyon` naming, edge-cron execution profile, edge-specific request budgets, and top-level deploy bias) after `EDGE-GW-ARCH-1` defines the later gateway contract.
  Blocker: do not refactor these assumptions piecemeal before the later edge architecture is chosen, because the right generic shape depends on that plan.

- [ ] DEP-ENT-2 Add ban-sync observability (<abbr title="Service Level Objective">SLO</abbr> metrics for sync result and lag) to support promotion and rollback decisions.
  Blocker: defer while shared-host remains the supported full control plane and the later edge/distributed-state architecture is not yet execution-ready.

- [ ] DEP-ENT-3 Add two-instance Spin integration coverage with shared Redis to prove ban and unban convergence behavior.
  Blocker: defer while shared-host remains the supported full control plane and the later edge/distributed-state architecture is not yet execution-ready.

- [ ] DEP-ENT-4 Add outage and partition tests for distributed state (Redis unavailable or degraded) and assert explicit configured behavior by mode.
  Blocker: defer while shared-host remains the supported full control plane and the later edge/distributed-state architecture is not yet execution-ready.

- [ ] DEP-ENT-5 Add deployment and runtime guardrails that validate enterprise distributed-state posture against outbound and backend requirements before authoritative operation.
  Blocker: defer while shared-host remains the supported full control plane and the later edge/distributed-state architecture is not yet execution-ready.

- [ ] OUT-1 Add explicit deployment guardrails that fail when `provider_backends.rate_limiter=external` or `provider_backends.ban_store=external` but required Redis outbound hosts are not allowlisted in `spin.toml` `allowed_outbound_hosts`.
  Blocker: defer with `DEP-ENT-2..5`; outbound guardrails should be revisited only once the later edge/distributed-state architecture is re-opened intentionally.

- [ ] OUT-2 Add a provider-to-outbound-requirements matrix in public docs (internal vs external backend, required host capabilities, required outbound host allowlists, fallback behavior).
  Blocker: defer with `DEP-ENT-2..5`; the truthful matrix depends on the later edge/distributed-state architecture.

- [ ] OUT-3 Add integration verification that exercises external Redis provider selection under restricted outbound policy and confirms safe fallback and guardrail behavior is deterministic.
  Blocker: defer with `DEP-ENT-2..5`; the meaningful verification target depends on the later edge/distributed-state architecture.

- [ ] AK-RG-2 Define config surface and naming for Rate and GEO Akamai integration controls, including defaults and whether each is a simple toggle or toggle-plus-mode control.
  Blocker: defer while the Akamai edge posture is a later gateway-only track rather than an active pre-launch runtime target.

- [ ] AK-RG-3 Implement admin API and runtime config validation for the new Rate and GEO Akamai controls with explicit guardrails and clear validation errors.
  Blocker: defer while the Akamai edge posture is a later gateway-only track rather than an active pre-launch runtime target.

- [ ] AK-RG-4 Implement runtime behavior wiring so Akamai Rate and GEO signals can influence decisions according to the defined mode semantics without bypassing Shuma policy ownership.
  Blocker: defer while the Akamai edge posture is a later gateway-only track rather than an active pre-launch runtime target.

- [ ] AK-RG-5 Add dashboard controls and help text for Rate and GEO Akamai integration in the top-level tabs, including disabled-state behavior and operator warnings.
  Blocker: defer while the Akamai edge posture is a later gateway-only track rather than an active pre-launch runtime target.

- [ ] AK-RG-6 Add observability and policy-event taxonomy coverage for Rate and GEO Akamai decisions (source, mode, action, fallback reason, and downgrade behavior).
  Blocker: defer while the Akamai edge posture is a later gateway-only track rather than an active pre-launch runtime target.

- [ ] AK-RG-7 Add integration and end-to-end tests for mode precedence, downgrade/fallback safety, and regression against internal-only behavior.
  Blocker: defer while the Akamai edge posture is a later gateway-only track rather than an active pre-launch runtime target.

- [ ] AK-RG-8 Document rollout and rollback guidance for enabling Rate and GEO Akamai integration in enterprise deployments, including promotion gates and emergency disable steps.
  Blocker: defer while the Akamai edge posture is a later gateway-only track rather than an active pre-launch runtime target.

## P1 Deferred Pre-Launch Roadmap Captures

Reference context:
- [`docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`docs/research/2026-03-17-operator-decision-support-telemetry-audit.md`](../docs/research/2026-03-17-operator-decision-support-telemetry-audit.md)
- [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`docs/research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../docs/research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](../docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`docs/plans/2026-03-15-agentic-era-oversight-design.md`](../docs/plans/2026-03-15-agentic-era-oversight-design.md)
- [`docs/plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](../docs/plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)

- [ ] TUNE-SURFACE-1 Complete the Tuning tab and related config-control surfaces as the full operator contract for route, defence, ban, recidive, and intelligence thresholds.
  Blocker: wait for `MON-OVERHAUL-1` so Tuning reuses the settled operator-facing monitoring and diagnostics projection patterns instead of inventing a second UI contract early. The backend prerequisites are now delivered or explicitly queued: `ARCH-OBS-1`, `OPS-BENCH-2`, `OPS-SNAPSHOT-2`, `OVR-RECON-1`, `OVR-AGENT-1`, `TRAFFIC-TAX-1`, `TRAFFIC-TAX-2`, `SIM-FULFILL-1`, `SIM-COVER-1`, `SIM-PROTECTED-1`, `OPS-OBJECTIVES-3`, `OPS-BENCH-3`, the delivered live-proven `OVR-APPLY-1` loop per [`../docs/research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md`](../docs/research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md), the delivered adversary-sim diagnostics truth fix per [`../docs/research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md`](../docs/research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md), the new pre-Monitoring host-impact cost proxy tranches `HOST-COST-1` and `HOST-COST-2`, the controller mutability-policy tranches `CTRL-SURFACE-1`, `CTRL-SURFACE-2`, and `CTRL-SURFACE-3`, and the local dashboard surfacing ownership captured in `UI-VID-1` and `UI-RED-1`. Tuning must expose proven patch families, protected-evidence rules, category-aware objectives, the operator-objectives and category-posture editor, taxonomy-backed lane coverage, stable operator-facing taxonomy labels, the bounded posture scale from `allowed` through `blocked`, classification-confidence lineage, rollback semantics, the settled host-impact cost semantics, and the canonical `never`/`manual_only`/`controller_tunable` boundary rather than speculate ahead of the backend loop. The first concrete UI slice is now defined in [`../docs/plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](../docs/plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md): a taxonomy posture matrix in `Tuning` with optional stance archetype seeding, while `Policy` remains responsible for declarative crawl and exemption rules. The next ownership follow-on is defined in [`../docs/plans/2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md`](../docs/plans/2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md) and [`../docs/plans/2026-03-24-identification-tab-rename-and-taxonomy-distinction-plan.md`](../docs/plans/2026-03-24-identification-tab-rename-and-taxonomy-distinction-plan.md): ratified controller-tunable botness and fingerprint controls should move into `Tuning`, while the current `Fingerprinting` tab should become `Identification` and explain both the signals used for non-human identification and how those signals distinguish the taxonomy categories. Those stance archetypes are operator-facing product presets; they must remain distinct from the later `Human-only / private` development reference stance used by `RSI-METH-1`.
  - [ ] TUNE-SURFACE-1A Make `Non-Human Traffic Posture` the visually primary section of `Tuning` and land the taxonomy posture matrix with preset seeding.
  - [ ] TUNE-SURFACE-1B Consolidate ratified controller-tunable botness and fingerprint controls into `Tuning`, rename `Fingerprinting` to `Identification`, and make that tab the explanation surface for identification signals and taxonomy distinction, with a first-wave sparse per-category presentation that shows only meaningful signal families rather than explicit `not useful` entries.
  - [ ] TUNE-SURFACE-1C Add later objective-budget editing and controller-explanation surfaces derived from the canonical mutability policy.

- [ ] SIM-RET-1 Define a dedicated retention and disposal model for adversary-sim telemetry distinct from real-traffic telemetry.
  Blocker: defer execution until mature adversary-sim lane planning settles the expected telemetry classes, retention value horizon, and audit residue needed after tune-confirm-act loops.

- [ ] CTI-ARCH-1 Plan central-intelligence storage and service architecture, including source-trust model, freshness, governance, and whether Shuma uses a standalone service, managed provider, or other shared data plane.
  Blocker: defer execution until the current local recidive/jitter/intelligence design and the delivered benchmark enrichment contract are ready to be broken into service/API/storage contracts; do not treat the Git repository itself as the default shared-intelligence transport.

- [ ] OVR-AGENT-2 Plan the later broader scheduled or autonomous analyzer/recommender/reconfigurer expansion, including hosted-worker ownership, model/runtime choice, config-vs-code scope, and whether fleet-aware or code-change suggestions belong in the same system or a separate reviewed path.
  Blocker: defer this later controller-expansion planning until the first shared-host agent loop is actually implemented and iterated, the first closed autonomous tuning loop is proven over protected evidence, monitoring and tuning projections exist over that proven loop, and the sim-evidence and central-intelligence contracts are mature enough that broader automation can be planned against truthful inputs and bounded outputs. The mature-sim roadmap and 2026-03-21 agent-first sequencing review now make clear that a first recommend-only analysis harness should consume machine-first contracts and emergent-lane evidence before the broader scheduled or autonomous system is planned. The 2026-03-22 autonomous-tuning safety review adds `SIM-PROTECTED-1`, `SIM-COVER-1`, `OPS-OBJECTIVES-3`, `OPS-BENCH-3`, and the delivered live-proven `OVR-APPLY-1` loop per [`../docs/research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md`](../docs/research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md) as explicit prerequisites. The 2026-03-22 canonical-taxonomy review adds `TRAFFIC-TAX-1`, `TRAFFIC-TAX-2`, and `SIM-FULFILL-1` ahead of those representativeness gates. The later refinement makes ongoing categorization-quality improvement part of the expected loop, while leaving taxonomy expansion as a non-critical-path later contingency. The new closed-loop bridge review adds `SIM-LLM-FIT-1` as the bounded LLM adversary prerequisite and narrows `OVR-AGENT-2` to the later LLM-backed diagnosis/config harness over the proven closed config loop, leaving code evolution to `OVR-CODE-1`. The later methodology is now further constrained by [`../docs/research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](../docs/research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md) and [`../docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md): later recursive-improvement work should begin from the `Human-only / private` development reference stance, run as bounded episodes until target-not-met and progress-not-flat are both true, and only then broaden into preset sweeps over relaxed operator stances. It must also inherit the canonical controller mutability policy from `CTRL-SURFACE-1`, `CTRL-SURFACE-2`, and `CTRL-SURFACE-3` so later LLM diagnosis cannot quietly widen the loop's action space beyond the ratified bounded config ring. Wait for the delivered machine-first operator-snapshot contract, the delivered static `benchmark_suite_v1` registry, the delivered nested `benchmark_results_v1` snapshot projection, the delivered benchmark enrichment contract, `OPS-BENCH-2`, `OPS-SNAPSHOT-2`, `ADV-PROMO-1`, `OVR-RECON-1`, `OVR-AGENT-1`, `TRAFFIC-TAX-1`, `TRAFFIC-TAX-2`, `SIM-LLM-FIT-1`, `SIM-FULFILL-1`, `SIM-COVER-1`, `SIM-PROTECTED-1`, `OPS-OBJECTIVES-3`, `OPS-BENCH-3`, the later Monitoring projection, `TUNE-SURFACE-1`, and `CTI-ARCH-1`.

- [ ] OVR-CODE-1 Plan the later benchmark-driven LLM code-evolution loop for code changes and optional PR generation.
  Blocker: defer execution until the delivered benchmark enrichment contract, central-intelligence architecture, the bounded config-tuning loop, and the later LLM diagnosis/config harness are all mature enough that code evolution can be judged against explicit benchmark suites rather than anecdotes or single-instance dashboard impressions. The current benchmark-family prerequisite is captured in [`../docs/plans/2026-03-20-benchmark-suite-v1-design.md`](../docs/plans/2026-03-20-benchmark-suite-v1-design.md), the benchmark enrichment contract in [`../docs/plans/2026-03-20-benchmark-fleet-and-intelligence-enrichment-contract.md`](../docs/plans/2026-03-20-benchmark-fleet-and-intelligence-enrichment-contract.md), the delivered static suite registry review in [`../docs/research/2026-03-20-benchmark-suite-contract-post-implementation-review.md`](../docs/research/2026-03-20-benchmark-suite-contract-post-implementation-review.md), the delivered first results-envelope review in [`../docs/research/2026-03-20-benchmark-results-contract-post-implementation-review.md`](../docs/research/2026-03-20-benchmark-results-contract-post-implementation-review.md), the delivered escalation-boundary review in [`../docs/research/2026-03-20-benchmark-escalation-boundary-post-implementation-review.md`](../docs/research/2026-03-20-benchmark-escalation-boundary-post-implementation-review.md), the delivered snapshot-projection review in [`../docs/research/2026-03-20-benchmark-results-snapshot-projection-post-implementation-review.md`](../docs/research/2026-03-20-benchmark-results-snapshot-projection-post-implementation-review.md), the 2026-03-21 loop-closure plan in [`../docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md), and the delivered `OVR-APPLY-1` live review in [`../docs/research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md`](../docs/research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md). The later methodology is now further constrained by [`../docs/research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](../docs/research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md) and [`../docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md): code-evolution proposals should optimize permissive target stances only while continuing to pass the strict `Human-only / private` reference stance as a regression anchor. They must also treat the canonical mutability policy from `CTRL-SURFACE-1`, `CTRL-SURFACE-2`, and `CTRL-SURFACE-3` as a hard boundary so code-evolution or later controller proposals do not smuggle operator-target or trust-topology mutation into the loop. Wait for `OVR-RECON-1`, `OVR-AGENT-1`, and `OVR-AGENT-2` to mature before reopening code-evolution execution planning.

- [ ] RSI-METH-1 Implement the later recursive-improvement methodology contract: development reference stance, run-to-homeostasis episode control, preset sweep regimen, and strict-reference regression anchor.
  Blocker: defer execution until `OVR-AGENT-2` planning is reopened. Do not retrofit the current first closed config loop into an indefinite autonomous runner before the later controller phases and plateau-detection semantics are mature. The methodology to implement is captured in [`../docs/research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](../docs/research/2026-03-24-reference-stance-and-run-to-homeostasis-review.md) and [`../docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../docs/plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md).

## P1 Blocked by Enterprise Baseline Maturity

- [ ] DEP-ENT-6 Optional asynchronous mirror of high-confidence bans to Akamai Network Lists.
  Blocker: wait until `DEP-ENT-1..5` establish the authoritative enterprise distributed-state baseline.

- [ ] OUT-4 ADR for non-Redis external integrations (for example webhook notifications or cross-service sync) that defines the approved pattern in Spin (`allowed_outbound_hosts` expansion vs sidecar/bridge service).
  Blocker: wait until a concrete non-Redis integration target is approved.

- [ ] OUT-5 External transport design for non-stub `challenge_engine=external` and `maze_tarpit=external`.
  Blocker: wait until there is an approved external provider path after the baseline deployment work is complete.
