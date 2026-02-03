# Shuma-Gorath Dashboard

Real-time dashboard for monitoring and managing Shuma-Gorath (Many-Angled Bot Defence).

## Features

- **Live Analytics**: Charts for blocked requests, ban rates, and traffic patterns
- **Event Log**: Real-time stream of bot detection events
- **Ban Management**: Manual IP ban/unban controls
- **Test Mode Toggle**: Enable/disable test mode from the dashboard
- **Top IPs**: View most frequently flagged IP addresses

## Files

| File | Purpose |
|------|---------|
| `index.html` | Main dashboard page with UI layout |
| `dashboard.js` | Chart rendering and data updates |
| `admin.js` | API helper functions for admin operations |
| `style.css` | Dashboard styling and responsive layout |

## Admin API Functions (admin.js)

### Ban Management
- `banIp(ip, reason, duration)` - Manually ban an IP address
- `unbanIp(ip)` - Remove an IP from the ban list

### Configuration
- `getConfig()` - Get current server configuration
- `updateConfig(updates)` - Update server configuration
- `setTestMode(enabled)` - Enable or disable test mode

## Test Mode

When test mode is enabled:
- All bot trap logic runs normally
- Events are logged with `[TEST MODE]` suffix
- **No requests are actually blocked**
- Dashboard shows amber "TEST MODE ACTIVE" banner

Toggle test mode via:
1. Dashboard toggle switch in Admin Controls
2. API: `POST /admin/config` with `{"test_mode": true}`
3. Environment variable: `TEST_MODE=true`

## Usage

1. Start the Spin server: `spin up`
2. Open browser to: `http://127.0.0.1:3000/dashboard/`
3. Configure API endpoint and key
4. Click "Refresh" to load data

## Development

The dashboard runs entirely client-side. To modify:

1. Edit HTML/JS/CSS files directly
2. Refresh browser to see changes
3. No build step required
