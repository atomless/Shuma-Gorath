Date: 2026-03-25
Status: Proposed planning driver

Related context:

- [`2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md`](2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md)
- [`../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../plans/2026-03-25-scrapling-full-attacker-capability-principle-plan.md`](../plans/2026-03-25-scrapling-full-attacker-capability-principle-plan.md)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)
- [Scrapling static fetchers docs](https://scrapling.readthedocs.io/en/latest/fetching/static/)
- [Scrapling dynamic fetchers docs](https://scrapling.readthedocs.io/en/latest/fetching/dynamic.html)
- [Scrapling stealth fetchers docs](https://scrapling.readthedocs.io/en/latest/fetching/stealthy.html)
- [Scrapling overview](https://scrapling.readthedocs.io/en/latest/overview.html)

# Question

What is the refreshed full-power Scrapling capability matrix for the non-agent or non-LLM adversary spectrum Shuma assigns to Scrapling, now that the project has explicitly rejected the earlier “request-native only unless a later local gap forces widening” posture?

# Findings

## 1. Shuma still only boots the request-native Scrapling subset

Today Shuma boots only:

1. `FetcherSession`,
2. `Request`,
3. `Spider`

through [`scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py), while [`scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh) only checks `FetcherSession` readiness.

That means the current lane is still a truthful request-native baseline, not a full-power Scrapling implementation.

## 2. Upstream Scrapling exposes three meaningful capability layers

Official upstream docs currently expose:

1. request-native fetchers and sessions with browser impersonation, stealthy header generation, session continuity, multiple HTTP verbs, and proxy support in the [static fetcher docs](https://scrapling.readthedocs.io/en/latest/fetching/static/),
2. dynamic browser fetchers and sessions with browser automation, `page_action`, `wait_selector`, `network_idle`, `real_chrome`, and shared browser sessions in the [dynamic fetcher docs](https://scrapling.readthedocs.io/en/latest/fetching/dynamic.html),
3. stealth browser fetchers and sessions with `solve_cloudflare`, `block_webrtc`, `hide_canvas`, `allow_webgl`, and stealth browser automation in the [stealth fetcher docs](https://scrapling.readthedocs.io/en/latest/fetching/stealthy.html).

The [overview](https://scrapling.readthedocs.io/en/latest/overview.html) and static docs also make clear that `impersonate` and `stealthy_headers` are part of the normal supported fetcher surface, not hidden extras.

## 3. Category ownership and capability ownership must stay separate

Shuma's current taxonomy ownership for Scrapling is still:

1. `indexing_bot`,
2. `ai_scraper_bot`,
3. `http_agent`

in [`src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs).

That does **not** mean Scrapling must remain request-native only.

It means:

1. dynamic or stealth Scrapling may be used where needed to exercise Scrapling-owned surfaces,
2. without automatically claiming `automated_browser`,
3. `browser_agent`,
4. or `agent_on_behalf_of_human`.

The category-ownership question remains separate from the tool-capability question.

## 4. The earlier `SIM-SCR-CAP-1` matrix is now too narrow

The earlier matrix freeze was useful when the project needed to stop overclaiming.

But it explicitly assigned most browser and stealth capability away from the current lane by default. Under the stronger attacker-faithfulness principle that is now too conservative:

1. it treats real attacker capability as optional,
2. it risks leaving Shuma tuned against a polite subset,
3. and it obscures which omissions are truly justified versus merely inherited from an older narrower remit.

So `SIM-SCR-FULL-1A` must supersede the earlier request-native-bounded matrix as the current mainline source of truth.

## 5. Refreshed full-power matrix

| Capability family | Upstream evidence | Current Shuma state | Refreshed outcome |
| --- | --- | --- | --- |
| Request-native browser impersonation, header shaping, session continuity, crawl or traversal, and hostile verb coverage | [static fetchers](https://scrapling.readthedocs.io/en/latest/fetching/static/) | Partly adopted already through `FetcherSession`, `Request`, `Spider`, and the request-native fidelity slice | **Keep adopted** |
| Request-native proxy support and routed-origin variation | [static fetchers](https://scrapling.readthedocs.io/en/latest/fetching/static/) | Not wired | **Explicit temporary exclusion** from the current shared-host mainline because it needs separate origin-distribution infrastructure and would otherwise blur local-loop comparability; keep it recorded as an omission rather than pretending it is unnecessary |
| Dynamic browser automation for owned surfaces, including `page_action`, wait conditions, `network_idle`, and `real_chrome` | [dynamic fetchers](https://scrapling.readthedocs.io/en/latest/fetching/dynamic.html) | Not wired | **Adopt in `SIM-SCR-FULL-1B` where owned surfaces need real browser execution** |
| Stealth browser automation for owned surfaces, including `solve_cloudflare`, `block_webrtc`, `hide_canvas`, and `allow_webgl` | [stealth fetchers](https://scrapling.readthedocs.io/en/latest/fetching/stealthy.html) | Not wired | **Adopt in `SIM-SCR-FULL-1B` where owned surfaces need real stealth execution** |
| Solver or bypass-style challenge capability as a class of malicious automation behavior | [stealth fetchers](https://scrapling.readthedocs.io/en/latest/fetching/stealthy.html) | Not wired | **In scope for Shuma-owned challenge surfaces where realistic**, but upstream Cloudflare support is only evidence of capability class, not proof Shuma can already solve its own challenge stack |
| Persistent browser-profile or CDP-attached operation | [dynamic fetchers](https://scrapling.readthedocs.io/en/latest/fetching/dynamic.html) | Not wired | **Adopt only where required by an owned surface and receipt-backed proof can show why**; do not widen by default without a surface-level need |
| Truthful `automated_browser` category fulfillment | N/A; this is a Shuma taxonomy question rather than an upstream feature | Still separate | **Keep separate in `SIM-SCR-BROWSER-1`** |

## 6. What counts as a justified omission now

Under the stronger principle, the only acceptable reasons not to adopt an upstream Scrapling capability are:

1. it does not map to a Scrapling-owned surface,
2. it belongs to a separate later category-ownership lane,
3. it cannot be safely or truthfully run inside the current harness,
4. or Shuma cannot yet receipt-prove its behavior.

Anything else is silent drift and should not survive into the next implementation tranche.

# Result

`SIM-SCR-FULL-1A` should replace the older request-native-bounded matrix as the live driver for the next coding slices.

That means:

1. request-native fidelity remains the truthful baseline already landed,
2. dynamic and stealth Scrapling for owned surfaces are now in-scope by default rather than assigned away by default,
3. `automated_browser` still remains a separate later category-ownership question,
4. proxy or origin-distribution behavior remains the one explicit temporary exclusion in the current shared-host mainline,
5. and `SIM-SCR-FULL-1B` must implement against this fuller matrix before the strict human-only loop is treated as operationally proven.
