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

The runner writes a machine-readable report to `scripts/tests/adversarial/latest_report.json` by default.

Notes:

- `abuse_regression` stale-token coverage is simulated with a signed expired Not-a-Bot seed so the test remains fast and deterministic.
- `akamai_smoke` uses canned JSON fixtures posted to local `/fingerprint-report`; it does not require a live Akamai edge deployment.
- `full_coverage` adds profile-level coverage gates (`gates.coverage_requirements`) using monitoring deltas captured over the run.
