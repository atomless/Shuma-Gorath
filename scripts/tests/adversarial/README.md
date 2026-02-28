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
- `frontier_action_contract.v1.json`
  - Versioned frontier execution contract for containerized black-box actions:
    - allowed tools/action types,
    - egress and forbidden path constraints,
    - runtime/request/query budgets,
    - reject-by-default DSL keys.
- `lane_contract.v1.json`
  - Canonical attacker/control capability boundary contract for black-box simulation lanes.
- `coverage_contract.v1.json`
  - Canonical `full_coverage` contract (minimum coverage categories + event/outcome obligations) used for drift checks against manifests and SIM2 plan rows.
- `scenario_intent_matrix.v1.json`
  - Canonical per-scenario intent matrix mapping each scenario to required defense categories, accepted evidence signals, minimum runtime evidence thresholds, and progression realism requirements.
  - Includes review-governance metadata (`cadence_days`, `stale_after_days`, row-level `last_reviewed_on`) used by periodic scenario quality checks.
- `real_traffic_contract.v1.json`
  - Canonical real-traffic evidence contract (required invariants, prohibited synthetic-success patterns, per-scenario runtime evidence fields, and control-plane lineage fields).
- `container_runtime_profile.v1.json`
  - Canonical hardened container runtime profile for frontier workers:
    - required hardening flags,
    - forbidden privileged/host-namespace flags,
    - forbidden host-control mount fragments.
- `../frontier_capability_envelope.py`
  - Shared host/worker capability-envelope signing and validation for executable frontier actions.
- `../adversarial_container/`
  - Container worker assets for black-box isolation lane:
    - `Dockerfile`
    - `worker.py`
- `../adversarial_browser_driver.mjs`
  - Deterministic Playwright-backed browser driver for `browser_realistic` scenarios.
  - Reads JSON payload on stdin and returns structured outcome + browser evidence JSON on stdout.

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
- `scripts/tests/adversarial/frontier_unavailability_policy.json` (from `make test-frontier-unavailability-policy`)
- `scripts/tests/adversarial/repeatability_report.json` (from `make test-adversarial-repeatability`)
- `scripts/tests/adversarial/promotion_candidates_report.json` (from `make test-adversarial-promote-candidates`)
- `scripts/tests/adversarial/container_isolation_report.json` (from `make test-adversarial-container-isolation`)
- `scripts/tests/adversarial/container_blackbox_report.json` (from `make test-adversarial-container-blackbox`)
- `scripts/tests/adversarial/sim2_realtime_bench_report.json` + `sim2_realtime_bench_summary.md` (from `make test-sim2-realtime-bench`)
- `scripts/tests/adversarial/sim2_adr_conformance_report.json` (from `make test-sim2-adr-conformance`)
- `scripts/tests/adversarial/sim2_ci_diagnostics.json` (from `make test-sim2-ci-diagnostics`)
- `scripts/tests/adversarial/sim2_verification_matrix_report.json` (from `make test-sim2-verification-matrix` / `make test-sim2-verification-e2e`)
- `scripts/tests/adversarial/sim2_operational_regressions_report.json` (from `make test-sim2-operational-regressions`)
- `scripts/tests/adversarial/sim2_governance_contract_report.json` (from `make test-sim2-governance-contract`)
- `scripts/tests/adversarial/preflight_report.json` (from `make test-adversarial-preflight`)
- `latest_report.json` and `attack_plan.json` include `execution_lane` metadata for auditability.
  - `latest_report.json` includes:
  - quantitative `gates` and `coverage_gates` sections (each check includes `threshold_source`),
  - `browser_execution_gates` checks proving browser-lane execution evidence (`js_executed`, DOM interactions, lineage/correlation),
  - `cohort_metrics` (persona-level outcome/latency/collateral summaries),
  - `realism_metrics` and `realism_gates` (deterministic traffic-model execution evidence for pacing, retry envelopes, and state-mode semantics),
  - `ip_range_suggestions` seed evidence (`seeded_summary`, `seeded_suggestions`, `matched_seed_suggestions`, `near_miss_suggestions`),
  - `plane_contract` metadata for attacker/control-plane guardrails,
  - `coverage_contract` metadata (schema/version/hash and canonical coverage keys).
  - `scenario_intent_gates` checks proving each passed scenario emitted intent-mapped defense evidence and matched progression constraints.
  - `real_traffic_contract` metadata (schema/version/hash and prohibited-pattern contract keys),
  - `retention_lifecycle` synthesized from runtime `retention_health` fields (`retention_hours`, `oldest_retained_ts`, `purge_lag_hours`, `pending_expired_buckets`, `last_error`, `state`, `guidance`),
  - `cost_governance` synthesized from runtime `details.cost_governance` fields (`cardinality_pressure`, `payload_budget_status`, `sampling_status`, `query_budget_status`, query/payload/compression thresholds),
  - `evidence` (`sim-run-evidence.v1`) with request-lineage, per-scenario runtime telemetry evidence rows, and control-plane lineage fields.

