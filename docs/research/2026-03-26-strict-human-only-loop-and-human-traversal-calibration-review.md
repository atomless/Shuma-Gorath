Date: 2026-03-26
Status: Proposed planning driver

Related context:

- [`2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](2026-03-24-reference-stance-and-run-to-homeostasis-review.md)
- [`2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md`](2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md)
- [`2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md`](2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md)
- [`../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md`](../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md)
- [`../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../plans/2026-03-26-acceptance-gate-and-completion-claim-discipline-plan.md`](../plans/2026-03-26-acceptance-gate-and-completion-claim-discipline-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/runtime/sim_public.rs`](../../src/runtime/sim_public.rs)
- [`../../docs/testing.md`](../../docs/testing.md)
- [`../../docs/dashboard-tabs/game-loop.md`](../../docs/dashboard-tabs/game-loop.md)

Implementation status note:

- Later on 2026-03-26, the seeded default was changed to `human_only_private`, legacy seeded `site_default_v1` profiles began auto-upgrading on load, and strict suspicious-leakage budgets started deriving from adversary-sim scope. This review captures the pre-fix reasoning that drove that implementation change.

# Purpose

Decide how the strict `human_only_private` stance should be expressed in policy, active tasks, and Game Loop proof given that, at the time of this review, the dashboard still showed mixed-site defaults such as a `10%` suspicious forwarded request target.

# Findings

## 1. Adversary-sim lanes are already the authoritative non-human pressure source

Shuma already treats adversary-sim traffic as a separate traffic origin and keeps it out of likely-human summaries.

That means the active Scrapling and later LLM adversary lanes are not a mixed cohort that includes people.

They are the deliberate hostile or non-human pressure source used to harden the strict loop.

Conclusion:

1. the first strict-baseline loop should assume the attacker traffic under test is `100%` non-human traffic,
2. and it should not preserve a tolerance budget that only makes sense for a mixed public-web default profile.

## 2. The `10%` suspicious forwarded target was a seeded site-default budget, not the strict reference stance

The seeded objectives at the time of this review in [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs) set:

1. `suspicious_forwarded_request_rate = 0.10`,
2. `suspicious_forwarded_byte_rate = 0.10`,
3. and `suspicious_forwarded_latency_share = 0.10`.

Those defaults are appropriate for the existing mixed public-web baseline the repo explicitly says should remain distinct from the later strict development reference stance.

But they are too loose for the first strict `human_only_private` loop when the attacker under test is fully non-human.

Conclusion:

1. the Game Loop must stop projecting the seeded `10%` suspicious forwarded budgets as though they were the strict reference target,
2. and the strict sim-only proof phase should instead use zero or equivalent fail-closed suppression targets for suspicious forwarded request, byte, and latency leakage.

## 3. Local development already has a usable public surface for the first strict-loop phase

Shuma already serves the dummy realism pages at:

1. `/sim/public/landing`,
2. `/sim/public/docs`,
3. `/sim/public/pricing`,
4. `/sim/public/contact`,
5. and `/sim/public/search?q=...`

through [`../../src/runtime/sim_public.rs`](../../src/runtime/sim_public.rs).

[`../../docs/testing.md`](../../docs/testing.md) already documents that these pages are available whenever adversary-sim availability is enabled and the effective desired state is on.

`make dev` also already prepares the local Scrapling artifacts before Spin starts.

Conclusion:

1. local loopback-hosted development is a truthful first environment for strict sim-only loop iteration,
2. and the repo should say that explicitly instead of implying Linode is required before the strict loop can even start.

## 4. Live Linode still matters, but for shared-host realism and later human verification

The live shared-host layer remains important because it adds:

1. the real wrapper and supervisor chain,
2. public-host and TLS behavior,
3. deployment-specific routing and origin reachability,
4. and the actual operator traversal environment a human will use later.

That is a different proof question from "can the strict loop locally find an aggressive config that suppresses all non-human adversary traffic on the dummy surface?"

Conclusion:

1. local should be the first strict-loop proof surface for speed,
2. Linode should remain the later realism and shared-host verification layer,
3. and those two proof rings should not be collapsed into one vague notion of "operational."

## 5. Human traversal calibration is a separate follow-on, not something to infer from sim traffic

The repo already has a strong lane-separation rule for human-friction denominators:

1. live likely-human summaries must stay separate from adversary-sim traffic,
2. and fallback `/sim/public/*` pages must not make adversary traffic count as likely-human evidence.

That means if Shuma discovers a very aggressive strict config that blocks all adversary traffic but also blocks the operator's real browser sessions, that does not invalidate the strict sim-only exclusion result.

It creates a second question:

1. how should a real human contribute to the testing loop,
2. and how should Shuma measure the friction imposed on that human traversal?

Conclusion:

1. human traversal against the discovered strict config must be a separate follow-on proof step,
2. it must use real human-driven sessions and likely-human telemetry rather than sim inference,
3. and local-first then live-host verification is the right sequence for that follow-on.

# Decisions

1. Express `human_only_private` as "deny or equivalently suppress all non-human traffic, including verified non-human identities and all adversary-sim lanes" during the strict reference loop.
2. Treat the current `10%` suspicious forwarded budgets as site-default mixed-web defaults only; do not use them as the strict reference target when `human_only_private` is active.
3. Use zero or equivalent fail-closed suspicious forwarded request, byte, and latency targets during the strict sim-only proof phase.
4. Use local loopback-hosted `/sim/public/*` surfaces as the first development and verification environment for the strict sim-only loop.
5. Keep live Linode as the later shared-host realism and public-network verification layer.
6. Split the overall proof story into:
   1. strict sim-only non-human exclusion proof,
   2. then separate human traversal calibration against the discovered strict config.
7. Update Game Loop, TODO, and testing language so operators do not read today's mixed-site defaults as the intended strict human-only target.
