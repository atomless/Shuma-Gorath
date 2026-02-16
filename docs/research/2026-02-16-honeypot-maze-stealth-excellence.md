# Honeypot + Maze Stealth Excellence (Research Synthesis)

Date: 2026-02-16  
Status: Active, pre-launch, no-compat constraints

## Why this batch now

Shuma is pre-launch, so we can remove detectable legacy surfaces instead of preserving them.
The target is to minimize stable deception fingerprints while preserving low-cost defender operation and strong escalation control.

## Latest intelligence snapshot (2025-2026)

1. Covert labyrinth patterns are now mainstream.
   Cloudflare AI Labyrinth docs (updated Jan 23, 2026) describe hidden links that are not visible to humans and do not alter site appearance/SEO, explicitly framing deceptive crawl deflection as invisible in-page routing rather than public trap paths.

2. Adversaries are now clearly adapting with stealth crawling.
   Cloudflare’s Aug 4, 2025 incident write-up documents crawler identity obfuscation (user-agent spoofing, ASN/IP rotation, robots non-compliance), reinforcing that static blocklists and semantic trap naming are weak if exposed.

3. Sequence/timing provenance is first-class in enterprise bot controls.
   Cloudflare sequence rules track operation order and elapsed timing, with explicit fields for current op, previous ops, and milliseconds since prior op. This directly validates Shuma’s operation ID and ordering-window primitives.

4. JavaScript verification is a signal plane, not a silver bullet.
   Cloudflare JavaScript detection guidance states first request often lacks JS outcome and enforcement should be done with explicit policy rules. This supports Shuma’s placement: collect JS/browser signals early, enforce via policy thresholds.

5. Bot identity based only on IP/UA is brittle.
   Cloudflare Web Bot Auth (May 15, 2025) argues for cryptographic bot identity (HTTP message signatures/mTLS) because UA spoofing and IP-range drift are operationally unreliable.

6. Tiered response by confidence is standard enterprise practice.
   Akamai’s adversarial bot guidance recommends monitor-first tuning and score-based escalation (monitor/challenge/deny), aligned with Shuma’s gradual escalation ladder.

7. `robots.txt` remains advisory, not access control.
   RFC 9309 states REP rules are not authorization. Robots directives must not be treated as a security gate.

8. Honeypot fingerprinting pressure is increasing.
   Recent literature and surveys continue to highlight detection/evasion progress and the need to reduce stable, repeated artifacts in deception systems.

## Research-backed design requirements

R1. Remove semantic deception route labels from live paths (`maze`, `trap`, `honeypot`).
R2. Use opaque, deployment-specific deception namespaces derived from secret material.
R3. Keep entry, support endpoints (`checkpoint`, `issue-links`), and maze assets under the same opaque namespace.
R4. Keep robots policy focused on crawler communication; do not advertise deception paths.
R5. Treat JS/browser verification as signal collection with policy-driven enforcement thresholds.
R6. Promote sequence primitives (operation IDs, ordering windows, timing thresholds) as canonical anti-automation signals.
R7. Preserve operator observability/admin preview while avoiding public giveaway markers in live content/headers.
R8. Prefer authoritative edge signals (when present) as confidence multipliers, not replacements for local stealth controls.

## Execution mapping for Stage 2.7

- `MZ-S1` + `MZ-S4`: remove explicit trap route/robots disclosure.
- `MZ-S2` + `MZ-S3`: ship opaque namespace + helper API + namespaced support/assets.
- `MZ-S5`: ensure preview/dashboard route handling uses helpers and remains non-operational.
- `MZ-S6`: add route stealth/canonicalization regressions (legacy `/maze/*` and `/trap/*` rejection, malformed prefix rejection).
- `MZ-S7`: update operator/public docs to opaque-route model.
- `MZ-S8`: run full verification and record behavior/resource delta.

## Operating split

### `self_hosted_minimal`

- Must implement and own route stealth, token/path integrity checks, preview safety, and regression tests.

### `enterprise_akamai`

- Can supply high-confidence outcomes and behavioral context to accelerate escalation.
- Should not replace Shuma’s internal token and traversal integrity controls.

## Sources

- Cloudflare AI Labyrinth docs (Last updated Jan 23, 2026): https://developers.cloudflare.com/bots/additional-configurations/ai-labyrinth/
- Cloudflare Sequence Rules docs: https://developers.cloudflare.com/bots/additional-configurations/sequence-rules/
- Cloudflare JavaScript detections docs: https://developers.cloudflare.com/bots/reference/javascript-detections/
- Cloudflare static resource protection docs: https://developers.cloudflare.com/bots/additional-configurations/static-resources/
- Cloudflare Web Bot Auth blog (May 15, 2025): https://blog.cloudflare.com/web-bot-auth/
- Cloudflare stealth crawler analysis (Aug 4, 2025): https://blog.cloudflare.com/perplexity-is-using-stealth-undeclared-crawlers-to-evade-website-no-crawl-directives/
- Cloudflare detection heuristics + detection IDs (Mar 19, 2025): https://blog.cloudflare.com/bots-heuristics/
- Akamai detection methods docs: https://techdocs.akamai.com/cloud-security/docs/detection-methods
- Akamai adversarial bot handling docs: https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- RFC 9309 (Robots Exclusion Protocol): https://www.rfc-editor.org/rfc/rfc9309.html
- PathMarker (Cybersecurity, SpringerOpen): https://cybersecurity.springeropen.com/articles/10.1186/s42400-019-0023-1
- Survey of contemporary honeypots (JNCA 2023): https://www.sciencedirect.com/science/article/pii/S108480452300156X
- TMA 2025 OT honeypot fingerprinting metadata: https://orbit.dtu.dk/en/publications/towards-agnostic-operational-technology-ot-honeypot-fingerprintin/
