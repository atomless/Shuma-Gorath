# 🐙 Research-Backed Value Proposition

This document explains where Shuma-Gorath is intentionally differentiated by research-driven design, low-energy operation, and attacker-cost asymmetry.

## 🐙 Core Product Value

Shuma-Gorath is designed around three non-negotiable goals:

- keep normal users low-friction,
- keep host cost bounded and energy-aware,
- move incremental cost onto malicious automation.

The implementation approach is research-first: synthesize papers, active ecosystem patterns, and enterprise operating models, then implement with explicit budget and rollback controls.

## 🐙 Capability Value Map

| Capability | Research grounding | Enterprise baseline followed | Shuma differentiation and advancement | Host-cost posture |
| --- | --- | --- | --- | --- |
| Tarpit (`L9_COST_IMPOSITION`) | [`docs/research/tarpit-research-2026-02-11.md`](research/tarpit-research-2026-02-11.md), [`docs/research/2026-02-22-http-tarpit-cost-shift-research-synthesis.md`](research/2026-02-22-http-tarpit-cost-shift-research-synthesis.md), [`docs/research/2026-02-23-tarpit-docs-rereview-addendum.md`](research/2026-02-23-tarpit-docs-rereview-addendum.md) | Follows proven bounded-response and concurrency/egress discipline seen in mature tarpit ecosystems; aligns with enterprise emphasis on operational guardrails before aggressive deception. | Uses signed progression tokens, strict replay/chain enforcement, proof-gated iterative steps, adaptive bounded work difficulty, deterministic fallback, and persistence escalation. Explicitly avoids unbounded slow-drip residency risk. | Bounded by per-flow, per-bucket, and global budgets plus concurrency caps; cost rises mostly on bot proof/step loops. |
| Maze deception (`L7/L8`) | [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](research/2026-02-14-maze-tarpit-research-synthesis.md), [`docs/research/2026-02-15-stage2.5-maze-efficiency-and-asymmetry.md`](research/2026-02-15-stage2.5-maze-efficiency-and-asymmetry.md) | Follows enterprise-style layered deception direction (for example selective decoy routing and adaptive complexity). | Adds signed traversal semantics, replay-safe progression, checkpointed issue-links, worker-based deep-tier compute, and CI-gated asymmetry benchmarks (`make test-maze-benchmark`) to prevent host-cost regressions. | Shared assets, bounded issuance, and deterministic budget fallback reduce repeated render cost. |
| Fingerprint and CDP signal stack | [`docs/research/2026-02-16-fingerprinting-research-synthesis.md`](research/2026-02-16-fingerprinting-research-synthesis.md) | Follows authoritative edge strengths for transport/global telemetry (for example Akamai edge outcomes). | Keeps internal-first signal model with provider provenance, family-level contribution caps, temporal coherence checks, pseudonymization controls, and explicit internal fallback when edge signals are absent/untrusted. | Bounded TTL/state windows and capped contribution budget reduce both data and compute overhead. |
| IP-range policy custom controls | [`docs/research/2026-02-20-ip-range-policy-research-synthesis.md`](research/2026-02-20-ip-range-policy-research-synthesis.md) | Follows enterprise-safe CIDR policy rollout patterns with explicit operator ownership. | Uses deterministic precedence (emergency allowlist > custom rules), advisory/enforce modes, and strict rule validation to reduce collateral-risk surprise. | Compile-once matching path and bounded rule/counter dimensions keep request-path cost stable. |
| Challenge stack (JS/PoW/Not-a-Bot/Puzzle routing) | Not-a-Bot: [`docs/research/2026-02-19-not-a-bot-challenge-research-synthesis.md`](research/2026-02-19-not-a-bot-challenge-research-synthesis.md) + [`docs/plans/2026-02-13-not-a-bot-excellence-plan.md`](plans/2026-02-13-not-a-bot-excellence-plan.md); Puzzle: archived baseline [`docs/plans/2026-02-13-puzzle-challenge-excellence-plan.md`](plans/2026-02-13-puzzle-challenge-excellence-plan.md) + active carry-forward [`docs/plans/2026-02-25-puzzle-challenge-carry-forward-plan.md`](plans/2026-02-25-puzzle-challenge-carry-forward-plan.md), with ARC-style transform-composition challenge design (`src/challenge/tests.rs`). | Follows managed-first philosophy: low-friction before interactive challenge escalation. | Keeps app-owned routing semantics with explicit separation between unsolved user attempts and confirmed attack signals, accessibility-aware challenge posture, ARC-style puzzle composition, and deterministic fallback between challenge, maze, tarpit, and block. | Escalation is staged; expensive controls are reserved for higher-confidence abuse paths. |
| Deployment and enterprise integration model | [`docs/research/2026-02-20-deployment-and-adversarial-simulation-research-synthesis.md`](research/2026-02-20-deployment-and-adversarial-simulation-research-synthesis.md) | Follows enterprise reality that multi-instance correctness needs distributed state and strict rollout gates. | Uses one policy engine with profile-gated state plane (`self_hosted_minimal` vs `enterprise_akamai`), explicit outage postures, and migration guardrails instead of silent divergence. | Self-hosted baseline remains efficient; enterprise overhead is applied only where needed for correctness. |

## 🐙 How Shuma Uses Enterprise Models Without Becoming Vendor-Dependent

- Treat authoritative edge systems as high-value signal providers where they are strongest (global telemetry, transport vantage, broad perimeter filtering).
- Keep application-context policy composition and deception ownership inside Shuma.
- Require explicit fallback semantics when external providers are degraded or unavailable.

This gives operators enterprise-grade optional integration without sacrificing self-hosted completeness.

## 🐙 Where to Validate Value in Production

Use these checks to confirm the expected cost placement:

- **Challenge and deception quality:** challenge success/failure mix, maze/tarpit entry rates, and false-positive envelope.
- **Host budget compliance:** tarpit budget-outcome counters, maze benchmark guardrails, and latency/error regressions.
- **Asymmetry trend:** attacker-visible work steps/proof burden increases while host CPU/egress and concurrency stay inside configured budgets.
- **Provider clarity:** provider/mode metrics and event annotations show whether decisions came from internal logic, edge signals, or hybrid composition.

## 🐙 Related Docs

- `bot-defence.md` (layering and ownership policy)
- `tarpit.md` (exact tarpit runtime mechanics)
- `maze.md` (maze runtime and asymmetry controls)
- `deployment.md` (profile and state-plane rollout)
- `research/README.md` (research corpus index)
