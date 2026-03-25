# Security Review Tracker

Last updated: 2026-03-24

Purpose: track review finding validity and closure status.
Active implementation planning lives in `todos/todo.md`; blocked or contingent follow-up lives in `todos/blocked-todo.md`.
Completed findings are archived in `todos/completed-todo-history.md`.

## Open Findings (Actionable)

### P1
- [ ] Enterprise multi-instance ban correctness is still not strict/fail-closed under backend outage. The external `ban_store` shares Redis state in steady state, but `is_banned`, `list_active_bans`, `ban_ip_with_fingerprint`, and `unban_ip` still silently fall back to local state on Redis errors or absence, so authoritative convergence under failover is not yet guaranteed. Track this under `DEP-ENT-1..5` in `todos/todo.md`.

### P2
- [ ] Logging is still mixed but predominantly unstructured (`eprintln!` component diagnostics alongside metrics/event records); request correlation and incident triage ergonomics can be improved.
- [ ] Event logs still persist raw IPs at rest for investigation value. Admin monitoring views pseudonymize by default and require forensic acknowledgement for raw display, but there is still no storage-level IP minimization mode for privacy-sensitive deployments (tracked as `SEC-GDPR-3` in `todos/todo.md`).

## Closed or Invalid Findings (Audit Trail)
- Retired findings from the 2026-03-06 audit were moved to `todos/completed-todo-history.md`.
- `SEC-GDPR-2` closed on 2026-03-24: fingerprint retention now performs bounded cadence-gated cleanup for stale `fp:state:*`, `fp:flow:*`, and `fp:flow:last_bucket:*` keys aligned to configured TTL and flow-window controls. Evidence: `make test-fingerprint-retention-cleanup`, `git diff --check`, and [`../docs/research/2026-03-24-sec-gdpr-2-fingerprint-retention-cleanup-post-implementation-review.md`](../docs/research/2026-03-24-sec-gdpr-2-fingerprint-retention-cleanup-post-implementation-review.md).
