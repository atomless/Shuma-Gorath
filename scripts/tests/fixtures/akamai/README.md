# Akamai Signal Fixtures

This directory contains deterministic canned fixtures for Shuma's Akamai integration paths.

## Fingerprint report payload fixtures (`POST /fingerprint-report`)

- `fingerprint_monitor_signal.json`
  - Low-confidence monitor outcome.
- `fingerprint_challenge_signal.json`
  - Mid-confidence challenge outcome.
- `fingerprint_additive_deny_signal.json`
  - High-confidence deny payload used in additive mode tests.
- `fingerprint_authoritative_deny_signal.json`
  - High-confidence deny payload used in authoritative mode tests.
- `fingerprint_invalid_score_signal.json`
  - Invalid score payload for negative validation tests.

## Trusted edge header fixtures

These are request-header sets for GEO/fingerprint transport signal simulations.

- `headers_geo_challenge_br.json`
- `headers_geo_maze_ru.json`
- `headers_transport_clean.json`
- `headers_transport_high_risk_mismatch.json`

Notes:
- Header names intentionally match runtime extraction keys (`x-geo-country`, `x-shuma-edge-ja3`, `x-shuma-edge-ja4`, `x-shuma-edge-browser-family`, `x-shuma-edge-bot-score`).
- Fixtures represent payload/header values only. Trusted ingestion still requires `X-Shuma-Forwarded-Secret` and a trusted `X-Forwarded-For` request source in test runs.
