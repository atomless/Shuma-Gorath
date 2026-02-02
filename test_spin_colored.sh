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
# This script runs 8 integration test scenarios:
#   1. Health check endpoint (GET /health)
#   2. Root endpoint behavior (GET /)
#   3. Honeypot ban detection (POST /bot-trap)
#   4. Admin API unban (POST /admin/unban)
#   5. Health check after ban/unban (GET /health)
#   6. Config API - get config (GET /admin/config)
#   7. Test mode toggle (POST /admin/config)
#   8. Test mode behavior verification

set -e

# Always clean before integration tests to ensure correct crate-type
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

# Test 1: Health check
info "Testing /health endpoint..."

health_resp=$(curl -s -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/health")
if echo "$health_resp" | grep -q OK; then
  pass "/health returns OK"
else
  fail "/health did not return OK"
  echo -e "${YELLOW}DEBUG /health response:${NC} $health_resp"
fi

# Test 2: Root endpoint (should return JS challenge or OK)
info "Testing root endpoint..."

root_resp=$(curl -s -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/")
if echo "$root_resp" | grep -q 'Access Blocked'; then
  pass "/ returns Access Blocked (not whitelisted or banned)"
else
  fail "/ did not return expected Access Blocked page"
  echo -e "${YELLOW}DEBUG / response:${NC} $root_resp"
fi

# Test 3: Honeypot triggers ban
info "Testing honeypot ban..."
curl -s -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/bot-trap" > /dev/null
resp=$(curl -s -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/")
if echo "$resp" | grep -q 'Access Blocked'; then
  pass "Honeypot triggers ban and / returns Access Blocked"
else
  fail "Honeypot did not trigger ban as expected"
fi

# Test 4: Unban 'unknown' via admin API
info "Testing admin unban for 'unknown'..."
curl -s -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/admin/unban?ip=unknown" -H "Authorization: Bearer $API_KEY" > /dev/null
resp=$(curl -s -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/")
if ! echo "$resp" | grep -q 'Blocked: Banned'; then
  pass "Unban for 'unknown' works"
else
  fail "Unban for 'unknown' did not work"
fi

# Test 5: Health check after ban/unban
info "Testing /health endpoint again..."
if curl -sf -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/health" | grep -q OK; then
  pass "/health returns OK after ban/unban"
else
  fail "/health did not return OK after ban/unban"
fi

# Test 6: Get config via admin API
info "Testing GET /admin/config..."
config_resp=$(curl -s -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/config")
if echo "$config_resp" | grep -q '"test_mode"'; then
  pass "GET /admin/config returns test_mode field"
else
  fail "GET /admin/config did not return test_mode"
  echo -e "${YELLOW}DEBUG config response:${NC} $config_resp"
fi

# Test 7: Enable test mode
info "Testing POST /admin/config to enable test_mode..."
enable_resp=$(curl -s -X POST -H "Authorization: Bearer $API_KEY" -H "Content-Type: application/json" \
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
curl -s -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.99" > /dev/null
# Hit honeypot with test IP
honeypot_resp=$(curl -s -H "X-Forwarded-For: 10.0.0.99" "$BASE_URL/bot-trap")
if echo "$honeypot_resp" | grep -q 'TEST MODE'; then
  pass "Test mode returns TEST MODE response for honeypot"
else
  fail "Test mode did not return expected TEST MODE response"
  echo -e "${YELLOW}DEBUG honeypot response:${NC} $honeypot_resp"
fi

# Verify IP was NOT actually banned
subsequent_resp=$(curl -s -H "X-Forwarded-For: 10.0.0.99" "$BASE_URL/")
if echo "$subsequent_resp" | grep -q 'TEST MODE'; then
  pass "Test mode: IP not actually banned after honeypot"
else
  fail "Test mode: IP was banned when it should not have been"
  echo -e "${YELLOW}DEBUG subsequent response:${NC} $subsequent_resp"
fi

# Test 9: Disable test mode and verify blocking resumes
info "Testing POST /admin/config to disable test_mode..."
disable_resp=$(curl -s -X POST -H "Authorization: Bearer $API_KEY" -H "Content-Type: application/json" \
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
curl -s -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.100" > /dev/null
# Hit honeypot - should now actually ban
curl -s -H "X-Forwarded-For: 10.0.0.100" "$BASE_URL/bot-trap" > /dev/null
block_resp=$(curl -s -H "X-Forwarded-For: 10.0.0.100" "$BASE_URL/")
if echo "$block_resp" | grep -q 'Access Blocked'; then
  pass "Blocking resumes: honeypot triggers real ban after test_mode disabled"
else
  fail "Blocking did not resume after test_mode disabled"
  echo -e "${YELLOW}DEBUG block response:${NC} $block_resp"
fi

# Cleanup: unban test IPs
curl -s -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.99" > /dev/null
curl -s -H "Authorization: Bearer $API_KEY" "$BASE_URL/admin/unban?ip=10.0.0.100" > /dev/null

# Test 11: Prometheus metrics endpoint
info "Testing GET /metrics (Prometheus format)..."
metrics_resp=$(curl -s "$BASE_URL/metrics")
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

echo -e "\n${GREEN}All integration tests complete.${NC}"
