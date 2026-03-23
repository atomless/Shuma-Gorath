Date: 2026-03-23
Status: Ready for implementation

Related context:

- [`../research/2026-03-23-ban-duration-family-truthfulness-review.md`](../research/2026-03-23-ban-duration-family-truthfulness-review.md)
- [`../configuration.md`](../configuration.md)
- [`../dashboard-tabs/policy.md`](../dashboard-tabs/policy.md)

# Goal

Make ban-duration configuration and the Policy tab truthful by aligning them to every shipped ban-producing family.

# Scope

In scope:

1. config/defaults, seeded KV config, and admin config patch parity for all shipped ban families
2. runtime duration resolution for every shipped ban family
3. Policy tab duration controls for every shipped ban family
4. operator-facing docs and focused verification

Out of scope:

1. ban jitter or recidive
2. changing which request outcomes ban vs block
3. inventing new ban families that do not exist in the runtime yet

# Canonical family set for this tranche

1. `honeypot`
2. `ip_range_honeypot`
3. `maze_crawler`
4. `rate_limit`
5. `cdp`
6. `edge_fingerprint`
7. `tarpit_persistence`
8. `not_a_bot_abuse`
9. `challenge_puzzle_abuse`
10. `admin`

Notes:

1. `cdp` remains the config-family name for the `cdp_automation` runtime reason.
2. `edge_fingerprint` is the config-family name for the `edge_fingerprint_automation` runtime reason.
3. `admin` is the config-family name for the `manual_ban` runtime reason.

# Implementation slices

## BAN-DUR-1.1 Contract and failing coverage

Add or extend focused tests to fail until:

1. `BanDurations` exposes the full canonical family set
2. admin config patch shape accepts the same family set
3. Advanced JSON parity includes the same family set
4. Policy tab renders one row for each family
5. key runtime ban paths no longer reuse mismatched buckets or hardcoded short-ban constants

Verification target:

1. add a focused `make` path for ban-duration contract and dashboard parity if one does not already exist

## BAN-DUR-1.2 Runtime and config parity

Implement the new config fields through:

1. `src/config/mod.rs`
2. `config/defaults.env`
3. `scripts/config_seed.sh`
4. `src/admin/api.rs`
5. `dashboard/src/lib/domain/config-schema.js`
6. `dashboard/static/assets/status-var-meanings.json`

Acceptance:

1. config load/default/seed/export/admin-write/admin-validate parity is preserved
2. manual admin default comes from config when duration is omitted

## BAN-DUR-1.3 Policy tab truthfulness

Update the Policy tab duration pane to expose the full family set with accurate labels and help text.

Acceptance:

1. no row label implies a different family than the one it edits
2. save payload writes the full family set
3. existing shared input-row styling is reused

## BAN-DUR-1.4 Docs and review

Update:

1. `docs/configuration.md`
2. `docs/api.md`
3. `docs/dashboard-tabs/policy.md`
4. `docs/testing.md`

Then perform a post-implementation review and capture any shortfall immediately.
