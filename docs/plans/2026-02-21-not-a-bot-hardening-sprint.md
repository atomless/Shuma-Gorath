# Not-a-Bot Hardening Sprint (Short Follow-up)

Date: 2026-02-21  
Status: Proposed (deferred follow-up)  
Owner: Challenge/Policy track

## Context

This sprint is a focused hardening pass for Not-a-Bot as a medium-strength checkpoint.
It is explicitly scoped to tighten pass issuance against high-fidelity automation while preserving accessibility-capable human completion.

## Sprint goals

1. Reduce low-evidence pass outcomes.
2. Require stronger corroboration before issuing a pass marker.
3. Add server-side continuity checks so single-request mimicry is less effective.
4. Preserve keyboard/touch/assistive pass-capability without allowing easy fast-path automation.

## Out of scope

- Replacing puzzle/maze escalation as the stronger barrier.
- “Unsolvable by any browser automation” as a design target.
- Third-party CAPTCHA dependency in default path.

## Hardening work items

### H1: Cap unknown modality below pass (escalate only)

Problem:
- `activation_method=unknown` currently remains pass-capable under sufficiently strong surrounding telemetry.

Change:
- Keep `unknown` modality valid for submission handling, but cap score/output so it cannot reach `pass`.
- Route `unknown` modality to `escalate_puzzle` at most (unless other hard-fail rules apply).

Acceptance criteria:
- `unknown` modality can never produce `NotABotDecision::Pass`.
- `unknown` modality can still produce `EscalatePuzzle` when non-malicious evidence is present.
- Existing keyboard/touch pathways remain unaffected.

### H2: Require stronger corroboration before pass marker issuance

Problem:
- Pass marker issuance may be reachable with insufficient corroborating evidence in edge cases.

Change:
- Introduce explicit pass-evidence gates in addition to numeric threshold, such as:
  - minimum evidence classes satisfied (timing + modality + focus stability),
  - stronger consistency expectations for pointer-mode pass.
- Keep replay/seed/binding/ordering checks as mandatory preconditions.

Acceptance criteria:
- Pass marker is only issued when threshold and corroboration gates both pass.
- Low-evidence “slow down and click” patterns no longer reach pass at default config.
- Legitimate keyboard-only and touch-first flows remain pass-capable under documented evidence rules.

### H3: Tie pass to server-side consistency across attempts/sessions

Problem:
- Current decision is primarily request-local once envelope/replay checks pass.

Change:
- Add bounded server-side continuity signals (low-cardinality, short retention), for example:
  - suspicious volatility across very recent attempts in the same IP/UA bucket,
  - repeated borderline outcomes within short windows.
- Use these signals as pass-gating modifiers (escalate instead of pass), not silent hard-blocks.

Acceptance criteria:
- Repeated inconsistent/borderline behavior degrades pass to escalate.
- Storage remains bounded by TTL/window and does not create long-term profiling.
- Monitoring includes counters for “pass downgraded by consistency gates”.

### H4: Keep keyboard/touch accessibility pass-capable while removing low-evidence fast paths

Problem:
- Hardening can easily regress accessibility or overfit to pointer-only behavior.

Change:
- Preserve modality-neutral accessibility policy:
  - keyboard/touch/assistive paths remain first-class pass-capable.
- Tighten low-evidence fast paths by combining:
  - operation envelope timing floor,
  - minimum interaction plausibility requirements,
  - stronger corroboration for pass marker issuance.

Acceptance criteria:
- Dedicated tests prove keyboard-only and touch-first pass remains possible.
- Dedicated negative tests prove too-fast/too-thin-evidence paths do not pass.
- No direct negative scoring for assistive usage indicators.

## Implementation checklist

- [ ] Add explicit modality cap logic for `unknown` -> max `EscalatePuzzle`.
- [ ] Add pass-evidence gate evaluation in submit decision path.
- [ ] Add short-lived server-side consistency gate state and decision wiring.
- [ ] Extend unit tests for new scoring/gating branches.
- [ ] Extend integration tests for pass/escalate under consistency modifiers.
- [ ] Update operator docs for new pass semantics and tuning impact.

## Verification

Required:

- `make test-unit`
- `make test-integration`
- `make test-dashboard-e2e` (browser-permitted environment)

If browser launch restrictions apply in sandboxed environments:

- `PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1 make test-dashboard-e2e`

In skip mode, record explicitly that browser E2E was environment-blocked.

## Exit criteria

- Unknown modality cannot pass.
- Pass marker issuance requires stronger corroboration than threshold alone.
- Additional server-side continuity checks influence pass vs escalate deterministically.
- Accessibility-capable keyboard/touch paths remain pass-capable and covered by tests.
