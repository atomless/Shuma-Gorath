## Akamai Integration Review: Shuma-Gorath

### Overview

The project integrates with Akamai's edge platform as an **optional upstream signal provider** for bot detection and request classification. The integration is designed with a trust boundary — edge-supplied headers are only consumed when cryptographically validated via `X-Shuma-Forwarded-Secret` (`src/lib.rs:48-62`). Three integration modes are supported via `EdgeIntegrationMode` (`src/config/mod.rs:212-223`):

| Mode | Behavior |
|---|---|
| **Off** | Ignores all Akamai edge outcomes |
| **Additive** | Uses Akamai scores as bounded input to local scoring (never sole decision-maker) |
| **Authoritative** | Allows strong-signal short-circuit actions from edge |

---

### Signal 1: GEO (Geolocation)

**How it works in the project:**
- Header extracted: `X-Geo-Country` in `src/signals/geo/mod.rs:26`
- Only consumed when `headers_trusted=true` (validated via forwarded secret)
- Used for geo-based routing and policy evaluation (e.g., challenge visitors from specific countries)
- Test fixtures exist for Brazil (`headers_geo_challenge_br.json`) and Russia (`headers_geo_maze_ru.json`)

**Akamai Documentation:**
- [Content Targeting / EdgeScape](https://techdocs.akamai.com/property-mgr/docs/content-tgting) — Primary docs for the `X-Akamai-Edgescape` header, field definitions (country_code, region_code, city, lat, long, timezone, asnum, network, etc.)
- [EdgeScape Behavior Reference (Property Manager API)](https://techdocs.akamai.com/property-mgr/reference/latest-edge-scape) — Technical API reference for configuring EdgeScape
- [User Location Object (EdgeWorkers)](https://techdocs.akamai.com/edgeworkers/docs/user-location-object) — Programmatic access to geo data within EdgeWorkers
- [User Location Data (Match Criteria)](https://techdocs.akamai.com/property-mgr/docs/user-loc-data) — Rule-based matching on user location
- [Akamai Developer — EdgeScape](https://developer.akamai.com/edgescape) — Developer overview

**Note:** The project extracts only `X-Geo-Country` (a single ISO country code). Akamai's EdgeScape provides much richer data (region, city, lat/long, ASN, network type, throughput) via the `X-Akamai-Edgescape` compound header. The project's header name (`X-Geo-Country`) suggests a custom mapping at the edge layer rather than consuming Akamai's native header format directly.

---

### Signal 2: Fingerprint (Bot Detection / Device Fingerprinting)

**How it works in the project:**

The fingerprint integration has two sub-planes:

**A. Transport fingerprints** (`src/signals/fingerprint.rs:172-178`):
- `X-Shuma-Edge-JA3` — TLS JA3 fingerprint hash
- `X-Shuma-Edge-JA4` — TLS JA4 fingerprint hash
- `X-Shuma-Edge-Browser-Family` — Detected browser family (chrome, firefox, safari, edge, other)
- `X-Shuma-Edge-Bot-Score` — Bot score (0.0–100.0)
- All extracted in `extract_transport_evidence()`, only trusted when forwarded secret validates

**B. Edge outcome payloads** (`src/providers/external.rs:37-47`):
- Received via `POST /fingerprint-report` endpoint
- `AkamaiEdgeOutcome` struct: `bot_score`, `action` (deny/block/challenge/monitor/allow), `detection_ids`, `tags`
- Normalized to confidence scores (`src/providers/external.rs:57-171`):
  - deny/block → confidence ≥ 9.5, `hard_signal=true`
  - challenge → confidence ≥ 6.5
  - monitor → confidence ≥ 3.5
  - allow → confidence ≥ 1.0
- In Additive mode, the edge signal is stored with TTL and contributes weight=2 to the composite botness score (`src/signals/fingerprint.rs:662-679`, signal key `fp_akamai_edge_additive`)

**Akamai Documentation:**
- [Detection Methods (Bot Manager)](https://techdocs.akamai.com/cloud-security/docs/detection-methods) — Core reference for how Bot Manager fingerprints requests (transparent, active, and behavioral detection)
- [Bot Score](https://techdocs.akamai.com/bot-manager/docs/bot-score) — The 0–100 algorithmic bot score, segment configuration (Cautious/Strict/Aggressive)
- [JA4 Client TLS Fingerprint Settings](https://techdocs.akamai.com/application-security/reference/get-ja4-fingerprint-settings) — JA4 fingerprint derived from TLS handshake
- [About Bots](https://techdocs.akamai.com/cloud-security/docs/about-bots) — Bot categories and classification
- [Handle Adversarial Bots](https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots) — Response strategies for sophisticated bots
- [Account Protector (Device Fingerprint + User Risk)](https://techdocs.akamai.com/cloud-security/docs/account-protector) — Device profiling and `Akamai-User-Risk` header
- [Client Reputation](https://techdocs.akamai.com/identity-cloud/docs/client-reputation-1) — IP-based reputation scoring (categories: DOSATCK, SCANTL, WEBATCK, WEBSCRP)
- [Client Reputation Reports](https://techdocs.akamai.com/security-ctr/docs/client-reputation-reports) — Dashboard for reputation traffic analysis
- [BotScore Object (EdgeWorkers)](https://techdocs.akamai.com/edgeworkers/docs/botscore-object) — Programmatic bot score access in EdgeWorkers

---

### Signal 3: Rate Limiting

**How it works in the project:**

The rate limiting integration is infrastructure-level rather than Akamai-signal-specific:
- `ExternalRateLimiterProvider` in `src/providers/external.rs:22-35` — Redis-backed rate limiter with outage modes
- `ExternalBanStoreProvider` — Redis-backed ban storage with sync support
- Rate usage feeds into the policy pipeline (`src/runtime/policy_pipeline.rs:154-160`) alongside fingerprint and geo signals
- The project does **not** currently parse Akamai-specific rate limiting headers (like `X-Throttling-Limit` or `X-Throttling-Rate`) — rate limiting runs independently in the local runtime, with Akamai's edge rate controls operating as a separate enforcement layer upstream

**Akamai Documentation:**
- [App & API Protector Overview](https://techdocs.akamai.com/cloud-security/docs/app-api-protector) — Umbrella product including WAF, bot mitigation, and rate controls
- [Set Protections (Rate Controls)](https://techdocs.akamai.com/cloud-security/docs/set-protections) — Configuring rate limiting policies, penalty box mechanism (10-min deny window)
- [API Throttling Configuration](https://techdocs.akamai.com/api-definitions/docs/api-throttling-config) — `X-Throttling-Limit` and `X-Throttling-Rate` headers forwarded to origin
- [API Throttling Overview](https://techdocs.akamai.com/key-traffic-mgmt/docs/api-throttling) — Per-second API traffic limiting
- [View WAF Rate Control Trends](https://techdocs.akamai.com/security-ctr/docs/view-waf-rate-control-trends) — Monitoring rate control activity
- [Rate Policy Options (Terraform)](https://techdocs.akamai.com/terraform/docs/rate-policy-options) — Infrastructure-as-code rate policy configuration
- [Create a Rate Policy (API)](https://techdocs.akamai.com/application-security/reference/post-rate-policies) — REST API for rate policy creation

---

### Architectural Observations

1. **Trust boundary is well-defined.** The `forwarded_ip_trusted()` gate in `src/lib.rs` ensures edge headers are never blindly trusted. All Akamai-sourced data flows through this check.

2. **Additive mode is the conservative default.** The project treats Akamai signals as one input among many rather than a sole authority, which is a sound defense-in-depth posture.

3. **Custom header namespace.** The project uses `X-Shuma-Edge-*` headers rather than Akamai's native `X-Akamai-*` headers, indicating an edge adapter layer (likely an EdgeWorker or Property Manager rule) that normalizes Akamai data into the project's expected format before forwarding to origin.

4. **GEO is narrow.** Only country code is consumed. If finer-grained geo decisions are needed (region, city, ASN), the integration could be expanded to parse more of EdgeScape's fields.

5. **Rate limiting is decoupled.** Unlike GEO and fingerprint signals which flow from Akamai's edge, rate limiting runs independently in the local runtime (Redis-backed). There is no consumption of Akamai's `X-Throttling-*` headers — the two systems operate as independent layers.

---

