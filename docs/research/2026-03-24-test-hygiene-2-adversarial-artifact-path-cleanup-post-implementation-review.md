# TEST-HYGIENE-2 post-implementation review

Date: 2026-03-24

## What landed

- Moved the routine adversarial/SIM2 generated receipt contract for:
  - `preflight_report.json`
  - `sim2_realtime_bench_report.json`
  - `sim2_realtime_bench_summary.md`
  - `sim2_ci_diagnostics.json`
  - `sim2_operational_regressions_report.json`
  from tracked `scripts/tests/adversarial/` paths to untracked `.spin/adversarial/` runtime storage.
- Added a shared Python path module:
  - `scripts/tests/adversarial_artifact_paths.py`
  so the generators and consumers use the same canonical locations.
- Added and wired the explicit contract lane:
  - `make test-adversarial-artifact-contract`
  and threaded the new helper/test into `make test-adversarial-python-unit`.
- Fixed the direct-entrypoint import seam in the touched SIM2 scripts so the shared helper also works when invoked through the canonical `make` targets, not only through `unittest`.
- Removed the previously tracked generated receipts from version control so canonical `make test` no longer dirties the worktree by regenerating them.

## Why this tranche mattered

- `TEST-HYGIENE-2` existed because normal verification was mutating tracked JSON and Markdown receipts that were generated afresh by routine adversarial and SIM2 lanes.
- That made the worktree look dirty after honest verification and blurred the boundary between source-of-truth fixtures and ephemeral run output.
- Moving those routine receipts into `.spin/adversarial/` keeps durable repo contracts in-repo while restoring truthful local verification hygiene.

## Verification

- `make test-adversarial-artifact-contract`
- `make test-adversarial-preflight`
- `make test-sim2-realtime-bench`
- `make test-sim2-ci-diagnostics`
- `make test-sim2-operational-regressions`
- `make test-sim2-adr-conformance`
- `make test-sim2-governance-contract`
- `make test-adversarial-python-unit`
- `git diff --check`

## Follow-on notes

- This tranche intentionally did not move durable repo-resident reports like `latest_report.json`, `attack_plan.json`, or governance/operator artifacts that are part of the adversarial contract surface rather than routine churn.
- If more routine generated receipts start dirtying the worktree in future, they should either join this shared `.spin/adversarial/` path contract or become reproducible non-generated fixtures with an explicit reason to stay tracked.
