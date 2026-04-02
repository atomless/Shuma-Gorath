# Root-Served Defence Confrontation And Scrapling Cadence Regression Plan

**Goal:** Recover hostile efficacy after the landed realism chain by proving that root-started Scrapling on the current generated public site now provokes, follows, and can be escalated by Shuma-served defences all the way into the tarpit, and by restoring bursty mode cadence without reintroducing fake route choreography or public links to defence surfaces.

**Architecture:** Reuse the existing generated root-hosted public site, policy-first request flow, Scrapling worker, owned-surface coverage contract, runtime surface gate, and Make-driven proof surface. Tighten worker-side defence recognition, generated-site traversal heuristics, owned-surface receipt honesty, and Scrapling per-mode time budgets. Preserve the same public-boundary and no-hidden-catalog rules.

**Tech Stack:** Rust adversary-sim realism profiles and lane runtime, Python Scrapling worker, Python runtime-surface integration gate, Makefile verification, backlog and sequencing docs.

**Related research:** [`../research/2026-04-01-root-served-defence-confrontation-and-scrapling-cadence-regression-review.md`](../research/2026-04-01-root-served-defence-confrontation-and-scrapling-cadence-regression-review.md)

---

## Guardrails

1. Do not make defence surfaces publicly discoverable through ordinary site navigation just to help Scrapling find them.
2. Do not add hidden route catalogs, simulator-only path hints, or Shuma-specific worker shortcuts.
3. Do not accept worker-declared hostile coverage when server-observed monitoring or event evidence says confrontation did not happen.
4. Do not recover cadence by making the lane unbounded; every stronger behavior must remain contract-backed and receipt-backed.
5. Do not reopen Game Loop or Tuning from narrative claims that realism is now “stronger”; closure must prove vigorous hostile pressure and real defence serving.
6. Do not satisfy tarpit proof by adding direct public tarpit navigation, hard-coded tarpit shortcuts, or any non-response-derived worker path knowledge.

## Task 1: `SIM-REALISM-REG-1A` Add An Honest Root-Served Defence Confrontation Gate

**Files:**
- Modify: `scripts/tests/adversary_runtime_toggle_surface_gate.py`
- Modify: `scripts/tests/test_adversary_runtime_toggle_surface_gate.py`
- Modify if evidence emission is missing: `src/providers/internal.rs`
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Tighten the runtime surface gate so root-started Scrapling on the current generated site must prove confrontation against a minimum hostile surface set through server-observed evidence rather than worker-side optimistic `surface_receipts` alone.
2. Require minimum mode breadth and minimum hostile-surface breadth in the recent run.
3. Require monitoring or event-log evidence showing that Shuma actually served or enforced relevant defences during the run.
4. When tarpit-owned abuse is claimed, require explicit tarpit-family runtime evidence rather than allowing closure from nearby challenge, PoW, maze, or rate-limit evidence alone.
5. If the existing tarpit entry/progress path does not emit sufficient server-observed evidence for that proof, add the minimal truthful runtime emission needed rather than weakening the gate.

**Acceptance criteria:**
1. A clean public `200 OK` root fetch alone can no longer satisfy `challenge_routing`, `rate_pressure`, or `geo_ip_policy` in the closure proof.
2. The live gate fails if Scrapling merely traverses public pages without provoking real defence interaction.
3. The proof starts at the host root and does not rely on public links to defence routes.
4. The live gate fails if Scrapling never earns observable tarpit escalation while still claiming tarpit-owned hostile coverage.

**Proof:**
1. Add and pass `make test-adversary-sim-root-served-defence-confrontation-unit`.
2. Add and pass `make test-adversary-sim-root-served-defence-confrontation`.

## Task 2: `SIM-REALISM-REG-1B` Fix Scrapling Root-Served Defence Handling And Generated-Site Hostile Traversal

**Files:**
- Modify: `scripts/supervisor/scrapling_worker.py`
- Modify: `scripts/tests/test_scrapling_worker.py`
- Modify: `src/observability/scrapling_owned_surface.rs` if the owned-surface success contract needs correction
- Modify: `docs/adversarial-operator-guide.md`

**Work:**
1. Stop recording `challenge_routing`, `rate_pressure`, and `geo_ip_policy` from ordinary clean public discovery responses.
2. Add honest response classification so root-served `NotABot`, challenge-puzzle, JS challenge, maze, rate-control, and related responses are recognized from the response itself.
3. Update request-native traversal heuristics so the generated site’s feed, archive, section, and dated-entry paths are treated as high-priority hostile terrain.
4. Make request-native and browser personas follow root-served challenge responses directly instead of requiring public links to defence routes.
5. Strengthen the aggressive request-native follow-through so challenge or not-a-bot abuse can persist long enough to earn tarpit escalation through Shuma’s normal defence policy rather than via direct tarpit shortcuts.

**Acceptance criteria:**
1. Request-native Scrapling can encounter and act on root-served not-a-bot or other defence responses even when the generated site never links to those defence paths.
2. Bulk-scraper and crawler heuristics now traverse the generated site’s feed and archive structure vigorously instead of remaining biased toward the old dummy-site `catalog` and `detail` shape.
3. Worker receipts no longer overclaim hostile-surface coverage from ordinary clean traversal alone.
4. At least one aggressive request-native persona can reach the tarpit from root-served hostile confrontation without any public tarpit link or hidden route hint.

**Proof:**
1. Keep and pass `make test-adversary-sim-scrapling-worker`.
2. Keep and pass `make test-adversary-sim-scrapling-category-fit`.
3. Keep and pass `make test-adversary-sim-scrapling-coverage-receipts`.

## Task 3: `SIM-REALISM-REG-1C` Rebalance Scrapling Cadence To Restore Bursty Hostile Pressure

**Files:**
- Modify: `src/admin/adversary_sim_realism_profile.rs`
- Modify: `scripts/tests/test_scrapling_worker.py`
- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`
- Modify: `docs/adversarial-operator-guide.md`

**Work:**
1. Reduce per-mode Scrapling time ceilings so one serialized five-mode cycle is no longer stretched across roughly one hundred seconds.
2. Preserve or strengthen hostile activity budgets and burst characteristics rather than recovering cadence by making the traffic milder.
3. Ensure the recovered cadence preserves enough persistent confrontational pressure for aggressive personas to be escalated into the tarpit on the real root-served defence path.
4. Leave same-lane multi-worker concurrency as a follow-on only if the honest acceptance gate still shows insufficient hostile pressure after budget rebalance.

**Acceptance criteria:**
1. Scrapling mode turnover in a normal watch window materially increases relative to the current regressed baseline.
2. Cadence recovery preserves or improves hostile action breadth rather than trading activity down for speed.
3. The implementation remains explicitly bounded and does not widen host knowledge.
4. Faster cadence does not come at the cost of weaker persistence through Shuma-served friction; aggressive personas must still be capable of earning tarpit escalation.

**Proof:**
1. Keep and pass `make test-adversary-sim-scrapling-realism`.
2. Keep and pass `make test-adversary-sim-pressure-envelope-realism`.
3. Keep and pass `make test-adversary-sim-root-served-defence-confrontation`.

## Recommended Implementation Order

1. `SIM-REALISM-REG-1A`
2. `SIM-REALISM-REG-1B`
3. `SIM-REALISM-REG-1C`
4. only then resume the contributor-facing environment-readiness chain in [`2026-04-01-contributor-friendly-adversary-proxy-pool-setup-plan.md`](./2026-04-01-contributor-friendly-adversary-proxy-pool-setup-plan.md)
