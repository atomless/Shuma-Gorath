#!/bin/bash
# test_all_colored.sh
# Runs all tests with colored output
#
# ⚠️ CRITICAL: Two separate test environments!
#
# 1. UNIT TESTS
#    - Run in NATIVE RUST environment (cargo test)
#    - NO Spin required
#    - Test individual functions in isolation
#
# 2. INTEGRATION TESTS
#    - Run in SPIN ENVIRONMENT ONLY (shell scripts via curl)
#    - Requires HTTP server, key-value store, real headers
#    - Tests full HTTP API end-to-end
#
# This script runs BOTH test types in their appropriate environments.

set -e

# Always clean before building/testing to avoid stale artifacts
cargo clean
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
NC="\033[0m" # No Color

pass() { echo -e "${GREEN}PASS${NC} $1"; }
fail() { echo -e "${RED}FAIL${NC} $1"; }
info() { echo -e "${YELLOW}INFO${NC} $1"; }

echo ""
echo -e "${YELLOW}============================================${NC}"
echo -e "${YELLOW}  UNIT TESTS (Native Rust Environment)${NC}"
echo -e "${YELLOW}  Run via: cargo test${NC}"
echo -e "${YELLOW}  Count: (see cargo test -- --list)${NC}"
echo -e "${YELLOW}============================================${NC}"
echo ""

info "Running Rust unit tests (cargo test)..."
cargo test -- --nocapture && pass "Unit tests passed" || { fail "Unit tests failed"; exit 1; }

echo ""
echo -e "${YELLOW}============================================${NC}"
echo -e "${YELLOW}  INTEGRATION TESTS (Spin Environment)${NC}"
echo -e "${YELLOW}  Run via: test_spin_colored.sh${NC}"
echo -e "${YELLOW}  Count: (see test_spin_colored.sh)${NC}"
echo -e "${YELLOW}============================================${NC}"
echo ""

info "Building Spin app..."
spin build

info "Running integration tests (test_spin_colored.sh)..."
./test_spin_colored.sh && pass "Integration tests passed" || { fail "Integration tests failed"; exit 1; }

echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  ALL TESTS COMPLETE${NC}"
echo -e "${GREEN}  Unit tests: passed${NC}"
echo -e "${GREEN}  Integration tests: passed${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
