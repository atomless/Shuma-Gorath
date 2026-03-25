Date: 2026-03-25
Status: Completed

Related plan:

- [`../plans/2026-03-25-test-hygiene-2-worktree-clean-implementation-plan.md`](../plans/2026-03-25-test-hygiene-2-worktree-clean-implementation-plan.md)
- [`2026-03-25-test-hygiene-2-worktree-clean-readiness-review.md`](2026-03-25-test-hygiene-2-worktree-clean-readiness-review.md)

# TEST-HYGIENE-2 Post-Implementation Review

## What landed

`TEST-HYGIENE-2` is now delivered as a narrow worktree-clean verification tranche.

The repo now has one explicit split for adversarial and SIM2 artifacts:

1. tracked files under `scripts/tests/adversarial/` remain committed contracts, manifests, schemas, and baselines,
2. routine generated outputs from `make` targets now land under `.spin/adversarial/`,
3. the committed diff baseline exception remains `scripts/tests/adversarial/latest_report.baseline.json`.

The Makefile wiring now routes the routine report-producing targets through canonical `.spin/adversarial/` variables, including:

1. preflight,
2. deterministic adversarial report producers,
3. SIM2 realtime, diagnostics, regressions, and governance reports,
4. promotion/report-diff outputs,
5. and the container isolation or black-box report paths.

The focused wiring proof was also tightened so the repo now fails fast if those late targets drift back to tracked generated paths.

## Verification

- `make test-testing-surface-artifact-path-contract`
- `make test-adversarial-preflight`
- `make test-adversarial-smoke`
- `make test-sim2-realtime-bench`
- `make test-sim2-ci-diagnostics`
- `make test-sim2-operational-regressions`
- `make test-sim2-governance-contract`
- `make test-adversarial-promote-candidates`
- `git diff --check`

## Outcome Against Plan

The tranche met the planned requirements:

1. the output-path contract is now executable, not only documented,
2. routine `make` workflows no longer need tracked generated JSON paths for their default outputs,
3. contributor docs now make the `.spin/adversarial/` split explicit,
4. and real Makefile paths that used to dirty tracked report locations were rerun successfully against the new artifact directory.

## Remaining Gap

No further unblocked work remains inside this specific worktree-clean artifact tranche.

Broader testing hygiene still remains as separate follow-on work:

1. `TEST-HYGIENE-3`
2. `TEST-HYGIENE-4`
3. `TEST-HYGIENE-5`
4. `TEST-HYGIENE-6C`
