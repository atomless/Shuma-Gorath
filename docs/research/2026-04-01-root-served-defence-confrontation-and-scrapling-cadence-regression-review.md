Date: 2026-04-01
Status: Proposed

Related context:

- [`2026-03-30-adversary-lane-traffic-realism-and-cadence-review.md`](2026-03-30-adversary-lane-traffic-realism-and-cadence-review.md)
- [`2026-03-30-adversary-lane-wild-traffic-gap-review.md`](2026-03-30-adversary-lane-wild-traffic-gap-review.md)
- [`../plans/2026-04-01-root-served-defence-confrontation-and-scrapling-cadence-regression-plan.md`](../plans/2026-04-01-root-served-defence-confrontation-and-scrapling-cadence-regression-plan.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)
- [`../../src/runtime/policy_graph.rs`](../../src/runtime/policy_graph.rs)
- [`../../src/runtime/effect_intents/response_renderer.rs`](../../src/runtime/effect_intents/response_renderer.rs)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/admin/adversary_sim_realism_profile.rs`](../../src/admin/adversary_sim_realism_profile.rs)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/tests/adversary_runtime_toggle_surface_gate.py`](../../scripts/tests/adversary_runtime_toggle_surface_gate.py)
- [`../../scripts/tests/test_scrapling_worker.py`](../../scripts/tests/test_scrapling_worker.py)

# Objective

Explain why the landed realism chain can currently produce less hostile Scrapling pressure on the root-hosted generated public site than before, why the current proof can still close without showing earned tarpit escalation, and define the exact recovery direction without reintroducing fake route choreography or public links to defence surfaces.

# Current Ground Truth

## 1. Shuma serves challenge and maze surfaces by policy, not by public navigation

The request path already routes through policy decisions before the generated public site is allowed to answer:

1. policy evaluation and response intents are built in [`../../src/runtime/policy_graph.rs`](../../src/runtime/policy_graph.rs),
2. those intents are rendered in [`../../src/runtime/effect_intents/response_renderer.rs`](../../src/runtime/effect_intents/response_renderer.rs),
3. and request handling in [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs) serves `NotABot`, `Challenge`, `JsChallenge`, `Maze`, and related friction before the generated public site handler is reached.

That means `/pow`, maze, not-a-bot, and related defences are not supposed to be ordinary public content links on the generated site. If Scrapling does not encounter them from root-started traffic, the realistic interpretations are:

1. Scrapling is not generating enough hostile or bot-like pressure to trigger them,
2. the worker is not recognizing or exploiting defence responses when they are served directly on ordinary public paths,
3. Shuma is not serving them when it should,
4. or some combination of those.

## 2. The generated site changed, but request-native Scrapling still prioritizes the old dummy-site shape

The current generated site is timeline and archive oriented. It primarily exposes:

1. `/`,
2. `/about/`,
3. `/research/`,
4. `/plans/`,
5. `/work/`,
6. `/page/N/`,
7. and dated entry pages.

But request-native Scrapling is still biased toward the old fake-site structure:

1. `_looks_bulk_scraper_public_target()` prefers `/`, `catalog`, and `/detail/`,
2. `_bulk_scraper_priority()` is still keyed to `catalog` and `/detail/`,
3. `_crawler_link_priority()` still prioritizes `redirect`, `/page`, `catalog`, `challenge`, `/pow`, and `/maze/`.

Those heuristics live in [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py).

So even before policy thresholds are considered, some request-native personas are simply less well adapted to the new public terrain than they were to the old dummy site.

## 3. Some current Scrapling owned-surface receipts can succeed on ordinary clean `200` pages

The current worker still records `challenge_routing`, `rate_pressure`, and `geo_ip_policy` from plain public discovery responses through `_public_discovery_surface_ids()` and `_coverage_status_for_http_status()` in [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py).

The canonical owned-surface contract in [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs) treats those surfaces as satisfied by `pass_observed` under the `mixed_outcomes` success contract.

That means a plain clean public `200 OK` response can currently satisfy:

1. `challenge_routing`,
2. `rate_pressure`,
3. and `geo_ip_policy`

without Shuma actually serving or enforcing those defences.

This is not only optimistic. It is actively dangerous because it can hide the exact regression we are trying to detect.

## 4. Current request-native challenge handling still assumes discovery through public links or forms

The worker’s request-native personas still mostly do this:

1. fetch the root,
2. extract links and forms,
3. look for explicit links to challenge or defence routes,
4. then attempt those follow-on interactions.

That logic is in the request-native persona handlers in [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py).

But a root-served `NotABot`, challenge-puzzle, JS challenge, or maze response is itself the confrontation surface. It should not depend on the root page containing a public link to `/challenge/not-a-bot-checkbox`, `/pow`, or `/maze/...`.

## 5. Scrapling cadence regressed because long per-mode budgets now combine with one-worker serialization

Scrapling dispatch is still serialized at the lane-runtime layer: the next worker is skipped while one is pending in [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs).

At the same time, current per-mode time budgets in [`../../src/admin/adversary_sim_realism_profile.rs`](../../src/admin/adversary_sim_realism_profile.rs) are large:

1. crawler `12s`,
2. bulk scraper `30s`,
3. browser automation `20s`,
4. stealth browser `24s`,
5. http agent `18s`.

That yields about `104s` of ceiling for one full five-mode cycle, far above the old flat `2s` ceiling per mode. In practice this means:

