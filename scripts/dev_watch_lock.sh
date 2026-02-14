#!/bin/bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "Usage: $0 <command> [args...]" >&2
  exit 1
fi

LOCK_FILE=".spin/dev-watch.lock"

acquire_lock() {
  mkdir -p .spin

  if ln -s "$$" "${LOCK_FILE}" 2>/dev/null; then
    return
  fi

  local existing_pid
  existing_pid="$(readlink "${LOCK_FILE}" 2>/dev/null || true)"
  if [[ -n "${existing_pid}" ]] && kill -0 "${existing_pid}" 2>/dev/null; then
    echo "A dev watcher is already running (pid ${existing_pid}). Stop it first with 'make stop'." >&2
    exit 1
  fi

  rm -f "${LOCK_FILE}"
  if ! ln -s "$$" "${LOCK_FILE}" 2>/dev/null; then
    echo "Another dev watcher is starting. Retry in a moment." >&2
    exit 1
  fi
}

cleanup() {
  if [[ "$(readlink "${LOCK_FILE}" 2>/dev/null || true)" == "$$" ]]; then
    rm -f "${LOCK_FILE}"
  fi
}

acquire_lock
trap cleanup EXIT INT TERM

"$@"
