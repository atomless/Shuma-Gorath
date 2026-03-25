Date: 2026-03-25
Status: Active

# `SIM-SCR-FULL-1C` Receipt Proof Review

## Context

`SIM-SCR-FULL-1B2B` changed the full-power Scrapling contract materially:

1. `not_a_bot_submit` is now browser-backed and should be able to pass,
2. `puzzle_submit_or_escalation` remains browser-backed and should fail honestly,
3. and `SIM-SCR-FULL-1B3` has now closed the remaining upstream power questions as explicit omissions or separate-lane issues.

That makes `SIM-SCR-FULL-1C` the next active mainline slice: prove the resulting lane more explicitly with receipt-backed touched/passed/failed/category evidence.

## Findings

1. The backend recent-run proof is stale against the new full-power contract.
   - `make test-adversary-sim-scrapling-coverage-receipts` now fails because `src/admin/api.rs` still seeds `not_a_bot_submit` as `fail_observed`, which no longer satisfies the current owned-surface contract.

2. The dashboard evidence proof still reflects the older contract shape.
   - The focused dashboard fixture still models `not_a_bot_submit` as `should_fail` / `fail_observed`.
   - The tests pass because they only prove the panel renders receipt rows, not that it matches the current full-power contract.

3. Red Team already has most of the raw evidence, but the operator-level summary is still too implicit.
   - The panel shows:
     - modes,
     - categories,
     - satisfied vs required counts,
     - per-surface contract and observed rows.
   - But it does not yet summarize:
     - how many owned surfaces were actually exercised,
     - how many expected passes were observed,
     - how many expected fails were observed.

4. Game Loop corroboration is still minimal.
   - It shows coverage status, satisfied vs required surfaces, and observed categories.
   - It does not yet echo the explicit pass/fail proof in a compact way.

## Conclusion

`SIM-SCR-FULL-1C` should:

1. refresh the stale backend recent-run receipt proof to the current browser-backed contract,
2. make Red Team summarize exercised surfaces plus expected pass/fail outcomes explicitly,
3. add a compact passed/failed corroboration to Game Loop,
4. and update the focused dashboard proof so it matches the current full-power Scrapling lane rather than the older request-native contract.
