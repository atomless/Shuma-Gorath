#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_FILE="${ROOT_DIR}/scripts/supervisor/adversary_sim_supervisor.rs"
TARGET_BIN="${ROOT_DIR}/target/tools/adversary_sim_supervisor"

build_binary() {
  mkdir -p "$(dirname "${TARGET_BIN}")"
  if [[ ! -x "${TARGET_BIN}" || "${SOURCE_FILE}" -nt "${TARGET_BIN}" ]]; then
    rustc --edition=2021 -O "${SOURCE_FILE}" -o "${TARGET_BIN}"
  fi
}

if [[ "${1:-}" == "--build-only" ]]; then
  build_binary
  exit 0
fi

build_binary
exec "${TARGET_BIN}" "$@"
