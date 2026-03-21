# ADV-RUN-ARCH-1 Post-Implementation Review

Date: 2026-03-21
Plan reference: `docs/plans/2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md`
Task: `ADV-RUN-ARCH-1`

## Scope Delivered

`ADV-RUN-ARCH-1` completed the planned structural decomposition of `scripts/tests/adversarial_simulation_runner.py` into focused helper modules while preserving the existing CLI, report artifacts, and regression-test surface:

- `scripts/tests/adversarial_runner/contracts.py`
  - contract loading, schema validation, and deterministic corpus metadata
- `scripts/tests/adversarial_runner/runtime_state.py`
  - HTTP/result/state carrier types plus attacker and control-plane client helpers
- `scripts/tests/adversarial_runner/shared.py`
  - shared lightweight normalization helpers
- `scripts/tests/adversarial_runner/reporting.py`
  - report-section builders for retention, cost, and security/privacy checks
- `scripts/tests/adversarial_runner/governance.py`
  - frontier payload schema enforcement, redaction, and attack-generation contract loading
- `scripts/tests/adversarial_runner/discovery_scoring.py`
  - frontier metadata, diversity/discovery scoring, and attack-plan shaping
- `scripts/tests/adversarial_runner/evidence.py`
  - monitoring snapshot extraction, execution evidence shaping, and coverage/intent checks
- `scripts/tests/adversarial_runner/execution.py`
  - execution-lane, realism, request-contract, and cohort/profile-coordination helpers
- `scripts/tests/adversarial_simulation_runner.py`
  - thin CLI/orchestration shell, stable public regression surface, and remaining runner-owned orchestration methods

## Plan Conformance Review

### 1. Contract loading and schema helpers

Delivered as planned in `scripts/tests/adversarial_runner/contracts.py`. The runner no longer owns the deterministic corpus, sim-tag contract, lane contract, and related schema-loading responsibilities directly.

### 2. Scenario execution and profile coordination

Delivered as planned in `scripts/tests/adversarial_runner/execution.py` and `scripts/tests/adversarial_runner/runtime_state.py`. The runner shell no longer owns retry/state-mode helpers, request-contract checks, realism/cohort shaping, or HTTP/result carrier types directly.

### 3. Evidence shaping and report-row materialization

Delivered as planned in `scripts/tests/adversarial_runner/evidence.py`. Monitoring snapshot extraction, scenario execution evidence, runtime/browser evidence checks, scenario-intent checks, and coverage-depth shaping are no longer mixed into the CLI shell.

### 4. Discovery scoring and frontier candidate shaping

Delivered as planned in `scripts/tests/adversarial_runner/discovery_scoring.py`. Frontier metadata, diversity/discovery scoring, and attack-plan construction now live with the frontier helper surface instead of staying inside the top-level runner file.

### 5. Governance and artifact-safety helpers

Delivered as planned in `scripts/tests/adversarial_runner/governance.py`. Frontier payload sanitization, schema validation, and attack-generation contract loading now live together instead of being interleaved with report rendering or execution logic.

### 6. Reporting helpers

Delivered as planned in `scripts/tests/adversarial_runner/reporting.py`. Retention, cost-governance, and security/privacy report sections are now isolated from the shell orchestration path.

## Verification Evidence

The behavior-preserving tranche was verified with the focused adversarial runner gates required by the plan:

- `make test-adversarial-runner-architecture`
- `make test-adversarial-python-unit`
- `make test-adversarial-lane-contract`
- `git diff --check`

Focused regression proof remained green after the tranche-local follow-up that moved `build_attack_plan` into `discovery_scoring.py`:

- `scripts/tests/test_adversarial_simulation_runner.py`
- `scripts/tests/adversarial_runner/discovery_scoring.py`
- `scripts/tests/adversarial_runner/evidence.py`
- `scripts/tests/adversarial_runner/execution.py`

## Architectural Result

The hotspot file is no longer the only home for contract loading, state carriers, frontier governance, frontier scoring, attack-plan shaping, evidence projection, and realism/profile coordination. After the tranche:

- `scripts/tests/adversarial_simulation_runner.py` is reduced from the pre-tranche `6950` lines to `4542` lines while keeping the stable CLI and regression surface.
- later `ADV-PROMO-1` work now has real helper homes around promotion/discovery semantics instead of landing back into one file.
- later `OPS-BENCH-2`, `OVR-RECON-1`, and `OVR-AGENT-1` work can consume adversarial outputs without re-concentrating unrelated helper logic into the runner shell.

## Shortfall Check

One tranche-local shortfall was found during review: `build_attack_plan` still remained in the runner shell even after the earlier discovery/governance extractions, which meant frontier candidate shaping had not fully moved behind the dedicated discovery helper surface.

That shortfall was closed immediately as `ADV-RUN-ARCH-1-REVIEW-1` by moving attack-plan shaping into `scripts/tests/adversarial_runner/discovery_scoring.py` while keeping the runner-facing wrapper and patch seam intact for the existing tests.

No `ADV-RUN-ARCH-1` shortfall remains open before proceeding to `OPS-BENCH-2`.
