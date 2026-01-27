#!/bin/bash
# test_unit_colored.sh
# Runs all Rust unit tests with colored, bannered output
set -e
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
NC="\033[0m" # No Color

pass() { echo -e "${GREEN}PASS${NC} $1"; }
fail() { echo -e "${RED}FAIL${NC} $1"; }
info() { echo -e "${YELLOW}INFO${NC} $1"; }

info "Running Rust unit tests (cargo test)..."
CARGO_TERM_COLOR=always cargo test -- --nocapture | tee unit_test_output.log
if grep -q "test result: ok" unit_test_output.log; then
  pass "All Rust unit tests passed"
else
  fail "Some Rust unit tests failed"
  exit 1
fi
