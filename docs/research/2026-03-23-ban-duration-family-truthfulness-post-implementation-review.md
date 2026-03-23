# BAN-DUR-1 Post-Implementation Review

Date: 2026-03-23

## Scope reviewed

- `src/config/mod.rs`
- `src/runtime/effect_intents/plan_builder.rs`
- `src/runtime/request_router.rs`
- `src/lib.rs`
- `src/providers/internal.rs`
- `src/providers/external.rs`
- `src/admin/api.rs`
- `dashboard/src/lib/components/dashboard/RobotsTab.svelte`
- `dashboard/src/lib/components/dashboard/config/ConfigDurationsSection.svelte`
- `dashboard/src/lib/domain/config-schema.js`
- `dashboard/static/assets/status-var-meanings.json`

## Planned outcome

`BAN-DUR-1` required four things:

1. truthful coverage for the full shipped ban-family set,
2. canonical config parity for every shipped ban-producing family,
3. one operator-visible Policy-tab duration row per shipped family,
4. docs and review closure.

## Delivered outcome

The shipped `ban_durations` family now covers every currently shipped ban-producing family that actually routes through a ban:

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

Runtime sites that previously reused a neighboring bucket or a hardcoded short-ban constant now resolve through the matching family key. The Policy tab renders and saves one duration row for each of those families, and the manual admin ban default now resolves through `ban_durations.admin` instead of a duplicated hardcoded fallback.

## Review findings

### Fixed during tranche

1. `manual_ban` still defaulted to `21600` seconds when the caller omitted `duration`.
   - Fixed by routing the default through `cfg.get_ban_duration("admin")`.

2. The Policy tab still only bound the legacy four duration controls.
   - Fixed by expanding the tab baseline, dirty-state, save payload, and rendered pane to the full shipped family set.

3. Local shared-host boot surfaced a stale persisted KV document after the new config keys landed.
   - Fixed operationally by updating the canonical defaults and `scripts/config_seed.sh`, then proving the backfill path with `make config-seed`.

4. The first rendered Playwright proof used overly broad text locators that collided with unrelated page content.
   - Fixed by tightening the proof to the Policy-tab input ids instead of page-wide text matches.

### Remaining open shortfall

None found for `BAN-DUR-1`.

## Verification

- `make test-ban-duration-family-truth`
- `make config-seed`
- `make test-dashboard-e2e-ban-duration-family-truth`
- `git diff --check`

## Conclusion

`BAN-DUR-1` is complete. The config contract, runtime routing, Policy-tab control surface, and operator docs are now aligned for the shipped ban-producing family set, with no tranche-local shortfall left open.
