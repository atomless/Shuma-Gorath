# 🐙 <abbr title="General Data Protection Regulation">GDPR</abbr> / Privacy Review (Telemetry, Logging, and Cookies)

Date: 2026-02-19  
Scope: current Shuma runtime, admin <abbr title="Application Programming Interface">API</abbr>/dashboard, telemetry, and storage behavior

Legal note: this is an engineering compliance review, not legal advice.

## 🐙 Decision Summary

- Default Shuma deployments do not ship advertising, profiling, or third-party analytics cookies.
- Current browser storage is limited to security/auth/session operations and short-lived dashboard operational caching.
- For <abbr title="European Union">EU</abbr>/<abbr title="United Kingdom">UK</abbr> ePrivacy-style rules, this is generally in the "strictly necessary" category for service security/authentication, so a consent banner is typically not required by default.
- A privacy/cookie disclosure is still required (what is stored, why, for how long, and who can access it).
- If operators add non-essential client storage (analytics, marketing, cross-site tracking, personalization), prior consent is required before setting that storage.

## 🐙 Data Inventory

### Client-side cookies/storage

| Item | Purpose | Lifetime | Data category | Consent posture |
| --- | --- | --- | --- | --- |
| `js_verified` cookie | <abbr title="JavaScript">JS</abbr> verification gate for bot defence | `Max-Age=86400` | security token (<abbr title="Internet Protocol">IP</abbr>-bound <abbr title="Hash-based Message Authentication Code">HMAC</abbr>) | generally strictly necessary (security) |
| `shuma_fp` cookie | short-lived persistence marker in verification flow | `Max-Age=1800` | marker token | generally strictly necessary (abuse detection) |
| `shuma_admin_session` cookie | dashboard/admin authentication session | `Max-Age=3600` | session identifier | strictly necessary (authenticated admin service) |
| `localStorage: shuma_dashboard_cache_monitoring_v1` | short-lived monitoring cache | app <abbr title="Time To Live">TTL</abbr> `60s` | operational telemetry snapshot | disclosure required; consent usually not required for admin-ops function |
| `localStorage: shuma_dashboard_cache_ip_bans_v1` | short-lived ban-list cache | app <abbr title="Time To Live">TTL</abbr> `60s` | operational telemetry snapshot | disclosure required; consent usually not required for admin-ops function |
| `localStorage: shuma_dashboard_auto_refresh_enabled` | operator preference | persistent until changed | <abbr title="User Interface">UI</abbr> preference flag | disclose; in strict jurisdictions confirm exemption scope with counsel |

Notes:
- Dashboard telemetry caches are now cleared on logout/session invalidation.
- Storage/cookie use should still be documented in deployment privacy/cookie notices.

### Server-side telemetry/logging

| Dataset | Stored shape | Retention/control |
| --- | --- | --- |
| Event log (`eventlog:v2:*`) | `ts`, `event`, `ip`, `ip_storage_mode`, `reason`, `outcome`, `admin` | requested by `SHUMA_EVENT_LOG_RETENTION_HOURS`; `SHUMA_EVENT_LOG_IP_STORAGE_MODE` decides whether newly written rows store raw IPs, coarse masked buckets, or keyed pseudonyms, and raw operator retention is capped to `72h` when raw rows exist |
| Monitoring counters (`monitoring:v1:*`) | aggregated counters by hour; dimensions include <abbr title="Internet Protocol">IP</abbr> bucket, normalized path, reason/outcome/country | bounded by `SHUMA_MONITORING_RETENTION_HOURS` |
| Monitoring rollups (`monitoring_rollup:v1:day:*`) | derived daily aggregates for longer-window summary reads | bounded by `SHUMA_MONITORING_ROLLUP_RETENTION_HOURS` |
| Ban records (`ban:*`) | <abbr title="Internet Protocol">IP</abbr>, reason, expiry, optional fingerprint summary | per-ban expiry (`ban_duration*`) |
| Fingerprint state (`fp:*`) | bounded-window mismatch/coherence state; pseudonymized when enabled | bounded by configured TTL and flow-window controls with cadence-gated cleanup for stale `fp:state:*`, `fp:flow:*`, and `fp:flow:last_bucket:*` keys |
| Admin session <abbr title="Key-Value">KV</abbr> (`admin_session:*`) | <abbr title="Cross-Site Request Forgery">CSRF</abbr> token + expiry | session <abbr title="Time To Live">TTL</abbr> (`3600s`) with expiry checks |

