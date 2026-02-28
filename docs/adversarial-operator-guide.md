# Adversarial Simulation Operator Guide

Date: 2026-02-27  
Status: Active (`SIM-6`)

This guide defines how operators must interpret adversarial simulation failures and how to tune safely without introducing collateral-risk regressions.

## Scope

Use this guide for:

- `make test-adversarial-fast`
- `make test-adversarial-soak`
- `make test-adversarial-live`
- `make test-adversarial-repeatability`
- `make test-adversarial-promote-candidates`
- `make test-adversarial-sim-tag-contract`
- `make test-frontier-unavailability-policy`
- `make test-adversarial-container-isolation`
- `make test-adversarial-container-blackbox`

## SIM Run Definition Of Done (`SIM2-GC-1`)

A run must be treated as complete only when all rules below are true:

1. `latest_report.json` has `passed=true`.
2. `latest_report.json` includes `real_traffic_contract` and `evidence` sections.
3. Every passed scenario includes runtime telemetry evidence in `evidence.scenario_execution`:
   - `runtime_request_count > 0`
   - plus at least one runtime telemetry delta (`monitoring_total_delta`, `coverage_delta_total`, or `simulation_event_count_delta`) above zero.
4. `evidence.control_plane_lineage` is present with:
   - `control_operation_id`, `requested_state`, `desired_state`, `actual_state`, `actor_session`.
5. No synthetic-success pattern is used:
   - no synthetic monitoring injection,
   - no out-of-band metrics writes,
   - no control-plane-only success signaling.
6. Every passed `browser_realistic` scenario includes browser execution evidence:
   - `browser_js_executed=true`,
   - `browser_dom_events > 0`,
   - non-empty `browser_challenge_dom_path`,
   - non-empty request-lineage correlation IDs.

Canonical contract reference:

- [`sim2-real-adversary-traffic-contract.md`](sim2-real-adversary-traffic-contract.md)

All profiles write a report to `scripts/tests/adversarial/latest_report.json` unless `ADVERSARIAL_REPORT_PATH` overrides it.
All runs also emit `scripts/tests/adversarial/attack_plan.json` with frontier mode/provider metadata and sanitized candidate payloads.
Promotion triage emits `scripts/tests/adversarial/promotion_candidates_report.json` with candidate -> replay -> promotion lineage records.
Frontier threshold policy emits `scripts/tests/adversarial/frontier_unavailability_policy.json`.
All manifests and reports are locked to `execution_lane=black_box`; non-black-box lane values are rejected at validation time.
Lane capability boundaries are versioned in `scripts/tests/adversarial/lane_contract.v1.json` and validated by `make test-adversarial-lane-contract`.
Simulation-tag signing contract is versioned in `scripts/tests/adversarial/sim_tag_contract.v1.json` and validated by `make test-adversarial-sim-tag-contract`.
Full-coverage category obligations are versioned in `scripts/tests/adversarial/coverage_contract.v1.json` and validated by `make test-adversarial-coverage-contract`.
Container frontier action grammar contract is versioned in `scripts/tests/adversarial/frontier_action_contract.v1.json` and enforced as reject-by-default by host and worker validators.
Browser-lane execution proof is enforced via `latest_report.json -> gates.browser_execution_gates`.
`make test-adversarial-live` now classifies failures as `transient` or `fatal`, retries transient cycles with capped backoff, and only terminates after `ADVERSARIAL_FATAL_CYCLE_LIMIT` consecutive fatal cycles.
Container lane emits:
1. `scripts/tests/adversarial/container_isolation_report.json`
2. `scripts/tests/adversarial/container_blackbox_report.json`
3. `container_blackbox_report.json` includes `frontier_action_source` and `frontier_action_lineage` to trace attack-plan candidates to executed requests.
4. `container_blackbox_report.json -> policy_audit` records explicit deny/allow boundary decisions for action validation and egress-policy enforcement.
5. `container_blackbox_report.json -> frontier_candidate_rejections` lists sanitized model outputs that were blocked before execution.
6. `container_blackbox_report.json -> frontier_lineage` reports end-to-end lineage completeness (`model suggestion -> executed action -> runtime events -> monitoring events`).
7. `container_blackbox_report.json -> frontier_runtime_state` surfaces degraded fallback/outage conditions and marks degraded runs as non-passing.

## Deterministic + Containerized Coexistence Contract (SIM-V2-15)

Current policy is explicit coexistence, not replacement:

1. Deterministic lanes are the canonical protected-lane and release blockers:
   - `make test-adversarial-smoke`
   - `make test-adversarial-abuse`
   - `make test-adversarial-akamai`
   - `make test-adversarial-coverage`
