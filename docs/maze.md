# 🐙 Maze

The maze is Shuma-Gorath's deception subsystem: a synthetic crawl space designed to absorb suspicious automation while keeping normal human traffic friction low.

## 🐙 Maze Excellence Mission

Maze excellence is about asymmetry:

- increase attacker cost (time, traversal effort, compute, and bandwidth),
- keep defender cost bounded and energy-aware,
- preserve human UX and avoid SEO regressions,
- maintain operator control and explainable policy outcomes.

This is the core stance for `L7_DECEPTION_EXPLICIT` behavior and related deception flows.

## 🐙 Guiding Aims and Principles

1. **Asymmetric cost placement**
   Most incremental cost should land on malicious visitors, not on host infrastructure.
2. **Adaptive depth by confidence**
   Low-suspicion traffic gets minimal or no maze friction; high-suspicion traffic gets deeper deception.
3. **Anti-fingerprinting by design**
   Avoid globally stable decoys by using rotating entropy, signed traversal tokens, and variant families.
4. **Bounded host budgets**
   Enforce hard limits (concurrency, bytes, time, per-bucket spend) with deterministic fallback.
5. **Progressive escalation**
   Maze is one control in a ladder, not a silo; saturated or bypassed maze paths escalate predictably.
6. **Observability first**
   Every major maze decision should remain measurable and explainable in metrics/events.

## 🐙 How Maze Pages Are Populated

### Current Baseline

- Requests to maze paths return synthetic pages linking to additional maze pages.
- Each maze hit increments counters for telemetry and threshold actions.
- If configured, repeated maze traversal can trigger auto-ban.

### Maze Excellence Population Model (Planned)

1. **Variant template families**
   Prebuilt layout/content families are selected per request window to reduce stable fingerprints.
2. **Signed, rotating seed selection**
   Variant choice is derived from short-lived signed entropy, not path-only deterministic hash.
3. **Graph shaping**
   Link graph depth/branching is tuned by suspicion tier and active budget limits.
4. **Signed traversal primitives**
   Maze links include signed tokens with TTL/depth/replay controls.
5. **Progress checkpoints**
   Suspicious-tier traversal uses checkpointed progress validation to curb blind prefetch crawling.
6. **Deterministic fallback**
   If budgets or proof checks fail, policy falls back to lower-cost controls (`challenge`/`block`) per escalation config.

### Content Sources and Safety Rules

- Use synthetic decoy corpora and neutral template content only.
- Do not mirror user/private application data into maze pages.
- Keep generated content disposable and non-authoritative.
- Preserve explicit robots signaling for trap routes and honeypot paths.

## 🐙 Signal Inputs That Shape Maze Behavior

Maze complexity/routing can consume:

- local signals (rate, geo posture, challenge outcomes, JS/CDP observations),
- traversal signals (ordering windows, timing thresholds, replay checks),
- trusted upstream enterprise signals (for example edge-provided bot outcomes) when configured.

Signal collection informs policy; the maze remains Shuma-controlled policy composition by default.

## 🐙 Configuration

These fields are part of the runtime config (`/admin/config`):

- `maze_enabled` (bool) - Enable or disable the maze.
- `maze_auto_ban` (bool) - Auto-ban after threshold.
- `maze_auto_ban_threshold` (u32) - Number of maze pages before auto-ban.

## 🐙 Admin Endpoint

- `GET /admin/maze` - Returns maze stats for the dashboard.

## 🐙 Metrics

- `bot_defence_maze_hits_total` tracks total maze page hits.

## 🐙 Notes

- If you do not want crawler trapping, set `maze_enabled` to `false`.
- Auto-ban uses the `maze_crawler` reason in metrics and events.
- For deeper implementation detail, see `docs/plans/2026-02-13-maze-excellence-plan.md`.
