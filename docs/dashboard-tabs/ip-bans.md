# Dashboard Tab: IP Bans

Route: `#ip-bans`  
Component: [`dashboard/src/lib/components/dashboard/IpBansTab.svelte`](../../dashboard/src/lib/components/dashboard/IpBansTab.svelte)

Purpose:

- Operate active bans and configure IP range policy and bypass allowlists.

Panels and behavior:

- Ban view filter (`All Active Bans` vs `IP Range Policy Only`).
- Active bans table with expandable details (signals, source, fallback/action metadata).
- Manual ban form (IP + duration tuple).
- Manual unban form.
- Suggested ranges panel (`Last 24h`):
  - Shows candidate CIDRs with confidence and collateral risk.
  - Shows recommended action/mode plus safer alternatives.
  - Supports one-click apply into custom rules (`Add as logging-only`, `Add as enforce`).
- IP Range Policy panel:
  - Mode: `off`, `logging-only` (runtime `advisory`), `enforce`.
  - Emergency allowlist: one CIDR per line.
  - Custom rules: one JSON object per line.
- Bypass Allowlists panel:
  - Enable toggle.
  - IP/CIDR allowlist textarea.
  - Path allowlist textarea.

Validation:

- Line-specific validation errors for CIDR and JSON-line inputs.
- `action` validation is constrained to the implemented action set.

Writes:

- Ban/unban APIs.
- `ip_range_*` policy keys.
- `bypass_allowlists_enabled`, `allowlist`, `path_allowlist`.

Suggestion endpoint used by this tab:

- `GET /admin/ip-range/suggestions?hours=24&limit=20`
