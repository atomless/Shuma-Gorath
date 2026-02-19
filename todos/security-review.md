# Security Review Tracker

Last updated: 2026-02-12

Purpose: track review finding validity and closure status.
Active implementation planning lives in `todos/todo.md`.
Completed findings are archived in `todos/completed-todo-history.md`.

## Open Findings (Actionable)

### P1
- [ ] Rate-limiter TOCTOU remains for high concurrency and multi-instance scenarios; fix requires atomic distributed counters (Redis `INCR`/Lua path tracked in `todos/todo.md`). This is enterprise/hybrid critical-path risk, but non-blocking for single-instance `self_hosted_minimal`.
- [ ] Admin abuse controls are partially operational and must be enforced in deployment: `SHUMA_ADMIN_IP_ALLOWLIST` plus CDN/WAF limits for `POST /admin/login` and `/admin/*`.
- [ ] Ban/unban propagation is not yet synchronized across edge instances; consistency drift remains possible under scale/failover.

### P2
- [ ] KV-backed operational telemetry remains acceptable at current scale but needs periodic reassessment against write volume and retention growth.
- [ ] Logging is still largely unstructured (`eprintln!`-first); request correlation and incident triage ergonomics can be improved.
- [ ] Fingerprint-state retention currently uses logical TTL windows but does not yet deterministically remove stale `fp:*` keys; tighten physical cleanup for strict privacy retention compliance (tracked as `SEC-GDPR-2` in `todos/todo.md`).
- [ ] Event logs store raw IPs by default for abuse triage; add optional minimization mode for privacy-sensitive deployments (tracked as `SEC-GDPR-3` in `todos/todo.md`).

## Closed or Invalid Findings (Audit Trail)
