# RSI-GAME-ARCH-1G Live Protected Evidence Post-Implementation Review

Date: 2026-03-27  
Status: implemented

## Scope

Closed `RSI-GAME-ARCH-1G`: make strong live Scrapling runtime board evidence eligible as protected tuning evidence without reopening simulator-label leakage or weakening the synthetic or advisory safety gates.

## What Landed

1. `benchmark_results_v1` now materializes an explicit `protected_evidence` summary alongside the existing `replay_promotion` provenance summary.
2. The effective protected basis now prefers strong live Scrapling runtime proof when the current board-state evidence is localized, shared-path, high-confidence, and reproduced across the recent window.
3. `tuning_eligibility` now gates on `protected_evidence` rather than on replay lineage alone.
4. Oversight patch shaping no longer drags replay-review requirements into bounded moves when the protected basis is live Scrapling runtime rather than replay-promoted lineage.
5. The Game Loop now projects `Protected Evidence` and `Replay Lineage` separately, and trust blockers now follow the effective protected-evidence blockers instead of replay-only blockers.

## Why This Matters

Before this slice, the live Scrapling loop could show strong repeated board-state pressure and still remain blocked purely because replay lineage had not been materialized. That made the loop read as less operational than it really was and prevented bounded config tuning from responding to the strongest currently observed live attacker evidence.

After this slice:

1. replay lineage remains an important provenance surface,
2. but it is no longer the only route by which the controller can regard evidence as protected,
3. and strong live Scrapling runtime proof can now unlock bounded tuning without smuggling simulator metadata into runtime or restriction scoring.

## Safety And Purity Checks

1. Simulator-known category or persona labels still do not enter runtime defenses or restriction tuning.
2. `synthetic_traffic` remains ineligible as protected tuning evidence.
3. Raw frontier or LLM discovery remains advisory until replay-promoted or equivalently confirmed elsewhere.
4. Replay lineage remains visible as provenance rather than being silently overwritten by the effective protected-evidence rail.

## Verification

- `make test-protected-tuning-evidence`
- `make test-benchmark-results-contract`
- `make test-rsi-score-move-selection`
- `make test-dashboard-game-loop-accountability`
- local live payload check on `GET /admin/benchmark-results` now shows:
  - `protected_evidence.protected_basis=live_scrapling_runtime`
  - `protected_evidence.tuning_eligible=true`
  - and `tuning_eligibility.blockers` no longer include `protected_lineage_missing` or `protected_tuning_evidence_not_ready` when strong live Scrapling runtime proof is present

## Remaining Follow-On

1. `RSI-GAME-ARCH-1E`
   - retire or demote the remaining replaced category-first surfaces only after full-path replacement proof.
2. Continue the Game Loop board-state mainline toward cleaner restriction-first controller and UI surfaces.
