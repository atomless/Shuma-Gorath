# Adversarial Simulation Fast-Smoke Design

Date: 2026-02-25  
Status: Active (initial executable slice)

## Goal

Deliver the first mandatory adversarial simulation gate with:

1. A versioned scenario manifest contract (`SIM-1`)
2. A unified runner under `scripts/tests/` (`SIM-2`)
3. Explicit replay/stale/ordering abuse scenarios (`SIM-3`)
4. Quantitative gate assertions for latency, outcome ratios, and telemetry amplification (`SIM-4`)

## Design Summary

### Manifest and schema

Canonical files:

- `scripts/tests/adversarial/scenario_manifest.schema.json`
- `scripts/tests/adversarial/scenario_manifest.v1.json`

Schema characteristics:

- scenario tiers: `SIM-T0`..`SIM-T4`
- expected outcomes: `allow`, `monitor`, `not-a-bot`, `challenge`, `maze`, `deny_temp`
- profile-level runtime budget and quantitative gates
- scenario-level runtime budget and max-latency assertion
- fixture references for Akamai payload simulations

### Unified runner

Runner path:

- `scripts/tests/adversarial_simulation_runner.py`

Runner behavior:

1. validates manifest/profile contracts before execution,
2. resets runtime config into deterministic baseline,
3. executes profile scenario list in declared order,
4. asserts per-scenario outcomes and latency budgets,
5. evaluates profile quantitative gates,
6. writes machine-readable report (`scripts/tests/adversarial/latest_report.json`).

### Mandatory fast-smoke profile

Profile name:

- `fast_smoke`

Mandatory scenario set:

1. `SIM-T0`: allowlist-backed legitimate browser allow
2. `SIM-T1`: successful Not-a-Bot pass
3. `SIM-T2`: GEO challenge route
4. `SIM-T3`: GEO maze route
5. `SIM-T4`: honeypot temporary deny
6. `SIM-T3`: Not-a-Bot replay abuse rejection

This profile intentionally mixes benign and adversarial tiers in one short run to gate both efficacy and collateral-risk posture.

### Abuse and Akamai profiles

Additional initial profiles are included:

- `abuse_regression`: replay + stale-token + ordering/cadence abuse
- `akamai_smoke`: additive and authoritative Akamai fixture-driven paths

Akamai fixtures live in `scripts/tests/fixtures/akamai/`.

## Make Target Design

New targets:

- `make test-adversarial-manifest`
  - validates manifest/schema/fixture references only (no running server required)
- `make test-adversarial-smoke`
  - executes mandatory `fast_smoke` profile (requires running Spin)
- `make test-adversarial-abuse`
  - executes explicit abuse profile
- `make test-adversarial-akamai`
  - executes Akamai fixture profile
- `make test-adversarial-fast`
  - executes the mandatory fast matrix (`smoke + abuse + akamai`)
- `make test-adversarial-soak`
  - executes deep soak coverage (`full_coverage`) for scheduled/manual gates

Umbrella integration:

- `make test` includes `make test-adversarial-fast` as the mandatory adversarial gate before dashboard e2e.
- `make test-adversarial-soak` is used by scheduled/manual CI for deep adversarial coverage.

## Quantitative Gate Contract (initial)

Profile gates currently assert:

1. latency band (`p95_max_ms`)
2. outcome distribution bounds by expected outcome key
3. telemetry write amplification limits:
   - `max_fingerprint_events_per_request`
   - `max_monitoring_events_per_request`

## Initial Calibration (2026-02-26)

Observed `fast_smoke` report on live local Spin:

- request count: `10`
- suite runtime: `145ms`
- latency p95: `13ms`
- outcome counts:
  - `allow=1`
  - `not-a-bot=1`
  - `challenge=1`
  - `maze=2`
  - `deny_temp=1`
- telemetry amplification:
  - fingerprint: `0.10` events/request (`delta=1`)
  - monitoring: `0.00` events/request (`delta=0`)

Threshold tuning applied from this baseline:

- `max_runtime_seconds`: `75 -> 30`
- `latency.p95_max_ms`: `2200 -> 1200`
- ratio bounds tightened around deterministic slice composition:
  - `allow`, `not-a-bot`, `challenge`, `deny_temp`: `[0.15, 0.18]`
  - `maze`: `[0.32, 0.35]`
- telemetry amplification tightened:
  - fingerprint: `2.0 -> 0.5`
  - monitoring: `3.0 -> 1.0`

## Cross-Environment Recalibration (2026-03-08)

Observed `fast_smoke` divergence between local macOS Chromium and Linux CI Chromium:

- local macOS Chromium:
  - fingerprint: `0.21` events/request (`delta=3`, `requests=14`)
  - monitoring: `0.50` events/request (`delta=7`, `requests=14`)
- Linux CI Chromium:
  - fingerprint: `3.00` events/request (`delta=42`, `requests=14`)
  - monitoring: `7.00` events/request (`delta=98`, `requests=14`)

Interpretation:

- outcomes and request lineage stayed deterministic across environments,
- the drift came from heavier CI browser-harness telemetry (`cdp_detections`, `fingerprint_events`, `not_a_bot_escalate`, and `rate_violations`), not from scenario outcome regressions,
- the original amplification ceilings had become a macOS-only calibration and were no longer truthful for the mandatory Linux CI lane.

Threshold tuning applied from the cross-environment baseline:

- `fast_smoke.telemetry_amplification.max_fingerprint_events_per_request`: `0.5 -> 3.5`
- `fast_smoke.telemetry_amplification.max_monitoring_events_per_request`: `1.0 -> 8.0`
- `full_coverage.telemetry_amplification.max_monitoring_events_per_request`: `5.0 -> 8.0`

## Risks and Follow-up

1. Runtime sensitivity: profile timing can drift across slower machines.
   - Mitigation: keep budgets explicit and tune from measured baselines.
2. Abuse-path strictness: some branches are mode-dependent.
   - Mitigation: deterministic baseline config reset before every scenario.
3. Coverage breadth:
   - Mitigation: deep soak profile (`full_coverage`) is available via `make test-adversarial-soak` and intended for scheduled/manual CI execution.