1. fewer mode rotations per watch window,
2. fewer category fulfillments observed,
3. fewer surfaces touched,
4. and fewer defence confrontations or bans,

even when each individual mode is “more realistic” on paper.

## 6. The current live proof can still pass without showing earned tarpit escalation

Shuma already has a real policy path that can sink sufficiently abusive challenge or not-a-bot failures into the tarpit when the tarpit provider is available:

1. challenge and not-a-bot abuse outcomes can classify to `TarpitOrShortBan` in [`../../src/runtime/request_router.rs`](../../src/runtime/request_router.rs),
2. `enforce_tarpit_or_short_ban(...)` first calls the maze or tarpit provider before falling back to a short ban,
3. and the internal provider already serves real tarpit entry and progress handling in [`../../src/providers/internal.rs`](../../src/providers/internal.rs).

So if Scrapling is sufficiently invasive and persistent, earned tarpit entry should be a natural downstream consequence of Shuma's current defence posture.

But the current runtime surface gate still allows closure without proving that path. It currently:

1. accepts any meaningful defence-event family rather than specifically requiring tarpit evidence when tarpit-owned abuse is claimed,
2. relies more heavily on hostile-surface breadth than on explicit tarpit-family runtime evidence,
3. and therefore can still pass when Scrapling provokes challenge, PoW, maze, or rate-limit friction without ever proving that the same malevolent progression reached the tarpit.

That is too weak for the current priority. The closure proof must now show both that Shuma's defences are functioning strongly enough to escalate a persistent hostile visitor into the tarpit and that Scrapling is actually forceful enough to get there honestly.

# Findings

## 1. The correct recovery is not richer public linking of defence routes

Adding ordinary crawlable links to `/pow`, maze, not-a-bot, or other defence paths on the generated site would damage the authenticity of Scrapling traffic and violate the public-boundary model.

The generated site should stay a realistic public terrain. Defence surfaces must be confronted because Shuma serves them in response to hostile signals, not because the site advertises them.

## 2. The next closure gate must be server-observed defence confrontation, not worker-declared coverage alone

The existing runtime surface gate already has access to live operator snapshot and monitoring evidence in [`../../scripts/tests/adversary_runtime_toggle_surface_gate.py`](../../scripts/tests/adversary_runtime_toggle_surface_gate.py).

That is the correct proof surface to tighten. Closure should require:

1. root-started Scrapling on the current generated site,
2. minimum observed fulfillment-mode breadth,
3. minimum hostile-surface coverage,
4. and minimum server-observed defence-event evidence,

instead of relying on optimistic worker-side `surface_receipts` alone.

## 3. The next closure gate must explicitly require earned tarpit evidence

The active regression is no longer only “does Scrapling hit enough friction?” It is also:

1. does Shuma escalate challenge abuse into tarpit when it should,
2. and does Scrapling persist malevolently enough to actually trigger that escalation from an ordinary public root entrypoint.

So the strengthened gate must fail unless:

1. root-started hostile traversal provokes real Shuma-served challenge or related friction,
2. the follow-through remains sufficiently invasive and persistent,
3. and the server-observed evidence includes real tarpit-family progression rather than only adjacent friction families.

## 4. Request-native Scrapling needs two simultaneous fixes

The worker must:

1. recognize and exploit root-served challenge responses directly,
2. and adapt its public traversal heuristics to the generated site’s feed, archive, and dated-entry structure.

Doing only one of those fixes would still leave meaningful efficacy on the table.

## 5. Scrapling also needs more persistent abusive follow-through once friction appears

The worker already knows how to follow response-derived tarpit paths without public links. That part is good.

But the remaining malevolence gap is whether the aggressive request-native personas keep enough pressure on root-served friction to earn the tarpit naturally. The relevant follow-through must remain:

1. root-started,
2. same-origin,
3. response-derived,
4. and bounded by persona contracts rather than simulator convenience.

The recovery therefore needs stronger persistent abuse behavior after challenge, not-a-bot, PoW, or related interstitial friction, especially for the more aggressive request-native personas.

## 6. Cadence must become violent and bursty again

The user-facing symptom is correct: current Scrapling cadence can feel pedestrian and impotent relative to the earlier baseline.

The simplest recovery is not necessarily multiple concurrent Scrapling workers. The first recovery should be:

1. shorter per-mode time ceilings,
2. preserved or increased hostile activity budgets,
3. and faster rotation across the mode set

so one full Scrapling cycle again fits a meaningful watch window and produces visible confrontation pressure.

If that still proves insufficient after honest gating, only then should same-lane multi-worker concurrency be reopened.

# Recommended Recovery Direction

1. Add a hard acceptance gate that proves root-started Scrapling reaches a minimum hostile surface set and minimum category coverage on the current generated site, using server-observed defence evidence.
2. Strengthen that gate further so it fails unless earned tarpit escalation is visible through server-observed evidence when tarpit-owned abuse is claimed.
3. Fix the worker so request-native and browser personas treat root-served challenge responses as confrontation surfaces rather than requiring public links to them.
4. Retune generated-site traversal heuristics so request-native Scrapling actually attacks the root-hosted feed and archive terrain vigorously.
5. Increase persistent hostile follow-through after Shuma-served friction so aggressive personas can naturally be escalated into the tarpit by Shuma's existing defence policy.
6. Rebalance Scrapling cadence so one serialized five-mode cycle is no longer stretched across about one hundred seconds.
7. Keep the no-hidden-catalog and no-public-defence-link rules intact.
