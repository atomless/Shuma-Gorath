#!/usr/bin/env bash
# smoke_single_host.sh
#
# Post-deploy smoke checks for self_hosted_minimal posture:
#   - health endpoint
#   - admin auth enforcement
#   - metrics endpoint
#   - challenge route sanity
#
# Usage:
#   ./scripts/tests/smoke_single_host.sh
#   ./scripts/tests/smoke_single_host.sh --base-url http://127.0.0.1:3000

set -euo pipefail

BASE_URL="${SHUMA_BASE_URL:-http://127.0.0.1:3000}"
FORWARDED_IP="${SHUMA_SMOKE_FORWARDED_IP:-127.0.0.1}"
CHALLENGE_PATH="${SHUMA_SMOKE_CHALLENGE_PATH:-}"
CHALLENGE_EXPECT="${SHUMA_SMOKE_CHALLENGE_EXPECT:-}"

GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
NC="\033[0m"

pass() { echo -e "${GREEN}PASS${NC} $1"; }
fail() { echo -e "${RED}FAIL${NC} $1"; exit 1; }
info() { echo -e "${YELLOW}INFO${NC} $1"; }

usage() {
  cat <<'EOF'
Usage: smoke_single_host.sh [options]

Options:
  --base-url URL             Base URL to test (default: SHUMA_BASE_URL or http://127.0.0.1:3000)
  --forwarded-ip IP          Value for X-Forwarded-For (default: SHUMA_SMOKE_FORWARDED_IP or 127.0.0.1)
  --challenge-path PATH      Challenge path to sanity-check (default: auto-detect from /admin/config)
  --challenge-expect REGEX   Regex expected in challenge response body (default: auto by challenge type)
  -h, --help                 Show help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base-url)
      BASE_URL="${2:-}"
      shift 2
      ;;
    --forwarded-ip)
      FORWARDED_IP="${2:-}"
      shift 2
      ;;
    --challenge-path)
      CHALLENGE_PATH="${2:-}"
      shift 2
      ;;
    --challenge-expect)
      CHALLENGE_EXPECT="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "Unknown argument: $1"
      ;;
  esac
done

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

if [[ -z "${SHUMA_API_KEY:-}" ]]; then
  SHUMA_API_KEY="$(read_env_local_value SHUMA_API_KEY || true)"
fi
if [[ -z "${SHUMA_API_KEY:-}" ]]; then
  fail "Missing SHUMA_API_KEY (export it or set it in .env.local)."
fi

if [[ -z "${SHUMA_FORWARDED_IP_SECRET:-}" ]]; then
  SHUMA_FORWARDED_IP_SECRET="$(read_env_local_value SHUMA_FORWARDED_IP_SECRET || true)"
fi
if [[ -z "${SHUMA_HEALTH_SECRET:-}" ]]; then
  SHUMA_HEALTH_SECRET="$(read_env_local_value SHUMA_HEALTH_SECRET || true)"
fi

FORWARDED_HEADERS=(-H "X-Forwarded-For: ${FORWARDED_IP}")
if [[ -n "${SHUMA_FORWARDED_IP_SECRET:-}" ]]; then
  FORWARDED_HEADERS+=(-H "X-Shuma-Forwarded-Secret: ${SHUMA_FORWARDED_IP_SECRET}")
fi

HEALTH_HEADERS=("${FORWARDED_HEADERS[@]}")
if [[ -n "${SHUMA_HEALTH_SECRET:-}" ]]; then
  HEALTH_HEADERS+=(-H "X-Shuma-Health-Secret: ${SHUMA_HEALTH_SECRET}")
fi

http_request() {
  local method="$1"
  local url="$2"
  shift 2
  local response
  response="$(
    curl -s --max-time 8 -X "$method" "$@" -w $'\n__HTTP_STATUS__:%{http_code}' "$url" 2>/dev/null || true
  )"
  HTTP_BODY="${response%$'\n'__HTTP_STATUS__:*}"
  HTTP_STATUS="${response##*$'\n'__HTTP_STATUS__:}"
}

body_matches_expect() {
  local pattern="$1"
  local body="$2"
  grep -Eq "$pattern" <<< "$body"
}

info "Smoke target: ${BASE_URL}"

http_request GET "${BASE_URL}/health" "${HEALTH_HEADERS[@]}"
if [[ "${HTTP_STATUS}" == "200" ]] && grep -q "OK" <<< "${HTTP_BODY}"; then
  pass "/health returns 200 + OK"
else
  fail "/health failed (status=${HTTP_STATUS})"
fi

