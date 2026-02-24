# 🐙 Features & Roadmap

## 🐙 Current Features (Implemented)

- Honeypot endpoints (default: `/instaban`)
- Per-<abbr title="Internet Protocol">IP</abbr> rate limiting
- Browser version blocking
- <abbr title="Geolocation">GEO</abbr> scoring + policy routing (`allow/challenge/maze/block`) via trusted `X-Geo-Country`
- <abbr title="JavaScript">JS</abbr> challenge with signed cookie
- Puzzle challenge step-up with single-use seeds
- Proof-of-work (<abbr title="Proof of Work">PoW</abbr>) step before <abbr title="JavaScript">JS</abbr> verification (edge-served)
- Optional browser whitelist to bypass <abbr title="JavaScript">JS</abbr> challenge
- Maze deception stack with signed traversal tokens, rotating entropy variants, checkpointed progression, optional deep-tier micro-<abbr title="Proof of Work">PoW</abbr>, and auto-ban threshold controls
- HTTP tarpit escalation path with bounded concurrency/time/byte controls and deterministic fallback
- <abbr title="Chrome DevTools Protocol">CDP</abbr> automation detection and reporting (`/cdp-report`)
- robots.txt generation and policy controls
- Admin <abbr title="Application Programming Interface">API</abbr> (ban/unban, analytics, events, config, maze, robots, <abbr title="Chrome DevTools Protocol">CDP</abbr>)
- Test mode (log-only, no enforcement)
- Event logging with retention (`SHUMA_EVENT_LOG_RETENTION_HOURS`)
- Prometheus metrics (`/metrics`)
- Composable defence modes per module (`off` / `signal` / `enforce` / `both`) for `rate`, `geo`, and `js`
- Effective-mode and signal-state observability for botness decisions
- Web dashboard for analytics and admin control
- Makefile-based setup, build, and test workflows

## 🐙 Why These Features Are Valuable

Shuma’s feature set is implemented with an explicit asymmetry target:

- keep normal-user friction low,
- keep host resource cost bounded,
- increase malicious-visitor cost progressively as confidence rises.

For detailed rationale by capability (research basis, enterprise baseline followed, and Shuma-specific advancement), see `value-proposition.md`.

## 🐙 Near-Term Roadmap
- Human verification tuning (usability vs abuse resistance) and accessibility path
- Webhook notifications (Slack/Discord/PagerDuty)
- CSV/<abbr title="JavaScript Object Notation">JSON</abbr> export for events and analytics
- Additional geo/<abbr title="Internet Protocol">IP</abbr> intelligence sources and fallbacks
- Expanded test coverage (edge cases + negative tests)
- Optional re-enable of challenge-on-ban feature

## 🐙 Longer-Term / Modern Threats

See [`docs/bot-defence.md`](bot-defence.md) for Shuma-Gorath layering strategy with managed edge bot defences (including Akamai Bot Manager).
