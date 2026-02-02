# Dashboard Documentation

## Overview

The bot trap now includes a real-time monitoring dashboard with visualizations and admin controls.

## Features

### 📊 Real-Time Statistics
- **Total Bans**: Lifetime count of all IP bans
- **Active Bans**: Currently banned IPs (non-expired)
- **Total Events**: All logged security events
- **Unique IPs**: Distinct IP addresses tracked

### 📈 Visualizations
1. **Event Types Distribution** (Doughnut Chart)
   - Shows breakdown of event types: honeypot_ban, admin_ban, js_challenge_fail, rate_limit, unban
   - Color-coded for easy identification

2. **Top IPs** (Bar Chart)
   - Displays the most active IP addresses
   - Sorted by event count
   - Helps identify repeat offenders

3. **Events Over Time** (Line Chart)
   - Time-series visualization of event activity
   - Toggle between 24 hours, 7 days, or 30 days
   - Buckets: hourly (24h view), daily (7d/30d views)
   - **Note**: Data retention not yet implemented - events accumulate indefinitely in current version

### 📋 Data Tables

#### Ban List
- IP addresses currently banned
- Reason for ban
- Expiry time
- Quick unban button for each entry

#### Recent Events
- Last 50 security events
- Color-coded badges for event types
- IP address and timestamp
- Optional reason/details

### 🛠️ Admin Controls

#### Manual Ban
- Input IP address to ban
- Set custom ban duration (default: 6 hours)
- Set custom ban reason (default: admin_ban)

#### Manual Unban
- Input IP address to unban
- Instant removal from ban list

## Access

### Development Mode (Current)
```bash
# Dashboard accessible at:
http://127.0.0.1:3000/dashboard/index.html

# Configuration:
- Endpoint: http://127.0.0.1:3000
- API Key: changeme-supersecret (default from spin.toml)
```

### Production Mode (TODO)
In production, the dashboard should be:
1. Protected with authentication
2. Only accessible from admin IP addresses
3. Served over HTTPS with proper certificates
4. API key stored securely (environment variable)

## Architecture

### Files
```
dashboard/
  ├── index.html      # Main dashboard page with Chart.js
  ├── style.css       # Modern, responsive CSS styling
  ├── dashboard.js    # Chart rendering and data updates
  ├── admin.js        # API interaction functions
  └── README.md       # Dashboard development notes
```

### API Endpoints Used
```
GET  /admin/analytics  # Stats (total bans, active bans, events, unique IPs)
GET  /admin/events     # Recent events log
POST /admin/ban        # Manual IP ban
POST /admin/unban      # Manual IP unban
```

### Data Flow
```
1. Page loads → initCharts() creates empty charts
2. updateConfig() reads endpoint + API key from inputs
3. Auto-refresh every 30 seconds:
   - fetchAnalytics() → updateStatCards()
   - fetchAnalytics() → updateEventTypesChart()
   - fetchAnalytics() → updateTopIpsChart()
   - fetchAnalytics() → updateBansTable()
   - fetchAnalytics() → updateEventsTable()
```

## Testing Checklist

### Functional Tests
- [ ] Dashboard loads at http://127.0.0.1:3000/dashboard/index.html
- [ ] Stat cards display correctly
- [ ] Event types doughnut chart renders
- [ ] Top IPs bar chart renders
- [ ] Ban list table shows current bans
- [ ] Recent events table shows last 50 events
- [ ] Manual ban function works
- [ ] Manual unban function works
- [ ] Quick unban buttons work from table
- [ ] Auto-refresh updates data every 30 seconds
- [ ] Endpoint and API key inputs work
- [ ] Configuration saves and persists

### Integration Tests (TODO)
```bash
# Test 1: Dashboard loads successfully
curl -I http://127.0.0.1:3000/dashboard/index.html

# Test 2: Analytics endpoint returns JSON
curl -H "X-API-Key: changeme-supersecret" \
  http://127.0.0.1:3000/admin/analytics

# Test 3: Events endpoint returns JSON
curl -H "X-API-Key: changeme-supersecret" \
  http://127.0.0.1:3000/admin/events

# Test 4: Ban IP via API
curl -X POST -H "X-API-Key: changeme-supersecret" \
  -H "Content-Type: application/json" \
  -d '{"ip":"1.2.3.4","reason":"test_ban","duration_secs":3600}' \
  http://127.0.0.1:3000/admin/ban

# Test 5: Unban IP via API
curl -X POST -H "X-API-Key: changeme-supersecret" \
  -H "Content-Type: application/json" \
  -d '{"ip":"1.2.3.4"}' \
  http://127.0.0.1:3000/admin/unban
```

