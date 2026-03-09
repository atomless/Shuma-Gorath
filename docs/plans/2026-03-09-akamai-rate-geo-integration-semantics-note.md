# Akamai Rate and GEO Integration Semantics Note

**Goal:** Define the exact semantics for future Akamai controls on Rate Limiting and GEO so the repo stops teaching unfinished behavior as if it already exists, and so later implementation work has one authoritative contract.

**Current reality:** The implemented Akamai-facing surface today is the fingerprint edge adapter on `POST /fingerprint-report`, plus a generic trusted edge GEO country-header path (`X-Geo-Country`) that is not a native Akamai EdgeScape parser. The current Rate tab toggle is infrastructure selection for `provider_backends.rate_limiter`, not Akamai rate-signal ingestion. The live proof target `make test-remote-edge-signal-smoke` proves only the implemented fingerprint and trusted GEO surfaces on a running `ssh_systemd` remote.

---

## Decisions

1. Do not overload today’s generic GEO-header and external-rate-backend toggles with future Akamai semantics.
2. Keep the current UI/docs honest:
   - GEO uses trusted-edge-header wording, not “Akamai GEO signal.”
   - Rate uses external-backend wording, not “Akamai rate signal.”
3. Future Akamai-specific Rate and GEO work must add explicit mode semantics instead of silently reinterpreting the existing toggles.
4. Trust boundary remains strict:
   - origin must never trust raw provider-native edge headers directly,
   - the edge layer must sanitize/map provider-native data into a Shuma-owned canonical surface behind the forwarded-secret trust boundary,
   - Shuma remains policy owner even when provider signals are strong.
5. `AK-RG-2` will define the exact config names and UI shape. This note defines semantics only.

## Semantics Contract

### GEO

Future Akamai GEO control modes must mean:

- `off`
  - ignore Akamai-derived GEO augmentation entirely,
  - the current `geo_edge_headers_enabled` generic header path may still exist as a separate infrastructure switch until it is deliberately replaced.
- `additive`
  - accept trusted provider-derived GEO evidence as input to Shuma’s GEO evaluation,
  - Shuma’s own `geo_allow` / `geo_challenge` / `geo_maze` / `geo_block` policy lists remain the decision owner,
  - missing or untrusted provider data must fail safe to normal Shuma behavior without implicit bypass.
- `authoritative`
  - permit trusted provider-derived GEO identity to become the canonical GEO source when present,
  - still do not allow the provider to inject a direct block/challenge command that bypasses Shuma’s GEO policy tables,
  - fallback on missing or untrusted data must be explicit and observable.

### Rate Limiting

Future Akamai Rate control modes must mean:

- `off`
  - ignore Akamai-derived rate outcomes entirely,
  - current local/external backend selection remains independent.
- `additive`
  - treat trusted provider rate evidence as bounded input to Shuma scoring and enforcement decisions,
  - local/internal or external Redis-backed Shuma rate limiting remains the primary enforcement owner.
- `authoritative`
  - permit trusted provider rate actions to short-circuit into an explicit Shuma action mapping only when the action taxonomy is normalized and observable,
  - downgrade and fallback behavior must be explicit when provider data is missing, stale, or untrusted.

## Precedence and Fallback Rules

1. Trust validation happens first. Untrusted provider input is ignored.
2. Provider-specific parsing and mapping happens at the edge adapter boundary, not spread through origin handlers.
3. Shuma normalizes trusted provider data into Shuma-owned signal and action vocabulary before policy evaluation.
4. In `additive`, provider data may strengthen local confidence but must not become a sole hidden enforcer.
5. In `authoritative`, provider data may short-circuit only through documented Shuma-owned action mapping and telemetry.
6. Fallback must never silently broaden access relative to configured Shuma local policy.

## Immediate Follow-on Work

- `AK-RG-2`: define the config surface and naming for the new Akamai Rate and GEO controls.
- `AK-RG-3..8`: runtime wiring, dashboard controls, observability, tests, and rollout guidance.
- Keep the live proof target focused on the already implemented fingerprint and trusted GEO surfaces until the new controls actually exist.
