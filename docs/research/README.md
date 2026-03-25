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
- [`2026-03-24-ctrl-surface-1-controller-mutability-policy-post-implementation-review.md`](2026-03-24-ctrl-surface-1-controller-mutability-policy-post-implementation-review.md) - Closeout for the canonical `never` / `manual_only` / `controller_tunable` policy across operator objectives and admin config
- [`2026-03-24-ctrl-surface-2-action-surface-parity-post-implementation-review.md`](2026-03-24-ctrl-surface-2-action-surface-parity-post-implementation-review.md) - Closeout for making `allowed_actions_v1`, benchmark escalation, and patch shaping obey the canonical mutability policy
- [`2026-03-24-ctrl-surface-3-hard-boundary-enforcement-post-implementation-review.md`](2026-03-24-ctrl-surface-3-hard-boundary-enforcement-post-implementation-review.md) - Closeout for explicit hard-boundary rejection and apply-side refusal over controller-forbidden surfaces
- [`2026-03-24-rsi-game-1a-canonical-game-contract-post-implementation-review.md`](2026-03-24-rsi-game-1a-canonical-game-contract-post-implementation-review.md) - Closeout for landing the canonical recursive-improvement game contract over rules, evaluator, move ring, gates, and anchors
- [`2026-03-24-rsi-game-1b-move-selection-policy-post-implementation-review.md`](2026-03-24-rsi-game-1b-move-selection-policy-post-implementation-review.md) - Closeout for replacing the coarse pressure bridge with explicit shortfall-attribution, bounded family guidance, and exact-move upgrade semantics
- [`2026-03-24-rsi-score-1-judge-scorecard-post-implementation-review.md`](2026-03-24-rsi-score-1-judge-scorecard-post-implementation-review.md) - Closeout for freezing the explicit machine-first judge scorecard over targets, guardrails, regression inputs, diagnostics, and homeostasis inputs
- [`2026-03-24-rsi-game-1c-episode-archive-post-implementation-review.md`](2026-03-24-rsi-game-1c-episode-archive-post-implementation-review.md) - Closeout for landing the bounded episode archive, terminal outcome memory, and conservative homeostasis summary
- [`2026-03-24-default-flips-verified-identity-and-scrapling-lane-post-implementation-review.md`](2026-03-24-default-flips-verified-identity-and-scrapling-lane-post-implementation-review.md) - Closeout for enabling verified identity by default and making Scrapling the default sim lane
- [`2026-03-24-mon-overhaul-1a-monitoring-ia-post-implementation-review.md`](2026-03-24-mon-overhaul-1a-monitoring-ia-post-implementation-review.md) - Closeout for the first Monitoring/Diagnostics accountability-vs-diagnostics information-architecture split
- [`2026-03-24-test-hygiene-6a-dashboard-behavior-proof-post-implementation-review.md`](2026-03-24-test-hygiene-6a-dashboard-behavior-proof-post-implementation-review.md) - Closeout for replacing the first dashboard runtime archaeology tests with behavior-first proof
- [`2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-review.md`](2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-review.md) - Narrows the remaining shell-wrapper and integration cleanup archaeology into explicit contract lanes
- [`2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-post-implementation-review.md`](2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-post-implementation-review.md) - Closeout for splitting live feedback-loop behavior proof from wrapper contracts and renaming the integration cleanup contract lane
- [`2026-03-25-testing-suite-structure-and-mainline-friction-review.md`](2026-03-25-testing-suite-structure-and-mainline-friction-review.md) - Assesses the current test surface professionally and makes a narrow active-mainline test-ergonomics tranche the next highest-leverage follow-on
- [`2026-03-25-test-mainline-1-active-verification-ergonomics-post-implementation-review.md`](2026-03-25-test-mainline-1-active-verification-ergonomics-post-implementation-review.md) - Closeout for adding the truthful active Scrapling -> game-loop aggregate verification bundle
- [`2026-03-25-sim-llm-1a-black-box-contract-readiness-review.md`](2026-03-25-sim-llm-1a-black-box-contract-readiness-review.md) - Confirms that the next backend mainline after attacker-faithful Scrapling and the first working game-loop proof is the LLM attacker black-box contract, not deferred dashboard cleanup
- [`2026-03-25-sim-llm-1a-black-box-contract-post-implementation-review.md`](2026-03-25-sim-llm-1a-black-box-contract-post-implementation-review.md) - Closeout for making the later LLM attacker boundary executable across the adversarial contract files, fulfillment plans, and internal beat payload
- [`2026-03-24-monitoring-reuse-first-diagnostics-cleanup-sequencing-review.md`](2026-03-24-monitoring-reuse-first-diagnostics-cleanup-sequencing-review.md) - Why Monitoring should reuse transitional shared Diagnostics chart/view-model surfaces before Diagnostics cleanup becomes aggressive
- [`2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-review.md`](2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-review.md) - Defines the three-way split between loop accountability, traffic visibility, and furniture-operational diagnostics
- [`2026-03-24-traffic-tab-1-post-implementation-review.md`](2026-03-24-traffic-tab-1-post-implementation-review.md) - Closeout for landing the first-class Traffic tab, shared refresh-bar reuse, and the first diagnostics title-cleanup fixes
- [`2026-03-24-game-loop-tab-rename-and-ordering-review.md`](2026-03-24-game-loop-tab-rename-and-ordering-review.md) - Rationale for renaming Monitoring to Game Loop and placing it after Red Team
- [`2026-03-24-ui-game-loop-1-tab-rename-post-implementation-review.md`](2026-03-24-ui-game-loop-1-tab-rename-post-implementation-review.md) - Closeout for the canonical Game Loop tab rename, reordering, and proof-path updates
- [`2026-03-24-dashboard-tab-framing-copy-cleanup-post-implementation-review.md`](2026-03-24-dashboard-tab-framing-copy-cleanup-post-implementation-review.md) - Closeout for removing redundant framing copy from Traffic, Game Loop, and Diagnostics
- [`2026-03-24-diagnostics-intro-restore-post-implementation-review.md`](2026-03-24-diagnostics-intro-restore-post-implementation-review.md) - Exact restoration of the Diagnostics ownership intro after it was incorrectly removed
- [`2026-03-24-dashboard-dom-de-shelling-post-implementation-review.md`](2026-03-24-dashboard-dom-de-shelling-post-implementation-review.md) - Closeout for removing needless nested section shells and empty top-level notice chrome from Traffic, Game Loop, and Diagnostics
- [`2026-03-24-diagnostics-breakdown-and-dom-review-post-implementation.md`](2026-03-24-diagnostics-breakdown-and-dom-review-post-implementation.md) - Follow-up closeout for restoring richer diagnostics breakdown content and cleaning the remaining shared section noise
- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md) - Defines the missing game contract between Shuma's evaluator, move set, and later recursive-improvement phases
- [`2026-03-24-rsi-game-1a-canonical-game-contract-post-implementation-review.md`](2026-03-24-rsi-game-1a-canonical-game-contract-post-implementation-review.md) - Closeout for the first landed machine-first game contract projected through snapshot and oversight history
- [`2026-03-24-rsi-score-1-judge-scorecard-post-implementation-review.md`](2026-03-24-rsi-score-1-judge-scorecard-post-implementation-review.md) - Closeout for making the judge scorecard explicit before episode archive and full game-loop execution
- [`2026-03-24-llm-player-role-decomposition-review.md`](2026-03-24-llm-player-role-decomposition-review.md) - Decomposes the later LLM attacker and defender roles under the non-LLM judge
- [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md) - Captures the remaining protocol-level contracts for the later game: judge scorecard, player schemas, and held-out evaluation separation
- [`2026-03-24-game-loop-audit-trail-and-github-provenance-review.md`](2026-03-24-game-loop-audit-trail-and-github-provenance-review.md) - Freezes the missing audit and provenance contract, with GitHub as the canonical code-lineage spine where feasible
- [`2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md`](2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md) - Clarifies how Game Loop should distinguish true numeric budgets from per-category target achievement
- [`2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md`](2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md) - Makes attacker-faithful Scrapling coverage an explicit prerequisite for the fuller attacker/defender game loop
- [`2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md`](2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md) - Captures the gap between upstream Scrapling challenge/browser capability and Shuma's current request-native-only integration
- [`2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-review.md`](2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-review.md) - Defines the machine-readable owned defense-surface matrix and success semantics for attacker-faithful Scrapling
- [`2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-post-implementation-review.md) - Closeout for landing the canonical Scrapling owned-surface matrix and focused contract gate
- [`2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-review.md`](2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-review.md) - Defines the attacker-faithful malicious request-native interactions Scrapling must add for the surfaces it owns
- [`2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-review.md`](2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-review.md) - Defines the bounded receipt and closure shape needed to prove Scrapling-owned defense-surface coverage per run
- [`2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-post-implementation-review.md) - Closeout for landing bounded owned-surface receipts and recent-run closure over the attacker-faithful Scrapling lane
- [`2026-03-24-mainline-resequence-scrapling-before-game-loop-review.md`](2026-03-24-mainline-resequence-scrapling-before-game-loop-review.md) - Reorders the active mainline so attacker-faithful Scrapling and the first working game loop come before later LLM runtime work
- [`2026-03-25-sim-llm-1a-black-box-contract-readiness-review.md`](2026-03-25-sim-llm-1a-black-box-contract-readiness-review.md) - Reopens the next backend mainline as the executable LLM attacker black-box contract over the now-landed Scrapling and game-loop baseline
- [`2026-03-25-sim-llm-1a-black-box-contract-post-implementation-review.md`](2026-03-25-sim-llm-1a-black-box-contract-post-implementation-review.md) - Closeout for landing the executable host-root-only and Shuma-blind attacker boundary ahead of the later LLM runtime actor
- [`2026-03-24-rsi-game-mainline-first-working-loop-review.md`](2026-03-24-rsi-game-mainline-first-working-loop-review.md) - Splits the first working game-loop proof into a concrete route-level mainline slice and a stronger follow-on proof
- [`2026-03-24-rsi-game-mainline-1a-first-working-loop-post-implementation-review.md`](2026-03-24-rsi-game-mainline-1a-first-working-loop-post-implementation-review.md) - Closeout for landing the first explicit mainline proof over the automatic post-sim hook and the route-level canary -> judged archive loop
- [`2026-03-24-tah-11-tarpit-observability-post-implementation-review.md`](2026-03-24-tah-11-tarpit-observability-post-implementation-review.md) - Closeout for expanding tarpit proof, chain, budget-reason, and offender-bucket observability across metrics and admin monitoring
- [`2026-03-24-tah-19-tarpit-persistence-collateral-risk-review.md`](2026-03-24-tah-19-tarpit-persistence-collateral-risk-review.md) - Explains why tarpit punitive escalation must stop using shared bucket counts even though coarse bucket visibility remains useful
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
- [`2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-review.md`](2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-review.md)
- [`2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2a-owned-surface-matrix-post-implementation-review.md)
- [`2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-review.md`](2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-review.md)
- [`2026-03-24-mainline-resequence-scrapling-before-game-loop-review.md`](2026-03-24-mainline-resequence-scrapling-before-game-loop-review.md)
- [`2026-03-24-rsi-game-mainline-first-working-loop-review.md`](2026-03-24-rsi-game-mainline-first-working-loop-review.md)
- [`2026-03-24-rsi-game-mainline-1a-first-working-loop-post-implementation-review.md`](2026-03-24-rsi-game-mainline-1a-first-working-loop-post-implementation-review.md)
- [`2026-03-24-rsi-game-mainline-1b-shared-host-verifier-review.md`](2026-03-24-rsi-game-mainline-1b-shared-host-verifier-review.md)
- [`2026-03-24-rsi-game-mainline-1b-shared-host-verifier-post-implementation-review.md`](2026-03-24-rsi-game-mainline-1b-shared-host-verifier-post-implementation-review.md) - Closeout for extending the first working game-loop proof into the shared-host verifier behavior layer
- [`2026-03-24-rsi-roles-1-triadic-role-contract-post-implementation-review.md`](2026-03-24-rsi-roles-1-triadic-role-contract-post-implementation-review.md) - Closeout for freezing the attacker/defender/judge split before later player protocol and autonomy work
- [`2026-03-24-rsi-proto-1-player-wire-contract-post-implementation-review.md`](2026-03-24-rsi-proto-1-player-wire-contract-post-implementation-review.md) - Closeout for freezing the canonical shared envelope plus attacker and defender schema families
- [`2026-03-24-rsi-eval-1-held-out-evaluation-boundary-post-implementation-review.md`](2026-03-24-rsi-eval-1-held-out-evaluation-boundary-post-implementation-review.md) - Closeout for freezing player-visible protected evidence versus judge-held-out evaluation contexts
- [`2026-03-24-rsi-audit-1a-shared-lineage-schema-post-implementation-review.md`](2026-03-24-rsi-audit-1a-shared-lineage-schema-post-implementation-review.md) - Closeout for freezing the shared episode and proposal lineage vocabulary across config and later code moves
- [`2026-03-24-build-hygiene-1-native-warning-cleanup-post-implementation-review.md`](2026-03-24-build-hygiene-1-native-warning-cleanup-post-implementation-review.md) - Closeout for removing native test-build warning debt and adding a focused warning-hygiene make gate
- [`2026-03-24-test-env-1-rust-env-isolation-post-implementation-review.md`](2026-03-24-test-env-1-rust-env-isolation-post-implementation-review.md) - Closeout for making `lock_env()` an executable repo-wide contract for env-mutating Rust tests
- [`2026-03-24-ci-wf-1-node24-action-refresh-post-implementation-review.md`](2026-03-24-ci-wf-1-node24-action-refresh-post-implementation-review.md) - Closeout for moving the workflow pins onto the Node24-backed official action majors and adding a focused repo-local version contract
- [`2026-03-24-mz-t1-live-maze-spin-integration-review.md`](2026-03-24-mz-t1-live-maze-spin-integration-review.md) - Pre-implementation review for the missing live Spin-path maze proof
- [`2026-03-24-mz-t1-live-maze-spin-integration-post-implementation-review.md`](2026-03-24-mz-t1-live-maze-spin-integration-post-implementation-review.md) - Closeout for landing the focused live opaque maze traversal gate
- [`2026-03-24-mz-t2-live-maze-browser-e2e-review.md`](2026-03-24-mz-t2-live-maze-browser-e2e-review.md) - Pre-implementation review for the missing live browser/session maze proof
- [`2026-03-24-mz-t2-live-maze-browser-e2e-post-implementation-review.md`](2026-03-24-mz-t2-live-maze-browser-e2e-post-implementation-review.md) - Closeout for landing the focused live browser/session maze proof
- [`2026-03-24-mz-t3-maze-state-concurrency-and-soak-review.md`](2026-03-24-mz-t3-maze-state-concurrency-and-soak-review.md) - Pre-implementation review for burst/concurrency closure over maze state primitives
- [`2026-03-24-mz-t3-maze-state-concurrency-and-soak-post-implementation-review.md`](2026-03-24-mz-t3-maze-state-concurrency-and-soak-post-implementation-review.md) - Closeout for burst/concurrency coverage and shared-host hardening over maze state primitives
- [`2026-03-24-mz-t4-maze-canonical-verification-wiring-review.md`](2026-03-24-mz-t4-maze-canonical-verification-wiring-review.md) - Defines the missing canonical Makefile and CI wiring needed to make the new maze proofs real merge gates
- [`2026-03-24-mz-t4-maze-canonical-verification-wiring-post-implementation-review.md`](2026-03-24-mz-t4-maze-canonical-verification-wiring-post-implementation-review.md) - Closeout for routing the live/browser/concurrency maze proofs through the canonical local and release verification paths
- [`2026-03-24-baseline-repair-after-mz-t4-full-suite-review.md`](2026-03-24-baseline-repair-after-mz-t4-full-suite-review.md) - Captures the non-maze baseline failures exposed by the canonical full-suite run after MZ-T4 and the repair boundaries needed before resuming the mainline
- [`2026-03-24-baseline-repair-after-mz-t4-full-suite-post-implementation-review.md`](2026-03-24-baseline-repair-after-mz-t4-full-suite-post-implementation-review.md) - Closeout for the baseline-repair tranche that restored a truthful green suite, repaired the Scrapling runtime gate, and rebaselined the operator-snapshot hot-read budget
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
- [`2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-review.md`](2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-review.md)
- [`2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-post-implementation-review.md`](2026-03-24-test-hygiene-6b-live-wrapper-and-integration-contract-post-implementation-review.md)
- [`2026-03-24-traffic-tab-1-post-implementation-review.md`](2026-03-24-traffic-tab-1-post-implementation-review.md)
- [`2026-03-24-ui-game-loop-1-tab-rename-post-implementation-review.md`](2026-03-24-ui-game-loop-1-tab-rename-post-implementation-review.md)
- [`2026-03-24-dashboard-tab-framing-copy-cleanup-post-implementation-review.md`](2026-03-24-dashboard-tab-framing-copy-cleanup-post-implementation-review.md)
- [`2026-03-24-diagnostics-intro-restore-post-implementation-review.md`](2026-03-24-diagnostics-intro-restore-post-implementation-review.md)
- [`2026-03-24-dashboard-dom-de-shelling-post-implementation-review.md`](2026-03-24-dashboard-dom-de-shelling-post-implementation-review.md)
- [`2026-03-24-rsi-game-1a-canonical-game-contract-post-implementation-review.md`](2026-03-24-rsi-game-1a-canonical-game-contract-post-implementation-review.md)
- [`2026-03-24-rsi-game-1b-move-selection-policy-post-implementation-review.md`](2026-03-24-rsi-game-1b-move-selection-policy-post-implementation-review.md)
- [`2026-03-24-rsi-game-1c-episode-archive-post-implementation-review.md`](2026-03-24-rsi-game-1c-episode-archive-post-implementation-review.md)
- [`2026-03-24-rsi-game-mainline-1a-first-working-loop-post-implementation-review.md`](2026-03-24-rsi-game-mainline-1a-first-working-loop-post-implementation-review.md)
- [`2026-03-24-tah-19-tarpit-persistence-collateral-risk-post-implementation-review.md`](2026-03-24-tah-19-tarpit-persistence-collateral-risk-post-implementation-review.md)
- [`2026-03-23-vid-tax-1-faithful-verified-identity-category-crosswalk-post-implementation-review.md`](2026-03-23-vid-tax-1-faithful-verified-identity-category-crosswalk-post-implementation-review.md)
- [`2026-03-23-vid-tax-2-bot-1-and-guard-1-calibration-and-no-harm-post-implementation-review.md`](2026-03-23-vid-tax-2-bot-1-and-guard-1-calibration-and-no-harm-post-implementation-review.md)
- [`2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-post-implementation-review.md)
- [`2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2d-receipt-backed-surface-coverage-post-implementation-review.md)

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
