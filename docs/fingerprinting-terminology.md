# 🐙 Fingerprinting Terminology

This page is the canonical terminology map for fingerprinting, JS verification, and Akamai signal integration.

Use this map in UI copy, docs, and runbooks to avoid ambiguous wording.

| Term | Definition | Used for | Not |
| --- | --- | --- | --- |
| `JS Verification` | The gate that requires a valid `js_verified` marker before normal request flow continues. | Early automation pressure and marker issuance before policy escalation. | Not a Not-a-Bot or Puzzle challenge. |
| `JS Verification Interstitial` | The rendered browser page/script path that performs JS verification work (and optional PoW) and posts telemetry to the active report endpoint. | Collecting verification telemetry and issuing `js_verified` on success. | Not a generic "challenge page" label. |
| `Browser CDP Automation Detection` | Shuma’s own browser-side automation probe and scoring path (`cdp_detection_*` controls, `/cdp-report` when internal provider is active). | Internal automation evidence and optional auto-ban at configured thresholds. | Not edge telemetry from Akamai. |
| `Passive Fingerprint Signals` | Request/header/temporal/persistence signals computed by Shuma without requiring a managed edge provider. | Botness scoring evidence that remains available in self-hosted mode. | Not provider-managed global bot intel. |
| `Akamai Bot Signal` | Normalized bot-risk outcomes ingested from Akamai-shaped edge payloads (`/fingerprint-report`) when enabled. | Edge-origin confidence input that can augment or dominate selected local decisions by mode. | Not direct browser-runtime CDP introspection. |
| `Additive` mode | Akamai signal contributes bounded points into fingerprint scoring; it does not directly short-circuit to ban. | Measured edge contribution during controlled rollout. | Not "observe-only"; it changes scoring. |
| `Authoritative` mode | Documented high-confidence Akamai outcomes can trigger immediate enforcement paths (for example auto-ban when enabled). | Explicit strong-evidence escalation with strict trust checks. | Not default mode; not required for baseline operation. |

## 🐙 Naming Rule

When describing these controls to operators:

- Use `Akamai Bot Signal` for edge ingestion.
- Use `Browser CDP Automation Detection` for Shuma’s internal browser automation probe controls.
- Use `Passive Fingerprint Signals` for internal request/behavior fingerprinting.
- Use `JS Verification Interstitial` for the JS gate page path.

Implementation note:
- In config keys, `provider_backends.fingerprint_signal=external` is the backend selector that enables the edge adapter path (currently Akamai).