2. Containerized black-box lanes are complementary and scheduled/manual in this phase:
   - `make test-adversarial-container-isolation`
   - `make test-adversarial-container-blackbox`
   - container executions must pass frontier action DSL validation before any request is emitted.
3. Frontier lane remains adaptive discovery input; deterministic replay confirmation remains the blocking regression oracle.

Capability mapping (must stay explicit):

| Requirement family | Deterministic lane (mandatory) | Containerized lane (complementary) |
|---|---|---|
| Merge/release blocking regression oracle | Primary and required | Not used as release blocker in this phase |
| Full category gate contract (`full_coverage`) | Primary and required | Observational/complementary |
| Replay/order/stale deterministic abuse regressions | Primary and required | Complementary realism signal |
| Akamai fixture policy behavior | Primary and required | Not primary coverage contract |
| Isolation boundary and runtime-hardening checks | Not primary | Primary (`container_isolation`) |
| Alternative runtime traffic realism signal | Secondary | Primary (`container_blackbox`) |
| Frontier candidate promotion confirmation | Primary (`promote-candidates` replay gate) | Not primary |

Keep-both-vs-replace decision record:

1. ADR: [`docs/adr/0005-adversarial-lane-coexistence-policy.md`](adr/0005-adversarial-lane-coexistence-policy.md).
2. Required migration checklist template: [`docs/adr/adversarial-lane-parity-signoff-checklist.md`](adr/adversarial-lane-parity-signoff-checklist.md).
3. Deterministic-lane demotion is forbidden without owner approval plus completed parity sign-off evidence.

## Simulation Metadata Tagging and Filtering (SIM-V2-20)

Adversary-generated traffic is tagged at request time with:

1. `sim_run_id`
2. `sim_profile`
3. `sim_lane`
4. `sim_ts`
5. `sim_nonce`
6. `sim_signature` (HMAC-SHA256 over canonical `sim-tag.v1` message)

Storage and read-path policy:

1. Simulation telemetry writes to canonical event/monitoring stores and is identified by metadata fields (`sim_run_id`, `sim_profile`, `sim_lane`, `is_simulation`).
2. Admin read endpoints (`/admin/events`, `/admin/cdp/events`, `/admin/monitoring`, `/admin/monitoring/delta`, `/admin/monitoring/stream`, `/admin/ip-bans/delta`, `/admin/ip-bans/stream`) include tagged simulation rows in runtime-dev by default.
3. Non-dev runtime remains default-safe because adversary simulation control surfaces are unavailable.
4. Unsigned/invalid/stale/replayed simulation tags must not activate simulation context; requests stay in normal telemetry partition.
5. Invalid simulation-tag attempts emit explicit policy-signal telemetry:
   - `S_SIM_TAG_MISSING_SECRET`
   - `S_SIM_TAG_MISSING_REQUIRED_HEADERS`
   - `S_SIM_TAG_INVALID_HEADER_VALUE`
   - `S_SIM_TAG_INVALID_TIMESTAMP`
   - `S_SIM_TAG_TIMESTAMP_SKEW`
   - `S_SIM_TAG_SIGNATURE_MISMATCH`
   - `S_SIM_TAG_NONCE_REPLAY`

Containerized attacker-lane handling:

1. Container black-box workers must not receive `SHUMA_*` secrets (including `SHUMA_SIM_TELEMETRY_SECRET`).
2. Host orchestrator issues bounded pre-signed sim-tag envelopes per run and passes only those non-secret envelopes into the container.
3. Replay-window checks in runtime enforce one-use nonce semantics for signed tags.

## Sim-Tag Secret Rotation and Troubleshooting

Rotation policy:

1. Rotate `SHUMA_SIM_TELEMETRY_SECRET` whenever adversarial runner hosts are reprovisioned or when simulation-tag validation anomalies are detected.
2. Rotate by updating the secret in `.env.local`/deploy environment, restarting runtime-dev, and re-running `make test-adversarial-fast`.
3. Do not share this secret with containerized attacker lanes; only deterministic host-side signers should hold it.

Troubleshooting sequence for failed sim tagging:

1. Confirm runtime guards: `SHUMA_RUNTIME_ENV=runtime-dev` and `SHUMA_ADVERSARY_SIM_AVAILABLE=true`.
2. Confirm secret presence on host runner and runtime process: `SHUMA_SIM_TELEMETRY_SECRET` is non-empty.
3. Run `make test-adversarial-sim-tag-contract` to verify contract parity.
4. Inspect `/metrics` for `bot_defence_policy_signals_total{signal=\"S_SIM_TAG_*\"}` counters and identify dominant failure reason.
5. If failures persist, restart `make dev` to clear stale process env and rerun `make test-adversarial-fast`.