### Security Tests (TODO - After Functionality Confirmed)
- [ ] Dashboard blocked without valid API key
- [ ] Admin endpoints reject requests without authentication
- [ ] XSS protection in event/ban data display
- [ ] CSRF protection for admin actions
- [ ] Rate limiting for admin API calls
- [ ] Input validation for IP addresses
- [ ] SQL injection prevention (N/A - using KV store)
- [ ] Production mode restricts access by IP

## Browser Compatibility

Tested on:
- Chrome/Edge (Chromium-based)
- Firefox
- Safari

Requirements:
- Modern browser with ES6+ support
- JavaScript enabled
- Chart.js 4.4.1 loaded from CDN

## Troubleshooting

### Dashboard Not Loading
```bash
# Check Spin server is running
ps aux | grep "spin up"

# Check dashboard route is configured
cat spin.toml | grep dashboard

# Check dashboard files exist
ls -la dashboard/

# Check Spin logs
tail -f /tmp/spin_bot_trap.log
```

### Charts Not Rendering
- Check browser console for JavaScript errors
- Verify Chart.js CDN is accessible
- Check API key is correct in config section
- Verify endpoint URL matches Spin server

### No Data Showing
```bash
# Generate test data by triggering events
curl http://127.0.0.1:3000/honeypot
curl http://127.0.0.1:3000/login
curl http://127.0.0.1:3000/admin

# Ban a test IP
curl -X POST -H "X-API-Key: changeme-supersecret" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.0.0.1","reason":"test","duration_secs":3600}' \
  http://127.0.0.1:3000/admin/ban
```

### API Authentication Failing
- Default API key: `changeme-supersecret`
- Set in `spin.toml`: `environment = { API_KEY = "..." }`
- Must be sent in `X-API-Key` header
- Dashboard reads from config input field

## Development

### Running Locally
```bash
# Build and start Spin server with dashboard
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/wasm_bot_trap.wasm src/bot_trap.wasm
nohup spin up --listen 127.0.0.1:3000 > /tmp/spin_bot_trap.log 2>&1 &

# Open dashboard in browser
open http://127.0.0.1:3000/dashboard/index.html
```

### Making Changes
```bash
# Edit dashboard files directly (no rebuild needed)
# Changes take effect immediately - just refresh browser

# Edit Rust code requires rebuild
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/wasm_bot_trap.wasm src/bot_trap.wasm
pkill -f "spin up"
nohup spin up --listen 127.0.0.1:3000 > /tmp/spin_bot_trap.log 2>&1 &
```

### Adding New Charts
1. Add new canvas element in `dashboard/index.html`
2. Create chart initialization in `initCharts()` (dashboard.js)
3. Add update function for chart data
4. Call update function in auto-refresh interval

### Adding New Admin Features
1. Add new function in `dashboard/admin.js`
2. Add UI controls in `dashboard/index.html`
3. Wire up event listeners in `dashboard/dashboard.js`
4. Add corresponding Rust endpoint in `src/admin.rs`

## Next Steps

### Immediate (Functionality First)
1. ✅ Dashboard loads successfully
2. ⏳ Test all dashboard features manually
3. ⏳ Add integration tests for API endpoints
4. ⏳ Add JavaScript unit tests for dashboard.js
5. ⏳ Document all discovered bugs

### Security Hardening (After Tests)
1. Add dev mode vs prod mode configuration
2. Implement IP-based access control
3. Add authentication layer (OAuth, JWT, etc.)
4. Rate limiting for admin API
5. Input sanitization and validation
6. HTTPS requirement in production
7. Secure API key storage
8. CORS configuration
9. Content Security Policy (CSP) headers
10. Audit logging for admin actions

### Features (Future)
1. Historical trend charts (last 24h, 7d, 30d)
2. Real-time WebSocket updates (vs polling)
3. Downloadable reports (CSV, JSON)
4. Alert notifications (email, Slack, etc.)
5. Geographic IP visualization (map)
6. Custom dashboard widgets
7. User preference saving
8. Dark mode toggle
9. Mobile-responsive improvements
10. Accessibility (WCAG compliance)
11. **Data retention policy**: Auto-cleanup of events older than configurable period (e.g., 90 days)
12. **Background cleanup job**: Periodic task to remove old events from KV store

## License

Same as main project.
