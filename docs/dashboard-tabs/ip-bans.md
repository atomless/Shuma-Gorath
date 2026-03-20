# Dashboard Tab: IP Bans

Route: `#ip-bans`  
Component: [`dashboard/src/lib/components/dashboard/IpBansTab.svelte`](../../dashboard/src/lib/components/dashboard/IpBansTab.svelte)

Purpose:

- Operate active bans and configure IP range policy and bypass allowlists.
- Keep contributor diagnostics off the main ban-operations surface; low-level freshness/transport/raw-feed diagnostics now live in Diagnostics' collapsed `Telemetry Diagnostics` section instead.
- Show the real active ban state for the site, including manual interventions, even though Monitoring excludes operator-originated actions from its external-traffic telemetry.

Panels and behavior:

- Ban view filter (`All Active Bans` vs `IP Range Policy Only`).
- Active bans table with expandable details (signals, source, fallback/action metadata).
- Manual ban form (IP + duration tuple).
- Manual unban form.
- Suggested ranges panel (`Last 24h`):
  - Shows candidate CIDRs with confidence and collateral risk.
  - Shows recommended action/mode plus safer alternatives.
  - Suggestion evidence is derived from external traffic telemetry, not operator-originated admin actions.
  - Supports one-click apply into custom rules (`Add as logging-only`, `Add as enforce`).
- IP Range Policy panel:
  - Mode: `off`, `logging-only` (runtime `advisory`), `enforce`.
  - Emergency allowlist: one CIDR per line.
  - Custom rules: one JSON object per line.
- Bypass Allowlists panel:
  - Enable toggle.
  - IP/CIDR allowlist textarea.

Validation:

- Line-specific validation errors for CIDR and JSON-line inputs.
- `action` validation is constrained to the implemented action set.

Writes:

- Ban/unban APIs.
- `ip_range_*` policy keys.
- `bypass_allowlists_enabled`, `allowlist`.

Suggestion endpoint used by this tab:

- `GET /admin/ip-range/suggestions?hours=24&limit=20`
