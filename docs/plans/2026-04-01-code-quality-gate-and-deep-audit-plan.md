# 2026-04-01 Code-Quality Gate And Deep Audit Plan

## Goal

Make code-quality analysis a compulsory part of closing every non-doc tranche, while preserving truthful target naming and acknowledging that the stronger semantic audit tools still expose existing repo debt.

## Scope

This tranche will:

1. add a mandatory `make test-code-quality` gate,
2. add a broader `make audit-code-quality-deep` lane,
3. wire the mandatory gate into policy, PR, and CI surfaces,
4. record the current limitation that the deeper audit is not yet green enough to become the universal completion gate.

This tranche will not:

1. burn down the existing strict Clippy or JS-aware Svelte semantic debt,
2. add new dashboard linting dependencies such as ESLint or Knip yet,
3. pretend the deeper audit is already clean enough to be mandatory.

## Acceptance Criteria

### AC1. Makefile exposes truthful code-quality gates

Pass when:

- `make test-code-quality` exists and runs:
  - `make test-code-quality-contract`
  - `make test-native-build-warning-hygiene`
  - `make test-dashboard-svelte-check`
- `make audit-code-quality-deep` exists and runs:
  - strict `cargo clippy --all-targets --all-features -- -D warnings`
  - JS-aware dashboard `svelte-check`
- target help text and docs make it explicit that the deep audit is broader than the mandatory gate

Proof:

- `make test-code-quality-contract`

### AC2. The mandatory gate is part of the repo completion contract

Pass when:

- `AGENTS.md`, `docs/project-principles.md`, and `CONTRIBUTING.md` all require `make test-code-quality` for non-doc tranche completion
- `.github/pull_request_template.md` includes a dedicated `make test-code-quality` checklist item
- `docs/testing.md` documents both the mandatory gate and the deeper audit lane

Proof:

- `make test-code-quality-contract`

### AC3. CI and the umbrella suite fail fast on missing code-quality proof

Pass when:

- `.github/workflows/ci.yml` runs `make test-code-quality` before the full suite
- `make test` begins with `make test-code-quality`

Proof:

- `make test-code-quality-contract`
- `make test-code-quality`

### AC4. Debt honesty is preserved

Pass when:

- research and plan docs record that strict Clippy and JS-aware dashboard semantic checks still expose known repo debt
- backlog carries a follow-on item to graduate the deep audit into the mandatory gate only after that debt is removed

Proof:

- dated docs in `docs/research/` and `docs/plans/`
- backlog updates in `todos/blocked-todo.md`

## Proof Surface

- Makefile target wiring
- package script surface
- CI workflow wiring
- contributor/policy docs
- PR template completion checklist
- backlog truth for the deferred promotion of the deep audit lane

## Verification Plan

Mandatory:

- `make test-code-quality-contract`
- `make test-code-quality`

Advisory baseline capture for this tranche:

- `make audit-code-quality-deep`

The advisory audit is expected to remain informative rather than green until the follow-on debt-cleanup tranche lands.
