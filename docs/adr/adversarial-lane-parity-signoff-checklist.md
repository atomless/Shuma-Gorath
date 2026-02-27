# Adversarial Lane Parity Sign-off Checklist (Template)

Use this checklist before proposing any deterministic-lane demotion/replacement.

## Ownership and Approval

- [ ] Requested by (team/person):
- [ ] Primary owner:
- [ ] Approver:
- [ ] Approval recorded (link to issue/PR/comment):

## Scope of Proposed Change

- [ ] Proposed lane migration summary:
- [ ] Deterministic targets affected:
- [ ] Candidate replacement lane set:
- [ ] Rollback plan:

## Required Parity Evidence

### 1) Category parity (`100%`)

- [ ] Evidence artifact links:
- [ ] Result:

### 2) Reproducibility parity (`>= 95%` deterministic replay confirmation over rolling 20 findings)

- [ ] Findings window reference:
- [ ] Confirmation rate:
- [ ] Evidence artifact links:

### 3) Stability parity (10 consecutive protected-lane runs without flaky nondeterministic failures)

- [ ] Run window reference:
- [ ] Consecutive pass count:
- [ ] Evidence artifact links:

### 4) Cost parity (`<= 20%` protected-lane runtime increase vs deterministic-only baseline unless explicitly approved)

- [ ] Deterministic-only baseline (minutes):
- [ ] Candidate runtime (minutes):
- [ ] Runtime delta (%):
- [ ] Explicit over-budget approval link (required if >20%):

## Governance and Documentation Updates

- [ ] `todos/todo.md` updated with explicit migration acceptance criteria.
- [ ] ADR updated or superseding ADR added (`docs/adr/`).
- [ ] `docs/testing.md` updated with new mandatory/scheduled lane contract.
- [ ] `docs/adversarial-operator-guide.md` updated with operator triage implications.
- [ ] Branch protection/check policy documentation updated.

## Final Sign-off

- [ ] Owner sign-off:
- [ ] Approver sign-off:
- [ ] Date:
- [ ] Effective change window:
