# Dashboard Auth Shell Flash Review

## Summary

Direct navigation to `/dashboard` or `/dashboard/index.html` while logged out currently renders the dashboard shell briefly before redirecting to `/dashboard/login.html`.

This is not a protected-data leak today because the rendered default panel is the placeholder Monitoring shell, not live monitoring data. It is still an auth-flow flaw and a poor security posture because the dashboard route currently follows a render-first, auth-later pattern.

## Evidence

- [`dashboard/src/routes/+page.svelte`](../../dashboard/src/routes/+page.svelte)
- [`dashboard/src/lib/runtime/dashboard-route-controller.js`](../../dashboard/src/lib/runtime/dashboard-route-controller.js)
- [`dashboard/src/lib/runtime/dashboard-native-runtime.js`](../../dashboard/src/lib/runtime/dashboard-native-runtime.js)
- [`dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`src/runtime/request_router.rs`](../../src/runtime/request_router.rs)
- [`src/admin/api.rs`](../../src/admin/api.rs)

Concrete traced behavior:

1. The dashboard store initializes with `initialTab: 'game-loop'`.
2. The route renders the dashboard shell and `GameLoopTab` unconditionally.
3. `bootstrapRuntime()` mounts the dashboard runtime before checking session state.
4. `bootstrapSession()` then performs `GET /admin/session`.
5. If unauthenticated, the client redirects to the login page.
6. The server-side `/dashboard` route only performs a blind redirect to `/dashboard/index.html`; it does not auth-gate dashboard HTML before render.

## Root Cause

The dashboard entry route prerenders and mounts an operator shell before session truth is known.

The immediate cause is the ordering in the Svelte route and dashboard route controller:

- render shell first,
- mount runtime,
- restore session,
- redirect if unauthenticated.

## Constraints

1. Keep the first remediation slice local to the dashboard auth flow.
2. Do not bundle Monitoring-overhaul work into this fix.
3. Preserve the existing login route, `next` handling, and same-origin session contract.
4. Add rendered proof, not only source-shape assertions.

## Decision

Land a local dashboard auth-pending gate first.

That means:

- prerender the dashboard route into a neutral auth-pending shell rather than the dashboard shell,
- do not render header, tabs, or tab panels until session restore confirms authentication,
- preserve the existing client redirect to login for unauthenticated sessions.

This is the smallest clean fix because it removes the flash now without first restructuring the static dashboard asset-serving path.

## Later Follow-on

A later hardening slice may still add server-side auth gating for `/dashboard/index.html` if the static-serving seam is brought under a clean request-router/auth decision point. That is not required for the first bugfix tranche.
