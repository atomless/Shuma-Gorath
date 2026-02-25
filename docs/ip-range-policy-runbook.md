# IP Range Policy Runbook

Date: 2026-02-20  
Owner: Bot defence operations

## Scope

This runbook explains how to operate IP Range Policy safely:

- how each setting works,
- how requests are matched,
- how to roll out without blocking legitimate traffic,
- how to recover quickly if a rule is wrong.

Settings covered:

- `ip_range_policy_mode`
- `ip_range_emergency_allowlist`
- `ip_range_custom_rules`
- `ip_range_managed_policies`
- `ip_range_managed_max_staleness_hours`
- `ip_range_allow_stale_managed_enforce`

Managed built-in sets currently supported:

- `openai_gptbot`
- `openai_oai_searchbot`
- `openai_chatgpt_user`
- `github_copilot`

DeepSeek managed sets are intentionally unavailable until an official machine-readable source exists.

## What each setting means

- `ip_range_policy_mode`
  - `off`: do not run IP range policy.
  - `advisory`: evaluate rules and record outcomes, but do not enforce actions.
  - `enforce`: apply the configured action for matching rules.
- `ip_range_emergency_allowlist`
  - CIDR ranges that bypass IP range actions.
  - Use this to quickly protect known-good traffic if a rule causes collateral impact.
- `ip_range_custom_rules`
  - Your own ordered rule list.
  - First matching rule wins.
- `ip_range_managed_policies`
  - Policy assignments for built-in managed IP sets.
  - Used after custom rules if no custom rule matched.
- `ip_range_managed_max_staleness_hours`
  - Maximum catalog age allowed for managed policy enforcement.
- `ip_range_allow_stale_managed_enforce`
  - `false` (recommended): stale managed sets do not enforce in `enforce` mode.
  - `true`: allow stale managed sets to keep enforcing as an explicit emergency override.

## How a request is decided

Decision order is fixed:

1. Emergency allowlist check.
2. Custom rules (top to bottom, first match wins).
3. Managed policies (if no custom match).
4. If mode is `advisory`, log/observe only.
5. If mode is `enforce`, run the matched action.

## Safe rollout sequence

1. Start with `ip_range_policy_mode=off` while you build rules.
2. Switch to `advisory` and monitor `ip_range_policy_advisory` outcomes.
3. Confirm no meaningful false positives on legitimate traffic.
4. Move to `enforce` only after advisory data is clean.
5. Prefer lower-friction actions first (`rate_limit` or `maze`) before hard blocking where possible.

## If you hit false positives

1. Add affected ranges to `ip_range_emergency_allowlist` immediately.
2. Keep that allowlist entry in place while investigating.
3. If impact is broad or unclear, set `ip_range_policy_mode=off` to stop enforcement globally.
4. Narrow, disable, or remove the offending custom/managed policy.
5. Return to `advisory` before re-entering `enforce`.

## Rollback procedure

1. Set `ip_range_policy_mode=off`.
2. Disable affected rules.
3. Add temporary emergency allowlist entries for known-good traffic.
4. Re-enable only in `advisory` until validated.

## Managed catalog refresh and staleness

Refresh managed catalog from official sources:

```bash
make ip-range-catalog-update
```

Refresh guardrails include:

- HTTPS-only sources on a strict host allowlist.
- Source schema validation.
- CIDR validation and broad-prefix rejection.
- Per-set entry caps and growth-delta guards.

Recommended operations:

- Refresh at least daily.
- Keep `ip_range_allow_stale_managed_enforce=false` by default.
- If freshness cannot be restored quickly, degrade mode to `advisory` or `off`.

## Efficiency and safety controls

- Keep CIDR lists precise; avoid very broad ranges.
- Keep policy lists focused and bounded.
- Use `advisory` when testing broad coverage changes.
- Treat managed catalog updates as controlled changes and review diffs before rollout.

Never ingest unofficial, scraped, or user-submitted list sources directly into managed sets.
