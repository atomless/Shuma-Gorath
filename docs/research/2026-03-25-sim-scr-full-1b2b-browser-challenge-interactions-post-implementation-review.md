Date: 2026-03-25
Status: Completed

Related context:

- [`2026-03-25-sim-scr-full-1b2b-browser-challenge-interactions-review.md`](2026-03-25-sim-scr-full-1b2b-browser-challenge-interactions-review.md)
- [`../plans/2026-03-25-sim-scr-full-1b2b-browser-challenge-interactions-plan.md`](../plans/2026-03-25-sim-scr-full-1b2b-browser-challenge-interactions-plan.md)

# What landed

`SIM-SCR-FULL-1B2B` is now complete.

The Scrapling worker now:

1. drives `not_a_bot_submit` through real browser-backed DOM interaction and records honest success receipts,
2. drives `puzzle_submit_or_escalation` through real browser-backed DOM interaction and records honest fail or escalation receipts,
3. keeps `pow_verify_abuse` and `tarpit_progress_abuse` on the existing request-native path in the same bounded `http_agent` run,
4. and aligns the owned-surface contract so those first DOM challenge surfaces now truthfully require `browser_or_stealth` transport.

# Why this was the right next slice

`SIM-SCR-FULL-1B2A` made the runtime executable, but the worker was still pushing the owned DOM challenges through direct request-native posts. That was no longer truthful enough once real browser sessions were available.

Landing the first browser-backed challenge tranche closes the first real full-power gap without overreaching into later browser-class surfaces that Scrapling does not yet need to own.

# Remaining gap

This slice does **not** yet claim the full Scrapling power track is complete.

The next tranche is still:

1. `SIM-SCR-FULL-1B3` to close the remaining full-power gaps left after this first browser-backed slice,
2. then `SIM-SCR-FULL-1C` to provide the fuller receipt-backed proof of which defenses Scrapling touched, passed where expected, failed where expected, and which categories and surfaces it exercised.
