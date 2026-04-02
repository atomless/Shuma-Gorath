#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "usage: $0 <command...>" >&2
  exit 2
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUPERVISOR_MANAGER_PID=""
SUPERVISOR_WORKER_PID=""
TRUSTED_INGRESS_PROXY_PID=""
PUBLIC_INGRESS_PROXY_PID=""
APP_PID=""

SIM_AVAILABLE_RAW="${SHUMA_ADVERSARY_SIM_AVAILABLE:-true}"
SIM_AVAILABLE="$(printf '%s' "${SIM_AVAILABLE_RAW}" | tr '[:upper:]' '[:lower:]')"
SUPERVISOR_ENABLED_RAW="${SHUMA_ADVERSARY_SIM_SUPERVISOR_ENABLE:-1}"
SUPERVISOR_ENABLED="$(printf '%s' "${SUPERVISOR_ENABLED_RAW}" | tr '[:upper:]' '[:lower:]')"
SUPERVISOR_MANAGER_POLL_SECONDS="${SHUMA_ADVERSARY_SIM_SUPERVISOR_MANAGER_POLL_SECONDS:-1}"
BASE_URL="${SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL:-http://127.0.0.1:3000}"
ADMIN_API_KEY="${SHUMA_API_KEY:-}"
FORWARDED_SECRET="${SHUMA_FORWARDED_IP_SECRET:-}"
TRUSTED_INGRESS_PROXY_URL="${ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL:-}"
TRUSTED_INGRESS_AUTH_TOKEN="${ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN:-}"
TRUSTED_INGRESS_LISTEN_HOST="${ADVERSARY_SIM_TRUSTED_INGRESS_LISTEN_HOST:-127.0.0.1}"
TRUSTED_INGRESS_LISTEN_PORT="${ADVERSARY_SIM_TRUSTED_INGRESS_LISTEN_PORT:-3871}"
LOCAL_CONTRIBUTOR_INGRESS_RAW="${SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE:-0}"
LOCAL_CONTRIBUTOR_INGRESS_ENABLED="$(printf '%s' "${LOCAL_CONTRIBUTOR_INGRESS_RAW}" | tr '[:upper:]' '[:lower:]')"
LOCAL_CONTRIBUTOR_ORIGIN_BASE_URL="${SHUMA_LOCAL_CONTRIBUTOR_ORIGIN_BASE_URL:-${BASE_URL}}"
LOCAL_CONTRIBUTOR_PUBLIC_INGRESS_LISTEN_HOST="${SHUMA_LOCAL_CONTRIBUTOR_PUBLIC_INGRESS_LISTEN_HOST:-127.0.0.1}"
LOCAL_CONTRIBUTOR_PUBLIC_INGRESS_LISTEN_PORT="${SHUMA_LOCAL_CONTRIBUTOR_PUBLIC_INGRESS_LISTEN_PORT:-3000}"
LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING_RAW="${SHUMA_LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING:-0}"
LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING="$(printf '%s' "${LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING_RAW}" | tr '[:upper:]' '[:lower:]')"
LOCAL_CONTRIBUTOR_DIRECT_CLIENT_IP="${SHUMA_LOCAL_CONTRIBUTOR_DIRECT_CLIENT_IP:-127.0.0.1}"

supervisor_attention_required() {
  if [[ -z "${ADMIN_API_KEY}" ]]; then
    return 1
  fi

  local headers=(
    -H "Authorization: Bearer ${ADMIN_API_KEY}"
    -H "X-Forwarded-For: 127.0.0.1"
    -H "X-Forwarded-Proto: https"
    -H "X-Shuma-Internal-Supervisor: adversary-sim"
  )
  if [[ -n "${FORWARDED_SECRET}" ]]; then
    headers+=(-H "X-Shuma-Forwarded-Secret: ${FORWARDED_SECRET}")
  fi

  local payload
  if ! payload="$(curl -fsS --max-time 2 "${headers[@]}" "${BASE_URL}/shuma/admin/adversary-sim/status" 2>/dev/null)"; then
    return 1
  fi

  if [[ "${payload}" == *"\"generation_active\":true"* || "${payload}" == *"\"supervisor_attention_required\":true"* ]]; then
    return 0
  fi
  return 1
}

cleanup_worker() {
  if [[ -n "${SUPERVISOR_WORKER_PID}" ]]; then
    kill "${SUPERVISOR_WORKER_PID}" 2>/dev/null || true
    wait "${SUPERVISOR_WORKER_PID}" 2>/dev/null || true
    SUPERVISOR_WORKER_PID=""
  fi
}

