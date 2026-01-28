# WASM Stealth Bot Trap (Fermyon Spin)

This project implements a customizable, behavior-based bot defense system for deployment at the edge using Fermyon Spin and WebAssembly.

## Structure
- `src/`: Rust source code for the Spin app
- `spin.toml`: Spin app manifest
- `README.md`: Project overview and setup
- `.gitignore`: Standard ignores

## Quick Start
1. Install [Spin](https://developer.fermyon.com/spin/install)
2. Build: `spin build --release`
3. Run locally: `spin up`
4. Deploy to Fermyon Cloud as needed

---


## Usage

### As a Site Owner
- Deploy the app to your edge environment (Fermyon Cloud or compatible platform).
- Configure honeypot URLs, rate limits, browser blocklist, geo risk countries, and whitelist via the admin API.
- Monitor and manage bans and analytics via the admin API.

### Endpoints

- `/health` — Health check endpoint. Returns `OK` only when accessed from localhost (127.0.0.1 or ::1). Used for liveness/readiness probes. All other sources receive 403 Forbidden.
- `/` — Main endpoint. Applies bot trap logic: whitelist, ban, honeypot, rate limit, JS challenge, browser/geo checks, and interactive quiz for banned users.
- `/quiz` — Interactive math quiz endpoint for banned users. Users must solve a randomized math challenge to regain access.
- `/admin/*` — Admin API endpoints (see below).

### Admin API Endpoints

All endpoints require an `Authorization: Bearer <API_KEY>` header. The API key is configurable via the `API_KEY` environment variable (see below).

#### Endpoints

- `GET /admin/ban` — List all current bans (JSON: IP, reason, expiry)
- `POST /admin/unban?ip=...` — Unban a specific IP (removes ban immediately)
- `GET /admin/analytics` — Get ban count analytics
- `GET /admin/events?hours=N` — Query recent security events, top IPs, and event statistics for dashboarding (see below)
- `GET /admin` — Usage help

#### `/admin/events` — Activity & Analytics API

Returns a JSON object with:

- `recent_events`: up to 100 most recent events (ban, unban, challenge, block, admin actions)
- `event_counts`: count of each event type in the time window
- `top_ips`: top 10 IPs by event count

Query params:
- `hours=N` — How many hours of history to include (default: 24)

**Example response:**
```json
{
	"recent_events": [
		{
			"ts": 1769577600,
			"event": "Ban",
			"ip": "203.0.113.42",
			"reason": "honeypot",
			"outcome": "banned",
			"admin": null
		},
		{
			"ts": 1769577700,
			"event": "Unban",
			"ip": "203.0.113.42",
			"reason": "admin_unban",
			"outcome": "unbanned",
			"admin": "admin"
		}
		// ...
	],
	"event_counts": {
		"Ban": 12,
		"Unban": 2,
		"Challenge": 8,
		"Block": 0,
		"AdminAction": 5
	},
	"top_ips": [
		["203.0.113.42", 7],
		["198.51.100.10", 3]
	]
}
```

**Example curl queries:**

List all bans:
```sh
curl -s -H "Authorization: Bearer $API_KEY" http://localhost:3000/admin/ban | jq
```

Unban an IP:
```sh
curl -X POST -H "Authorization: Bearer $API_KEY" "http://localhost:3000/admin/unban?ip=203.0.113.42"
```

Get ban count analytics:
```sh
curl -s -H "Authorization: Bearer $API_KEY" http://localhost:3000/admin/analytics | jq
```

Get recent events and statistics (last 24 hours):
```sh
curl -s -H "Authorization: Bearer $API_KEY" "http://localhost:3000/admin/events?hours=24" | jq
```

Get API usage/help:
```sh
curl -s -H "Authorization: Bearer $API_KEY" http://localhost:3000/admin
```

#### API Key Configuration
- The admin API key is set via the `API_KEY` environment variable in your Spin manifest or deployment environment. If not set, it defaults to `changeme-supersecret` for development.
- Example (in `spin.toml`):
	```toml
	[component.bot-trap]
	environment = { API_KEY = "changeme-supersecret" }
	```


### Interactive Quiz for Banned Users

When a user is banned (e.g., by honeypot, rate limit, or admin action), they are presented with an interactive math quiz. Features:

- **Randomized question types**: Addition, subtraction, and multiplication
- **User-friendly HTML**: Styled, accessible, and mobile-friendly
- **Automatic unban**: Correct answer removes the ban and restores access
- **Security**: Quiz answers are stored securely per IP

This feature helps reduce false positives and allows legitimate users to regain access easily.


### Configuration
- Ban duration, rate limit, honeypot URLs, browser blocklist, geo risk, whitelist (with CIDR and comments), path-based whitelist for integrations/webhooks, and test mode are stored in edge KV and can be managed via future admin endpoints or direct KV updates.

#### Whitelist Features
- **IP/CIDR support:** Whitelist entries can be single IPs (e.g., `1.2.3.4`) or CIDR ranges (e.g., `192.168.0.0/24`).
- **Inline comments:** Entries can include comments after a `#` (e.g., `10.0.0.0/8 # corp network`).
- **Path-based whitelisting:** The `path_whitelist` config allows you to specify exact paths (e.g., `/webhook/stripe`) or wildcard prefixes (e.g., `/api/integration/*`) that should always bypass bot protections. Useful for trusted webhooks and integrations.

Example config snippet:
```json
{
	"whitelist": ["1.2.3.4", "192.168.0.0/24 # office", "10.0.0.0/8 # corp"],
	"path_whitelist": ["/webhook/stripe", "/api/integration/* # trusted integrations"]
}
```

#### Test Mode (Safe Deployment/Tuning)

Test mode allows you to safely deploy and tune the bot trap in production without impacting real users. When enabled, all block/ban/challenge actions are logged but not enforced—users are always allowed through. This is ideal for initial rollout, tuning, and validation.

**How to enable:**
- Set the environment variable `TEST_MODE=1` or `TEST_MODE=true` in your deployment (e.g., in `spin.toml`):
	```toml
	[component.bot-trap]
	environment = { TEST_MODE = "1" }
	```
- Or set `"test_mode": true` in the config KV object.

**When enabled:**
- All actions (ban, block, challenge) are logged with a `[TEST MODE]` prefix
- No user is actually blocked, banned, or challenged
- Useful for safe validation and tuning in production

**Disable test mode** to enforce real blocking/ban logic.

---



## Testing

### Full Test Suite (Unit + Integration, Colorized)

To run all tests (unit and integration) with clear, colorized output, use:

```sh
./test_all_colored.sh
```

**Note:** All test scripts now automatically run `cargo clean` before building or testing. This ensures the correct crate-type is set for each build mode (native or WASM), preventing build/test errors due to stale build artifacts. You never need to remember to clean manually.

**How crate-type switching works:**
- When building for native (unit/integration tests), the crate-type is set to `["rlib"]`.
- When building for WASM (Spin), the crate-type is set to `["cdylib"]`.
- This is handled automatically by `build.rs` based on the build target.

If you see errors about missing crates or WASM output, ensure you are using the provided scripts or run `cargo clean` before switching build modes.

This script will:
- Run all Rust unit tests (including quiz and ban logic) with colored output
- Build the Spin app
- Run the full integration test suite (endpoints, ban logic, admin, etc.) with colored output

All results are easy to review. See `test_all_colored.sh` for details.

### Unit Tests Only (Colorized)

To run only the Rust unit tests (with colored output):

```sh
./test_unit_colored.sh
```

### Integration Tests Only (Colorized)

To run only the integration tests (Spin endpoints, colorized):

```sh
spin build && ./test_spin_colored.sh
```

### Manual Testing: Triggering Bot Trap Responses

To manually trigger and test each bot trap response in your browser or with curl, you can simulate the following scenarios:


1. **Whitelist**: Add your IP to the whitelist in the config (or remove it to test blocks).
2. **Ban**: Manually ban your IP using the admin API, or trigger a honeypot or rate limit. You will be presented with an interactive math quiz to regain access.
3. **Honeypot**: Visit a honeypot path (e.g., http://127.0.0.1:3000/bot-trap).
4. **Rate Limit**: Send many requests quickly (e.g., with a script or curl loop) to exceed the rate limit.
5. **JS Challenge**: Clear cookies and visit the root endpoint; you should see the JS challenge page.
6. **Outdated Browser**: Use a custom User-Agent string with an old version (e.g., Chrome/100) to trigger a block.
7. **Geo Risk**: Add a high-risk country to the config and set the X-Geo-Country header.

You can use browser dev tools or curl to set headers and test these scenarios. See the admin API section above for ban management.

---

- Modular Rust code: see `src/` for ban, rate, JS, browser, geo, whitelist, honeypot, admin, and interactive quiz logic.
- Integration test script: see `test_spin_colored.sh` for automated end-to-end tests.
- Unit tests: see `src/ban_tests.rs` for ban logic tests.
- Logging: Security events and ban actions are logged using Spin's logging macros.
- Performance: Early returns, minimal KV access, lightweight parsing, and optimized WASM build.

## Performance Checklist
- Early returns: Whitelist and ban checks short-circuit further logic
- Minimal key-value store reads/writes per request
- Lightweight header/cookie parsing
- Fixed time window for rate limiting
- No large in-memory state; all persistent state in edge KV
- Build with `--release` for optimized WASM

---

## Security
- All admin endpoints require API key authentication.
- Input validation and sanitization for all admin operations.
- JS challenge uses a secure, tamper-proof token tied to the visitor's IP.

---

## Roadmap
- Expand admin API for full configuration management
- Add more analytics and export options
- Integrate with additional edge geo/IP sources
- Add more unit and integration tests

---

See `src/` for implementation details and extend as needed.
