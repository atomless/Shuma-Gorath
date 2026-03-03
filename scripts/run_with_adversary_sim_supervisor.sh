#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "usage: $0 <command...>" >&2
  exit 2
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUPERVISOR_MANAGER_PID=""
APP_PID=""

SIM_AVAILABLE_RAW="${SHUMA_ADVERSARY_SIM_AVAILABLE:-false}"
SIM_AVAILABLE="$(printf '%s' "${SIM_AVAILABLE_RAW}" | tr '[:upper:]' '[:lower:]')"
SUPERVISOR_ENABLED_RAW="${SHUMA_ADVERSARY_SIM_SUPERVISOR_ENABLE:-1}"
SUPERVISOR_ENABLED="$(printf '%s' "${SUPERVISOR_ENABLED_RAW}" | tr '[:upper:]' '[:lower:]')"
SUPERVISOR_MANAGER_POLL_SECONDS="${SHUMA_ADVERSARY_SIM_SUPERVISOR_MANAGER_POLL_SECONDS:-1}"
BASE_URL="${SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL:-http://127.0.0.1:3000}"
ADMIN_API_KEY="${SHUMA_API_KEY:-}"
FORWARDED_SECRET="${SHUMA_FORWARDED_IP_SECRET:-}"

is_generation_active() {
  if [[ -z "${ADMIN_API_KEY}" ]]; then
    return 1
  fi

  local headers=(-H "Authorization: Bearer ${ADMIN_API_KEY}" -H "X-Forwarded-For: 127.0.0.1")
  if [[ -n "${FORWARDED_SECRET}" ]]; then
    headers+=(-H "X-Shuma-Forwarded-Secret: ${FORWARDED_SECRET}")
  fi

  local payload
  if ! payload="$(curl -fsS --max-time 2 "${headers[@]}" "${BASE_URL}/admin/adversary-sim/status" 2>/dev/null)"; then
    return 1
  fi

  if [[ "${payload}" == *"\"generation_active\":true"* ]]; then
    return 0
  fi
  return 1
}

run_supervisor_manager() {
  local worker_pid=""

  cleanup_worker() {
    if [[ -n "${worker_pid}" ]]; then
      kill "${worker_pid}" 2>/dev/null || true
      wait "${worker_pid}" 2>/dev/null || true
      worker_pid=""
    fi
  }

  trap cleanup_worker EXIT INT TERM

  while kill -0 "${APP_PID}" 2>/dev/null; do
    if [[ -n "${worker_pid}" ]] && ! kill -0 "${worker_pid}" 2>/dev/null; then
      worker_pid=""
    fi

    if is_generation_active; then
      if [[ -z "${worker_pid}" ]]; then
        SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL="${BASE_URL}" \
          SHUMA_ADVERSARY_SIM_SUPERVISOR_EXIT_WHEN_OFF=1 \
          "${ROOT_DIR}/scripts/adversary_sim_supervisor_launch.sh" --exit-when-off --base-url "${BASE_URL}" &
        worker_pid=$!
      fi
    fi

    sleep "${SUPERVISOR_MANAGER_POLL_SECONDS}"
  done
}

cleanup() {
  if [[ -n "${SUPERVISOR_MANAGER_PID}" ]]; then
    kill "${SUPERVISOR_MANAGER_PID}" 2>/dev/null || true
    wait "${SUPERVISOR_MANAGER_PID}" 2>/dev/null || true
    SUPERVISOR_MANAGER_PID=""
  fi
}

trap cleanup EXIT INT TERM

"$@" &
APP_PID=$!

if [[ "${SIM_AVAILABLE}" == "true" || "${SIM_AVAILABLE}" == "1" || "${SIM_AVAILABLE}" == "yes" || "${SIM_AVAILABLE}" == "on" ]]; then
  if [[ "${SUPERVISOR_ENABLED}" == "1" || "${SUPERVISOR_ENABLED}" == "true" || "${SUPERVISOR_ENABLED}" == "yes" || "${SUPERVISOR_ENABLED}" == "on" ]]; then
    if [[ -z "${ADMIN_API_KEY}" ]]; then
      echo "[adversary-sim-supervisor-manager] disabled: SHUMA_API_KEY is empty; cannot poll /admin/adversary-sim/status" >&2
    else
      run_supervisor_manager &
      SUPERVISOR_MANAGER_PID=$!
    fi
  fi
fi

APP_EXIT=0
wait "${APP_PID}" || APP_EXIT=$?

cleanup
exit "${APP_EXIT}"
