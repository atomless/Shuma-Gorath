#!/bin/bash
# test_spin_colored.sh
# Integration test suite for Spin app with colored output
#
# ⚠️ IMPORTANT: These tests MUST run in the Spin environment!
# They require HTTP server, key-value store, and real headers.
#
# PREREQUISITES:
#   1. Start Spin server: spin up
#   2. Run this script: ./test_spin_colored.sh
#
# This script runs integration test scenarios:
#   1. Health check endpoint (GET /health)
#   2. Root endpoint behavior (GET /)
#   3. Honeypot ban detection (POST /bot-trap)
#   4. Admin API unban (POST /admin/unban)
#   5. Health check after ban/unban (GET /health)
#   6. Config API - get config (GET /admin/config)
#   7. Test mode toggle (POST /admin/config)
#   8. Test mode behavior verification
#   9. Test mode disable and blocking resumes
#   10. Verify blocking after test mode disabled
#   11. Prometheus metrics endpoint
#   12. CDP report endpoint (POST /cdp-report)
#   13. CDP auto-ban with high score
#   14. CDP config via admin API
#   15. Unban functionality test

set -e

# Always clean before integration tests to avoid stale artifacts
cargo clean
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
NC="\033[0m" # No Color

pass() { echo -e "${GREEN}PASS${NC} $1"; }
fail() { echo -e "${RED}FAIL${NC} $1"; }
info() { echo -e "${YELLOW}INFO${NC} $1"; }

BASE_URL="http://127.0.0.1:3000"
API_KEY="changeme-supersecret"

FORWARDED_SECRET_HEADER=()
if [[ -n "${FORWARDED_IP_SECRET:-}" ]]; then
  FORWARDED_SECRET_HEADER=(-H "X-Shuma-Forwarded-Secret: ${FORWARDED_IP_SECRET}")
fi

# Test 1: Health check
info "Testing /health endpoint..."

health_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/health")
if echo "$health_resp" | grep -q OK; then
  pass "/health returns OK"
else
  fail "/health did not return OK"
  echo -e "${YELLOW}DEBUG /health response:${NC} $health_resp"
fi

# Test 2: Root endpoint (should return JS challenge or OK)
info "Testing root endpoint..."

root_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/")
if echo "$root_resp" | grep -qE '(js_verified|JavaScript|Verifying|pow|Proof-of-work)'; then
  pass "/ returns JS challenge (PoW or standard)"
else
  fail "/ did not return expected JS challenge page"
  echo -e "${YELLOW}DEBUG / response:${NC} $root_resp"
fi

# Test 3: Honeypot triggers ban
info "Testing honeypot ban..."
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/bot-trap" > /dev/null
resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/")
if echo "$resp" | grep -q 'Access Blocked'; then
  pass "Honeypot triggers ban and / returns Access Blocked"
else
  fail "Honeypot did not trigger ban as expected"
fi

# Test 4: Unban 'unknown' via admin API
info "Testing admin unban for 'unknown'..."
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/admin/unban?ip=unknown" -H "Authorization: Bearer $API_KEY" > /dev/null
resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/")
if ! echo "$resp" | grep -q 'Blocked: Banned'; then
  pass "Unban for 'unknown' works"
else
  fail "Unban for 'unknown' did not work"
fi

# Test 5: Health check after ban/unban
info "Testing /health endpoint again..."
if curl -sf "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/health" | grep -q OK; then
  pass "/health returns OK after ban/unban"
else
  fail "/health did not return OK after ban/unban"
fi

# Test 6: Get config via admin API
info "Testing GET /admin/config..."
config_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/config")
if echo "$config_resp" | grep -q '"test_mode"'; then
  pass "GET /admin/config returns test_mode field"
else
  fail "GET /admin/config did not return test_mode"
  echo -e "${YELLOW}DEBUG config response:${NC} $config_resp"
fi

# Test 7: Enable test mode
info "Testing POST /admin/config to enable test_mode..."
enable_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $API_KEY" -H "Content-Type: application/json" \
  -d '{"test_mode": true}' "$BASE_URL/admin/config")
if echo "$enable_resp" | grep -q '"test_mode":true'; then
  pass "POST /admin/config enables test_mode"
else
  fail "POST /admin/config did not enable test_mode"
  echo -e "${YELLOW}DEBUG enable response:${NC} $enable_resp"
fi

# Test 8: Test mode allows honeypot access without blocking
info "Testing test_mode behavior (honeypot should not block)..."
# First, unban the test IP to ensure clean state
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.99" > /dev/null
# Hit honeypot with test IP
honeypot_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.99" "$BASE_URL/bot-trap")
if echo "$honeypot_resp" | grep -q 'TEST MODE'; then
  pass "Test mode returns TEST MODE response for honeypot"
