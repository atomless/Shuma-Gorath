# 🐙 Research Index

Use this index when you want the current research drivers without trawling the full date-sorted folder.

Top-level research in `docs/research/` now serves three purposes:

1. current design drivers for upcoming work,
2. recent proof and closeout notes that still matter to the active planning chain.
3. dated historical baselines and outdated deferred-edge notes preserved for auditability.

The directory is intentionally flat. Use this index, not nested folders, to distinguish active drivers from historical receipts.

## Start Here

- [`2026-03-21-feedback-loop-and-architecture-debt-review.md`](2026-03-21-feedback-loop-and-architecture-debt-review.md) - Current architectural debt and loop-gap review
- [`2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md`](2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md) - Why the mainline is shared-host-first
- [`2026-03-22-live-linode-feedback-loop-proof.md`](2026-03-22-live-linode-feedback-loop-proof.md) - Current live proof of the shared-host loop
- [`2026-03-23-documentation-audit-and-information-architecture-review.md`](2026-03-23-documentation-audit-and-information-architecture-review.md) - Current documentation IA audit and cleanup boundaries
- [`2026-03-23-testing-surface-audit.md`](2026-03-23-testing-surface-audit.md) - Current audit of what the test surface really proves
- [`2026-03-23-test-tier-1-post-implementation-review.md`](2026-03-23-test-tier-1-post-implementation-review.md) - Closeout for canonical test-tier and target-scope truthfulness
- [`2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-review.md`](2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-review.md) - Monitoring reframed around closed-loop accountability and diagnostics-first deep inspection
- [`2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md) - Hard boundary audit for what the controller may and must not mutate
- [`2026-03-24-ctrl-surface-1-controller-mutability-policy-post-implementation-review.md`](2026-03-24-ctrl-surface-1-controller-mutability-policy-post-implementation-review.md) - Closeout for landing the canonical path-level controller mutability rings and exposing them through `allowed_actions_v1`
- [`2026-03-24-ctrl-surface-2-action-surface-and-proposer-parity-post-implementation-review.md`](2026-03-24-ctrl-surface-2-action-surface-and-proposer-parity-post-implementation-review.md) - Closeout for aligning `allowed_actions_v1`, benchmark escalation, and the bounded proposer around the same explicit auto-proposable controller surface
- [`2026-03-24-ctrl-surface-3-hard-boundary-enforcement-post-implementation-review.md`](2026-03-24-ctrl-surface-3-hard-boundary-enforcement-post-implementation-review.md) - Closeout for proving hard-never controller boundaries and threading the canonical mutability truth into Advanced and Tuning docs
- [`2026-03-24-sim-scr-geo-1-public-network-identity-post-implementation-review.md`](2026-03-24-sim-scr-geo-1-public-network-identity-post-implementation-review.md) - Closeout for landing request-native public-network identity receipts so Scrapling can touch `geo_ip_policy` attacker-faithfully
- [`2026-03-24-default-flips-verified-identity-and-scrapling-lane-post-implementation-review.md`](2026-03-24-default-flips-verified-identity-and-scrapling-lane-post-implementation-review.md) - Closeout for enabling verified identity by default and making Scrapling the default sim lane
- [`2026-03-24-mon-overhaul-1a-monitoring-ia-post-implementation-review.md`](2026-03-24-mon-overhaul-1a-monitoring-ia-post-implementation-review.md) - Closeout for the first Monitoring/Diagnostics accountability-vs-diagnostics information-architecture split
- [`2026-03-24-test-hygiene-6a-dashboard-behavior-proof-post-implementation-review.md`](2026-03-24-test-hygiene-6a-dashboard-behavior-proof-post-implementation-review.md) - Closeout for replacing the first dashboard runtime archaeology tests with behavior-first proof
- [`2026-03-24-monitoring-reuse-first-diagnostics-cleanup-sequencing-review.md`](2026-03-24-monitoring-reuse-first-diagnostics-cleanup-sequencing-review.md) - Why Monitoring should reuse transitional shared Diagnostics chart/view-model surfaces before Diagnostics cleanup becomes aggressive
- [`2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-review.md`](2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-review.md) - Defines the three-way split between loop accountability, traffic visibility, and furniture-operational diagnostics
- [`2026-03-24-traffic-tab-1-post-implementation-review.md`](2026-03-24-traffic-tab-1-post-implementation-review.md) - Closeout for landing the first-class Traffic tab, shared refresh-bar reuse, and the first diagnostics title-cleanup fixes
- [`2026-03-24-game-loop-tab-rename-and-ordering-review.md`](2026-03-24-game-loop-tab-rename-and-ordering-review.md) - Rationale for renaming Monitoring to Game Loop and placing it after Red Team
- [`2026-03-24-ui-game-loop-1-tab-rename-post-implementation-review.md`](2026-03-24-ui-game-loop-1-tab-rename-post-implementation-review.md) - Closeout for the canonical Game Loop tab rename, reordering, and proof-path updates
- [`2026-03-24-sim-scr-geo-1-public-network-identity-post-implementation-review.md`](2026-03-24-sim-scr-geo-1-public-network-identity-post-implementation-review.md) - Closeout for request-native public-network identity coverage on Scrapling's final owned `geo_ip_policy` surface
- [`2026-03-24-dashboard-tab-framing-copy-cleanup-post-implementation-review.md`](2026-03-24-dashboard-tab-framing-copy-cleanup-post-implementation-review.md) - Closeout for removing redundant framing copy from Traffic, Game Loop, and Diagnostics
- [`2026-03-24-diagnostics-intro-restore-post-implementation-review.md`](2026-03-24-diagnostics-intro-restore-post-implementation-review.md) - Exact restoration of the Diagnostics ownership intro after it was incorrectly removed
- [`2026-03-24-dashboard-dom-de-shelling-post-implementation-review.md`](2026-03-24-dashboard-dom-de-shelling-post-implementation-review.md) - Closeout for removing needless nested section shells and empty top-level notice chrome from Traffic, Game Loop, and Diagnostics
- [`2026-03-24-ctrl-surface-1-controller-mutability-policy-post-implementation-review.md`](2026-03-24-ctrl-surface-1-controller-mutability-policy-post-implementation-review.md) - Closeout for the first canonical controller mutability-ring surface and config/dashboard parity proof
- [`2026-03-24-diagnostics-breakdown-and-dom-review-post-implementation.md`](2026-03-24-diagnostics-breakdown-and-dom-review-post-implementation.md) - Follow-up closeout for restoring richer diagnostics breakdown content and cleaning the remaining shared section noise
- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md) - Defines the missing game contract between Shuma's evaluator, move set, and later recursive-improvement phases
- [`2026-03-24-llm-player-role-decomposition-review.md`](2026-03-24-llm-player-role-decomposition-review.md) - Decomposes the later LLM attacker and defender roles under the non-LLM judge
- [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md) - Captures the remaining protocol-level contracts for the later game: judge scorecard, player schemas, and held-out evaluation separation
- [`2026-03-24-game-loop-audit-trail-and-github-provenance-review.md`](2026-03-24-game-loop-audit-trail-and-github-provenance-review.md) - Freezes the missing audit and provenance contract, with GitHub as the canonical code-lineage spine where feasible
- [`2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md`](2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md) - Clarifies how Game Loop should distinguish true numeric budgets from per-category target achievement
- [`2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md`](2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md) - Makes attacker-faithful Scrapling coverage an explicit prerequisite for the fuller attacker/defender game loop
- [`2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md`](2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md) - Captures the gap between upstream Scrapling challenge/browser capability and Shuma's current request-native-only integration
- [`2026-03-24-scrapling-owned-defense-surface-matrix-and-success-contract-review.md`](2026-03-24-scrapling-owned-defense-surface-matrix-and-success-contract-review.md) - Freezes the first request-native Scrapling-owned defense surfaces and success semantics
- [`2026-03-24-scrapling-geo-ip-policy-source-diversification-review.md`](2026-03-24-scrapling-geo-ip-policy-source-diversification-review.md) - Explains why the remaining `geo_ip_policy` gap is a request-native public-network identity problem, not a browser-stealth problem
- [`2026-03-24-sim-scr-challenge-2b-malicious-request-native-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2b-malicious-request-native-post-implementation-review.md) - Closeout for turning Scrapling's `http_agent` persona into a hostile request-native challenge actor with persisted surface-interaction receipts
- [`2026-03-24-sim-scr-challenge-2d-defense-receipt-surface-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2d-defense-receipt-surface-post-implementation-review.md) - Closeout for surfacing matrix-aligned defense receipts in recent sim runs and operator snapshot
- [`2026-03-24-sim-scr-challenge-2d-coverage-closure-and-gap-assignment-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2d-coverage-closure-and-gap-assignment-post-implementation-review.md) - Closeout for comparing Scrapling receipt coverage against the owned-surface matrix and explicitly assigning the remaining geo gap
- [`2026-03-24-mainline-resequence-scrapling-before-game-loop-review.md`](2026-03-24-mainline-resequence-scrapling-before-game-loop-review.md) - Reorders the active mainline so attacker-faithful Scrapling and the first working game loop come before later LLM runtime work
- [`2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-review.md`](2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-review.md) - Ownership split for making Tuning primary and reducing Fingerprinting to truthful diagnostics
- [`2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md`](2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md) - Why the future Fingerprinting tab should become Identification and explain taxonomy distinction

