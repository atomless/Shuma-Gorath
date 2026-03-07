#!/bin/bash
# verify-runtime.sh - Validate runtime-only prerequisites for single-host deployment
#
# Usage: make verify-runtime

set -euo pipefail

GREEN="\033[0;32m"
YELLOW="\033[1;33m"
CYAN="\033[0;36m"
RED="\033[0;31m"
NC="\033[0m"

pass() { echo -e "${GREEN}PASS${NC} $1"; }
fail() { echo -e "${RED}FAIL${NC} $1"; FAILED=1; }
warn() { echo -e "${YELLOW}WARN${NC} $1"; }
info() { echo -e "${CYAN}INFO${NC} $1"; }

FAILED=0

read_env_local_value() {
  local key="$1"
  if [[ ! -f ".env.local" ]]; then
    return 1
  fi
  local line
  line=$(grep -E "^${key}=" .env.local | tail -1 || true)
  if [[ -z "$line" ]]; then
    return 1
  fi
  local value="${line#*=}"
  value="${value%\"}"
  value="${value#\"}"
  value="${value%\'}"
  value="${value#\'}"
  printf '%s' "$value"
}

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════╗"
echo "║  WASM Bot Defence - Runtime Verify               ║"
echo "╚═══════════════════════════════════════════════════╝"
echo -e "${NC}"

if command -v rustc >/dev/null 2>&1; then
  pass "Rust installed: $(rustc --version)"
else
  fail "Rust not installed (run: make setup-runtime)"
fi

if command -v cargo >/dev/null 2>&1; then
  pass "Cargo installed: $(cargo --version)"
else
  fail "Cargo not installed (run: make setup-runtime)"
fi

if command -v rustup >/dev/null 2>&1; then
  if rustup target list --installed 2>/dev/null | grep -q "wasm32-wasip1"; then
    pass "wasm32-wasip1 target installed"
  else
    fail "wasm32-wasip1 target missing (run: rustup target add wasm32-wasip1)"
  fi
else
  fail "rustup missing (required for target management)"
fi

if command -v spin >/dev/null 2>&1; then
  pass "Spin installed: $(spin --version | head -1)"
else
  fail "Spin not installed (run: make setup-runtime)"
fi

if [[ -f "config/defaults.env" ]]; then
  pass "config/defaults.env present"
else
  fail "config/defaults.env missing"
fi

if [[ -f ".env.local" ]]; then
  pass ".env.local present"
else
  fail ".env.local missing (run: make setup-runtime)"
fi

api_key_value="$(read_env_local_value SHUMA_API_KEY || true)"
if [[ -z "${api_key_value}" ]]; then
  fail "SHUMA_API_KEY is empty in .env.local"
elif [[ "${api_key_value}" =~ ^(changeme-dev-only-api-key|changeme-supersecret|changeme-prod-api-key)$ ]]; then
  fail "SHUMA_API_KEY still uses placeholder value in .env.local"
else
  pass ".env.local has a non-placeholder SHUMA_API_KEY"
fi

js_secret_value="$(read_env_local_value SHUMA_JS_SECRET || true)"
if [[ -z "${js_secret_value}" ]]; then
  fail "SHUMA_JS_SECRET is empty in .env.local"
else
  pass ".env.local has SHUMA_JS_SECRET"
fi

forwarded_secret_value="$(read_env_local_value SHUMA_FORWARDED_IP_SECRET || true)"
if [[ -z "${forwarded_secret_value}" ]]; then
  warn "SHUMA_FORWARDED_IP_SECRET is empty in .env.local (set this when using trusted forwarded headers)"
else
  pass ".env.local has SHUMA_FORWARDED_IP_SECRET"
fi

sim_secret_value="$(read_env_local_value SHUMA_SIM_TELEMETRY_SECRET || true)"
if [[ -z "${sim_secret_value}" ]]; then
  fail "SHUMA_SIM_TELEMETRY_SECRET is empty in .env.local"
elif [[ "${sim_secret_value}" == "changeme-dev-only-sim-telemetry-secret" ]]; then
  fail "SHUMA_SIM_TELEMETRY_SECRET still uses placeholder value in .env.local"
elif [[ ! "${sim_secret_value}" =~ ^[0-9a-fA-F]{64,}$ ]]; then
  fail "SHUMA_SIM_TELEMETRY_SECRET must be hex and at least 64 characters in .env.local"
else
  pass ".env.local has a valid SHUMA_SIM_TELEMETRY_SECRET"
fi

info "Running read-only config verification via Makefile target..."
if make --no-print-directory config-verify >/dev/null; then
  pass "make config-verify succeeded"
else
  fail "make config-verify failed"
fi

info "Running runtime build verification via Makefile target..."
if make --no-print-directory build-runtime >/dev/null; then
  pass "make build-runtime succeeded"
else
  fail "make build-runtime failed"
fi

echo ""
if [[ $FAILED -eq 0 ]]; then
  echo -e "${GREEN}All runtime checks passed.${NC}"
  exit 0
fi

echo -e "${RED}Runtime verification failed. Resolve the failed checks and re-run make verify-runtime.${NC}"
exit 1