http_request GET "${BASE_URL}/admin/config" "${FORWARDED_HEADERS[@]}"
if [[ "${HTTP_STATUS}" == "401" || "${HTTP_STATUS}" == "403" ]]; then
  pass "/admin/config requires auth"
else
  fail "/admin/config should reject unauthenticated access (status=${HTTP_STATUS})"
fi

http_request GET "${BASE_URL}/admin/config" "${FORWARDED_HEADERS[@]}" -H "Authorization: Bearer ${SHUMA_API_KEY}"
if [[ "${HTTP_STATUS}" == "200" ]] && grep -q '"rate_limit"' <<< "${HTTP_BODY}"; then
  pass "/admin/config accepts authenticated access"
else
  fail "/admin/config auth check failed (status=${HTTP_STATUS})"
fi
ADMIN_CONFIG_BODY="${HTTP_BODY}"

http_request GET "${BASE_URL}/metrics" "${FORWARDED_HEADERS[@]}"
if [[ "${HTTP_STATUS}" == "200" ]] && grep -q "bot_defence_requests_total" <<< "${HTTP_BODY}"; then
  pass "/metrics returns Prometheus families"
else
  fail "/metrics check failed (status=${HTTP_STATUS})"
fi

if [[ -z "${CHALLENGE_PATH}" || -z "${CHALLENGE_EXPECT}" ]]; then
  auto_probe="$(
    python3 -c '
import json
import sys
try:
    cfg = json.loads(sys.stdin.read())
except Exception:
    cfg = {}
not_a_bot = bool(cfg.get("not_a_bot_enabled", True))
puzzle = bool(cfg.get("challenge_puzzle_enabled", True))
if not_a_bot and puzzle:
    print("/challenge/not-a-bot-checkbox")
    print("I am not a bot")
elif puzzle:
    print("/challenge/puzzle")
    print("Puzzle")
else:
    print("/")
    print("JavaScript|Proof-of-work|Verifying|Access Blocked|data-link-kind=\"maze\"")
' <<< "${ADMIN_CONFIG_BODY}" 2>/dev/null || true
  )"
  if [[ -z "${CHALLENGE_PATH}" ]]; then
    CHALLENGE_PATH="$(printf '%s\n' "${auto_probe}" | sed -n '1p')"
  fi
  if [[ -z "${CHALLENGE_EXPECT}" ]]; then
    CHALLENGE_EXPECT="$(printf '%s\n' "${auto_probe}" | sed -n '2p')"
  fi
fi

if [[ -z "${CHALLENGE_PATH}" ]]; then
  CHALLENGE_PATH="/challenge/not-a-bot-checkbox"
fi
if [[ -z "${CHALLENGE_EXPECT}" ]]; then
  CHALLENGE_EXPECT="I am not a bot|Puzzle"
fi

http_request GET "${BASE_URL}${CHALLENGE_PATH}" "${FORWARDED_HEADERS[@]}" -H "User-Agent: ShumaSmoke/1.0"
if [[ "${HTTP_STATUS}" != "200" ]] || ! body_matches_expect "${CHALLENGE_EXPECT}" "${HTTP_BODY}"; then
  if [[ "${HTTP_STATUS}" == "404" && "${CHALLENGE_PATH}" != "/challenge/puzzle" ]]; then
    CHALLENGE_PATH="/challenge/puzzle"
    CHALLENGE_EXPECT="Puzzle"
    http_request GET "${BASE_URL}${CHALLENGE_PATH}" "${FORWARDED_HEADERS[@]}" -H "User-Agent: ShumaSmoke/1.0"
  fi
fi

if [[ "${HTTP_STATUS}" != "200" ]] || ! body_matches_expect "${CHALLENGE_EXPECT}" "${HTTP_BODY}"; then
  if [[ "${HTTP_STATUS}" == "404" && "${CHALLENGE_PATH}" != "/" ]]; then
    CHALLENGE_PATH="/"
    CHALLENGE_EXPECT="JavaScript|Proof-of-work|Verifying|Access Blocked|data-link-kind=\"maze\""
    http_request GET "${BASE_URL}${CHALLENGE_PATH}" "${FORWARDED_HEADERS[@]}" -H "User-Agent: ShumaSmoke/1.0"
  fi
fi

if [[ "${HTTP_STATUS}" == "200" ]] && grep -Eq "${CHALLENGE_EXPECT}" <<< "${HTTP_BODY}"; then
  pass "${CHALLENGE_PATH} challenge route responds with expected content"
else
  fail "${CHALLENGE_PATH} sanity check failed (status=${HTTP_STATUS})"
fi

echo -e "${GREEN}Single-host smoke checks passed.${NC}"
