#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${SHUMA_BASE_URL:-http://127.0.0.1:3000}"
EXPECTED_RUNTIME_ENVIRONMENT="runtime-dev"

usage() {
  cat <<'EOF'
Usage: verify_test_runtime_environment.sh [--base-url URL] [--expected-runtime-environment runtime-dev|runtime-prod]

Checks GET /admin/session on an already-running local Spin server and verifies the
reported runtime_environment matches the expected full-suite test contract.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base-url)
      BASE_URL="${2:-}"
      shift 2
      ;;
    --expected-runtime-environment)
      EXPECTED_RUNTIME_ENVIRONMENT="${2:-}"
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

if [[ "${EXPECTED_RUNTIME_ENVIRONMENT}" != "runtime-dev" && "${EXPECTED_RUNTIME_ENVIRONMENT}" != "runtime-prod" ]]; then
  echo "Invalid --expected-runtime-environment value: ${EXPECTED_RUNTIME_ENVIRONMENT}" >&2
  exit 1
fi

response="$(curl -fsS --max-time 5 "${BASE_URL}/admin/session" 2>/dev/null || true)"
if [[ -z "${response}" ]]; then
  echo "Failed to read ${BASE_URL}/admin/session during full-suite runtime preflight." >&2
  echo "Ensure the local Spin server is running and reachable before invoking make test." >&2
  exit 1
fi

runtime_environment="$(
  python3 - <<'PY' "${response}"
import json
import sys

payload = sys.argv[1]
try:
    data = json.loads(payload)
except json.JSONDecodeError:
    print("")
    raise SystemExit(0)

value = data.get("runtime_environment")
print("" if value is None else str(value).strip())
PY
)"

if [[ -z "${runtime_environment}" ]]; then
  echo "Full-suite runtime preflight could not determine runtime_environment from ${BASE_URL}/admin/session." >&2
  echo "Expected runtime_environment=${EXPECTED_RUNTIME_ENVIRONMENT}." >&2
  exit 1
fi

if [[ "${runtime_environment}" != "${EXPECTED_RUNTIME_ENVIRONMENT}" ]]; then
  echo "make test requires a ${EXPECTED_RUNTIME_ENVIRONMENT} server from make dev." >&2
  echo "Current runtime_environment=${runtime_environment}." >&2
  if [[ "${EXPECTED_RUNTIME_ENVIRONMENT}" == "runtime-dev" ]]; then
    echo "Stop the current server and restart with make dev before rerunning make test." >&2
  fi
  exit 1
fi

echo "Full-suite runtime preflight passed (runtime_environment=${runtime_environment})."
