# IP Range Policy Runbook

Date: 2026-02-26  
Owner: Bot defence operations

## Scope

This runbook explains how to operate IP Range Policy safely and how to use the new suggestion workflow to shift range-discovery burden away from operators.

Settings covered:

- `ip_range_policy_mode`
- `ip_range_emergency_allowlist`
- `ip_range_custom_rules`
- `GET /admin/ip-range/suggestions?hours=&limit=`

## Modes and rule behavior

- `ip_range_policy_mode=off`
  - The system must not evaluate IP-range rules.
- `ip_range_policy_mode=advisory` (dashboard label: `logging-only`)
  - The system must evaluate rules and emit telemetry.
  - The system must not enforce actions.
- `ip_range_policy_mode=enforce`
  - The system must evaluate and enforce matching rule actions.

Rule order is deterministic:

1. Emergency allowlist check.
2. Custom rules from top to bottom.
3. First match wins.

## Suggestions workflow (Last 24h)

`/admin/ip-range/suggestions` returns candidate CIDRs with:

- bot evidence score,
- human evidence score,
- confidence,
- collateral risk band (`low`/`medium`/`high`),
- recommended action (`deny_temp`, `tarpit`, `logging-only`),
- safer alternatives when broad candidates are high collateral.

Operator rules:

- You must treat `high` collateral candidates as investigation-only unless safer alternatives are acceptable.
- You must start with `logging-only` apply for unfamiliar ranges.
- You must promote to enforce only after monitoring confirms acceptable collateral.
- You must use emergency allowlist first when legitimate traffic is impacted.

Implementation note:

- Suggestion action `deny_temp` maps to existing runtime action `honeypot` when added to custom rules (current pre-launch naming alignment path).

## Safe rollout sequence

1. Keep `ip_range_policy_mode=off` while preparing rule candidates.
2. Add candidate rules from suggestions as `logging-only` first.
3. Set global mode to `advisory` and observe impact for at least one traffic cycle.
4. Promote selected rules to enforce and switch mode to `enforce` only when collateral is acceptable.
5. Keep emergency allowlist entries for known-good ranges during rollout.

## False-positive response (must-do)

1. Add affected CIDR or IP to `ip_range_emergency_allowlist` immediately.
2. If impact is broad, set `ip_range_policy_mode=off`.
3. Remove, narrow, or demote the offending custom rule.
4. Return to `advisory` before re-enabling enforcement.
5. Keep incident notes with rule ID, matched CIDR, and observed collateral window.

## Rollback

1. Set `ip_range_policy_mode=off`.
2. Disable or remove recent custom-rule changes.
3. Keep temporary emergency allowlist entries until user impact is clear.
4. Re-enter `advisory` before any enforce re-rollout.

## Efficiency guardrails

- You must keep CIDRs as narrow as practical.
- You must not apply broad enforce rules without a low-collateral confidence signal.
- You should prefer `tarpit` over immediate deny when confidence is high but collateral is not definitively low.
