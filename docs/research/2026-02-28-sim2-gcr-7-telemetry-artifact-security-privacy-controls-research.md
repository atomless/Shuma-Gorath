# SIM2-GCR-7 Research: Telemetry and Adversary-Artifact Security/Privacy Controls

Date: 2026-02-28  
Status: Recommended control model selected

## Objective

Define robust security/privacy controls for telemetry and adversary artifacts, covering secret-exposure prevention, data minimization, pseudonymization, retention-risk controls, and incident-response hooks.

## Repository Baseline (Current State)

1. Event logs store rich fields (`ip`, `reason`, `outcome`, `admin`) and simulation metadata when present.
2. Simulation-tag authenticity checks are in place (`sim-tag.v1` with HMAC/nonce/timestamp validation).
3. Fingerprint pipeline has a pseudonymization toggle (`fingerprint_pseudonymize`), but event-log/monitoring privacy controls remain broader TODO territory.
4. Frontier/adversarial artifact governance exists, but centralized field-classification and retention-risk controls are not yet formalized as one contract.

## Primary-Source Findings

1. Logs must avoid sensitive data exposure by design, with explicit exclusion/redaction policy.
   Source: [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)
2. Secret handling requires strict lifecycle controls and must avoid accidental disclosure in logs/artifacts.
   Source: [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
3. PII protection guidance emphasizes minimization and safeguards proportional to confidentiality risk.
   Source: [NIST SP 800-122](https://www.nist.gov/publications/guide-protecting-confidentiality-personally-identifiable-information-pii)
4. GDPR requires data minimization and defines pseudonymization as a risk-reduction control.
   Sources:
   - [GDPR Article 5 (data minimization)](https://gdpr-info.eu/art-5-gdpr/)
   - [GDPR Article 4(5) (pseudonymization)](https://gdpr-info.eu/art-4-gdpr/)
   - [GDPR Article 32 (security of processing)](https://gdpr-info.eu/art-32-gdpr/)
5. Incident handling guidance requires preparation, detection, containment, eradication, recovery, and post-incident lessons.
   Source: [NIST SP 800-61 Rev.2](https://www.nist.gov/publications/computer-security-incident-handling-guide)

## Inferences for Shuma (Derived from Sources)

1. Telemetry fields need explicit data-class classification (`public`, `internal`, `sensitive`, `secret-prohibited`) with enforcement in code paths.
2. Artifact retention should be tiered by sensitivity (short-lived raw artifacts, longer-lived redacted summaries).
3. Secret-leak canary detection should be integrated into adversary artifact generation/ingest and wired to incident-response hooks.
4. Pseudonymization should be default-on for IP-linked analytics paths unless explicitly disabled with risk acknowledgement.

## Architecture Options

### Option A: Convention-Based Hygiene (Current Tendency)

Rely on contributor discipline and scattered checks to avoid secret/PII leakage.

### Option B: Redaction-Only on Output

Keep internal raw capture but redact only at API/UI presentation boundary.

### Option C: Classified Data-Flow + Enforcement + Retention Tiers + Incident Hooks (Recommended)

Classify fields at ingress, enforce allow/deny policy in telemetry/artifact pipelines, pseudonymize sensitive identifiers, apply sensitivity-tiered retention, and trigger incident workflows on leak detection.

### Option D: External DLP-first approach

Outsource leakage prevention to external DLP/SIEM controls while keeping internal pipeline mostly unchanged.

## Decision Matrix

| Option | Benefits | Risks | Resource Cost | Security Impact | Rollback Complexity |
|---|---|---|---|---|---|
| A. Convention-based | Low engineering effort | High drift and leak risk | Low | Weak | Low |
| B. Redaction-only output | Better UI/API hygiene | Raw storage still high risk | Low-medium | Moderate | Low |
| C. Classified+enforced+tiered+incident hooks (recommended) | Strong prevention and containment posture, auditable controls | Requires broad schema/pipeline updates | Medium | Strong | Medium |
| D. External DLP-first | Potentially strong centralized controls | Vendor dependence and integration complexity | High | Strong but externalized | High |

## Recommendation

Adopt **Option C**.

Required controls:

1. **Field classification and enforcement**
   1. Define telemetry/artifact field classes and prohibited classes.
   2. Enforce at ingestion and before persistence (not only at UI output).
2. **Secret exposure prevention**
   1. Add deterministic scrubber for secret-like values in `reason/outcome/artifact payloads`.
   2. Add canary detectors for frontier artifact outputs and fail-closed policy on detection.
3. **Privacy minimization and pseudonymization**
   1. Pseudonymize IP-linked fields by default in telemetry surfaces where raw value is not operationally required.
   2. Keep controlled break-glass path for raw forensic access with explicit operator action.
4. **Retention-risk controls**
   1. Tier artifacts by sensitivity and apply strict max retention windows.
   2. Prefer redacted summaries for long-lived retention.
5. **Incident-response hooks**
   1. Emit structured security events for leak detections, policy violations, and scrubber failures.
   2. Provide deterministic operator workflows for containment and artifact quarantine.

## Quantitative Targets (for TODO enforcement)

1. Secret canary leakage tolerance: `0` accepted leaked canary values in persisted telemetry/artifacts.
2. High-risk raw artifact retention ceiling: `<=72h` by default unless explicit override is logged.
3. Pseudonymization default coverage: `100%` of configured sensitive identifier fields in non-forensic views.
4. Incident hook latency: leak detection events must appear in monitoring/security streams within one refresh/stream cycle.

## Plan and TODO Impact

1. New plan doc: `docs/plans/2026-02-28-sim2-gcr-7-telemetry-artifact-security-privacy-controls-plan.md`.
2. Add dedicated implementation slice (`SIM2-GC-17`) for classification, redaction, pseudonymization, retention tiers, and incident hooks.
3. Extend `SIM2-GC-11` verification matrix with secret-leak canary and privacy-mode regression coverage.
