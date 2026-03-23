# PoW Carry-Forward Plan

Date: 2026-02-25
Status: Active (carry-forward)
Supersedes: Historical baseline in [`docs/plans/2026-02-13-pow-excellence-plan.md`](2026-02-13-pow-excellence-plan.md)

## Scope

This plan captures remaining PoW work after partial delivery of the 2026-02-13 plan.

Delivered baseline already includes:
- Signed PoW seeds with expiry.
- Replay/tamper checks and sequence/binding/timing validation in verify path.
- Core PoW failure taxonomy and baseline verification tests.
- Config bounds for current static difficulty and TTL.

## Remaining Work

1. POW-1: Implement bounded adaptive difficulty.
   - Difficulty should respond to risk and abuse pressure within strict min/max guardrails.

2. POW-3: Add per-bucket attempt caps and cooldown windows for PoW verify.
   - Keep retry cost mostly on attackers while bounding host verification pressure.

3. POW-4: Add optional Web Worker solve path.
   - Move heavier solve loops off the main thread for constrained-device safety.

4. POW-5: Add verifier latency and stage-outcome metrics.
   - Add low-cardinality telemetry for solve/verify latency and failure-stage classification.

5. POW-6: Finalize PoW failure escalation matrix.
   - Make failure outcomes explicit by reason and confidence (retry, challenge, maze, tarpit, block).

6. POW-9: Add Akamai-aware trigger hooks.
   - Use Akamai evidence to influence when PoW is required, without replacing local PoW policy controls.

7. POW-10: Publish rollout thresholds and rollback runbook.

## Definition of Done

- Adaptive difficulty is deployed with bounded guardrails and test coverage.
- Attempt/cooldown protections are enforced and observable.
- Worker mode and escalation matrix are explicitly documented and verified.
