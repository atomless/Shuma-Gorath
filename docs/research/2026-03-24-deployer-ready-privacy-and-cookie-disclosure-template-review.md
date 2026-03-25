Date: 2026-03-24
Status: Proposed

Related context:

- [`../../docs/privacy-gdpr-review.md`](../../docs/privacy-gdpr-review.md)
- [`../../docs/configuration.md`](../../docs/configuration.md)
- [`../../docs/security-hardening.md`](../../docs/security-hardening.md)
- [GDPR (EUR-Lex)](https://eur-lex.europa.eu/legal-content/ENG/HIS/?uri=CELEX:32016R0679)
- [ICO: The right to be informed](https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/individual-rights/the-right-to-be-informed/)
- [ICO: Legitimate interests](https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/lawful-basis/legitimate-interests/)
- [ICO: Guidance on the use of cookies and similar technologies (PDF)](https://ico.org.uk/media2/kz0doybw/guidance-on-the-use-of-cookies-and-similar-technologies-1-0.pdf)

# Deployer-Ready Privacy And Cookie Disclosure Template Review

## Question

What should a Shuma deployer-ready privacy and cookie disclosure template contain so operators can publish a truthful notice without reverse-engineering storage behavior from code and scattered docs?

## Why the current state is not enough

Shuma already has:

1. a good engineering review of privacy and cookie posture in [`docs/privacy-gdpr-review.md`](../../docs/privacy-gdpr-review.md),
2. a detailed configuration reference in [`docs/configuration.md`](../../docs/configuration.md),
3. and a short deployment hardening note in [`docs/security-hardening.md`](../../docs/security-hardening.md).

But those are not yet a deployer-ready artifact. They explain the system; they do not yet give operators a clear starting notice they can adapt and publish.

That leaves three problems:

1. deployers must synthesize their own lawful-basis and retention wording from multiple docs,
2. the current storage inventory is implementation-grounded but not shaped as a publishable notice,
3. and the difference between Shuma's default strictly-necessary security storage and any later optional analytics or tracking additions is easy to blur.

## What official guidance says matters most

The most relevant requirements for this template are stable and practical:

1. the GDPR transparency rules require operators to tell people the purposes of processing, retention periods, and sharing or recipient information;
2. when relying on legitimate interests, operators must say that and explain what those legitimate interests are;
3. privacy information must be concise, transparent, intelligible, easily accessible, and written in clear language;
4. cookie or similar-storage consent is generally not required for storage that is strictly necessary for the service requested, but operators still need a clear notice describing what is stored, why, and for how long;
5. if operators add non-essential analytics, marketing, or tracking storage, they must add a consent mechanism and update the notice accordingly.

The ICO guidance is especially useful here because it is operational rather than abstract:

- `The right to be informed` explicitly calls out purposes, retention periods, and sharing as core privacy-information content.
- `Legitimate interests` says deployers must explain both that they rely on legitimate interests and what those legitimate interests are.
- the cookie guidance says security cookies may fall within the strictly-necessary exemption, but that only remains true when they are used solely for that necessary purpose.

## Recommended Shuma template structure

The cleanest template should be a single deployer-facing Markdown document with these sections:

1. **How to use this template**
   - clearly marked placeholders,
   - explicit reminder that this is not legal advice,
   - note that operators must adapt it to their deployment and jurisdiction.
2. **Controller and contact details**
   - who operates the site,
   - privacy contact route,
   - complaints contact or supervisory authority pointer.
3. **What Shuma collects**
   - request metadata, IP-derived identifiers, security outcomes, and admin/session storage.
4. **Why it is collected and the lawful basis**
   - primary row for security and abuse prevention,
   - likely basis: legitimate interests for private operators, or public task where that is the real basis for a public authority deployment,
   - optional row for authenticated admin operations.
5. **Cookie and browser-storage table**
   - `js_verified`
   - `shuma_fp`
   - `shuma_admin_session`
   - dashboard localStorage cache entries
   - note that these are generally strictly necessary by default.
6. **Server-side storage and retention**
   - event log, including the `raw` / `masked` / `pseudonymized` write-mode distinction,
   - monitoring counters and rollups,
   - bans,
   - fingerprint state,
   - admin sessions.
7. **Who data is shared with**
   - hosting providers,
   - Redis or managed data-store providers if used,
   - any external providers intentionally enabled by the operator.
8. **International transfers**
   - only as a placeholder section because deployment-specific.
9. **Rights-handling workflow**
   - access,
   - erasure,
   - restriction or objection where applicable,
   - complaint route,
   - what information the operator will likely need to locate records.
10. **Optional add-on section for non-essential analytics or tracking**
   - explicit instruction that if the operator adds these, the default Shuma notice is no longer sufficient.

## Recommended Shuma-specific truthfulness points

The template should state a few Shuma-specific truths explicitly so operators do not accidentally overclaim or underclaim:

1. event-log IP sensitivity depends on `SHUMA_EVENT_LOG_IP_STORAGE_MODE`;
2. default Shuma storage is security and operations focused, not advertising or profiling;
3. security cookies can only be described as strictly necessary if they are used solely for security or requested service operation;
4. the deployer must update the template if they enable external providers, analytics, marketing, or other extra storage beyond Shuma's default posture.

## Result

`SEC-GDPR-4` should land as a docs-only tranche that adds one deployer-ready template and links it from the existing privacy and hardening docs. It should not widen into jurisdiction-specific legal advice or a generic compliance program.