Notes:

- `abuse_regression` stale-token coverage is simulated in black-box mode by mutating a valid issued seed before submit (no runner-side token re-signing and no signing-secret dependency).
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
- `full_coverage` now emits `coverage_gates.defense_noop_checks` for `pow`, `challenge`, `maze`, `honeypot`, `cdp`, `rate_limit`, and `geo`; any targeted defense with zero telemetry delta fails the run.
- `full_coverage` now includes explicit PoW success/failure, challenge puzzle-failure fallback, replay-to-tarpit abuse, CDP deny path, rate-limit enforcement, and GEO block scenarios in addition to existing challenge/maze/honeypot/Akamai families.
- `full_coverage` also enforces persona and taxonomy gates:
  - `human_like_collateral_max_ratio`,
  - `required_event_reasons`,
  - `ip_range_suggestion_seed_required` (with deterministic seed traffic + evidence snapshot),
  - `persona_scheduler=round_robin` and `realism.required_retry_attempts.retry_storm>=1`.
- Plane separation contract:
  - attacker-plane requests are restricted to public paths and reject privileged headers (`Authorization`, health/admin/signing secret headers),
  - orchestrator-only setup/reset/config hooks remain on the control plane via admin-authenticated calls.
  - attacker-plane contract is versioned in `lane_contract.v1.json` and verified by `make test-adversarial-lane-contract`.
- Coverage contract governance:
  - canonical `full_coverage` requirements are versioned in `coverage_contract.v1.json`,
  - drift checks run via `make test-adversarial-coverage-contract` and are wired into `make test-adversarial-manifest`, `make test-adversarial-fast`, and `make test-adversarial-coverage`.
- Scenario intent governance:
  - canonical per-scenario intent mappings are versioned in `scenario_intent_matrix.v1.json`,
  - parity/freshness checks run via `make test-adversarial-scenario-review` and are wired into `make test-adversarial-manifest`, `make test-adversarial-fast`, and `make test-adversarial-coverage`.
  - runs now fail when a scenario is marked passed but its intent-mapped defense categories lack qualifying evidence signals.
- Real-traffic evidence governance:
  - canonical invariants and prohibited synthetic-success patterns are versioned in `real_traffic_contract.v1.json`,
  - passed scenarios must include runtime telemetry evidence (`runtime_request_count > 0` and telemetry deltas) or the run fails.
- Browser-realistic execution governance:
  - browser-realistic drivers execute via Playwright (`scripts/tests/adversarial_browser_driver.mjs`) instead of HTTP emulation,
  - each browser-realistic passed scenario must emit browser evidence fields (`browser_js_executed`, `browser_dom_events`, `browser_storage_mode`, `browser_challenge_dom_path`, and request-lineage correlation IDs),
  - transient browser-driver failures retry deterministically (`SHUMA_ADVERSARIAL_BROWSER_RETRIES`, default `2`) and fail with explicit diagnostics (`error_code`, attempt/exit metadata).
- Simulation telemetry tagging:
  - deterministic runner and container worker attach `X-Shuma-Sim-Run-Id`, `X-Shuma-Sim-Profile`, and `X-Shuma-Sim-Lane` on attacker-plane traffic,
  - backend event/monitoring read APIs include tagged rows in runtime-dev by default.
