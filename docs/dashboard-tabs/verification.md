# Dashboard Tab: Verification

Route: `#verification`  
Component: [`dashboard/src/lib/components/dashboard/VerificationTab.svelte`](../../dashboard/src/lib/components/dashboard/VerificationTab.svelte)

Purpose:

- Configure core verification and challenge controls.

Panels:

- `JS Required` toggle (`js_required_enforced`).
- `Browser CDP Automation Probe`:
  - enable toggle (`cdp_detection_enabled`),
  - auto-ban toggle (`cdp_auto_ban`),
  - threshold slider (`cdp_detection_threshold`).
  - CDP controls are disabled when JS Required is off.
- `Proof-of-Work (PoW)` toggle (`pow_enabled`).
- `Challenge: Not-a-Bot`:
  - enable toggle (`not_a_bot_enabled`),
  - pass score (`not_a_bot_pass_score`),
  - fail score (`not_a_bot_fail_score`).
  - UI enforces `fail < pass` and `pass > fail`.
- `Challenge: Puzzle` toggle (`challenge_puzzle_enabled`).
- `Browser Policy`:
  - toggle (`browser_policy_enabled`),
  - minimum-version rules (`browser_block`).

Notes:

- Challenge and PoW advanced controls remain in [`advanced.md`](advanced.md).
- This tab no longer includes IP range policy controls (those are in [`ip-bans.md`](ip-bans.md)).
