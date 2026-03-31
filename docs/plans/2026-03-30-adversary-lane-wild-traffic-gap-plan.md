# SIM-REALISM-2 Wild Traffic Gap Closure Plan

**Goal:** Close the remaining field-grounded realism gaps after `SIM-REALISM-1C` and `SIM-REALISM-1D` so Shuma's adversary lanes no longer stop at local burst-shape realism and can instead represent the pressure, identity, trusted-ingress client-IP posture, transport, traversal, public discoverability, and recurrence patterns actually seen in current hostile traffic.

**Architecture:** Reuse the existing lane realism contract, lane planner, supervisor, and worker seams. Extend that contract with pressure-envelope, identity-envelope, trusted-ingress identity restoration, transport-envelope, browser-secondary-traffic, exploration-envelope, exploration-receipt, public-discoverability, and recurrence concepts. Preserve the existing public-discovery-only Scrapling boundary and the existing black-box Shuma-blind Agentic boundary. Do not widen host knowledge to achieve realism.

**Tech Stack:** Rust adversary-sim planner and state, Python Scrapling worker, Python LLM runtime worker and adversarial container worker, existing monitoring and observer read models, Makefile verification, backlog and sequencing docs.

**Related research:** [`../research/2026-03-30-adversary-lane-wild-traffic-gap-review.md`](../research/2026-03-30-adversary-lane-wild-traffic-gap-review.md)

**Status note:** `SIM-REALISM-2A`, `SIM-REALISM-2B`, `SIM-REALISM-2C`, `SIM-REALISM-2D`, `SIM-REALISM-2E`, `SIM-REALISM-2F`, `SIM-REALISM-2G`, `SIM-REALISM-2I`, `SIM-REALISM-2J`, and `SIM-REALISM-3A` are now landed. The generated root-hosted contributor public site that satisfied the old `SIM-REALISM-2H` discoverability need is also already landed. The next active realism tranche is therefore `SIM-REALISM-3B`.

---

## Guardrails

1. Do not fake proxy-pool realism when no pool is configured. Receipts and docs must say realism is degraded rather than inventing network identity churn.
2. Do not achieve higher pressure by widening host knowledge, introducing internal route catalogs, or leaking Shuma-specific hints.
3. Do not let simulator-only identity, proxy, or category metadata leak into defence truth or category truth.
4. Do not turn realism work into uncontrolled aggression. Every stronger behavior must remain bounded by an explicit contract and receipt-backed proof.
5. Do not reintroduce hidden or internal route catalogs, worker-only path hints, or fake public-surface inventories to make traversal look deeper.
6. Do not grant attacker-plane workers privileged ingress headers or sim-only trust shortcuts. If realistic client IPs are needed, restore them through the same trusted ingress boundary external traffic must satisfy.

## Shared Acceptance Contract

All remaining `SIM-REALISM-2*` tranches now inherit the acceptance and envelope-governance doctrine in [`2026-03-31-adversary-realism-acceptance-and-envelope-governance-plan.md`](./2026-03-31-adversary-realism-acceptance-and-envelope-governance-plan.md).

That means:

1. no tranche closes from “more bans” alone,
2. every realism envelope must be justified as a hostile persona model rather than a simulator comfort limit,
3. every tranche must declare which realism scorecard dimensions it is expected to improve,
4. and closure now requires measurable baseline-to-post-tranche change in the relevant dimensions, not just receipt presence or feature existence.