- `monitoring_after` snapshot includes nested tarpit metrics so live-loop output can report activation/progression/fallback/escalation coverage without manual JSON digging.
- Protected-lane frontier probe output (`frontier_lane_status.json`) is advisory only; deterministic coverage/replay gates remain blocking.
- Repeatability lane (`make test-adversarial-repeatability`) executes deterministic profiles three times with fixed reset/seed posture and fails on scenario/gate/coverage drift.
- Promotion lane (`make test-adversarial-promote-candidates`) normalizes frontier findings, attempts deterministic replay for regression candidates, and emits candidate -> replay -> promotion lineage with owner-review requirements.
- Promotion lane now emits hybrid-governance thresholds (`>=95%` deterministic confirmation, `<=20%` false discovery, owner disposition SLA `<=48h`) and marks blocking-required state when thresholds drift.
- Verification matrix governance:
  - `verification_matrix.v1.json` maps defense categories to required scenarios, lanes, and evidence assertions.
  - `make test-sim2-verification-matrix` validates matrix structure and report evidence diagnostics.
  - `make test-sim2-verification-e2e` executes matrix-required deterministic + frontier lanes and fails on missing row/evidence/lineage diagnostics.
- Operational regressions governance:
  - `make test-sim2-operational-regressions` enforces failure-injection, prod non-sim freshness, retention, cost, and security/privacy regression thresholds.
  - Failures emit explicit taxonomy labels for threshold and policy drifts.
- Hybrid governance contract:
  - `hybrid_lane_contract.v1.json` defines deterministic vs emergent lane boundaries, promotion thresholds, cadence ownership, and KPI/rollback governance.
  - `make test-sim2-governance-contract` validates contract structure and required operator/promotion policy markers.
- Frontier threshold lane (`make test-frontier-unavailability-policy`) tracks protected-lane degraded streaks and emits actionability state for model-refresh ownership workflows.
- Secret/setup preflight:
  - `make test-adversarial-preflight` fails early on missing placeholder or malformed `SHUMA_API_KEY`/`SHUMA_SIM_TELEMETRY_SECRET`.
  - Use preflight before smoke/coverage/promote/container black-box lanes to keep failures actionable.
- Container lane is complementary and non-replacing in this phase:
  - run `make test-adversarial-container-isolation` to validate isolation contract first,
  - then run `make test-adversarial-container-blackbox` for bounded black-box traffic execution.
  - black-box mode converts `attack_plan.json` candidates into executable action DSL steps by default, then validates and executes them under `frontier_action_contract.v1.json`.
  - container black-box actions must pass `frontier_action_contract.v1.json` reject-by-default validation before execution.
  - unsafe candidate payloads are rejected before execution, with reasons emitted in `container_blackbox_report.json -> frontier_candidate_rejections`.
  - policy violations emit structured deny/audit events in `container_blackbox_report.json -> policy_audit.events`.
  - lineage evidence is emitted in `container_blackbox_report.json -> frontier_lineage` linking model suggestion -> executed action -> runtime/admin event surfaces.
  - degraded execution state is explicit in `container_blackbox_report.json -> frontier_runtime_state` and marks fallback/outage conditions as non-passing.
  - runtime launch is blocked when `container_runtime_profile.v1.json` hardening requirements are violated.
  - each executable action must include a signed capability envelope; signature/expiry/replay/scope violations are fail-closed before request execution.
  - artifact lifecycle cleanup runs each container lane invocation with bounded TTL-based deletion and diagnostics in `container_*_report.json -> cleanup_policy`.
  - command channel semantics are explicit and bounded (`host -> worker`, queue-capacity enforced, overflow fail-closed) with evidence-channel append-only expectations in report metadata.
  - emergency stop and teardown are fail-closed: kill-switch, heartbeat-timeout, and hard-deadline paths force termination and emit terminal diagnostics in `execution_control`/`worker_failure_detail`.
  - negative-path regression tests cover secret canary leakage, out-of-scope URL/path attempts, privileged header injection attempts, and replay envelope misuse.
- `adversarial_sim_selftest.py` is intentionally tiny and non-circular: it validates simulator mechanics against fixed stub routes without asserting product defense efficacy.
