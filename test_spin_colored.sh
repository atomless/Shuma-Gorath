#!/bin/bash
# test_spin_colored.sh
# Integration test suite for Spin app with colored output

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
if curl -sf "$BASE_URL/health" | grep -q OK; then
  pass "/health returns OK"
else
  fail "/health did not return OK"
fi

# Test 2: Root endpoint (should return JS challenge or OK)
info "Testing root endpoint..."
resp=$(curl -s "$BASE_URL/")
if echo "$resp" | grep -q 'js_verified' || echo "$resp" | grep -q 'OK (passed bot trap)'; then
  pass "/ returns JS challenge or OK"
else
  fail "/ did not return expected response"
fi

# Test 3: Honeypot triggers ban
info "Testing honeypot ban..."
curl -s "$BASE_URL/bot-trap" > /dev/null
resp=$(curl -s "$BASE_URL/")
if echo "$resp" | grep -q 'Blocked: Banned'; then
  pass "Honeypot triggers ban and / returns Blocked: Banned"
else
  fail "Honeypot did not trigger ban as expected"
fi

# Test 4: Unban 'unknown' via admin API
info "Testing admin unban for 'unknown'..."
curl -s "$BASE_URL/admin/unban?ip=unknown" -H "Authorization: Bearer $API_KEY" > /dev/null
resp=$(curl -s "$BASE_URL/")
if ! echo "$resp" | grep -q 'Blocked: Banned'; then
  pass "Unban for 'unknown' works"
else
  fail "Unban for 'unknown' did not work"
fi

# Test 5: Health check after ban/unban
info "Testing /health endpoint again..."
if curl -sf "$BASE_URL/health" | grep -q OK; then
  pass "/health returns OK after ban/unban"
else
  fail "/health did not return OK after ban/unban"
fi

echo -e "\n${GREEN}All integration tests complete.${NC}"
