Date: 2026-03-23
Status: Informing implementation

Related context:

- [`../plans/2026-03-23-ban-duration-family-truthfulness-implementation-plan.md`](../plans/2026-03-23-ban-duration-family-truthfulness-implementation-plan.md)
- [`../plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](../plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)
- [`../configuration.md`](../configuration.md)
- [`../dashboard-tabs/policy.md`](../dashboard-tabs/policy.md)

# Objective

Make the `Ban Durations` operator surface truthful: every currently shipped ban-producing family must have a configurable duration and must appear in the Policy tab.

# Findings

## Current operator contract is incomplete

The current dashboard pane exposes only four duration families:

1. `honeypot`
2. `rate_limit`
3. `cdp`
4. `admin`

That does not match the shipped runtime.

## Current shipped ban-producing families

The current codebase issues bans for the following distinct families:

1. `honeypot`
2. `ip_range_honeypot`
3. `maze_crawler`
4. `rate_limit`
5. `cdp_automation`
6. `edge_fingerprint_automation`
7. `tarpit_persistence`
8. `not_a_bot_abuse`
9. `challenge_puzzle_abuse`
10. `manual_admin_ban`

These are grounded in the current runtime call sites:

1. policy-graph/effect-intent bans for `honeypot`, `ip_range_honeypot`, and `rate_limit`
2. maze threshold auto-ban for `maze_crawler`
3. native CDP auto-ban for `cdp_automation`
4. authoritative edge-fingerprint auto-ban for `edge_fingerprint_automation`
5. tarpit persistence escalation short-ban for `tarpit_persistence`
6. challenge abuse short-ban paths for `not_a_bot_abuse` and `challenge_puzzle_abuse`
7. admin manual ban endpoint for `manual_ban`

## Current mismatches

1. `maze_crawler` currently reuses the `honeypot` duration bucket.
2. `ip_range_honeypot` currently reuses the `honeypot` duration bucket.
3. `edge_fingerprint_automation` currently reuses the `cdp` duration bucket.
4. `tarpit_persistence` uses a fixed short-ban constant instead of config.
5. `not_a_bot_abuse` uses a fixed short-ban constant instead of config.
6. `challenge_puzzle_abuse` uses a fixed short-ban constant instead of config.
7. manual admin ban defaults are split: the Policy tab exposes `ban_durations.admin`, but `POST /admin/ban` still defaults to a hardcoded `21600` when duration is omitted.
8. the Policy tab label `Maze Threshold Exceeded` is currently wired to the `honeypot` bucket, which is actively misleading.

# Design decisions

## 1. Ban duration families should follow shipped runtime families

The operator surface should reflect the actual ban families the runtime can issue today, not a smaller bucket abstraction.

## 2. One row per ban-producing family

For this tranche, the pane should expose one duration row for each shipped family above.

That keeps the contract truthful without inventing a second hidden mapping layer.

## 3. Challenge abuse remains grouped by runtime family, not by every sub-outcome

Multiple challenge-abuse outcomes currently collapse into two enforced ban families:

1. `not_a_bot_abuse`
2. `challenge_puzzle_abuse`

The pane should expose those two families directly rather than a row for every replay, binding, or timing violation subtype.

## 4. Manual admin default must actually drive manual bans

The `admin` duration row is only truthful if `POST /admin/ban` uses that configured default when the caller omits an explicit duration.

## 5. Legacy fallback remains legacy

`ban_duration` should remain available as the legacy catch-all fallback, but the operator-facing pane should not imply that it configures any specific shipped family directly.

# Acceptance criteria

1. Every shipped ban-producing family has a dedicated `ban_durations.*` config key.
2. Every ban-producing call site resolves its duration through the matching family key.
3. `GET /admin/config`, `POST /admin/config`, config export, defaults seeding, Advanced JSON parity, and Policy tab all expose the same family set.
4. `POST /admin/ban` uses the configured manual-admin default when duration is omitted.
5. Policy tab labels map one-to-one to the shipped runtime families.
