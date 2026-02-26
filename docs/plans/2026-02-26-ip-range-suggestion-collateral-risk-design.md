# IP Range Suggestion + Collateral Risk Design

Date: 2026-02-26  
Status: Proposed (design-first; implementation not started)

## Goal

Reduce operator burden in IP-range tuning by auto-generating candidate CIDR rules from the last 24h of telemetry, while quantifying likely collateral impact before enforcement.

Success means:

1. Operators can apply high-quality suggestions with one click.
2. Suggestions include an explicit collateral-risk estimate based on both botness and humanness evidence.
3. Auto-suggested ranges are safely bounded (narrow-first, guardrailed broadness).

## Motivation Alignment (Project Core Goals)

This feature exists to reinforce Shuma’s core defense posture:

- Keep defense friction low for legitimate humans by explicitly measuring and suppressing collateral-risk.
- Shift bot-onslaught burden away from operators and humans by automating high-signal range identification and safer bounds.
- Increase attacker cost asymmetrically by accelerating policy response where abuse concentration is clear, without broad blind blocking.

In short: less manual toil and fewer false positives for humans, faster and more targeted cost-imposition on abusive bot traffic.

## Scope

In scope:

- Candidate CIDR suggestion engine (24h lookback).
- Collateral-risk estimator combining strong bot and human signals.
- Safer-bound recommendations (split/narrow suggestions when blast radius is high).
- Dashboard suggestions pane integrated with existing custom rules flow.

Out of scope (for first slice):

- External reputation feed ingestion.
- ASN-wide or country-wide auto-suggestions.
- Automatic enforcement without operator action.

## Current State (what we can leverage now)

Existing strengths:

- IP range policy pipeline emits structured outcomes in event logs (`source`, `source_id`, `action`, `matched_cidr`, taxonomy data).
- Monitoring already tracks top offender buckets for challenge/pow/rate/honeypot and recent event rows.
- Not-a-Bot outcomes include `pass/escalate/fail/replay`, including `not_a_bot_pass` events with client IP.

Current gaps for high-quality collateral scoring:

- No explicit per-IP evidence for successful puzzle challenge solves.
- No low-cost baseline traffic sampling that captures likely-human allowed traffic per bucket.
- `recent_events` window in monitoring payload is capped and optimized for UI, not suggestion computation.

## Approaches Considered

### Option A: Event-log only heuristic (minimal change)

- Build suggestions only from existing event reasons and ban fingerprints.
- Pros: fastest implementation, no telemetry additions.
- Cons: weak denominator for collateral-risk; underestimates human blast radius where events are sparse.

### Option B: Full per-request per-IP telemetry

- Log request-level score/outcome for all traffic, then compute suggestions precisely.
- Pros: strongest accuracy.
- Cons: high KV write/read amplification, cardinality risk, violates cost-discipline goals.

### Option C (Recommended): Hybrid bounded telemetry + event synthesis

- Keep event-driven evidence as primary signal.
- Add bounded/sampled bucket counters for likely-human traffic and challenge-solved signals.
- Compute suggestions from a dedicated server-side summarizer (not UI row scraping).
- Pros: materially better collateral estimates with controlled overhead.
- Cons: moderate implementation scope.

Recommendation: Option C.

## Evidence Model (24h)

Evidence is aggregated by IP bucket (`/24` for IPv4, `/64` for IPv6) first, then rolled up into CIDR candidates.

### Strong bot evidence (positive abuse weight)

- Honeypot hits, tarpit escalation, short-ban abuse paths.
- Replay/sequence violations (`expired_replay`, `sequence_violation`, ordering/cadence abuse outcomes).
- Not-a-Bot `fail/replay/escalate` outcomes.
- Challenge failures weighted by severity.
- Existing ban reasons tied to policy/abuse routes.

### Strong human evidence (collateral protection weight)

- `not_a_bot_pass` outcomes.
- Puzzle challenge solved events (new event reason to add in implementation).
- Sampled allowed-flow bucket counters that did not trigger abuse/challenge outcomes (bounded, privacy-preserving bucket labels only).

### Scoring outputs per candidate

For each candidate CIDR:

- `bot_evidence_score`
- `human_evidence_score`
- `collateral_risk` = `human_score / (human_score + bot_score + epsilon)`
- `confidence` (function of evidence volume + signal consistency)
- `recommended_action` and `recommended_mode`

## Candidate Generation + Safer Bounds

1. Start from high-abuse buckets that meet minimum evidence thresholds.
2. Merge adjacent buckets into provisional parent CIDRs.
3. Evaluate parent collateral-risk.
4. If risk too high, split into child CIDRs and keep only low-collateral children.
5. Emit final suggestions sorted by confidence and expected operator value.

### Broadness guardrails (auto-suggest only)

- IPv4: do not auto-suggest broader than `/24`.
- IPv6: do not auto-suggest broader than `/48`.
- Minimum evidence gates (example initial values):
  - `>= 30` total observations in window,
  - `>= 8` weighted bot-evidence,
  - confidence `>= 0.60`.
- Suppress any suggestion where collateral-risk exceeds configured threshold band (initial target: >0.25 becomes "high risk").

Note: backend hard-validation constraints for user-entered CIDRs remain unchanged; these guardrails are for suggestion generation only.

## Suggested Action Policy

Action is advisory and editable by operator at apply time.

Default mapping (initial):

- Very high confidence + low collateral: `deny_temp` (temporary ban).
- High confidence + low-to-medium collateral: `tarpit` (narrow CIDR only; budget-guardrailed).
- High collateral: recommend `logging-only` mode and narrower alternative bounds (no enforce suggestion).

Implementation note: honeypot semantics are currently global in Shuma: a honeypot hit leads directly to immediate ban. There is no separate delayed honeypot lane today. For operator clarity, the suggestion surface should use `deny_temp` language and map it to the existing runtime action until naming is aligned.

This keeps action intent explicit, avoids semantic confusion, and preserves safe-first defaults.

## API Contract (proposed)

Add a dedicated endpoint for suggestions rather than overloading `/admin/monitoring`:

- `GET /admin/ip-range/suggestions?hours=24&limit=20`

Response shape (illustrative):

- `generated_at`
- `hours`
- `summary`:
  - `suggestions_total`
  - `low_risk`
  - `medium_risk`
  - `high_risk`
- `suggestions[]`:
  - `cidr`
  - `ip_family`
  - `bot_evidence_score`
  - `human_evidence_score`
  - `collateral_risk`
  - `confidence`
  - `recommended_action`
  - `recommended_mode` (`logging-only` or `enforce`)
  - `evidence_counts` (keyed map)
  - `safer_alternatives[]` (narrower CIDRs)
  - `guardrail_notes[]`

## Dashboard UX (proposed)

In IP Range Policy pane, add **Suggested Ranges (Last 24h)** above custom-rule JSON editor:

- Row fields: CIDR, confidence, collateral risk, bot vs human evidence, recommended action.
- Inline risk badge: `low`, `medium`, `high`.
- One-click controls:
  - `Add as logging-only rule`
  - `Add as enforce rule`
  - `Ignore suggestion`
- Expand row for evidence details and safer alternatives.
- Applying a suggestion appends a normalized custom-rule JSON object into the existing editor flow (preserves current save semantics and unsaved-change handling).

## Data/Resource Safety

- Use IP buckets only (no expansion of raw IP storage beyond existing event log behavior).
- Use bounded dimensions and fixed key families to prevent cardinality blowups.
- Prefer sampled baseline counters for likely-human denominator instead of full request logging.
- Add explicit read/write amplification checks in tests for suggestion computations.

## Testing Plan (implementation phase)

1. Unit tests:
   - evidence weighting and risk calculation,
   - merge/split candidate logic,
   - guardrail suppression behavior.
2. Integration tests:
   - synthetic event/counter fixtures -> deterministic suggestions,
   - API contract and sorting stability,
   - amplification budget assertions.
3. Dashboard unit/e2e:
   - rendering of suggestion rows,
   - apply-to-custom-rule wiring,
   - risk badge and safer-alternative display.

## Rollout Plan

