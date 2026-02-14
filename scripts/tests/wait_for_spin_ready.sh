#!/usr/bin/env bash
set -euo pipefail

TIMEOUT_SECONDS=90
BASE_URL="${SHUMA_BASE_URL:-http://127.0.0.1:3000}"

usage() {
  cat <<'EOF'
Usage: wait_for_spin_ready.sh [--timeout-seconds N] [--base-url URL]

Waits until GET /health returns HTTP 200 with an "OK" body.
Reads SHUMA_FORWARDED_IP_SECRET and SHUMA_HEALTH_SECRET from environment,
falling back to .env.local when unset.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --timeout-seconds)
      TIMEOUT_SECONDS="${2:-}"
      shift 2
      ;;
    --base-url)
      BASE_URL="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! [[ "$TIMEOUT_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "Invalid --timeout-seconds value: ${TIMEOUT_SECONDS}" >&2
  exit 1
fi

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

if [[ -z "${SHUMA_FORWARDED_IP_SECRET:-}" ]]; then
  SHUMA_FORWARDED_IP_SECRET="$(read_env_local_value SHUMA_FORWARDED_IP_SECRET || true)"
fi
if [[ -z "${SHUMA_HEALTH_SECRET:-}" ]]; then
  SHUMA_HEALTH_SECRET="$(read_env_local_value SHUMA_HEALTH_SECRET || true)"
fi

headers=(-H "X-Forwarded-For: 127.0.0.1")
if [[ -n "${SHUMA_FORWARDED_IP_SECRET:-}" ]]; then
  headers+=(-H "X-Shuma-Forwarded-Secret: ${SHUMA_FORWARDED_IP_SECRET}")
fi
if [[ -n "${SHUMA_HEALTH_SECRET:-}" ]]; then
  headers+=(-H "X-Shuma-Health-Secret: ${SHUMA_HEALTH_SECRET}")
fi

last_status="000"
last_body=""
deadline=$((SECONDS + TIMEOUT_SECONDS))

while (( SECONDS <= deadline )); do
  response="$(curl -s --max-time 2 "${headers[@]}" -w $'\n__HTTP_STATUS__:%{http_code}' "${BASE_URL}/health" 2>/dev/null || true)"
  body="${response%$'\n'__HTTP_STATUS__:*}"
  status="${response##*$'\n'__HTTP_STATUS__:}"

  if [[ "$status" == "200" ]] && grep -q "OK" <<< "$body"; then
    echo "Spin server is ready at ${BASE_URL}/health"
    exit 0
  fi

  last_status="$status"
  last_body="$body"
  sleep 1
done

echo "Timed out waiting for Spin server after ${TIMEOUT_SECONDS}s (${BASE_URL}/health)." >&2
if [[ "$last_status" == "403" ]]; then
  echo "Last status was 403; verify SHUMA_FORWARDED_IP_SECRET and SHUMA_HEALTH_SECRET." >&2
fi
if [[ -n "$last_body" ]]; then
  short_body="$(printf '%s' "$last_body" | tr '\n' ' ' | cut -c1-180)"
  echo "Last response body (truncated): ${short_body}" >&2
fi
exit 1
