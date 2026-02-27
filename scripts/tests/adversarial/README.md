# Adversarial Simulation Manifest

## Files

- `scenario_manifest.schema.json`
  - Draft schema contract for scenario tiers, expected outcomes, profile gates, and runtime budgets.
- `scenario_manifest.v1.json`
  - Versioned canonical scenario matrix (`SIM-T0`..`SIM-T4`) with profile groupings:
    - `fast_smoke` (mandatory release gate)
    - `abuse_regression`
    - `akamai_smoke`
    - `full_coverage`
- `scenario_manifest.v2.json`
  - Manifest v2 contract with explicit per-scenario `driver_class`, `traffic_model`, `expected_defense_categories`, `coverage_tags`, and `cost_assertions`.
  - Driver families are manifest-driven: `browser_realistic`, `http_scraper`, `edge_fixture`, `cost_imposition`.
- `scenario_manifest.v1.json` + `scenario_manifest.v2.json`
  - Both declare `execution_lane: black_box`; non-black-box lane values are rejected.
  - Makefile simulation targets execute `scenario_manifest.v2.json`; `v1` remains as a compatibility-validation contract.
- `frontier_payload_schema.v1.json`
  - Versioned outbound allowlist contract for frontier payload redaction/minimization.

## Runner

Execute via the unified runner in `scripts/tests/adversarial_simulation_runner.py`.

Examples:

```bash
python3 scripts/tests/adversarial_simulation_runner.py --validate-only
python3 scripts/tests/adversarial_simulation_runner.py --profile fast_smoke
python3 scripts/tests/adversarial_simulation_runner.py --profile abuse_regression
python3 scripts/tests/adversarial_simulation_runner.py --profile akamai_smoke
python3 scripts/tests/adversarial_simulation_runner.py --profile full_coverage
python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile fast_smoke --validate-only
python3 scripts/tests/adversarial_sim_selftest.py
```

The runner writes machine-readable artifacts to:
- `scripts/tests/adversarial/latest_report.json`
- `scripts/tests/adversarial/attack_plan.json`
- `scripts/tests/adversarial/frontier_lane_status.json` (from `make test-adversarial-frontier-attempt`)
- `latest_report.json` and `attack_plan.json` include `execution_lane` metadata for auditability.
- `latest_report.json` includes:
  - quantitative `gates` and `coverage_gates` sections (each check includes `threshold_source`),
  - `cohort_metrics` (persona-level outcome/latency/collateral summaries),
  - `ip_range_suggestions` seed evidence (`seeded_summary`, `seeded_suggestions`, `matched_seed_suggestions`, `near_miss_suggestions`),
  - `plane_contract` metadata for attacker/control-plane guardrails.

Notes:

- `abuse_regression` stale-token coverage is simulated with a signed expired Not-a-Bot seed so the test remains fast and deterministic.
- `abuse_regression` is fail-fast and now includes replay/stale/ordering plus retry-storm, fingerprint inconsistency, and forwarded-header spoof probes with invariant-oriented diagnostics.
- `akamai_smoke` uses canned JSON fixtures posted to local `/fingerprint-report`; it does not require a live Akamai edge deployment.
- Live-loop behavior can be tuned with environment variables:
  - `SHUMA_ADVERSARIAL_PRESERVE_STATE=1|0` controls whether run-final cleanup (baseline reset + unban cleanup) is skipped.
  - `SHUMA_ADVERSARIAL_ROTATE_IPS=1|0` controls per-run scenario IP rotation to avoid long-loop per-IP window collisions.
  - `make test-adversarial-live` defaults to preserve + rotate (`1/1`) for operator monitoring; CI-focused profile targets force deterministic cleanup + static IPs (`0/0`).
  - `ADVERSARIAL_CLEANUP_MODE=1` forces cleanup-per-cycle behavior in live loops (default `0` preserves state).
  - Live-loop resilience uses transient/fatal classification with capped backoff and fails only after `ADVERSARIAL_FATAL_CYCLE_LIMIT` consecutive fatal cycles.
  - Live-loop logs include failure classification, retry/backoff counters, and terminal failure reason on exit.
  - Live-loop quality gate rejects admin-only event noise; cycles must emit meaningful defense event reasons.
- `full_coverage` adds profile-level coverage gates (`gates.coverage_requirements`) using monitoring deltas captured over the run.
- `full_coverage` now includes explicit PoW success/failure, challenge puzzle-failure fallback, replay-to-tarpit abuse, CDP deny path, rate-limit enforcement, and GEO block scenarios in addition to existing challenge/maze/honeypot/Akamai families.
- `full_coverage` also enforces persona and taxonomy gates:
  - `human_like_collateral_max_ratio`,
  - `required_event_reasons`,
  - `ip_range_suggestion_seed_required` (with deterministic seed traffic + evidence snapshot).
- Plane separation contract:
  - attacker-plane requests are restricted to public paths and reject privileged headers (`Authorization`, health/admin/signing secret headers),
  - orchestrator-only setup/reset/config hooks remain on the control plane via admin-authenticated calls.
- `monitoring_after` snapshot includes nested tarpit metrics so live-loop output can report activation/progression/fallback/escalation coverage without manual JSON digging.
- Protected-lane frontier probe output (`frontier_lane_status.json`) is advisory only; deterministic coverage/replay gates remain blocking.
- `adversarial_sim_selftest.py` is intentionally tiny and non-circular: it validates simulator mechanics against fixed stub routes without asserting product defense efficacy.