## Frontier Architecture Modes

Frontier attack-candidate generation must run in one of two explicit modes:

1. `single_provider_self_play`
2. `multi_provider_playoff`

Mode semantics:

1. `single_provider_self_play`
   - One configured provider key.
   - Planner/attacker/critic roles remain isolated but share one model family.
   - Discovery confidence is lower because role diversity is reduced.
2. `multi_provider_playoff`
   - Two or more configured provider keys.
   - Cross-provider role assignment increases adversarial diversity.
   - Discovery confidence is higher and this is the recommended protected-lane posture.

Operator guidance:

1. `provider_count=0`: run remains deterministic-only; frontier lane is degraded advisory mode.
2. `provider_count=1`: run remains valid but reduced-diversity warning must be treated as a confidence downgrade.
3. `provider_count>=2`: preferred minimum for higher-confidence discovery.

## Protected-Lane Policy (Deterministic Oracle + Frontier Advisory)

Protected lanes must run both:

1. Deterministic coverage oracle (`make test-adversarial-coverage`) as a blocking gate.
2. Frontier lane attempt (`make test-adversarial-frontier-attempt`) as advisory telemetry.

Rules:

1. Frontier degraded status (missing key, auth error, timeout, provider outage) is non-blocking.
2. Deterministic coverage/replay failures remain merge/release blockers.
3. Frontier attempt output (`scripts/tests/adversarial/frontier_lane_status.json`) must be archived for PR/release auditing.
4. If frontier status remains degraded for 10 consecutive protected-lane runs or 7 days (whichever comes first), operators must open and assign a supported-model refresh action and update frontier model documentation.
5. Protected-lane automation uses `make test-frontier-unavailability-policy` (with `FRONTIER_POLICY_ENABLE_GITHUB=1`) to update tracker state and open/assign refresh action issues when the threshold is crossed.

## Frontier Finding Triage + Promotion (SIM-V2-18)

`make test-adversarial-promote-candidates` is the canonical triage/promotion lane.

Pipeline contract:

1. Normalize frontier findings into stable IDs (`finding_id`) with scenario family, path, headers, cadence pattern, observed outcome, severity, and risk metadata.
2. Carry frontier diversity metadata on every finding (`frontier_mode`, `provider_count`, provider/model list, `diversity_confidence`).
3. Attempt deterministic replay for each regression candidate and classify:
   - `confirmed_reproducible`
   - `not_reproducible`
   - `needs_manual_review`
4. Require owner review before any confirmed finding can become a blocking regression case.
5. Enforce diversity policy:
   - `single_provider_self_play`: owner review is mandatory and confidence is reduced.
   - `multi_provider_playoff`: higher initial confidence, but deterministic confirmation and owner review are still mandatory.

SLA for unresolved high-severity findings:

1. `PR` lanes: unresolved high-severity findings (`confirmed_reproducible` or `needs_manual_review`) must be dispositioned within 24 hours.
2. `Release` lanes: unresolved high-severity findings must be dispositioned before release cut; release remains blocked when deterministic replay confirms a high-severity regression.

## Live Loop Guardrails (SIM-V2-9)

Live-loop defaults are operator-observability-first:

1. `ADVERSARIAL_CLEANUP_MODE=0` (default) preserves state between cycles.
2. `ADVERSARIAL_CLEANUP_MODE=1` enables explicit cleanup-per-cycle mode.
3. Cycles that emit only admin/config noise (no meaningful defense event reasons) are classified as fatal-quality failures.
4. Loop logs include cycle classification, retry count, backoff seconds, and terminal failure reason.

## Inputs You Must Capture

For every failing run, operators must capture:

1. Exact command used (`make` target + env overrides).
2. Report artifact (`scripts/tests/adversarial/latest_report.json`).
3. Attack plan artifact (`scripts/tests/adversarial/attack_plan.json`).
4. Runtime config snapshot (`GET /admin/config`) from the failing environment.
5. Monitoring snapshot (`GET /admin/monitoring?hours=24&limit=10` in dev runtime) from the same time window.
6. Commit SHA and environment (`runtime-dev` or `runtime-prod`).
7. Runner plane-separation evidence (`latest_report.json` -> `plane_contract`).
8. Coverage contract evidence (`latest_report.json` -> `coverage_contract`) including schema/hash and category obligations.
9. Realism evidence (`latest_report.json` -> `realism_metrics` + `realism_gates`) for pacing/retry/state-mode conformance.

