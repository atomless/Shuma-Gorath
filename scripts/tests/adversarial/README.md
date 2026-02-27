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
```

The runner writes machine-readable artifacts to:
- `scripts/tests/adversarial/latest_report.json`
- `scripts/tests/adversarial/attack_plan.json`
- `scripts/tests/adversarial/frontier_lane_status.json` (from `make test-adversarial-frontier-attempt`)

Notes:

- `abuse_regression` stale-token coverage is simulated with a signed expired Not-a-Bot seed so the test remains fast and deterministic.
- `akamai_smoke` uses canned JSON fixtures posted to local `/fingerprint-report`; it does not require a live Akamai edge deployment.
- Live-loop behavior can be tuned with environment variables:
  - `SHUMA_ADVERSARIAL_PRESERVE_STATE=1|0` controls whether run-final cleanup (baseline reset + unban cleanup) is skipped.
  - `SHUMA_ADVERSARIAL_ROTATE_IPS=1|0` controls per-run scenario IP rotation to avoid long-loop per-IP window collisions.
  - `make test-adversarial-live` defaults to preserve + rotate (`1/1`) for operator monitoring; CI-focused profile targets force deterministic cleanup + static IPs (`0/0`).
- `full_coverage` adds profile-level coverage gates (`gates.coverage_requirements`) using monitoring deltas captured over the run.
- `full_coverage` now includes explicit PoW success/failure, rate-limit enforcement, and GEO block scenarios in addition to existing challenge/maze/honeypot/Akamai families.
- Protected-lane frontier probe output (`frontier_lane_status.json`) is advisory only; deterministic coverage/replay gates remain blocking.
