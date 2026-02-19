#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:3000}"
LOCAL_INDEX="${2:-dist/dashboard/index.html}"
REMOTE_INDEX_URL="${BASE_URL%/}/dashboard/index.html"

if [[ ! -f "${LOCAL_INDEX}" ]]; then
  echo "Dashboard asset verification failed: local file missing (${LOCAL_INDEX})."
  echo "Run: make dashboard-build"
  exit 1
fi

TMP_REMOTE="$(mktemp)"
cleanup() {
  rm -f "${TMP_REMOTE}"
}
trap cleanup EXIT

if ! curl -fsS "${REMOTE_INDEX_URL}" -o "${TMP_REMOTE}"; then
  echo "Dashboard asset verification failed: unable to fetch ${REMOTE_INDEX_URL}."
  exit 1
fi

if cmp -s "${LOCAL_INDEX}" "${TMP_REMOTE}"; then
  echo "Dashboard asset verification passed: Spin is serving current dist/dashboard assets."
  exit 0
fi

echo "Dashboard asset verification failed: Spin is serving stale dashboard assets."
echo "Expected latest local build: ${LOCAL_INDEX}"
echo "Fetched from server: ${REMOTE_INDEX_URL}"
echo "Restart Spin after dashboard build (for example, restart \`make dev\` or \`make run-prebuilt\`) and rerun tests."
exit 1

