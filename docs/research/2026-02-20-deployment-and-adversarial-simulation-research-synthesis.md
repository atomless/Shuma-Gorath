# Deployment and Adversarial Simulation Research Synthesis

Date: 2026-02-20  
Status: Completed

## Scope

This synthesis addresses two decisions:

1. What are the best deployment paths for:
   - single-host `self_hosted_minimal`
   - multi-instance `enterprise_akamai` on Akamai/Fermyon edge runtime
2. What test architecture best simulates crawler/scraper/bot behavior across the full botness and threat spectrum?

## Primary Source Findings

### Runtime and state constraints

1. Spin outbound networking is deny-by-default and requires explicit `allowed_outbound_hosts` entries in `spin.toml`.
2. Spin supports Redis from Rust via `spin_sdk::redis::Connection`.
3. Fermyon <abbr title="Key-Value">KV</abbr> storage has explicit limits, no compare-and-swap semantics, and global replication behavior that can surface stale reads.

Implication:
- Strict multi-instance ban consistency should use a shared distributed store path (Redis) for active-ban source of truth, not local-only per-instance stores.

### Akamai/Fermyon deployment posture

1. Akamai Functions is a Spin-based runtime on Akamai Connected Cloud.
2. Akamai Property Manager integration patterns exist for routing requests to Wasm Functions.
3. Akamai AppSec Network Lists are API-managed and can be activated with fast propagation (Akamai documents under-10-minute activation in current docs).

Implication:
- A production enterprise path can combine:
  - edge routing and first-pass controls in Akamai
  - Shuma policy enforcement at the Spin/Wasm layer
  - distributed Redis-backed ban/rate state for cross-instance consistency
  - optional asynchronous mirror of high-confidence ban state to Akamai Network Lists for perimeter suppression

### Adversarial simulation stack

1. Playwright is strong for browser-realistic end-to-end interaction simulation.
2. Crawlee supports both <abbr title="Hypertext Markup Language">HTML</abbr>-parser crawling and browser-driven crawling at scale.
3. Scrapy remains a mature non-browser crawler framework for high-volume scraper profiles.
4. k6 and Locust provide complementary load/traffic generation models for throughput/shape testing.

Implication:
- No single tool covers the full threat gradient. A layered harness should combine:
  - browser-realistic agents
  - non-browser scrapers
  - high-rate load generators
  - replay/evasion scenario drivers

## Research-Derived Gaps In Current Shuma State

1. Full-sync enterprise ban posture is not yet explicit:
   - external Redis exists, but strict no-drift operating mode and convergence <abbr title="Service Level Objective">SLO</abbr>s are not fully defined in docs/tests.
2. Multi-instance proof tests are limited:
   - current integration paths validate functional behavior, but not full two-instance convergence and outage/partition behavior as a required gate.
3. Bot simulation breadth is incomplete:
   - deterministic integration tests exist, but there is no canonical bot-profile matrix covering low-risk automation through high-threat adversarial traffic with pass/fail acceptance thresholds.

## Source Links

- Spin manifest `allowed_outbound_hosts`: [spinframework.dev](https://spinframework.dev/v2/manifest-reference)
- Spin Rust SDK Redis API: [spinframework.dev](https://spinframework.dev/v2/rust-components)
- Fermyon key-value limits/consistency caveats: [developer.fermyon.com](https://developer.fermyon.com/wasm-functions/kvs)
- Akamai Functions overview: [developer.fermyon.com](https://developer.fermyon.com/wasm-functions/quickstart)
- Akamai Property Manager integration for Wasm Functions: [developer.fermyon.com](https://developer.fermyon.com/wasm-functions/guides/integrate-akamai)
- Akamai Network Lists <abbr title="Application Programming Interface">API</abbr>: [techdocs.akamai.com](https://techdocs.akamai.com/application-security/reference/get-network-lists)
- Akamai Network List activation timing note: [techdocs.akamai.com](https://techdocs.akamai.com/application-security/reference/get-network-list-network-list-id-fast-activate)
- Playwright docs: [playwright.dev](https://playwright.dev/docs/intro)
- Crawlee docs: [crawlee.dev](https://crawlee.dev/)
- Scrapy docs: [docs.scrapy.org](https://docs.scrapy.org/en/latest/)
- k6 docs: [grafana.com](https://grafana.com/docs/k6/latest/)
- Locust docs: [docs.locust.io](https://docs.locust.io/en/stable/)
