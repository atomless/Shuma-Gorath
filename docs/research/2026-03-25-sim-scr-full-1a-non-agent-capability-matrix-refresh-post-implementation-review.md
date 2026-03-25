Date: 2026-03-25
Status: Completed

Related context:

- [`2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-review.md`](2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-review.md)
- [`../plans/2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-plan.md`](../plans/2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-plan.md)

# What landed

`SIM-SCR-FULL-1A` is now complete as a docs-first tranche.

The repo now explicitly says:

1. the earlier `SIM-SCR-CAP-1` matrix was a truthful request-native baseline,
2. it is no longer the live maturity target for Scrapling,
3. dynamic and stealth Scrapling capability for Scrapling-owned surfaces is in-scope by default where the refreshed matrix says it is needed,
4. `SIM-SCR-FULL-1B` is the next active implementation slice,
5. and `SIM-SCR-BROWSER-1` remains only the later category-ownership question for `automated_browser`.

# Why it mattered

Without this refresh, the planning chain still carried an older assumption that most browser or stealth Scrapling capability was effectively assigned away unless a later gap proved otherwise.

That was out of line with the now-settled attacker-faithfulness principle and with the stricter human-only game-loop gate that depends on full-power Scrapling before the loop is treated as operationally proven.

# Remaining follow-on

This tranche intentionally did not change code.

The next slice is `SIM-SCR-FULL-1B`, which must implement the remaining dynamic or stealth capability the refreshed matrix assigns to the Scrapling-owned non-agent remit, then `SIM-SCR-FULL-1C` must prove it with receipts before `RSI-GAME-HO-1` reopens.
