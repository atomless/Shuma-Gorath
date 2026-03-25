# 🐙 Deployer Privacy And Cookie Disclosure Template

This document is a deployer-ready starting template for Shuma deployments.

It is **not legal advice**. You must adapt it to your organisation, deployment, enabled providers, jurisdiction, and actual operational choices before publishing it.

Use this template together with:

- [`privacy-gdpr-review.md`](privacy-gdpr-review.md)
- [`configuration.md`](configuration.md)
- [`security-hardening.md`](security-hardening.md)

## How To Use This Template

1. Replace every placeholder in square brackets.
2. Remove sections that do not apply to your deployment.
3. Add any extra processors, analytics tools, tracking technologies, or transfers you actually use.
4. Re-check the cookie and storage tables whenever you change Shuma config or add non-Shuma tooling.
5. If you add non-essential analytics, advertising, personalization, or cross-site tracking storage, this default template is no longer sufficient on its own; add the required consent mechanism and update the notice accordingly.

---

# Template Starts Here

## Privacy And Cookie Notice

Last updated: [DATE]

[ORGANISATION NAME] operates this website and uses Shuma as part of its security and abuse-prevention controls.

This notice explains:

- what information we collect,
- why we collect it,
- how long we keep it,
- what cookies or similar browser storage we use,
- and how you can exercise your privacy rights.

If you have questions about this notice or about how your information is used, contact us at:

- Controller: [LEGAL ENTITY NAME]
- Contact email: [PRIVACY CONTACT EMAIL]
- Postal address: [POSTAL ADDRESS]
- Data Protection Officer or privacy contact, if applicable: [DPO OR PRIVACY CONTACT]

If you are in a jurisdiction that gives you the right to complain to a supervisory authority, you may also contact [SUPERVISORY AUTHORITY NAME OR EXPLANATION].

## Why We Process This Information And The Lawful Basis

We use Shuma primarily to protect this site and its visitors against abuse, scraping, hostile automation, fraud, and other malicious or unwanted traffic.

### Core security and abuse-prevention processing

| Purpose | Typical data used | Typical lawful basis |
| --- | --- | --- |
| Detect, rate-limit, challenge, or block abusive traffic; investigate incidents; protect service availability and integrity | IP-derived identifiers, request metadata, headers, user-agent data, security outcomes, challenge or verification results, botness or classification signals | [For most private-sector deployments: legitimate interests in network and information security, abuse prevention, and service protection. If you rely on legitimate interests, describe those interests plainly here.] |

Suggested legitimate-interests wording for many private deployments:

> We process this information because we have a legitimate interest in keeping our website, users, and infrastructure secure; preventing abuse, fraud, scraping, and hostile automation; maintaining service availability; and investigating security incidents.

If you are a public authority or rely on a different lawful basis, replace that wording with the lawful basis that actually applies to your deployment and explain it clearly.

### Admin and operational access

| Purpose | Typical data used | Typical lawful basis |
| --- | --- | --- |
| Authenticate administrators, operate the dashboard, and maintain service security | Admin session identifiers, CSRF token material, admin action metadata, operational telemetry | [Legitimate interests / contract / public task, depending on your organisation and deployment context] |

## What We Collect

Depending on how this site is configured, we may process:

- IP addresses or IP-derived identifiers,
- request metadata such as path, method, headers, and timing,
- device or browser signals used for security verification,
- challenge, proof-of-work, or verification outcomes,
- security event logs and monitoring counters,
- admin authentication and session data for authorised operators.

We do not use default Shuma storage for advertising, profiling, or third-party marketing analytics.

## Cookies And Browser Storage

The table below reflects Shuma's default browser-side storage. Replace or extend it if your deployment changes these defaults or adds other tooling.

| Cookie or storage item | Purpose | Typical lifetime | Default necessity posture |
| --- | --- | --- | --- |
| `js_verified` cookie | Records successful JavaScript verification for the security gate | Up to `86400` seconds | Generally strictly necessary for security |
| `shuma_fp` cookie | Short-lived persistence marker in verification flow | Up to `1800` seconds | Generally strictly necessary for abuse detection |
| `shuma_admin_session` cookie | Authenticated admin dashboard session | Up to `3600` seconds | Strictly necessary for admin access |
| `localStorage: shuma_dashboard_cache_monitoring_v1` | Short-lived admin monitoring cache | Approximately `60` seconds application TTL | Operational admin storage; usually not consent-based consumer tracking |
| `localStorage: shuma_dashboard_cache_ip_bans_v1` | Short-lived admin ban-list cache | Approximately `60` seconds application TTL | Operational admin storage; usually not consent-based consumer tracking |
| `localStorage: shuma_dashboard_auto_refresh_enabled` | Stores an admin dashboard preference | Until changed | Preference storage for the admin interface |

