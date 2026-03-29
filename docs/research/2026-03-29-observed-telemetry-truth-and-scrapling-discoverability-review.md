Date: 2026-03-29
Status: Proposed

Related context:

- [`../plans/2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md`](../plans/2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md)
- [`../plans/2026-03-29-game-loop-exact-observer-truth-plan.md`](../plans/2026-03-29-game-loop-exact-observer-truth-plan.md)
- [`../research/2026-03-29-game-loop-exact-observer-truth-review.md`](2026-03-29-game-loop-exact-observer-truth-review.md)
- [`../research/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md`](2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md)
- [`../plans/2026-03-25-sim-llm-1a-black-box-contract-implementation-plan.md`](../plans/2026-03-25-sim-llm-1a-black-box-contract-implementation-plan.md)
- [`../../AGENTS.md`](../../AGENTS.md)

# Observed Telemetry Truth And Scrapling Discoverability Review

## Questions

1. What explicit repo policy should govern presentation layers after the recent Game Loop observer-truth regression?
2. Why is Game Loop still showing Scrapling receipts like `GET /sim/public/search?q=scrapling-bulk-scraper`?
3. Are Scrapling and the bounded LLM attacker currently operating from attacker-faithful discoverable knowledge, or are they still being helped by Shuma-specific choreography?
4. Is Shuma currently using simulation-only tagging as a defense input rather than an attribution input?

## Findings

### 1. The repo still lacked an explicit release-blocking ban on presentation-side fabrication

The core policy chain already says telemetry is the map and that simulator labels must not become runtime or tuning truth, but it did not yet state the narrower UI rule strongly enough:

1. presentation layers must not invent or stitch execution claims just to satisfy a UI requirement,
2. absence of exact recorded evidence must render as absence,
3. and a plausible-looking fallback is worse than an explicit unknown.

That gap is exactly what allowed the recent Game Loop observer regression: the UI tried to stay populated and silently crossed from projection into fabrication.

### 2. Scrapling still contained out-of-band choreography

The current Scrapling worker still does two things that violate the attacker-faithful intent:

1. it emits persona-coded public requests such as `q=scrapling-bulk-scraper`, `q=scrapling-http-agent`, and `q=scrapling-crawler-probe`,
2. and it receives a `runtime_paths` bundle that hands it direct knowledge of `/sim/public/search`, `/challenge/not-a-bot-checkbox`, `/challenge/puzzle`, `/pow`, `/pow/verify`, `/tarpit/progress`, and `/maze/...` before it has discovered any of them from the host.

That means the worker is not acting from root URL plus bounded public hints. It is being given a private host map for convenience.

### 3. The bounded LLM attacker is closer to the intended boundary than Scrapling

The LLM lane already keeps the stronger black-box contract:

1. `host_root_only_entrypoint=true`,
2. `public_knowledge_only=true`,
3. `shuma_blind=true`,
4. allowed hint sources limited to `robots_txt`, sitemap references, and traversal-visible pages,
5. forbidden knowledge sources explicitly include Shuma repo, source, internal routes, and judge state.

It still receives a category objective because lane ownership is explicit, but it is not being handed internal route maps the way Scrapling is.

### 4. The visible Game Loop smell came from real request content, not only from UI projection

The `Rate Pressure` row the user saw is not just a display bug. The worker really is issuing requests whose query string contains persona names. The UI simply surfaced a truthful sample path for a bad simulator implementation.

### 5. Simulation tags are still an attribution path, but the current audit did not show them driving core enforcement actions

The request path still validates signed sim headers and records `traffic_origin=adversary_sim` for monitoring and benchmarking. That metadata currently appears to be used for:

1. origin attribution,
2. recent-run and operator-snapshot projection,
3. adversary-sim filtering in benchmarks and recognition evaluation,
4. and suppression of likely-human monitoring side effects for sim traffic.

This audit did not find evidence that core defense surfaces are choosing allow, block, challenge, or score outcomes from `sim_run_id`, `sim_profile`, or `sim_lane`. The risk is still architectural: those headers reach the request boundary, so the repo must freeze the rule that they remain attribution-only and must not become a shortcut defense signal.

## Conclusions

1. Add a new non-negotiable AGENTS rule: presentation layers must never fabricate or stitch fallback execution claims beyond what telemetry actually recorded.
2. Remove Scrapling `runtime_paths` from the worker contract.
3. Rewrite Scrapling personas so they operate from discoverable public inputs only:
   - root URL,
   - optional `robots.txt`/sitemap hints,
   - traversal-visible links,
   - and forms or redirects actually observed during the run.
4. Remove persona/category strings and synthetic `/agent/*` helper routes from request emission.
5. Keep simulation tags as attribution-only in this tranche; do not widen them into defense truth, and add documentation pressure that this remains a hard boundary.

## Acceptance boundary for the implementation tranche

This tranche is complete only when all of the following are true:

1. AGENTS contains the explicit no-fabrication rule for presentation layers.
2. Scrapling worker plans no longer carry out-of-band route maps.
3. Scrapling receipts no longer expose persona/category strings in public request paths.
4. Scrapling request-native personas reach any owned surfaces only through discoverable host content observed during the run.
5. The bounded LLM lane remains host-root/public-hint only.
6. Focused proof covers the worker contract, admin beat payload shape, and at least one rendered Game Loop receipt path.