1. Phase 1: API-only suggestions (read-only) + tests.
2. Phase 2: Dashboard suggestions pane with "apply" actions.
3. Phase 3: Tune thresholds from observed pre-launch traffic and adversarial smoke outputs.
4. Phase 4: Reassess whether additional signals are needed before GA.

## Atomic Implementation TODOs

### IRS-1: Define suggestion domain model + config defaults

Acceptance Criteria:

- Introduce typed runtime structs/enums for suggestion evidence, risk band, and recommendation output.
- Add canonical defaults (threshold bands, minimum evidence gates, guardrail widths) to `config/defaults.env` and runtime config loader.
- Config validation rejects out-of-range values with explicit error text.

Definition of Done:

- Unit tests for config parsing/validation pass.
- `make test-unit` passes.
- Docs mention new config keys and defaults.

### IRS-2: Add missing human-evidence telemetry hooks

Acceptance Criteria:

- Challenge solved outcomes are logged with a normalized reason suitable for suggestion scoring.
- Bounded likely-human baseline counter path is added using bucketed dimensions only.
- No new unbounded key families are introduced.

Definition of Done:

- Telemetry unit tests verify key format/cardinality guardrails.
- Monitoring retention/cleanup behavior remains unchanged.
- `make test-unit` passes.

### IRS-3: Build 24h evidence aggregation pipeline

Acceptance Criteria:

- Server-side aggregator reads bounded window data (`hours` query param, clamped).
- Aggregation produces per-bucket evidence totals for bot and human evidence.
- Aggregation is deterministic for fixed fixtures.

Definition of Done:

- Deterministic fixture-based tests validate output.
- Read amplification assertions are added for representative windows.
- `make test-integration` passes for new aggregation tests.

### IRS-4: Implement collateral-risk + confidence scoring

Acceptance Criteria:

- Bot and human evidence are weighted into `bot_evidence_score` and `human_evidence_score`.
- `collateral_risk` and `confidence` are computed and bounded to valid ranges.
- Risk band (`low`/`medium`/`high`) assignment uses configured thresholds.

Definition of Done:

- Unit tests cover edge cases: zero evidence, conflicting evidence, sparse evidence.
- Score math is documented in code comments for maintainability.
- `make test-unit` passes.

### IRS-5: Implement CIDR candidate merge/split + guardrails

Acceptance Criteria:

- Candidate builder merges adjacent buckets into provisional parent CIDRs.
- Guardrails enforce max auto-suggest width (IPv4 `/24`, IPv6 `/48` by default).
- High-collateral candidates are split into safer alternatives where possible.

Definition of Done:

- Unit tests cover merge-only, split-only, and mixed scenarios.
- Tests validate suppression when no safe candidate survives.
- `make test-unit` passes.

### IRS-6: Add admin endpoint for suggestions

Acceptance Criteria:

- `GET /admin/ip-range/suggestions?hours=<1-720>&limit=<1-50>` returns stable JSON contract.
- Endpoint is auth-protected and respects existing admin read-rate controls.
- Response includes summary counters, suggestions list, evidence counts, and safer alternatives.

Definition of Done:

- API tests cover auth, validation, and nominal responses.
- `docs/api.md` updated with endpoint contract and example.
- `make test-integration` passes.

### IRS-7: Wire dashboard API client + data adapters

Acceptance Criteria:

- Dashboard client exposes typed fetch method for suggestions endpoint.
- Adapter normalizes payload safely (defensive defaults for missing fields).
- Fetch lifecycle integrates with existing tab loading/error handling.

Definition of Done:

- Dashboard domain unit tests cover adapter normalization and edge payloads.
- `make test-dashboard-unit` passes.

### IRS-8: Add IP Range “Suggested Ranges (Last 24h)” UI section

Acceptance Criteria:

- New section renders CIDR, confidence, collateral risk, evidence split, and recommended action.
- Risk badges render deterministically (`low`/`medium`/`high`).
- Empty/loading/error states follow existing dashboard patterns.

Definition of Done:

- Dashboard unit tests cover render states and formatting.
- No design-language drift from existing dashboard components.
- `make test-dashboard-unit` passes.

### IRS-9: Implement one-click apply flow to custom rules

Acceptance Criteria:

- “Add as logging-only rule” and “Add as enforce rule” append normalized rule JSON into existing custom-rule editor state.
- Duplicate suppression prevents repeated insertion of the same candidate/rule id.
- Existing save bar/dirty-state and validation behavior remains intact.

Definition of Done:

- Unit + e2e tests cover apply actions and save integration.
- Regression tests confirm no breakage in existing custom-rule editing flow.
- `make test-dashboard-e2e` passes.

### IRS-10: Enforce action policy for suggestions (`deny_temp` + `tarpit`)

Acceptance Criteria:

- Suggestion recommender emits only `deny_temp`, `tarpit`, or `logging-only`.
- High-collateral candidates cannot return an enforce recommendation.
- Terminology in UI/docs avoids implying a separate honeypot lane for this feature.

Definition of Done:

- Unit tests assert action selection matrix behavior.
- Docs/runbook language updated for operator clarity.
- `make test-unit` and `make test-dashboard-unit` pass.

### IRS-11: Add end-to-end verification and amplification gates

Acceptance Criteria:

- Add integration fixtures and assertions for suggestion output stability.
- Add write/read amplification guardrails for suggestion pipeline.
- Add a focused Make target for suggestion validation (or extend existing adversarial target) without bypassing canonical `make test`.

Definition of Done:

- `make test` passes with new coverage included.
- CI path confirms no timeout regressions from suggestion computations.
- Threshold values are documented and justified in docs/plans.

### IRS-12: Runbook + operator tuning guidance

Acceptance Criteria:

- Update runbooks with interpretation workflow for risk bands and safer alternatives.
- Document rollback steps and emergency allowlist usage if false positives appear.
- Provide clear pre-launch tuning steps for thresholds and confidence gates.

Definition of Done:

- `docs/ip-range-policy-runbook.md` and related dashboard docs updated.
- Guidance includes concrete examples and “must/must not” phrasing.
- Documentation review completed in the same PR slice.

## Sequenced Execution Plan (Atomic Commits)

Execution rules:

1. One IRS task per commit.
2. Do not mix unrelated refactors into any IRS commit.
3. For commits that require integration/e2e verification, run `make dev` in a separate terminal first.

### Commit 1 (IRS-1)

Scope: suggestion model + config defaults + validation.

Likely files:

- `config/defaults.env`
- `src/config/mod.rs`
- `src/admin/api.rs`
- `src/config/tests.rs`
- `dashboard/src/lib/domain/config-schema.js` (if admin-writable keys are introduced)
- `docs/configuration.md`

Verification:

- `make test-unit`

Commit message:

- `feat(ip-range): add suggestion config/model defaults (IRS-1)`

### Commit 2 (IRS-2)

Scope: challenge-solved and likely-human telemetry hooks with bounded bucket keys.

Likely files:

- `src/runtime/request_router.rs`
- `src/observability/monitoring.rs`
- `src/observability/metrics.rs` (if counters exported)
- `src/admin/api.rs` tests (monitoring payload checks)

Verification:

- `make test-unit`

Commit message:

- `feat(ip-range): add bounded human-evidence telemetry hooks (IRS-2)`

### Commit 3 (IRS-3)

Scope: 24h evidence aggregation pipeline (deterministic, bounded reads).

Likely files:

- `src/signals/ip_range_suggestions.rs` (new)
- `src/signals/mod.rs` (wire new module)
- `src/admin/api.rs` (internal usage stubs or helpers)
- `src/*/tests.rs` fixtures for aggregation

Verification:

- `make test-unit`
- `make dev` (separate terminal)
- `make test-integration`

Commit message:

- `feat(ip-range): add 24h evidence aggregation pipeline (IRS-3)`

### Commit 4 (IRS-4)

Scope: collateral-risk/confidence scoring and risk-band assignment.

Likely files:

- `src/signals/ip_range_suggestions.rs`
- unit tests for scoring edge cases

Verification:

- `make test-unit`

Commit message:

- `feat(ip-range): implement collateral-risk scoring (IRS-4)`

### Commit 5 (IRS-5)

Scope: CIDR merge/split candidate generation and width guardrails.

Likely files:

- `src/signals/ip_range_suggestions.rs`
- candidate builder tests (merge/split/suppression)

Verification:

- `make test-unit`

Commit message:

- `feat(ip-range): add candidate merge-split guardrails (IRS-5)`

### Commit 6 (IRS-6)

Scope: authenticated admin suggestions endpoint and API docs.

Likely files:

- `src/admin/api.rs`
- `src/admin/api.rs` tests
- `docs/api.md`

Verification:

- `make test-unit`
- `make dev` (separate terminal)
- `make test-integration`

Commit message:

- `feat(admin): add ip-range suggestions endpoint (IRS-6)`

### Commit 7 (IRS-7)

Scope: dashboard API client method + payload adapters.

Likely files:

- `dashboard/src/lib/domain/api-client.js`
- `e2e/dashboard.modules.unit.test.js`

Verification:

- `make test-dashboard-unit`

Commit message:

- `feat(dashboard): wire ip-range suggestions client adapter (IRS-7)`

### Commit 8 (IRS-8)

Scope: new “Suggested Ranges (Last 24h)” UI section (read-only render).

Likely files:

- `dashboard/src/lib/components/dashboard/IpBansTab.svelte`
- `dashboard/src/lib/components/dashboard/monitoring/IpRangeSection.svelte` (if summary mirrors are added)
- `dashboard/src/lib/components/dashboard/primitives/*` (only if existing primitives insufficient)
- `e2e/dashboard.modules.unit.test.js`

Verification:

- `make test-dashboard-unit`

Commit message:

- `feat(dashboard): add suggested ranges panel (IRS-8)`

### Commit 9 (IRS-9)

Scope: one-click apply actions into existing custom-rule edit/save flow.

Likely files:

- `dashboard/src/lib/components/dashboard/IpBansTab.svelte`
- `dashboard/src/lib/domain/config-form-utils.js` (if helper extraction needed)
- `e2e/dashboard.modules.unit.test.js`
- dashboard e2e specs touching IP bans flow

Verification:

- `make test-dashboard-unit`
- `make dev` (separate terminal)
- `make test-dashboard-e2e`

Commit message:

- `feat(dashboard): add apply-suggestion rule actions (IRS-9)`

### Commit 10 (IRS-10)

Scope: recommender action constraints (`deny_temp`, `tarpit`, `logging-only`) and terminology alignment.

Likely files:

- `src/signals/ip_range_suggestions.rs`
- `dashboard/src/lib/components/dashboard/IpBansTab.svelte`
- `dashboard/src/lib/domain/ip-range-policy.js`
- `docs/ip-range-policy-runbook.md`

Verification:

- `make test-unit`
- `make test-dashboard-unit`

Commit message:

- `feat(ip-range): enforce suggestion action policy (IRS-10)`

### Commit 11 (IRS-11)

Scope: end-to-end verification + amplification gates + Make target wiring.

Likely files:

- `scripts/tests/*` (suggestion fixtures and assertions)
- `Makefile` (focused target wiring if needed)
- integration test scripts and assertions for bounded read/write amplification

Verification:

- `make dev` (separate terminal)
- `make test`

Commit message:

- `test(ip-range): add suggestion amplification and e2e gates (IRS-11)`

### Commit 12 (IRS-12)

Scope: runbook, dashboard tab docs, and operator tuning guidance.

Likely files:

- `docs/ip-range-policy-runbook.md`
- `docs/dashboard-tabs/ip-bans.md`
- `docs/configuration.md`
- `docs/value-proposition.md` (if operator burden/cost-shift language is updated)

Verification:

- Docs-only slice: no tests required.
- If any behavior/config code changed while updating docs, run `make test-unit` minimum.

Commit message:

- `docs(ip-range): add suggestion ops and tuning guidance (IRS-12)`

## Open Decisions To Confirm Before Implementation

1. Initial collateral-risk threshold bands (`low/medium/high`) exact cutoffs.
2. Whether to rename IP range action `honeypot` to `deny_temp`/`temporary_ban` in config/API/dashboard now (clean pre-launch break) versus keep internal action key and only relabel in UI/docs.
3. Sampling rate for likely-human baseline counters (if made tunable vs fixed).