cleanup_trusted_ingress_proxy() {
  if [[ -n "${TRUSTED_INGRESS_PROXY_PID}" ]]; then
    kill "${TRUSTED_INGRESS_PROXY_PID}" 2>/dev/null || true
    wait "${TRUSTED_INGRESS_PROXY_PID}" 2>/dev/null || true
    TRUSTED_INGRESS_PROXY_PID=""
  fi
  if [[ -n "${PUBLIC_INGRESS_PROXY_PID}" ]]; then
    kill "${PUBLIC_INGRESS_PROXY_PID}" 2>/dev/null || true
    wait "${PUBLIC_INGRESS_PROXY_PID}" 2>/dev/null || true
    PUBLIC_INGRESS_PROXY_PID=""
  fi
}

spin_command_has_env_binding() {
  local env_name="$1"
  shift
  local expect_binding=0
  local arg=""
  for arg in "$@"; do
    if [[ "${expect_binding}" == "1" ]]; then
      if [[ "${arg}" == "${env_name}="* ]]; then
        return 0
      fi
      expect_binding=0
      continue
    fi
    if [[ "${arg}" == "--env" ]]; then
      expect_binding=1
    fi
  done
  return 1
}

launch_app_command() {
  local app_command=("$@")

  if [[ $# -ge 2 && "$1" == "spin" && "$2" == "up" ]]; then
    if [[ -n "${ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL:-}" ]] && ! spin_command_has_env_binding "ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL" "${app_command[@]}"; then
      app_command+=(
        --env "ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL=${ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL}"
      )
    fi
    if [[ -n "${ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN:-}" ]] && ! spin_command_has_env_binding "ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN" "${app_command[@]}"; then
      app_command+=(
        --env "ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN=${ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN}"
      )
    fi
  fi

  "${app_command[@]}" &
  APP_PID=$!
}

start_trusted_ingress_proxy_if_needed() {
  local contributor_ingress_enabled=0
  if [[ "${LOCAL_CONTRIBUTOR_INGRESS_ENABLED}" == "1" || "${LOCAL_CONTRIBUTOR_INGRESS_ENABLED}" == "true" || "${LOCAL_CONTRIBUTOR_INGRESS_ENABLED}" == "yes" || "${LOCAL_CONTRIBUTOR_INGRESS_ENABLED}" == "on" ]]; then
    contributor_ingress_enabled=1
  fi

  if [[ -z "${FORWARDED_SECRET}" ]]; then
    if [[ "${contributor_ingress_enabled}" == "1" ]]; then
      echo "[adversary-sim-supervisor-manager] local contributor ingress requires SHUMA_FORWARDED_IP_SECRET" >&2
      exit 1
    fi
    return 0
  fi
  if [[ "${contributor_ingress_enabled}" != "1" && -n "${TRUSTED_INGRESS_PROXY_URL}" && -n "${TRUSTED_INGRESS_AUTH_TOKEN}" ]]; then
    export ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL="${TRUSTED_INGRESS_PROXY_URL}"
    export ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN="${TRUSTED_INGRESS_AUTH_TOKEN}"
    return 0
  fi

  TRUSTED_INGRESS_AUTH_TOKEN="$(
    python3 - <<'PY'
import secrets
print(secrets.token_urlsafe(18))
PY
  )"
  TRUSTED_INGRESS_PROXY_URL="http://${TRUSTED_INGRESS_LISTEN_HOST}:${TRUSTED_INGRESS_LISTEN_PORT}"
  local trusted_ingress_origin_base_url="${BASE_URL}"
  if [[ "${contributor_ingress_enabled}" == "1" ]]; then
    trusted_ingress_origin_base_url="${LOCAL_CONTRIBUTOR_ORIGIN_BASE_URL}"
  fi

  if ! command -v python3 >/dev/null 2>&1; then
    if [[ "${contributor_ingress_enabled}" == "1" ]]; then
      echo "[adversary-sim-supervisor-manager] local contributor ingress requires python3" >&2
      exit 1
    fi
    echo "[adversary-sim-supervisor-manager] trusted ingress disabled: python3 unavailable; sim worker IP realism will remain degraded" >&2
    TRUSTED_INGRESS_PROXY_URL=""
    TRUSTED_INGRESS_AUTH_TOKEN=""
    return 0
  fi

  local worker_proxy_args=(
    python3 "${ROOT_DIR}/scripts/supervisor/trusted_ingress_proxy.py"
    --listen-host "${TRUSTED_INGRESS_LISTEN_HOST}"
    --listen-port "${TRUSTED_INGRESS_LISTEN_PORT}"
    --public-base-url "${BASE_URL}"
    --origin-base-url "${trusted_ingress_origin_base_url}"
    --auth-token "${TRUSTED_INGRESS_AUTH_TOKEN}"
    --forwarded-secret "${FORWARDED_SECRET}"
  )
  local public_proxy_args=()
  if [[ "${contributor_ingress_enabled}" == "1" ]]; then
    public_proxy_args=(
      python3 "${ROOT_DIR}/scripts/supervisor/trusted_ingress_proxy.py"
      --listen-host "${LOCAL_CONTRIBUTOR_PUBLIC_INGRESS_LISTEN_HOST}"
      --listen-port "${LOCAL_CONTRIBUTOR_PUBLIC_INGRESS_LISTEN_PORT}"
      --public-base-url "${BASE_URL}"
      --origin-base-url "${trusted_ingress_origin_base_url}"
      --auth-token "${TRUSTED_INGRESS_AUTH_TOKEN}"
      --forwarded-secret "${FORWARDED_SECRET}"
      --allow-direct-browser-requests
      --direct-browser-client-ip "${LOCAL_CONTRIBUTOR_DIRECT_CLIENT_IP}"
    )
    if [[ "${LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING}" == "1" || "${LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING}" == "true" || "${LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING}" == "yes" || "${LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING}" == "on" ]]; then
      public_proxy_args+=(--allow-local-trusted-forwarded-passthrough)
    fi
  fi

  if [[ "${contributor_ingress_enabled}" == "1" ]]; then
    "${public_proxy_args[@]}" &
    PUBLIC_INGRESS_PROXY_PID=$!
    sleep 0.2
    if ! kill -0 "${PUBLIC_INGRESS_PROXY_PID}" 2>/dev/null; then
      echo "[adversary-sim-supervisor-manager] local contributor ingress failed to start" >&2
      cleanup_trusted_ingress_proxy
      exit 1
    fi
  fi

  "${worker_proxy_args[@]}" &
  TRUSTED_INGRESS_PROXY_PID=$!

  sleep 0.2
  if ! kill -0 "${TRUSTED_INGRESS_PROXY_PID}" 2>/dev/null; then
    if [[ "${contributor_ingress_enabled}" == "1" ]]; then
      echo "[adversary-sim-supervisor-manager] trusted ingress worker proxy failed to start" >&2
      cleanup_trusted_ingress_proxy
      exit 1
    fi
    echo "[adversary-sim-supervisor-manager] trusted ingress disabled: local proxy failed to start; sim worker IP realism will remain degraded" >&2
    cleanup_trusted_ingress_proxy
    TRUSTED_INGRESS_PROXY_URL=""
    TRUSTED_INGRESS_AUTH_TOKEN=""
    return 0
  fi

  export ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL="${TRUSTED_INGRESS_PROXY_URL}"
  export ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN="${TRUSTED_INGRESS_AUTH_TOKEN}"
}

run_supervisor_manager() {
  trap cleanup_worker EXIT INT TERM

  while kill -0 "${APP_PID}" 2>/dev/null; do
    if [[ -n "${SUPERVISOR_WORKER_PID}" ]] && ! kill -0 "${SUPERVISOR_WORKER_PID}" 2>/dev/null; then
      SUPERVISOR_WORKER_PID=""
    fi

    if supervisor_attention_required; then
      if [[ -z "${SUPERVISOR_WORKER_PID}" ]]; then
        SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL="${BASE_URL}" \
          SHUMA_ADVERSARY_SIM_SUPERVISOR_EXIT_WHEN_OFF=1 \
          "${ROOT_DIR}/scripts/adversary_sim_supervisor_launch.sh" --exit-when-off --base-url "${BASE_URL}" &
        SUPERVISOR_WORKER_PID=$!
      fi
    fi

    sleep "${SUPERVISOR_MANAGER_POLL_SECONDS}"
  done
}

cleanup() {
  cleanup_trusted_ingress_proxy
  if [[ -n "${SUPERVISOR_MANAGER_PID}" ]]; then
    kill "${SUPERVISOR_MANAGER_PID}" 2>/dev/null || true
    wait "${SUPERVISOR_MANAGER_PID}" 2>/dev/null || true
    SUPERVISOR_MANAGER_PID=""
  fi
}

trap cleanup EXIT INT TERM

start_trusted_ingress_proxy_if_needed

launch_app_command "$@"

if [[ "${SIM_AVAILABLE}" == "true" || "${SIM_AVAILABLE}" == "1" || "${SIM_AVAILABLE}" == "yes" || "${SIM_AVAILABLE}" == "on" ]]; then
  if [[ "${SUPERVISOR_ENABLED}" == "1" || "${SUPERVISOR_ENABLED}" == "true" || "${SUPERVISOR_ENABLED}" == "yes" || "${SUPERVISOR_ENABLED}" == "on" ]]; then
    if [[ -z "${ADMIN_API_KEY}" ]]; then
      echo "[adversary-sim-supervisor-manager] disabled: SHUMA_API_KEY is empty; cannot poll /shuma/admin/adversary-sim/status" >&2
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
