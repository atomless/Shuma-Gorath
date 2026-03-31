# Post-2J Adversary Realism Sufficiency Review

Date: 2026-03-31
Status: Current design driver for the post-`SIM-REALISM-2J` realism chain

Related context:

- [`2026-03-30-adversary-lane-wild-traffic-gap-review.md`](./2026-03-30-adversary-lane-wild-traffic-gap-review.md)
- [`../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md`](./2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md)
- [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs)
- [`../../src/admin/adversary_sim_realism_profile.rs`](../../src/admin/adversary_sim_realism_profile.rs)
- [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py)
- [`../../scripts/tests/adversarial_container/worker.py`](../../scripts/tests/adversarial_container/worker.py)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

## Purpose

Determine whether the remaining scheduled realism work (`SIM-REALISM-2F..2J`) is enough to make Shuma's adversary lanes representative of current hostile crawler, scraper, and agentic traffic for Game Loop and Tuning work, or whether a further realism chain is still required.

This note is intentionally narrow. It assumes the current `SIM-REALISM-2F..2J` chain lands successfully and asks what material realism gaps would still remain after that.

## Current Conclusion

`SIM-REALISM-2F..2J` is necessary, but not sufficient.

Those tasks will materially improve:

1. traversal depth and frontier truth,
2. richer public terrain,
3. trusted-ingress client-IP restoration,
4. and truthful degraded-identity wording.

But even after they land, Shuma would still fall short of representative wild-attacker pressure in five important ways:

1. overlapping attacker populations and true mixed-lane concurrency,
2. agentic offensive action capability beyond `GET` or `navigate/click`,
3. true hours-to-days dormancy and campaign-style return behavior,
4. deeper transport or network fingerprint realism,
5. and an explicit operational gate that prevents representative claims when the required identity infrastructure is absent.

## Findings

### 1. The sim is still orchestrated as one worker at a time

The top-level adversary-sim runtime still hard-caps itself at one concurrent run in [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs):

1. `MAX_CONCURRENT_RUNS = 1`
2. one shared pending-worker slot per running state

Supervisor dispatch also returns early whenever one worker tick is already pending in [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs).

That means the current architecture can shape one active attacker stream at a time, but it still does not model:

1. overlapping Scrapling plus Agentic pressure,
2. multiple simultaneous attacker identities,
3. or the mixed-arrival reality where one attack family is scraping broadly while another is making narrower agentic probes at the same time.

This matters because current field traffic is not just one persona taking turns.

### 2. The agentic lane is still too narrow in what it can actually do

The current agentic capability envelope in [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs) still allows:

1. request mode: `http_get` only
2. browser mode: `browser_navigate`, `browser_snapshot`, and `browser_click` only

And the degraded fallback in [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py) still collapses to:

1. `GET /`
2. then `GET` of public hint paths

That remains substantially short of real agentic abuse, which often includes:

1. form submission,
2. query variation,
3. repeated targeted extraction over narrow page sets,
4. pagination or result walking,
5. and other small but consequential multi-step behaviors.

The current agentic lane is therefore still underpowered even if its request cadence and traversal depth improve.

### 3. The current recurrence model is still too short-horizon

The live realism profiles in [`../../src/admin/adversary_sim_realism_profile.rs`](../../src/admin/adversary_sim_realism_profile.rs) currently model recurrence as:

1. `bounded_single_tick_reentry`
2. `within_run`
3. gaps of a few seconds

That was a worthwhile first recurrence slice, but it is not the same as the field patterns described in the research:

1. dormancy over hours or days,
2. re-entry after absence,
3. and campaign behavior that resumes with the same or similar identities later.

Until those longer windows exist, Shuma still cannot truthfully claim to be evaluating defenses against the return patterns operators actually report.

### 4. Transport realism is still shallow compared with field observations

The current transport envelope is a good step forward, but it remains named at a coarse application-client posture such as:

1. `curl_impersonate`
2. `urllib_direct`
3. `playwright_chromium`

in [`../../src/admin/adversary_sim_realism_profile.rs`](../../src/admin/adversary_sim_realism_profile.rs).

That is still shallower than the field observations that emphasize:

1. TLS or HTTP stack fingerprints,
2. protocol posture,
3. coherent request-header families,
4. and transport traits that distinguish common automation stacks from real browser traffic.

So even after `SIM-REALISM-2C` and the remaining `2I/2J`, the transport layer will still not be deep enough for strong representativeness claims.

### 5. Identity realism still depends on infrastructure, not just code

The current trusted-ingress plan is correct in [`2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md`](./2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md), but it still points to a larger truth:

1. representative identity realism requires actual trusted ingress,
2. and realistic pool-backed identities require real backing infrastructure.

So even a perfect code implementation of `SIM-REALISM-2I` and `SIM-REALISM-2J` would still be degraded when that infrastructure is absent.

Shuma therefore needs an explicit representativeness gate:

1. what identity infrastructure is present,
2. whether representative claims are allowed,
3. and how operator surfaces fail closed when the infrastructure is not there.

Without that, the system could still overstate realism after `2J`.

## External Grounding

These remaining gaps are not theoretical. They align with the external field observations already captured in the earlier realism review:

1. distributed scraping often uses very large residential pools with low requests per identity rather than one serialized worker stream,
2. agentic abuse often shows narrow but sharp focused sessions rather than simple broad `GET`-only retrieval,
3. operators report dormancy and return behavior over materially longer windows than a few seconds,
4. and transport-stack coherence is part of how serious scrapers avoid obvious attribution.

See:

1. [DataDome: Anatomy of a Distributed Scraping Attack](https://datadome.co/threat-research/anatomy-of-a-distributed-scraping-attack/)
2. [DataDome: The AI Agent Identity Crisis](https://datadome.co/threat-research/ai-agent-identity-crisis/)
3. [Glade Art article](https://gladeart.com/blog/the-bot-situation-on-the-internet-is-actually-worse-than-you-could-imagine-heres-why)
4. [Imperva 2025 Bad Bot Report](https://www.imperva.com/resources/wp-content/uploads/sites/6/reports/2025-Bad-Bot-Report.pdf)

## Recommended Direction

Treat `SIM-REALISM-2F..2J` as the end of the second realism chain, not the end of realism.

The next explicit chain should cover:

1. overlapping mixed-lane and multi-identity concurrency realism,
2. richer agentic action capability and a stronger degraded fallback floor,
3. true long-window dormancy and return realism,
4. deeper transport and network fingerprint realism,
5. and an operational representativeness gate for identity infrastructure.

Only after those are defined and landed should Shuma reopen claims that adversary-sim pressure is representative enough for Game Loop and Tuning work.

## Consequence For The Backlog

The active blocker language should now be tightened again:

1. do not treat `SIM-REALISM-2F..2J` alone as sufficient for representative attacker pressure,
2. add a new post-`2J` realism chain,
3. and keep later Game Loop and Tuning work blocked until that chain is closed as well.
