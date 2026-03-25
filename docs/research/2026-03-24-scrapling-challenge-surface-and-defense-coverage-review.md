Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/tests/test_scrapling_worker.py`](../../scripts/tests/test_scrapling_worker.py)

# Scrapling Challenge Surface And Defense Coverage Review

## Question

Is Shuma using all of Scrapling's practical capability, especially for challenge or anti-bot interaction, and what planning follow-on is needed if the answer is no?

## Current Shuma-grounded state

Shuma's current Scrapling integration is still narrower than upstream Scrapling capability, even though the request-native lane is now materially wider than the original fetch-only baseline.

The runtime bootstrap pins `scrapling[fetchers]==0.4.2`, but its readiness check only proves `FetcherSession` and related request-fetcher dependencies, not browser or stealth fetchers.

Source: [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)

The live Scrapling worker imports only:

1. `FetcherSession`,
2. `Request`,
3. `Spider`.

It then constrains the worker plan to only three fulfillment modes:

1. `crawler`,
2. `bulk_scraper`,
3. `http_agent`.

Source: [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)

Those three modes are all request-native. They cover:

1. traversal and link discovery,
2. bulk retrieval and pagination-style scraping,
3. direct HTTP request mixes with cookies, JSON bodies, redirects, and headers.

They now do attempt request-native challenge and abuse surfaces such as:

1. Shuma `not_a_bot` submits,
2. puzzle submits,
3. PoW verification abuse,
4. and tarpit progress abuse.

But they still do not cover:

1. browser automation,
2. stealth browser interaction,
3. or Cloudflare-style interstitial solving.

Source: [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)

Shuma's explicit challenge-abuse attempts currently live in the deterministic sim lane, not the Scrapling lane:

1. generic challenge submit,
2. `not_a_bot` fail,
3. `not_a_bot` escalate-to-puzzle,
4. PoW verify,
5. tarpit progress.

Source: [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)

## Upstream Scrapling capability findings

Scrapling's official docs now clearly advertise broader capability than the current Shuma worker uses.

The official index states that Scrapling fetchers can bypass Cloudflare Turnstile or Interstitial protections and that browser automation is available through `DynamicFetcher` and `StealthyFetcher`.

Source: [Scrapling docs index](https://scrapling.readthedocs.io/en/latest/)

The official `StealthyFetcher` page is even more explicit. It says:

1. `StealthyFetcher` is a stealthy browser fetcher built on browser automation and Playwright-facing APIs,
2. it "easily bypasses all types of Cloudflare's Turnstile/Interstitial automatically",
3. it exposes `solve_cloudflare`,
4. and it claims support even for custom pages with embedded captcha.

Source: [Scrapling stealthy fetcher docs](https://scrapling.readthedocs.io/en/latest/fetching/stealthy/)

The upstream README also shows:

1. `StealthySession(..., solve_cloudflare=True)`,
2. one-off `StealthyFetcher.fetch(...)`,
3. `DynamicFetcher` for full browser automation,
4. and CLI examples using `stealthy-fetch ... --solve-cloudflare`.

Source: [Scrapling GitHub README](https://github.com/D4Vinci/Scrapling)

## Important distinction: upstream Cloudflare solving is not the same thing as Shuma challenge coverage

Even if Scrapling can solve Cloudflare Turnstile or interstitial challenges upstream, that does not automatically mean Shuma currently gains:

1. truthful `not_a_bot` coverage,
2. truthful puzzle coverage,
3. truthful PoW abuse coverage,
4. or truthful `automated_browser` category coverage.

Those are separate questions.

Shuma's current `not_a_bot` and puzzle semantics are app-specific. They are not generic Cloudflare challenges. Using Scrapling's browser or stealth fetchers would still require:

1. explicit worker-plan semantics,
2. exact allowed interaction boundaries,
3. new receipts proving which defenses were actually exercised,
4. and a clear category-vs-defense distinction.

## Category coverage vs defense-surface coverage

The current request-native Scrapling work correctly focused on category fulfillment first:

1. `indexing_bot`,
2. `ai_scraper_bot`,
3. `http_agent`.

That was the right first boundary.

But the challenge question exposes a second representativeness axis that Shuma does not yet model explicitly enough:

1. which taxonomy categories Scrapling can fulfill,
2. and which Shuma defense surfaces Scrapling should be expected to encounter or exercise.

Those are not identical.

A request-native `http_agent` persona may still be expected to hit:

1. rate pressure,
2. geo or IP policy,
3. challenge routing,
4. `not_a_bot`,
5. puzzle escalation,
6. and PoW verification abuse,

without thereby becoming a browser-agent lane.

## Governing principle

Adversary lanes must be attacker-faithful within Shuma's scope and safety boundaries.

That means:

1. if a lane claims to represent malicious traffic for a surface, it must use the tool the way a real attacker would use it for that surface,
2. not a neutered subset chosen only because it is easy to wire or cheap to run,
3. and not a library capability claim that has not been proven through Shuma's own runtime, receipts, and coverage surfaces.

So for Scrapling specifically:

1. if Shuma expects the Scrapling lane to stand in for malicious Scrapling-powered automation against challenge or anti-bot surfaces,
2. then the lane must actually attempt those surfaces in the way an attacker would,
3. or Shuma must state explicitly that the lane does not yet cover them.

## Decision

Shuma should not silently assume that the current Scrapling worker already captures Scrapling's full potential.

The stronger principle is:

1. current Shuma Scrapling integration proves a truthful request-native attacker baseline,
2. upstream Scrapling appears capable of significantly more through `StealthyFetcher` and `DynamicFetcher`,
3. Shuma should adopt attacker-relevant upstream Scrapling capability by default for Scrapling-owned surfaces,
4. and any omission should be explicit, justified, and auditable rather than left as a passive default.

## Recommended follow-on split

### 1. `SIM-SCR-CAP-1`

Add an active capability-maintenance tranche for Scrapling.

This should answer:

1. which upstream Scrapling capabilities are attacker-relevant for Scrapling-owned surfaces,
2. which of those Shuma already adopts,
3. which of those should now be adopted next,
4. and which are explicitly excluded with a recorded reason.

### 2. `SIM-SCR-CHALLENGE-2C`

Keep broader browser or stealth Scrapling adoption bounded to the owned-surface subset the capability matrix says Scrapling should own next.

### 3. `SIM-SCR-BROWSER-1`

Keep browser-runtime adoption for `automated_browser` separate.

That tranche is broader than owned request-native challenge interaction. It should stay focused on browser-runtime ownership of the `automated_browser` category and not blur into every widened Scrapling capability question.

## Result

The next question is no longer only "can Scrapling fulfill these categories?"

It is now also:

1. which Shuma defense surfaces should Scrapling be able to exercise,
2. how much of that remains request-native,
3. when does Shuma need Scrapling's browser or stealth runtime rather than its current fetcher-only worker,
4. and how will Shuma prove those claims with receipts instead of aspiration.
