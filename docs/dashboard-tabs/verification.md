# Dashboard Tab: Verification

Route: `#verification`  
Component: [`dashboard/src/lib/components/dashboard/VerificationTab.svelte`](../../dashboard/src/lib/components/dashboard/VerificationTab.svelte)

Purpose:

- Configure core verification and challenge controls.
- Configure trusted verification-source posture.
- Review bounded verified-identity health without leaving the Verification tab.

Panels:

- `Akamai Bot Signal`:
  - enable toggle (`provider_backends.fingerprint_signal` internal/external),
  - influence mode selector (`edge_integration_mode`: `additive` or `authoritative`),
  - current additive scored contribution (`fp_akamai_edge_additive`) when Akamai edge posture controls are available.
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
- `Verified Identity`:
  - enable toggle (`verified_identity.enabled`),
  - native Web Bot Auth toggle (`verified_identity.native_web_bot_auth_enabled`),
  - provider assertions toggle (`verified_identity.provider_assertions_enabled`),
  - replay window (`verified_identity.replay_window_seconds`),
  - clock skew (`verified_identity.clock_skew_seconds`),
  - directory cache TTL (`verified_identity.directory_cache_ttl_seconds`),
  - directory freshness requirement (`verified_identity.directory_freshness_requirement_seconds`).
- `Verified Identity Health`:
  - availability,
  - attempts,
  - verified,
  - failed,
  - unique identities,
  - named policy and service-profile counts,
  - top failure reasons,
  - top schemes,
  - top categories.

Notes:

- Challenge and PoW advanced controls remain in [`advanced.md`](advanced.md).
- This tab no longer includes IP range policy controls (those are in [`ip-bans.md`](ip-bans.md)).
- This pane now owns verification mechanics plus trusted verification-source posture. Verified-identity category defaults, richer identity policy editing, and bounded tuning controls still stay out of this tab.
