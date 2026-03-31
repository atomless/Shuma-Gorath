# Adversary Realism Acceptance And Envelope Governance Review

Date: 2026-03-31
Status: Current design driver for realism acceptance and closure claims

Related context:

- [`2026-03-30-adversary-lane-wild-traffic-gap-review.md`](./2026-03-30-adversary-lane-wild-traffic-gap-review.md)
- [`2026-03-31-post-2j-adversary-realism-sufficiency-review.md`](./2026-03-31-post-2j-adversary-realism-sufficiency-review.md)
- [`../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`../plans/2026-03-31-post-2j-adversary-realism-sufficiency-plan.md`](../plans/2026-03-31-post-2j-adversary-realism-sufficiency-plan.md)
- [`../../src/admin/adversary_sim_realism_profile.rs`](../../src/admin/adversary_sim_realism_profile.rs)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py)
- [`../../todos/todo.md`](../../todos/todo.md)

## Purpose

Define how the remaining adversary-realism tranches should be accepted or rejected, and freeze the rule that `pressure_envelope`, `exploration_envelope`, and later realism envelopes must model real hostile persona operating patterns rather than acting as simulator comfort limits.

## Current Conclusion

Adversary realism must close on a persona scorecard, not on one generic metric.

Three acceptance mistakes now need to be ruled out explicitly:

1. treating higher ban or rate-limit counts as the universal sign of better realism,
2. accepting tighter or lower envelopes merely because they keep the simulator neat or bounded,
3. and closing realism tranches from receipt shape alone without proving meaningful hostile escalation against the protected public site.

The remaining realism work should therefore be accepted only when it shows measurable baseline-to-post-tranche change in the realism dimensions that actually matter for the persona being modeled:

1. emission shape and cadence,
2. terrain reach and frontier penetration,
3. defence engagement,
4. identity and transport truth,
5. long-window return behavior where relevant,
6. and representativeness readiness.

## Findings

### 1. Higher ban counts are useful evidence, but not a universal acceptance gate

Burstier or more reckless personas should often produce more:

1. rate-limit triggers,
2. honeypot hits,
3. bans,
4. or visible challenge escalation.

But low-and-slow distributed scraping often does the opposite: it spreads pressure across many identities precisely to avoid per-identity rate controls while still extracting aggressively. So realism cannot be accepted or rejected from ban counts alone.

### 2. Public-surface penetration must increase in observable ways

For the current protected public site, realism should produce materially better reach into what is publicly discoverable. That means future tranches should be expected to move one or more of:

1. `visited_url_count`,
2. `discovered_url_count`,
3. `deepest_depth_reached`,
4. `canonical_public_pages_reached`,
5. and `frontier_remaining_count`.

This is the clearest way to distinguish “stronger attacker model” from “same shallow model with different labels”.

### 3. Defence engagement must escalate in persona-appropriate ways

A realistic adversary should not merely exist in the recent-run list. It should interact more meaningfully with the defensive surfaces it encounters. Depending on persona, that can mean:

1. more challenge issuance or re-entry after challenge friction,
2. more PoW or puzzle follow-through attempts,
3. more maze touches or deeper maze progression,
4. more honeypot, tarpit, or rate-control interaction,
5. more denied retries or focused persistence after friction,
6. and more bounded hostile action sequences on the agentic lane.

Not every persona should escalate every defence interaction equally, but every tranche should specify which ones ought to rise.

### 4. Emission receipts are necessary, but not sufficient

Receipts already prove burst shape, dwell, concurrency, recurrence, and exploration truth more honestly than before. That is necessary.

It is not sufficient.

Realism should close only when those receipts line up with externally grounded hostile persona expectations and with observable consequences on the protected public surface. A neat receipt for an underpowered or overly polite persona is still an underpowered attacker.

### 5. Envelopes are legitimate only as persona models

`pressure_envelope`, `exploration_envelope`, `identity_envelope`, `transport_envelope`, and later realism envelopes are valid only when they answer:

1. which hostile persona is being modeled,
2. which field observation or research supports that operating range,
3. which realism dimension is expected to increase or sharpen,
4. and how the tranche will prove that the envelope changed hostile behavior rather than merely constraining it.

If an envelope exists only to keep load low, keep the simulator tidy, or prevent Shuma from being stressed, it is anti-realism and should fail acceptance.

### 6. Representativeness still depends on infrastructure truth

Even strong persona modeling must fail closed when the environment cannot support it. Trusted ingress, pool-backed identities, and deeper transport posture are not optional details; they are part of whether realism claims are truthful.

So realism closure must continue to separate:

1. model quality,
2. emitted behavior,
3. observed impact,
4. and infrastructure readiness.

## Recommended Acceptance Scorecard

Every remaining realism tranche should declare which of these dimensions it is expected to improve, and closure should require measurable pre/post change in at least one applicable dimension while keeping the others truthful:

1. `Emission realism`
   - burst size, dwell, concurrency, requests-per-identity, pause windows, recurrence timing, or action-family breadth.
2. `Terrain realism`
   - frontier penetration, deeper traversal, better sitemap or navigation reach, and lower unexplained frontier remainder.
3. `Defence-engagement realism`
   - more realistic challenge, maze, rate-control, honeypot, tarpit, or retry engagement for the modeled persona.
4. `Identity and transport realism`
   - parseable client IPs through trusted ingress, pool-backed identities, coherent geo or locale posture, or deeper transport truth.
5. `Safety and truthfulness`
   - same-origin, root-started, public-hint-only, Shuma-blind, and no privileged trust-header shortcuts.
6. `Representativeness readiness`
   - explicit truth about whether the environment is representative, partially representative, or degraded.

## Envelope Governance Rule

Every remaining realism envelope change must satisfy all of the following:

1. it names the hostile persona or campaign style it is modeling,
2. it cites the relevant external or prior in-repo research that justifies that persona,
3. it states which scorecard dimensions should measurably change,
4. it defines the proof surface and exact `make` target or evidence path for that change,
5. and it must not be accepted if the new envelope only lowers pressure or penetration without a persona-grounded reason.

## External Grounding

These acceptance rules align with the external sources already informing the realism chain:

1. MITRE ATT&CK and the Center for Threat-Informed Defense both frame adversary emulation as testing defenses against real-world adversary behavior rather than against arbitrary or convenient tool behavior:
   - [MITRE ATT&CK Adversary Emulation Plans](https://attack.mitre.org/resources/adversary-emulation-plans)
   - [Center for Threat-Informed Defense Adversary Emulation Library](https://ctid.mitre.org/resources/adversary-emulation-library/)
2. NIST's cyber-range guidance treats emulation as more than simple simulation and explicitly calls out traffic generation that emulates protocols, source patterns, traffic flows, and attacks:
   - [NIST Cyber Range Guide](https://www.nist.gov/system/files/documents/2023/09/29/The%20Cyber%20Range_A%20Guide.pdf)
3. Field reports continue to show that hostile web traffic is often distributed, persistent, and shaped to avoid simple per-identity throttles:
   - [DataDome: Anatomy of a Distributed Scraping Attack](https://datadome.co/threat-research/anatomy-of-a-distributed-scraping-attack/)
   - [DataDome: The AI Agent Identity Crisis](https://datadome.co/threat-research/ai-agent-identity-crisis/)
   - [Glade Art: The bot situation on the internet is actually worse than you could imagine. Here's why](https://gladeart.com/blog/the-bot-situation-on-the-internet-is-actually-worse-than-you-could-imagine-heres-why)

## Consequence For The Remaining Realism Chain

The remaining `SIM-REALISM-2I..3E` work should now inherit one explicit acceptance doctrine:

1. no remaining tranche closes from “more bans” alone,
2. no envelope change closes unless it is justified as a hostile persona model,
3. every tranche must state its expected realism escalation dimensions up front,
4. and Game Loop or Tuning must remain blocked until those dimensions are proven, not merely described.