## Monitoring And Machine-First Operator Surfaces

- [`2026-03-17-operator-decision-support-telemetry-audit.md`](2026-03-17-operator-decision-support-telemetry-audit.md)
- [`2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`2026-03-19-controller-readiness-telemetry-foundation-review.md`](2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
- [`2026-03-23-dashboard-auth-shell-flash-review.md`](2026-03-23-dashboard-auth-shell-flash-review.md)
- [`2026-03-23-dashboard-auth-gate-post-implementation-review.md`](2026-03-23-dashboard-auth-gate-post-implementation-review.md)
- [`2026-03-23-dashboard-operator-surfacing-gap-review.md`](2026-03-23-dashboard-operator-surfacing-gap-review.md)
- [`2026-03-23-tuning-tab-taxonomy-posture-matrix-and-policy-archetypes-review.md`](2026-03-23-tuning-tab-taxonomy-posture-matrix-and-policy-archetypes-review.md)
- [`2026-03-23-ui-vid-1-verified-identity-pane-post-implementation-review.md`](2026-03-23-ui-vid-1-verified-identity-pane-post-implementation-review.md)
- [`2026-03-23-ui-red-1-red-team-truth-basis-post-implementation-review.md`](2026-03-23-ui-red-1-red-team-truth-basis-post-implementation-review.md)
- [`2026-03-23-host-impact-cost-proxy-and-benchmark-review.md`](2026-03-23-host-impact-cost-proxy-and-benchmark-review.md)
- [`2026-03-23-host-cost-1-and-2-host-impact-proxy-post-implementation-review.md`](2026-03-23-host-cost-1-and-2-host-impact-proxy-post-implementation-review.md)
- [`2026-03-23-ban-duration-family-truthfulness-review.md`](2026-03-23-ban-duration-family-truthfulness-review.md)

## Closed Feedback Loop, Taxonomy, And Tuning Eligibility

- [`2026-03-20-adversary-evolution-loop-role-synthesis.md`](2026-03-20-adversary-evolution-loop-role-synthesis.md)
- [`2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md`](2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md)
- [`2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-review.md`](2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-review.md)
- [`2026-03-22-closed-loop-sequence-execution-readiness-review.md`](2026-03-22-closed-loop-sequence-execution-readiness-review.md)
- [`2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md`](2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](2026-03-24-reference-stance-and-run-to-homeostasis-review.md)
- [`2026-03-24-llm-player-role-decomposition-review.md`](2026-03-24-llm-player-role-decomposition-review.md)
- [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md)
- [`2026-03-24-game-loop-audit-trail-and-github-provenance-review.md`](2026-03-24-game-loop-audit-trail-and-github-provenance-review.md)
- [`2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md`](2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md)
- [`2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md`](2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md)
- [`2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md`](2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md)
- [`2026-03-24-scrapling-owned-defense-surface-matrix-and-success-contract-review.md`](2026-03-24-scrapling-owned-defense-surface-matrix-and-success-contract-review.md)
- [`2026-03-24-scrapling-geo-ip-policy-source-diversification-review.md`](2026-03-24-scrapling-geo-ip-policy-source-diversification-review.md)
- [`2026-03-24-mainline-resequence-scrapling-before-game-loop-review.md`](2026-03-24-mainline-resequence-scrapling-before-game-loop-review.md)
- [`2026-03-24-open-backlog-and-plan-reference-stance-alignment-review.md`](2026-03-24-open-backlog-and-plan-reference-stance-alignment-review.md)
- [`2026-03-24-monitoring-multi-loop-benchmark-progress-review.md`](2026-03-24-monitoring-multi-loop-benchmark-progress-review.md)
- [`2026-03-24-monitoring-reuse-first-diagnostics-cleanup-sequencing-review.md`](2026-03-24-monitoring-reuse-first-diagnostics-cleanup-sequencing-review.md)
- [`2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-review.md`](2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-review.md)
- [`2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md)
- [`2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-review.md`](2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-review.md)
- [`2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md`](2026-03-24-identification-tab-remit-and-taxonomy-distinction-review.md)
- [`2026-03-23-scrapling-non-human-category-capability-review.md`](2026-03-23-scrapling-non-human-category-capability-review.md)
- [`2026-03-23-scrapling-and-verified-identity-sequence-readiness-review.md`](2026-03-23-scrapling-and-verified-identity-sequence-readiness-review.md)
- [`2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md`](2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md)
- [`2026-03-23-vid-tax-1-faithful-verified-identity-category-crosswalk-post-implementation-review.md`](2026-03-23-vid-tax-1-faithful-verified-identity-category-crosswalk-post-implementation-review.md)
- [`2026-03-23-vid-tax-2-bot-1-and-guard-1-calibration-and-no-harm-post-implementation-review.md`](2026-03-23-vid-tax-2-bot-1-and-guard-1-calibration-and-no-harm-post-implementation-review.md)
- [`2026-03-23-ban-duration-family-truthfulness-post-implementation-review.md`](2026-03-23-ban-duration-family-truthfulness-post-implementation-review.md)