## 🐙 <abbr title="General Data Protection Regulation">GDPR</abbr> Posture Assessment

- Personal data is present:
  - event-log IP identifiers whose sensitivity now depends on `SHUMA_EVENT_LOG_IP_STORAGE_MODE` (`raw`, coarse masked bucket, or keyed pseudonym) plus raw IPs in ban records,
  - pseudonymous/bucketed identifiers in monitoring/fingerprint stores.
- Recommended legal basis for operators: legitimate interests in service security and abuse prevention, with an <abbr title="Legitimate Interests Assessment">LIA</abbr> documented by the deployer.
- Data minimization posture:
  - good: monitoring uses bucketed IPs and normalized low-cardinality paths.
  - improved: event-log storage can now be deployer-selected. `raw` preserves investigation value, `masked` stores only coarse buckets, and `pseudonymized` stores stable keyed pseudonyms. Default admin monitoring views remain pseudonymized and raw display still requires forensic acknowledgement, but that acknowledgement no longer implies raw IPs exist at rest when storage minimization is enabled.
- Retention posture:
  - event/monitoring retention is explicitly configurable and now deterministically cleaned.
  - fingerprint-state cleanup is now bounded and cadence-gated to the configured TTL/window controls.

Event-log IP storage tradeoff summary:

- `raw`
  - highest investigation value,
  - highest at-rest privacy sensitivity.
- `masked`
  - preserves rough source grouping and locality,
  - prevents later raw-IP recovery for new rows.
- `pseudonymized`
  - preserves cross-row correlation for the same source,
  - prevents later raw-IP recovery for new rows,
  - does not preserve geo/prefix locality in stored rows.

## 🐙 Cookie-Consent Determination by Deployment Context

| Deployment context | Consent banner required by default? | Required disclosure controls |
| --- | --- | --- |
| Public site with default Shuma bot defence only | Usually no | privacy notice + cookie table for security cookies |
| Public site + admin dashboard | Usually no | privacy notice + cookie table + admin telemetry disclosure |
| Internal/admin-only deployment | Usually no | internal privacy notice and retention policy still required |
| Any deployment with non-essential analytics/marketing/tracking storage | Yes | consent banner prior to write + withdrawal controls + full cookie policy |

## 🐙 Required Operator Disclosures (Minimum)

Operators deploying Shuma should publish:

1. Security-monitoring notice:
   - categories collected (<abbr title="Internet Protocol">IP</abbr>, request metadata, security outcomes),
   - purpose (abuse prevention, service protection),
   - retention window and where it is configured,
   - whether event-log IPs are stored as raw, masked, or pseudonymized identifiers.
2. Cookie/storage notice:
   - `js_verified`, `shuma_fp`, `shuma_admin_session`,
   - dashboard localStorage items and short-lived cache behavior.
3. Data-subject rights/process:
   - access/erasure contact path,
   - process to locate data by <abbr title="Internet Protocol">IP</abbr>/time window where legally required.
4. Processor/transfer details when external providers are enabled (for example Redis services in managed environments).

## 🐙 Review Outcome for `SEC-GDPR-1`

Completed: <abbr title="General Data Protection Regulation">GDPR</abbr>/privacy and cookie-consent review performed with a deployment-context determination.

Follow-up engineering tasks are tracked in `todos/todo.md` under <abbr title="General Data Protection Regulation">GDPR</abbr> follow-up items.