## Triage Order

Operators must triage in this order:

1. Scenario failures in `results` where `passed=false`.
2. Gate failures in `gates.checks` where `passed=false`.
3. Coverage gate failures in `coverage_gates.checks` where `passed=false`.
4. Defense no-op detector failures in `coverage_gates.defense_noop_checks` (`full_coverage`) where `passed=false`.
5. Coverage deltas in `coverage_gates.coverage.deltas` (for `full_coverage`/soak).
6. Persona collateral and cost envelopes in `cohort_metrics`.
7. Realism gate failures in `realism_gates.checks` and persona/runtime evidence in `realism_metrics`.
8. Seeded IP-range evidence in `ip_range_suggestions`.
9. Tarpit progression/fallback/escalation counters in Monitoring tab (`Tarpit Progression` section) and `monitoring_after.tarpit.metrics` in report artifacts.

Operators must not tune thresholds before confirming whether failures are scenario mismatches versus gate regressions.

## Coverage Contract Update Protocol

When `full_coverage` obligations must change, update in this order:

1. Update SIM2 plan coverage table in `docs/plans/2026-02-26-adversarial-simulation-v2-plan.md`.
2. Update canonical contract `scripts/tests/adversarial/coverage_contract.v1.json`.
3. Update manifest `profiles.full_coverage.gates` parity in both `scenario_manifest.v1.json` and `scenario_manifest.v2.json`.
4. Run `make test-adversarial-coverage-contract`, `make test-adversarial-manifest`, and `make test-adversarial-coverage`.

`full_coverage` drift is expected to fail fast if any of these artifacts diverge.

## Scenario Failure Interpretation

When `passed=false`, use `driver`, `expected_outcome`, `observed_outcome`, and `detail`.

### Driver-to-Action Mapping

| Driver | Expected posture | Primary checks | Typical operator action |
|---|---|---|---|
| `allow_browser_allowlist` | `allow` | browser allowlist and policy mode | Correct allowlist entries; avoid broad wildcarding |
| `not_a_bot_pass` | `not-a-bot` | Not-a-Bot token flow and pass scoring | Adjust pass/fail scores in small increments |
| `not_a_bot_replay_abuse` / `not_a_bot_stale_token_abuse` / `not_a_bot_ordering_cadence_abuse` | `maze` | replay/order/timing protections | Keep abuse escalation strict; fix sequence checks if downgraded |
| `not_a_bot_replay_tarpit_abuse` | `tarpit` | replay abuse escalation through tarpit entry path | Keep tarpit enabled + budgeted; investigate fallback/escalation if downgraded to block |
| `challenge_puzzle_fail_maze` | `maze` | puzzle failure routing and sequence envelope checks | Preserve incorrect-answer fallback semantics and sequence validation |
| `pow_success` | `allow` | `/pow` issue + `/pow/verify` success | Validate PoW difficulty/TTL and sequence timing envelope |
| `pow_invalid_proof` | `monitor` | PoW invalid proof rejection path | Ensure invalid proof remains rejected; do not downgrade to allow |
| `rate_limit_enforce` / `retry_storm_enforce` | `deny_temp` | limiter thresholds and enforcement mode under burst traffic | Verify `rate_limit`, provider mode, retry-storm posture, and outage posture |
| `geo_challenge` / `geo_maze` / `geo_block` | `challenge` / `maze` / `deny_temp` | GEO lists and trusted header gating | Confirm country list routing and trusted header behavior |
| `header_spoofing_probe` | `monitor` | untrusted forwarded/header spoof rejection semantics | Ensure spoofed headers do not trigger privileged GEO enforcement |
| `honeypot_deny_temp` | `deny_temp` | honeypot path and ban enforcement | Verify honeypot remains active and banning works |
| `fingerprint_inconsistent_payload` | `monitor` | malformed external fingerprint ingestion handling | Keep invalid payload rejection deterministic (`400`) without bypassing telemetry |
| `cdp_high_confidence_deny` | `deny_temp` | CDP ingest + auto-ban deny path | Confirm follow-up request is denied and event taxonomy is present |
| `akamai_additive_report` | `monitor` | additive edge signal ingest | Keep additive mode non-authoritative |
| `akamai_authoritative_deny` | `deny_temp` | authoritative edge deny path | Verify deny only in authoritative mode |

## Gate Failure Interpretation

`gates.checks` includes quantitative assertions.

Common SIM-v2 checks and expected operator response:

