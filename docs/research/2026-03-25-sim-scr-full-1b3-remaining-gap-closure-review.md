Date: 2026-03-25
Status: Completed

# `SIM-SCR-FULL-1B3` Remaining Gap Closure Review

## Context

`SIM-SCR-FULL-1B2B` landed the first real browser-backed Scrapling challenge interactions for the currently owned DOM challenge surfaces:

1. `not_a_bot_submit`,
2. `puzzle_submit_or_escalation`,
3. while `pow_verify_abuse` and `tarpit_progress_abuse` remained request-native in the same bounded `http_agent` path.

That means the next question is no longer "can Scrapling execute owned browser-backed challenge paths at all?" It is "what still remains in-scope for the current Scrapling-owned surface set, and what is now an explicit omission or separate-lane question?"

## Findings

1. No additional currently owned surface now requires another immediate browser or stealth implementation slice.
   - The owned surface contract in `src/observability/scrapling_owned_surface.rs` now truthfully places:
     - `not_a_bot_submit` on `browser_or_stealth`,
     - `puzzle_submit_or_escalation` on `browser_or_stealth`,
     - and keeps `pow_verify_abuse` plus `tarpit_progress_abuse` request-native.
   - After `SIM-SCR-FULL-1B2B`, each of those owned surfaces is now exercised by the worker and protected by focused proof.

2. The most obvious remaining upstream Scrapling powers are not presently missing owned-surface behaviors.
   - `real_chrome`, `cdp_url`, and explicit persistent profile controls are available upstream, but the current owned surface set does not yet include a ratified surface whose truthful proof depends on them.
   - Proxy or origin-distribution support remains the one explicit temporary shared-host exclusion already recorded in the full-power matrix.

3. `solve_cloudflare` is an explicit non-gap for the current Shuma-owned surfaces.
   - Upstream support demonstrates capability class, but Shuma's current browser-backed owned surfaces are internal app DOM challenges, not Cloudflare interstitials.
   - Keeping `solve_cloudflare=False` on the internal stealth browser challenge path is therefore a truthful omission, not a missing implementation.

4. The remaining browser-class surfaces are still separate-lane questions, not hidden Scrapling-owned gaps.
   - `maze_navigation`
   - `js_verification_execution`
   - `browser_automation_detection`
   remain outside current Scrapling ownership in the owned-surface matrix.
   - `automated_browser` remains a separate later category-ownership question.

## Conclusion

`SIM-SCR-FULL-1B3` should close as a truth and omission-ledger tranche, not as another behavior implementation tranche.

The current shared-host mainline can now say:

1. the still-ratified Scrapling-owned surfaces have their required request-native vs browser-backed behaviors implemented,
2. the remaining upstream Scrapling powers are either:
   - explicit temporary exclusions,
   - not applicable to current Shuma-owned surfaces,
   - or separate-lane ownership questions,
3. and the next meaningful mainline step is `SIM-SCR-FULL-1C`, which should prove the full lane more explicitly with receipt-backed touched/passed/failed/category evidence.