## Task 1: Refresh Backlog And Sequencing Truth

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`

**Work:**
1. Make `SIM-REALISM-2A..2E` explicit as the field-grounded realism follow-on after `SIM-REALISM-1D`.
2. Stop implying that `SIM-REALISM-1D` alone reopens representative mixed-attacker or tuning-quality claims.
3. Record that `SIM-REALISM-1B` landed local pacing-shape realism, not the full wild-attacker realism contract.

**Acceptance criteria:**
1. The active backlog names the second realism chain explicitly.
2. The sequencing docs place `SIM-REALISM-2` ahead of any claim that adversary lanes are representative enough for mixed-attacker proof.

## Task 2: `SIM-REALISM-2A` Unclip Pressure Envelopes And Add Per-Persona Concurrency

**Status:** Landed on 2026-03-30.

**Files:**
- Later code targets: `src/admin/adversary_sim.rs`, `src/admin/adversary_sim_lane_runtime.rs`, `src/admin/adversary_sim_realism_profile.rs`, `scripts/supervisor/scrapling_worker.py`, `src/admin/adversary_sim_llm_lane.rs`, `scripts/tests/adversarial_container/worker.py`
- Later proof targets: focused realism tests, `Makefile`

**Work:**
1. Replace the current one-size global request/time clipping behavior with explicit per-persona and per-mode pressure envelopes.
2. Allow request-native personas to exceed the current `8 requests / 2 seconds` ceiling when their realism contract calls for it, while preserving explicit bounded safety limits.
3. Add bounded concurrency semantics where they materially affect realism instead of forcing every persona through a single-file sequential shape.

**Acceptance criteria:**
1. Request-native personas no longer collapse to the same clipped pressure envelope.
2. Receipts prove actual peak per-burst counts, effective request cadence, and any concurrency grouping used.
3. Pressure remains explicitly bounded by contract and does not widen host knowledge.

**Proof:**
1. Add and pass `make test-adversary-sim-pressure-envelope-realism`.
2. Keep existing Scrapling and Agentic realism targets green.

## Task 3: `SIM-REALISM-2B` Add Proxy-Pool And Identity-Envelope Realism

**Files:**
- Later code targets: `src/admin/adversary_sim_worker_plan.rs`, `src/admin/adversary_sim_lane_runtime.rs`, `scripts/supervisor/scrapling_worker.py`, `src/admin/adversary_sim_llm_lane.rs`, `scripts/supervisor/llm_runtime_worker.py`
- Later proof targets: contract tests, focused runtime tests, `Makefile`

**Work:**
1. Introduce a bounded identity-envelope contract that can represent:
   - residential versus mobile versus datacenter identity class,
   - geo affinity,
   - session stickiness,
   - reuse and churn cadence,
   - and truthful degraded mode when no proxy pool exists.
2. Support pool-backed identity rotation for both Scrapling and Agentic request traffic without claiming network diversity when only local process diversity exists.
3. Project identity-envelope truth into realism receipts without leaking those labels into Shuma defences.

**Acceptance criteria:**
1. Lanes can truthfully emit realistic identity rotation when backing infrastructure exists.
2. When that infrastructure does not exist, receipts explicitly state degraded identity realism rather than presenting synthetic session churn as full network realism.
3. Geo, locale, and session continuity can be tied to the chosen identity envelope.

**Proof:**
1. Add and pass `make test-adversarial-identity-envelope-contract`.
2. Keep recent-run projection and machine-contract tests green if receipt or observer shapes change.

## Task 4: `SIM-REALISM-2C` Add Coherent Header, Locale, And Transport Envelopes

**Status:** landed on 2026-03-30. The shared realism contract now carries a bounded `transport_envelope`, request-native Scrapling and Agentic request-mode now emit coherent persona or geo-aligned header posture, and Agentic browser-mode now carries explicit browser locale plus client posture into the Playwright session and receipt contract.

**Files:**
- Later code targets: `scripts/supervisor/scrapling_worker.py`, `scripts/tests/adversarial_container/worker.py`, `scripts/tests/adversarial_runner/llm_fulfillment.py`, `src/admin/adversary_sim_realism_profile.rs`
- Later proof targets: focused runtime tests, `Makefile`

**Work:**
1. Replace the current mostly static locale and header posture with coherent persona and identity envelopes covering:
   - user-agent family,
   - Accept-Language,
   - Accept and related resource headers,
   - browser locale,
   - and transport or fingerprint posture where the current libraries expose it.
2. Keep those envelopes coherent with geo affinity and persona rather than randomizing headers independently.
3. Preserve the current same-origin, public-discovery-only, Shuma-blind boundaries.

**Acceptance criteria:**
1. Traffic no longer pins every request-native run to one default `en-GB` Chrome-like posture.
2. Header and locale variation is coherent enough to model the field patterns described in the research note.
3. Receipts or focused proofs can show which envelope was actually applied without turning it into defence truth.

**Proof:**
1. Add and pass `make test-adversary-sim-header-transport-realism`.
2. Keep relevant lane-contract and worker tests green.

## Task 5: `SIM-REALISM-2D` Capture Browser Secondary Traffic And Background Request Truth

**Status:** landed on 2026-03-30. Scrapling browser personas now preserve compact XHR-backed secondary-traffic counts, Agentic browser-mode now preserves compact same-origin request-event secondary-traffic counts, and recent-run plus operator-snapshot read models now project those counts without turning hot reads into raw browser traces.

**Files:**
- Later code targets: `scripts/supervisor/scrapling_worker.py`, browser-mode execution under `scripts/tests/adversarial_container/`, read-model and recent-run projection files if needed
- Later proof targets: focused runtime tests, machine-contract tests, `Makefile`

**Work:**
1. Capture and receipt meaningful browser secondary traffic:
   - subresource fetches,
   - background XHR or fetch activity,
   - and top-level versus background separation.
2. Use the already-upgraded upstream Scrapling capability where applicable instead of inventing a parallel capture surface.
3. Preserve compact observer truth so browser lanes are not understated as mere top-level page visits.

**Acceptance criteria:**
1. Browser personas no longer hide their emitted background traffic behind top-level-only receipts.
2. Recent-run or machine-facing truth can distinguish top-level actions from secondary background requests.
3. Proof remains compact and does not bloat the hot observer payload with raw browser traces.

**Proof:**
1. Add and pass `make test-adversary-sim-browser-secondary-traffic-realism`.
2. Keep relevant recent-run or machine-contract tests green if read models change.

## Task 6: `SIM-REALISM-2E` Add Long-Horizon Dormancy, Recurrence, And Re-Entry Realism

**Files:**
- Later code targets: `src/admin/adversary_sim_state.rs`, `src/admin/adversary_sim_lane_runtime.rs`, relevant worker state or receipt files, later tuning or observer docs if timing truth changes
- Later proof targets: focused runtime tests, `Makefile`

**Work:**
1. Add a bounded recurrence model for adversary lanes so they can express:
   - re-entry after absence,
   - repeated narrow sessions over longer windows,
   - and longer-horizon persistence patterns that current short runs cannot represent alone.
2. Keep this separate from ban-policy tuning; the aim is attacker realism, not sanction logic.
3. Make any longer-horizon profile visible in machine receipts so later tuning and evaluation surfaces do not infer it from coincidence.

**Acceptance criteria:**
1. The realism contract can express more than one isolated short burst.
2. Longer-window attacker behavior is modeled explicitly rather than assumed from repeated beat execution.
3. The resulting state remains bounded and testable.

**Proof:**
1. Add and pass `make test-adversary-sim-recurrence-realism`.
2. Keep adversary-sim state and dispatch tests green.

## Task 7: `SIM-REALISM-2F` Replace Flat Discovery Caps With Per-Persona Exploration Envelopes

**Status:** Landed on 2026-03-31.

**Files:**
- Later code targets: `src/admin/adversary_sim.rs`, `src/admin/adversary_sim_lane_runtime.rs`, `src/admin/adversary_sim_realism_profile.rs`, `scripts/supervisor/scrapling_worker.py`, `scripts/supervisor/llm_runtime_worker.py`
- Later proof targets: focused realism tests, retained worker proofs, `Makefile`

**Work:**
1. Replace the current flat discovery clipping model with explicit per-persona and per-mode exploration envelopes covering request, depth, byte, and time budgets.
2. Allow deeper traversal where the realism contract calls for it while keeping the public-discovery-only boundary intact.
3. Preserve proof of the effective exploration envelope in runtime receipts so shallow coverage can be distinguished from exhausted frontier.

**Acceptance criteria:**
1. Scrapling and Agentic traversal no longer collapse to one flat discovery cap.
2. Deeper exploration remains bounded by explicit contract rather than ad hoc worker freedom.
3. No internal route catalogs or simulator-only helper hints are introduced to manufacture depth.

**Proof:**
1. Add and pass `make test-adversary-sim-exploration-envelope-realism`.
2. Keep retained Scrapling and Agentic worker tests green.

## Task 8: `SIM-REALISM-2G` Add Compact Exploration Receipts And Observer Truth

**Status:** Landed on 2026-03-31.

**Files:**
- Later code targets: worker receipt emitters, `src/observability/hot_read_documents.rs`, recent-run projection files, relevant admin or dashboard observer adapters
- Later proof targets: machine-contract tests, rendered observer proofs if UI changes, `Makefile`

**Work:**
1. Add compact traversal receipts that persist:
   - `visited_url_count`
   - `discovered_url_count`
   - `deepest_depth_reached`
   - `sitemap_documents_seen`
   - `frontier_remaining_count`
   - `canonical_public_pages_reached`
2. Preserve those facts as observer-only truth so operators can tell whether a lane stopped early or exhausted the reachable public frontier.
3. Keep the receipts compact rather than turning hot reads into raw path traces.

**Acceptance criteria:**
1. Recent-run and observer surfaces can distinguish shallow traversal from frontier exhaustion.
2. The receipt contract stays compact and machine-readable.
3. No new receipt field leaks simulator-only hints into defence truth.

**Proof:**
1. Add and pass `make test-adversary-sim-exploration-receipts`.
2. Keep relevant machine-contract tests green, and keep dashboard accountability proof green if rendered observer surfaces change.

## Task 9: `SIM-REALISM-2H` Make The Dummy Site More Richly Publicly Discoverable Without Choreography

**Status:** Landed through the generated contributor-site and route-namespace chains on 2026-03-31.

**Files:**
- Later code targets: `src/runtime/sim_public.rs`, generated-site build surfaces, `robots.txt` or sitemap support surfaces, shared-host seed/discovery helpers where needed
- Later proof targets: focused discoverability tests, shared-host seed-contract proof, `Makefile`

**Work:**
1. Replace the current thin hard-coded dummy site with a contributor-generated public-content site on the protected host root rather than a nested sim-only prefix.
2. Expose broader site reachability through realistic public mechanisms:
   - root links,
   - realistic navigation,
   - `robots.txt` sitemap entries,
   - and sitemap documents.
3. Keep the generated HTML semantic and well structured, with extremely minimal hypertext-style presentation rather than dashboard-style chrome.
4. Make the contributor site viewable on local `make dev` even when adversary sim is idle.
5. Keep hidden or internal route catalogs out of the workers and seed contract.
6. Make the site itself, not private worker knowledge, responsible for richer discoverability.
7. Remove the old five-page hard-coded dummy site once the generated path is live.

**Acceptance criteria:**
1. Broader site surfaces become publicly reachable without simulator choreography.
2. Contributors can browse the site locally without first running adversary sim.
3. The workers still discover those surfaces only through public traversal and accepted hint documents.
4. The discovery contract continues to treat sitemap documents as bounded public hints, not authoritative surface truth.
5. The old hard-coded dummy-site model is removed rather than preserved as a parallel surface.

**Proof:**
1. Add and pass `make test-sim-public-discoverability-contract`.
2. Keep shared-host seed-contract proof green.
3. See the dedicated generated-site execution chain in [`2026-03-30-contributor-generated-public-content-sim-site-plan.md`](./2026-03-30-contributor-generated-public-content-sim-site-plan.md).

## Task 10: `SIM-REALISM-2I` Add Trusted-Ingress Client-IP Realism Without Attacker-Plane Privilege Creep

**Status:** Landed on 2026-03-31.

Detailed topology and trust-boundary addendum:

- [`2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md`](./2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md)

**Files:**
- Later code targets: trusted ingress or proxy adapter surfaces, adversary-sim supervisor or harness wiring, relevant runtime client-IP extraction or deployment docs
- Later proof targets: forwarded-header security tests, adversary-sim identity realism tests, `Makefile`

**Work:**
1. Add a truthful path for adversary-sim traffic to arrive through a Shuma-owned trusted ingress or proxy that can restore real client IP headers.
2. Keep the attacker workers forbidden from emitting privileged trust headers such as `x-shuma-forwarded-secret`.
3. Reuse the same forwarded-header trust gate used for external traffic rather than inventing a sim-only bypass.

**Acceptance criteria:**
1. When trusted sim ingress is configured, Shuma can observe parseable client IPs for sim traffic.
2. When it is not configured, the system explicitly reports degraded identity realism instead of silently collapsing to misleading values.
3. The attacker plane gains no new privilege to impersonate trusted ingress directly.
4. Closure includes measurable baseline-to-post-tranche change in the identity-realism dimensions this slice owns, rather than a qualitative claim that IP truth “looks better”.

**Proof:**
1. Add and pass `make test-adversary-sim-trusted-ingress-ip-realism`.
2. Keep attacker-plane contract checks and forwarded-header security tests green.

## Task 11: `SIM-REALISM-2J` Add Explicit Identity-Realism Receipts And Observer Wording

**Status:** Landed on 2026-03-31.

Detailed topology and trust-boundary addendum:

- [`2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md`](./2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md)

**Files:**
- Later code targets: receipt emitters, recent-run/read-model adapters, dashboard observer surfaces, related docs
- Later proof targets: machine-contract tests, rendered dashboard proofs, `Makefile`

**Work:**
1. Add explicit identity-realism receipt fields that record whether observed identity was:
   - trusted-ingress-backed,
   - pool-backed,
   - bucketed only,
   - or degraded.
2. Update operator surfaces so `unknown` and hashed bucket values like `h382` are labeled truthfully as degraded or bucketed identity, not as source IPs.
3. Keep those fields observer-only rather than turning them into defence truth.

**Acceptance criteria:**
1. Operators can distinguish real observed client-IP realism from degraded bucket-only identity.
2. UI wording no longer makes hashed bucket values look like realistic source addresses.
3. Receipt truth remains compact and machine-readable.
4. Closure includes a measurable before/after improvement in observer honesty about identity provenance, not just a new receipt field in isolation.

**Proof:**
1. Add and pass `make test-adversary-sim-identity-observer-truth`.
2. Keep relevant dashboard accountability and machine-contract tests green.

## Recommended Execution Order

1. `SIM-REALISM-3A`
2. `SIM-REALISM-3B`
3. `SIM-REALISM-3C`
4. `SIM-REALISM-3D`
5. `SIM-REALISM-3E`
6. only then reopen representative mixed-attacker or tuning-quality claims
