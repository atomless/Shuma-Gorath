#!/bin/bash
# test_all_colored.sh
# Runs all Rust unit tests and integration tests with colored output
set -e

# Always clean before building/testing to ensure correct crate-type
cargo clean
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
NC="\033[0m" # No Color

pass() { echo -e "${GREEN}PASS${NC} $1"; }
fail() { echo -e "${RED}FAIL${NC} $1"; }
info() { echo -e "${YELLOW}INFO${NC} $1"; }

info "Running Rust unit tests (cargo test)..."
cargo test -- --nocapture && pass "All Rust unit tests passed" || { fail "Rust unit tests failed"; exit 1; }

info "Building Spin app..."
spin build

info "Running integration tests (test_spin_colored.sh)..."
./test_spin_colored.sh && pass "All integration tests passed" || { fail "Integration tests failed"; exit 1; }

info "All tests complete."
