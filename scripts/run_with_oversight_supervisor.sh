#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "usage: $0 <command...>" >&2
  exit 2
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_PID=""
OVERSIGHT_MANAGER_PID=""

BASE_URL="${SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL:-http://127.0.0.1:3000}"
ADMIN_API_KEY="${SHUMA_API_KEY:-}"
FORWARDED_SECRET="${SHUMA_FORWARDED_IP_SECRET:-}"
PERIODIC_INTERVAL_SECONDS=300

post_periodic_agent_run() {
  if [[ -z "${ADMIN_API_KEY}" ]]; then
    return 1
  fi

  local headers=(
    -H "Authorization: Bearer ${ADMIN_API_KEY}"
    -H "Content-Type: application/json"
    -H "X-Forwarded-For: 127.0.0.1"
    -H "X-Forwarded-Proto: https"
    -H "X-Shuma-Internal-Supervisor: oversight-agent"
  )
  if [[ -n "${FORWARDED_SECRET}" ]]; then
    headers+=(-H "X-Shuma-Forwarded-Secret: ${FORWARDED_SECRET}")
  fi

  curl -fsS --max-time 5 -X POST \
    "${headers[@]}" \
    "${BASE_URL}/internal/oversight/agent/run" \
    --data '{"trigger_kind":"periodic_supervisor"}' >/dev/null
}

run_oversight_manager() {
  while kill -0 "${APP_PID}" 2>/dev/null; do
    post_periodic_agent_run || true
    sleep "${PERIODIC_INTERVAL_SECONDS}"
  done
}

cleanup() {
  if [[ -n "${OVERSIGHT_MANAGER_PID}" ]]; then
    kill "${OVERSIGHT_MANAGER_PID}" 2>/dev/null || true
    wait "${OVERSIGHT_MANAGER_PID}" 2>/dev/null || true
    OVERSIGHT_MANAGER_PID=""
  fi
}

trap cleanup EXIT INT TERM

"${ROOT_DIR}/scripts/run_with_adversary_sim_supervisor.sh" "$@" &
APP_PID=$!

if [[ -z "${ADMIN_API_KEY}" ]]; then
  echo "[oversight-supervisor] disabled: SHUMA_API_KEY is empty; cannot post /internal/oversight/agent/run" >&2
else
  run_oversight_manager &
  OVERSIGHT_MANAGER_PID=$!
fi

APP_EXIT=0
wait "${APP_PID}" || APP_EXIT=$?

cleanup
exit "${APP_EXIT}"