Default Shuma cookie and storage posture:

- these items are used for security, abuse prevention, authentication, or admin operations,
- they are not used by default for advertising or cross-site tracking,
- and in many EU or UK ePrivacy contexts they are generally treated as strictly necessary when used only for those purposes.

If we add non-essential analytics, advertising, personalization, or other tracking technologies, we will update this notice and, where required, request consent before setting them.

## Server-Side Security And Operational Storage

The table below is a deployment-ready baseline for Shuma's default server-side storage. Replace the retention wording with the values you actually use.

| Dataset | Typical contents | Retention or control |
| --- | --- | --- |
| Event log (`eventlog:v2:*`) | timestamp, event type, IP-derived identifier, reason, outcome, optional admin actor metadata | Controlled by `SHUMA_EVENT_LOG_RETENTION_HOURS`; event-log IP storage may be `raw`, `masked`, or `pseudonymized` depending on deployment configuration |
| Monitoring counters (`monitoring:v1:*`) | aggregated hourly counters and low-cardinality operational dimensions | Controlled by `SHUMA_MONITORING_RETENTION_HOURS` |
| Monitoring rollups (`monitoring_rollup:v1:day:*`) | derived daily operational summaries | Controlled by `SHUMA_MONITORING_ROLLUP_RETENTION_HOURS` |
| Ban records (`ban:*`) | IP address or related source identifier, reason, expiry, optional fingerprint summary | Retained until ban expiry |
| Fingerprint state (`fp:*`) | bounded-window mismatch and coherence state; may be pseudonymized | Retained according to configured TTL and bounded cleanup rules |
| Admin sessions (`admin_session:*`) | admin session and CSRF state | Retained for the configured session TTL, normally up to `3600` seconds |

### Event-log IP storage mode

Our deployment currently stores new event-log IP data as: [RAW / MASKED / PSEUDONYMIZED].

Use one of the following explanations and delete the others:

- `raw`
  - We store raw IP addresses in new event-log rows for security investigation value.
- `masked`
  - We store only a coarse masked IP bucket in new event-log rows, not the raw IP address.
- `pseudonymized`
  - We store a stable keyed pseudonymous identifier in new event-log rows, not the raw IP address.

Historical rows may reflect the write mode in use at the time they were created.

## Who We Share Information With

We share information only where necessary to operate and secure the service.

Typical recipients or processors may include:

- our hosting or infrastructure providers: [LIST]
- managed data-store providers, if used: [LIST]
- content-delivery, gateway, or edge providers, if used: [LIST]
- other processors or service providers involved in operating this site: [LIST]

We do not share Shuma security data for advertising purposes as part of the default product behavior.

## International Transfers

[Explain whether personal data is processed outside the UK, EEA, or other relevant jurisdiction, which providers are involved, and what transfer safeguards apply.]

If no relevant international transfers apply, state that clearly.

## How Long We Keep Information

We keep security and operational data only for as long as it is needed for abuse prevention, service protection, investigation, and compliance purposes.

Current baseline retention settings for this deployment:

- event log: [VALUE]
- monitoring counters: [VALUE]
- monitoring rollups: [VALUE]
- admin sessions: [VALUE]
- any additional processor-specific retention: [VALUE]

If you use a shorter plain-language table instead, keep it aligned with the exact configured values in your deployment.

## Your Rights

Depending on your location and the laws that apply, you may have rights to:

- request access to your personal data,
- request correction of inaccurate data,
- request deletion of data in some circumstances,
- object to processing, including processing based on legitimate interests in some circumstances,
- request restriction of processing in some circumstances,
- lodge a complaint with a relevant supervisory authority.

To exercise your rights, contact us at [RIGHTS CONTACT EMAIL OR FORM].

To help us locate relevant security records, we may ask for information such as:

- the time and date of the interaction,
- the relevant page or request,
- the IP address or network used at the time, where available,
- and any other details needed to identify the event safely and accurately.

We may not always be able to identify a person directly from pseudonymized or masked security records alone.

## Automated Security Decisions

We use automated security rules to decide whether to allow, challenge, rate-limit, or block requests where necessary to protect the site and its users from abuse or hostile automation.

[If your deployment wants to explain this in more detail, describe the high-level logic and any human review or escalation path here.]

## Changes To This Notice

We may update this notice when we change how we operate the service, when we change Shuma configuration in a way that affects privacy or storage, or when legal requirements change.

The latest version will be available at [NOTICE URL].

---

# Template Ends Here