## Current Proof And Closeout Chain

- [`2026-03-22-live-linode-feedback-loop-post-verification-review.md`](2026-03-22-live-linode-feedback-loop-post-verification-review.md)
- [`2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md`](2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md)
- [`2026-03-23-test-tier-1-post-implementation-review.md`](2026-03-23-test-tier-1-post-implementation-review.md)
- [`2026-03-23-adv-diag-1-adversary-sim-status-truth-review.md`](2026-03-23-adv-diag-1-adversary-sim-status-truth-review.md)
- [`2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md`](2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md)
- [`2026-03-23-ban-duration-family-truthfulness-post-implementation-review.md`](2026-03-23-ban-duration-family-truthfulness-post-implementation-review.md)
- [`2026-03-23-host-cost-1-and-2-host-impact-proxy-post-implementation-review.md`](2026-03-23-host-cost-1-and-2-host-impact-proxy-post-implementation-review.md)
- [`2026-03-23-ui-vid-1-verified-identity-pane-post-implementation-review.md`](2026-03-23-ui-vid-1-verified-identity-pane-post-implementation-review.md)
- [`2026-03-23-ui-red-1-red-team-truth-basis-post-implementation-review.md`](2026-03-23-ui-red-1-red-team-truth-basis-post-implementation-review.md)
- [`2026-03-24-default-flips-verified-identity-and-scrapling-lane-post-implementation-review.md`](2026-03-24-default-flips-verified-identity-and-scrapling-lane-post-implementation-review.md)
- [`2026-03-24-mon-overhaul-1a-monitoring-ia-post-implementation-review.md`](2026-03-24-mon-overhaul-1a-monitoring-ia-post-implementation-review.md)
- [`2026-03-24-test-hygiene-6a-dashboard-behavior-proof-post-implementation-review.md`](2026-03-24-test-hygiene-6a-dashboard-behavior-proof-post-implementation-review.md)
- [`2026-03-24-traffic-tab-1-post-implementation-review.md`](2026-03-24-traffic-tab-1-post-implementation-review.md)
- [`2026-03-24-ui-game-loop-1-tab-rename-post-implementation-review.md`](2026-03-24-ui-game-loop-1-tab-rename-post-implementation-review.md)
- [`2026-03-24-dashboard-tab-framing-copy-cleanup-post-implementation-review.md`](2026-03-24-dashboard-tab-framing-copy-cleanup-post-implementation-review.md)
- [`2026-03-24-diagnostics-intro-restore-post-implementation-review.md`](2026-03-24-diagnostics-intro-restore-post-implementation-review.md)
- [`2026-03-24-dashboard-dom-de-shelling-post-implementation-review.md`](2026-03-24-dashboard-dom-de-shelling-post-implementation-review.md)
- [`2026-03-24-ctrl-surface-2-action-surface-and-proposer-parity-post-implementation-review.md`](2026-03-24-ctrl-surface-2-action-surface-and-proposer-parity-post-implementation-review.md)
- [`2026-03-24-ctrl-surface-3-hard-boundary-enforcement-post-implementation-review.md`](2026-03-24-ctrl-surface-3-hard-boundary-enforcement-post-implementation-review.md)
- [`2026-03-24-sim-scr-challenge-2b-malicious-request-native-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2b-malicious-request-native-post-implementation-review.md)
- [`2026-03-24-sim-scr-challenge-2d-defense-receipt-surface-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2d-defense-receipt-surface-post-implementation-review.md)
- [`2026-03-24-sim-scr-challenge-2d-coverage-closure-and-gap-assignment-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2d-coverage-closure-and-gap-assignment-post-implementation-review.md)
- [`2026-03-23-vid-tax-1-faithful-verified-identity-category-crosswalk-post-implementation-review.md`](2026-03-23-vid-tax-1-faithful-verified-identity-category-crosswalk-post-implementation-review.md)
- [`2026-03-23-vid-tax-2-bot-1-and-guard-1-calibration-and-no-harm-post-implementation-review.md`](2026-03-23-vid-tax-2-bot-1-and-guard-1-calibration-and-no-harm-post-implementation-review.md)

## Topic Collections

### Tarpit

- [`tarpit-research-2026-02-11.md`](tarpit-research-2026-02-11.md)
- [`2026-02-14-maze-tarpit-research-synthesis.md`](2026-02-14-maze-tarpit-research-synthesis.md)
- [`2026-02-22-http-tarpit-cost-shift-research-synthesis.md`](2026-02-22-http-tarpit-cost-shift-research-synthesis.md)
- [`2026-02-23-tarpit-docs-rereview-addendum.md`](2026-02-23-tarpit-docs-rereview-addendum.md)

### Adversarial simulation and LLM testing

- [`2026-02-25-llm-adversarial-testing-research-synthesis.md`](2026-02-25-llm-adversarial-testing-research-synthesis.md)
- [`2026-03-02-sim-runtime-architecture-overview-and-gap-report.md`](2026-03-02-sim-runtime-architecture-overview-and-gap-report.md)
- [`2026-03-20-sim-deploy-2-readiness-review.md`](2026-03-20-sim-deploy-2-readiness-review.md)
- [`2026-03-20-sim-deploy-2-post-implementation-review.md`](2026-03-20-sim-deploy-2-post-implementation-review.md)
- [`2026-03-22-sim-llm-fit-1-bounded-llm-fulfillment-post-implementation-review.md`](2026-03-22-sim-llm-fit-1-bounded-llm-fulfillment-post-implementation-review.md)
- [`2026-03-22-sim-fulfill-1-category-to-lane-fulfillment-post-implementation-review.md`](2026-03-22-sim-fulfill-1-category-to-lane-fulfillment-post-implementation-review.md)
- [`2026-03-22-sim-cover-1-category-coverage-receipts-post-implementation-review.md`](2026-03-22-sim-cover-1-category-coverage-receipts-post-implementation-review.md)
- [`2026-03-22-sim-protected-1-protected-tuning-evidence-post-implementation-review.md`](2026-03-22-sim-protected-1-protected-tuning-evidence-post-implementation-review.md)
- [`2026-03-23-sim-scr-fit-1-request-native-category-ownership-post-implementation-review.md`](2026-03-23-sim-scr-fit-1-request-native-category-ownership-post-implementation-review.md)
- [`2026-03-23-sim-scr-fit-2-request-personas-post-implementation-review.md`](2026-03-23-sim-scr-fit-2-request-personas-post-implementation-review.md)
- [`2026-03-23-sim-scr-cover-2-request-native-coverage-post-implementation-review.md`](2026-03-23-sim-scr-cover-2-request-native-coverage-post-implementation-review.md)

### Deferred edge gateway track

- [`2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md`](2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md)
- [`2026-03-10-fermyon-akamai-edge-live-proof-blockers.md`](2026-03-10-fermyon-akamai-edge-live-proof-blockers.md)
- [`2026-03-12-fermyon-akamai-edge-live-proof.md`](2026-03-12-fermyon-akamai-edge-live-proof.md)
- [`2026-03-14-fermyon-edge-signal-and-blank-slate-live-proof.md`](2026-03-14-fermyon-edge-signal-and-blank-slate-live-proof.md)

## Historical Delivered Baselines

- [`2026-02-15-stage2.5-maze-efficiency-and-asymmetry.md`](2026-02-15-stage2.5-maze-efficiency-and-asymmetry.md)
- [`2026-02-16-fingerprinting-research-synthesis.md`](2026-02-16-fingerprinting-research-synthesis.md)
- [`2026-02-20-ip-range-policy-research-synthesis.md`](2026-02-20-ip-range-policy-research-synthesis.md)

## Notes

- If you need older tranche-by-tranche evidence, search the folder directly; the dated flat layout is now the canonical filesystem structure.
- Deferred edge blocker and proof notes stay in this folder as historical context for a later gateway-only track, but they are not current mainline guidance.
