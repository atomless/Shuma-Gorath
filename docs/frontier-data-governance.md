# Frontier Data Governance

Date: 2026-02-27  
Status: Active (`SIM-V2-19` foundation)

This document defines outbound data-minimization rules for frontier adversary payloads.

Canonical schema: `scripts/tests/adversarial/frontier_payload_schema.v1.json`

## Allowed vs Forbidden

| Category | Rule |
| --- | --- |
| Allowed | Public crawl content summaries, synthetic attack metadata, scenario IDs/tiers/drivers, target base URL/path hints, deterministic run/profile identifiers. |
| Forbidden | Secrets, API keys, auth headers/tokens, cookies/session identifiers, raw admin payloads, raw user identifiers, raw IP addresses. |
| Quasi-identifiers | Fields that look like IP/user/contact identifiers are masked before schema validation. |

## Redaction Pipeline

Outbound payload preparation is ordered and deterministic:

1. Canonicalize payload structure and key ordering.
2. Classify fields (`allowed`, `forbidden`, `quasi_identifier`).
3. Drop forbidden fields.
4. Mask quasi-identifiers.
5. Validate against `frontier_payload_schema.v1`.
6. Emit sanitized payload only.

## Retention Rules

| Artifact | Default policy | Limit |
| --- | --- | --- |
| Raw frontier request/response payloads | Disabled | No persistence unless explicit future dev-only debug mode |
| Dev-only debug payload persistence | Optional (future explicit opt-in) | Max TTL `24h` |
| Normalized findings + replay metadata | Enabled | Max retention `30d` |

## Examples

Allowed examples:

- `scenario.id=scenario_allow`
- `traffic_model.cohort=adversarial`
- `target.base_url=http://127.0.0.1:3000`
- `public_crawl_content.scenario_description="allow scenario"`

Forbidden examples:

- `api_key=...`
- `authorization=Bearer ...`
- `cookie=session=...`
- `session_token=...`
- `scenario.ip=203.0.113.10` (must be masked to `[masked]`)
