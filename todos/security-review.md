# Security Review Tracker

Last updated: 2026-03-06

Purpose: track review finding validity and closure status.
Active implementation planning lives in `todos/todo.md`; blocked or contingent follow-up lives in `todos/blocked-todo.md`.
Completed findings are archived in `todos/completed-todo-history.md`.

## Open Findings (Actionable)

### P1
- [ ] Enterprise multi-instance ban correctness is still not strict/fail-closed under backend outage. The external `ban_store` shares Redis state in steady state, but `is_banned`, `list_active_bans`, `ban_ip_with_fingerprint`, and `unban_ip` still silently fall back to local state on Redis errors or absence, so authoritative convergence under failover is not yet guaranteed. Track this under `DEP-ENT-1..5` in `todos/todo.md`.

### P2
- [ ] KV-backed operational telemetry now has bucket-indexed retention cleanup and buffered metric writes, but it still needs reassessment against real shared-host traffic volume, retention lag, and monitoring query cost once deployment evidence exists. Track the execution slice under `TEL-STORE-1` in `todos/todo.md` and [`docs/plans/2026-03-11-telemetry-storage-query-efficiency-excellence-plan.md`](../docs/plans/2026-03-11-telemetry-storage-query-efficiency-excellence-plan.md).
- [ ] Logging is still mixed but predominantly unstructured (`eprintln!` component diagnostics alongside metrics/event records); request correlation and incident triage ergonomics can be improved.
- [ ] Fingerprint-state cleanup is now opportunistic rather than absent: stale `fp:state:*` and `fp:edge:*` keys are deleted on read, and prior `fp:flow:*` buckets are deleted on rollover, but cold/orphaned `fp:flow:*` and `fp:flow:last_bucket:*` keys still lack deterministic sweeping aligned to configured windows (tracked as `SEC-GDPR-2` in `todos/todo.md`).
- [ ] Event logs still persist raw IPs at rest for investigation value. Admin monitoring views pseudonymize by default and require forensic acknowledgement for raw display, but there is still no storage-level IP minimization mode for privacy-sensitive deployments (tracked as `SEC-GDPR-3` in `todos/todo.md`).

## Closed or Invalid Findings (Audit Trail)
- Retired findings from the 2026-03-06 audit were moved to `todos/completed-todo-history.md`.