else
  fail "Test mode did not return expected TEST MODE response"
  echo -e "${YELLOW}DEBUG honeypot response:${NC} $honeypot_resp"
fi

# Verify IP was NOT actually banned
subsequent_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.99" "$BASE_URL/")
if echo "$subsequent_resp" | grep -q 'TEST MODE'; then
  pass "Test mode: IP not actually banned after honeypot"
else
  fail "Test mode: IP was banned when it should not have been"
  echo -e "${YELLOW}DEBUG subsequent response:${NC} $subsequent_resp"
fi

# Test 9: Disable test mode and verify blocking resumes
info "Testing POST /admin/config to disable test_mode..."
disable_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $API_KEY" -H "Content-Type: application/json" \
  -d '{"test_mode": false}' "$BASE_URL/admin/config")
if echo "$disable_resp" | grep -q '"test_mode":false'; then
  pass "POST /admin/config disables test_mode"
else
  fail "POST /admin/config did not disable test_mode"
  echo -e "${YELLOW}DEBUG disable response:${NC} $disable_resp"
fi

# Test 10: Verify blocking works again after test mode disabled
info "Testing that blocking resumes after test_mode disabled..."
# Unban first to get clean state
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.100" > /dev/null
# Hit honeypot - should now actually ban
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.100" "$BASE_URL/bot-trap" > /dev/null
block_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 10.0.0.100" "$BASE_URL/")
if echo "$block_resp" | grep -q 'Access Blocked'; then
  pass "Blocking resumes: honeypot triggers real ban after test_mode disabled"
else
  fail "Blocking did not resume after test_mode disabled"
  echo -e "${YELLOW}DEBUG block response:${NC} $block_resp"
fi

# Cleanup: unban test IPs
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.99" > /dev/null
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.100" > /dev/null

# Test 11: Prometheus metrics endpoint
info "Testing GET /metrics (Prometheus format)..."
metrics_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" "$BASE_URL/metrics")
if echo "$metrics_resp" | grep -q 'bot_trap_requests_total'; then
  pass "/metrics returns Prometheus-formatted metrics"
else
  fail "/metrics did not return expected Prometheus format"
  echo -e "${YELLOW}DEBUG metrics response:${NC} $metrics_resp"
fi

# Verify metrics contain expected counters
if echo "$metrics_resp" | grep -q 'bot_trap_bans_total'; then
  pass "/metrics includes ban counters"
else
  fail "/metrics missing ban counters"
fi

# Test 12: CDP report endpoint exists
info "Testing POST /cdp-report endpoint..."
cdp_report='{"cdp_detected":true,"score":0.5,"checks":["webdriver"]}'
cdp_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Content-Type: application/json" -H "X-Forwarded-For: 10.0.0.200" -d "$cdp_report" "$BASE_URL/cdp-report")
if echo "$cdp_resp" | grep -qiE 'received|disabled|detected'; then
  pass "/cdp-report endpoint accepts detection reports"
else
  fail "/cdp-report endpoint not working"
  echo -e "${YELLOW}DEBUG cdp response:${NC} $cdp_resp"
fi

# Test 13: CDP report with high score triggers action (when enabled)
info "Testing CDP auto-ban with high score..."
cdp_high='{"cdp_detected":true,"score":0.95,"checks":["webdriver","automation_props","cdp_timing"]}'
cdp_high_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Content-Type: application/json" -H "X-Forwarded-For: 10.0.0.201" -d "$cdp_high" "$BASE_URL/cdp-report")
if echo "$cdp_high_resp" | grep -qiE 'banned|received|disabled'; then
  pass "/cdp-report handles high-score detection"
else
  fail "/cdp-report failed for high-score detection"
  echo -e "${YELLOW}DEBUG cdp high response:${NC} $cdp_high_resp"
fi

# Test 14: CDP config available via admin API
info "Testing CDP config in /admin/cdp..."
cdp_config=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/cdp")
if echo "$cdp_config" | grep -qE '"enabled"|cdp_detection'; then
  pass "/admin/cdp returns CDP configuration"
else
  fail "/admin/cdp not returning expected config"
  echo -e "${YELLOW}DEBUG cdp config:${NC} $cdp_config"
fi

# Test 15: unban_ip function works via admin endpoint  
info "Testing unban functionality..."
# First ban an IP
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/ban?ip=10.0.0.202&reason=test" > /dev/null
# Then unban it
unban_resp=$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.202")
if echo "$unban_resp" | grep -qi 'unbanned'; then
  pass "Unban via admin API works correctly"
else
  fail "Unban did not work as expected"
  echo -e "${YELLOW}DEBUG unban response:${NC} $unban_resp"
fi

# Cleanup: unban test CDP IPs
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.200" > /dev/null
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.201" > /dev/null

echo -e "\n${GREEN}All integration tests complete.${NC}"