- `human_like_collateral_ratio`
  - Investigate `cohort_metrics.human_like.collateral_ratio` first.
  - Tune challenge/maze/tarpit escalation thresholds before editing ratio bounds.
- `event_reason_prefix_*`
  - Confirm required event taxonomy is still emitted and prefixed consistently.
  - Fix route/reason wiring before relaxing required prefixes.
- `ip_range_suggestion_seed_match`
  - Inspect `ip_range_suggestions.seed_evidence`, `matched_seed_suggestions`, and `near_miss_suggestions`.
  - Do not suppress this gate; fix seeding prerequisites or suggestion aggregation drift.

## Dashboard Toggle Orchestration (SIM-V2-9A)

The dashboard `Adversary Sim` global toggle is the only supported UI control path for dev orchestration lifecycle.

Control-plane endpoints:

1. `POST /admin/adversary-sim/control` for explicit ON/OFF transitions.
2. `GET /admin/adversary-sim/status` for phase + guardrail visibility.
3. `POST /admin/adversary-sim/history/cleanup` for explicit retained-telemetry cleanup.

Lifecycle semantics:

1. `generation_active` describes whether adversary traffic producers are currently running.
2. `historical_data_visible` remains `true` after auto-off; retained telemetry stays queryable until retention expiry or explicit cleanup.
3. `history_retention` status fields expose retention window and cleanup command.

Guardrail constants (hard-coded, not operator-configurable):

1. `max_duration_seconds=900` (runtime key `adversary_sim_duration_seconds` is bounded to `30..900`, default `180`).
2. `max_concurrent_runs=1`.
3. `cpu_cap_millicores=1000`.
4. `memory_cap_mib=512`.
5. `queue_policy=reject_new`.

Lifecycle state diagram:

```mermaid
stateDiagram-v2
  [*] --> Off
  Off --> Running: "toggle ON (control endpoint)"
  Running --> Stopping: "toggle OFF (manual_off)"
  Running --> Stopping: "window expiry (auto_window_expired)"
  Running --> Stopping: "config disabled / unavailable"
  Stopping --> Off: "active counts == 0"
  Stopping --> Off: "stop timeout -> forced_kill_timeout"
```

Failure-handling rules:

1. Unauthenticated, unauthorized, and CSRF-invalid control attempts must be rejected and written to admin event log.
2. If stop does not converge to zero-active state before stop timeout, orchestrator must force-kill and return to safe `off` state.
3. If runtime is not `runtime-dev` or `SHUMA_ADVERSARY_SIM_AVAILABLE=false`, control/status endpoints must fail closed (`404`).
4. Status polling and lifecycle-state rendering are presentation only; defense behavior remains server-authoritative.
5. Use `make adversary-sim-history-clean` only when explicit history reset is required; auto-off must not be treated as data deletion.

### `latency_p95` Failure

- Operators must verify runtime saturation before relaxing latency limits.
- Operators must not widen thresholds by more than 20% in one change.

### `ratio_*` Failure

- Operators must confirm scenario composition did not change.
- Operators must tune policy inputs (for example rate/GEO/Not-a-Bot thresholds), not the ratio bounds first.
- Operators must update ratio bounds only after observed behavior is intentionally changed and documented.

### `telemetry_*_amplification` Failure

- Operators must treat this as a resource/cost regression first.
- Operators must reduce noisy writes (event volume, duplicate logging paths) before relaxing amplification limits.

### `coverage_*` Failure (Soak)

- Operators must confirm the corresponding scenario driver actually executed.
- Operators must confirm monitoring counters are still mapped to the same semantic event.
- Operators must not disable coverage checks to make failures disappear.

## Safe Tuning Rules

1. Operators must change one control family at a time (for example only `rate_limit` knobs, then rerun).
2. Operators must rerun `make test-adversarial-fast` after every tuning change.
3. Operators must rerun `make test-adversarial-soak` before promotion when tuning touched PoW/rate/GEO/Akamai pathways.
4. Operators must document every threshold change with before/after values and reason.
5. Operators must not combine unrelated policy and observability changes in one promotion.

## Rollback Rules

Rollback must be immediate when any of the following occurs:

1. `fast` profile fails after a tuning change.
2. Any abuse scenario downgrades from `maze`/`deny_temp` to `allow`/`monitor`.
3. Telemetry amplification exceeds bounds by more than 2x baseline.
4. GEO/Rate/PoW enforcement drops below expected coverage deltas in soak.

Rollback action:

1. Restore last known-good config snapshot.
2. Re-run `make test-adversarial-fast`.
3. Re-run `make test-adversarial-soak` before reattempting promotion.
