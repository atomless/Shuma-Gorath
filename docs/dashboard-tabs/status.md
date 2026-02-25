# Dashboard Tab: Status

Route: `#status`  
Component: [`dashboard/src/lib/components/dashboard/StatusTab.svelte`](../../dashboard/src/lib/components/dashboard/StatusTab.svelte)

Purpose:

- Fast, human-readable status summary of key defense features and runtime behavior.

What it shows:

- Feature status cards derived from current runtime config snapshot (fail mode, HTTPS, PoW, challenges, CDP, maze, tarpit, JS required, GEO, IP range policy, rate limiting).
- Runtime performance telemetry for dashboard refresh behavior:
  - Fetch latency (last, avg, p95).
  - Render timing (last, avg, p95).
  - Polling skip/resume counters.

Writes:

- Read-only tab (no config writes).
