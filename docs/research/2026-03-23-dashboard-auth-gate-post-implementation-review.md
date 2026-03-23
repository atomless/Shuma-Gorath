# Dashboard Auth Gate Post-Implementation Review

## Tranche

- `DASH-AUTH-1` Gate the dashboard shell behind authenticated session restore so logged-out navigation to `/dashboard` or `/dashboard/index.html` does not render the dashboard shell before redirecting to `/dashboard/login.html`.

## Delivered

1. Added a route-local auth bootstrap state in [`dashboard/src/routes/+page.svelte`](../../dashboard/src/routes/+page.svelte).
2. Changed the prerendered dashboard route to emit a no-copy auth gate instead of the dashboard shell until authenticated session restore succeeds.
3. Added focused source-contract and rendered Playwright coverage proving:
   - the auth gate exists in the route contract,
   - logged-out dashboard entry does not render the tab shell while `/admin/session` is unresolved,
   - the route still redirects cleanly to `/dashboard/login.html`.
4. Added a focused Makefile verification target and updated operator/testing docs.

## Verification

- `make test-dashboard-auth-gate`
- `git diff --check`

## Architectural Review

The fix stayed within the local dashboard auth flow:

- no global session contract changes,
- no Monitoring-overhaul coupling,
- no broader route/polling/auth architecture refactor,
- no server-side static asset serving changes.

That was the correct first slice because the root cause was the prerendered client route exposing the shell before auth truth, and the rendered no-copy auth gate removes that exposure cleanly without inventing a second login-like screen.

## Shortfall Check

No tranche-local shortfall remains open.

The route still relies on client-side session restore and redirect rather than a server-side auth gate for `/dashboard/index.html`, but that was an intentional scope boundary, not a missed requirement, for this first fix.
