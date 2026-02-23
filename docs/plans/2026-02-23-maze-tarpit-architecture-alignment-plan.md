# Maze/Tarpit Architecture Alignment Plan

Date: 2026-02-23  
Status: Proposed

Related:

- `docs/module-boundaries.md`
- `docs/adr/0004-tarpit-v2-progression-contract.md`
- `docs/plans/2026-02-23-tarpit-v2-progression-envelope.md`
- `todos/todo.md` (`TAH-ARCH-*`, `TAH-*`)

## Why now (pre-launch window)

Maze and tarpit currently work, but tarpit runtime logic is still concentrated in provider internals and older `maze_plus_drip` behavior paths. Before finishing tarpit v2, this is the best point to cleanly separate responsibilities and avoid locking in coupling.

## Current architecture pressure points

1. Provider-layer coupling:
   - `src/providers/internal.rs` still contains substantial tarpit runtime logic instead of being a thin adapter.
2. Mixed route ownership:
   - Maze and challenge endpoints have clearer domain ownership than tarpit progression paths (which are still pending).
3. Shared primitive drift risk:
   - Maze already has token/replay/chain/budget helpers; tarpit v2 could duplicate them unless extracted deliberately.
4. Legacy config model overlap:
   - Existing tarpit knobs were designed for static drip mode and do not map cleanly to progression-gated v2 behavior.

## Target architecture

1. Keep provider contracts as adapters:
   - provider interface remains stable (`MazeTarpitProvider`), runtime logic moves to domain modules.
2. Establish explicit tarpit domain modules:
   - `src/tarpit/http.rs` for progression endpoint handling,
   - `src/tarpit/runtime.rs` for orchestration,
   - `src/tarpit/proof.rs` for server-verified work gate,
   - `src/tarpit/types.rs` for progression outcomes/reasons.
3. Extract shared maze+tarpit primitives once:
   - shared token-chain/replay/budget helpers in a common module consumed by both maze and tarpit.
4. Align config with v2 semantics:
   - keep main-pane exposure minimal (`tarpit_enabled`),
   - move v2 tuning to Advanced JSON,
   - preserve strict env-only safety ceilings.

## Execution sequence

1. Architecture refactor tranche (`TAH-ARCH-1`..`TAH-ARCH-6`).
2. Core tarpit v2 runtime tranche (`TAH-3`, `TAH-4`, `TAH-6`, `TAH-7`).
3. Budget lifecycle tranche (`TAH-8`, `TAH-9`, `TAH-10`).
4. Observability and operator tranche (`TAH-11`, `TAH-12`, `TAH-15`).
5. Test/CI tranche (`TAH-13`, `TAH-14`).
6. Fingerprint-resistance and safety tranche (`TAH-16`, `TAH-17`, `TAH-18`).

## Guardrails

- No new third-party runtime dependencies.
- No separate tarpit-only token format if shared primitives can express the same guarantees.
- No broadening of main-pane tarpit controls beyond `tarpit_enabled`.
- Keep deterministic fallback (`maze`/`block`) and short-ban escalation semantics explicit.
