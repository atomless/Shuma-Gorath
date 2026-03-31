#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PLAYWRIGHT_BROWSERS_PATH_USER_SET=0
if [[ -n "${PLAYWRIGHT_BROWSERS_PATH:-}" ]]; then
  PLAYWRIGHT_BROWSERS_PATH_USER_SET=1
fi
PLAYWRIGHT_BROWSER_CACHE="${PLAYWRIGHT_BROWSERS_PATH:-${ROOT_DIR}/.cache/ms-playwright}"
PLAYWRIGHT_LOCAL_HOME="${PLAYWRIGHT_HOME:-${ROOT_DIR}/.cache/playwright-home}"
ORIGINAL_HOME="${HOME:-}"
PLAYWRIGHT_FORCE_LOCAL_HOME="${PLAYWRIGHT_FORCE_LOCAL_HOME:-1}"
using_repo_browser_cache=true

mkdir -p "${PLAYWRIGHT_BROWSER_CACHE}"

export PLAYWRIGHT_BROWSERS_PATH="${PLAYWRIGHT_BROWSER_CACHE}"
using_local_home=false
if [[ "${PLAYWRIGHT_FORCE_LOCAL_HOME}" == "1" ]]; then
  mkdir -p "${PLAYWRIGHT_LOCAL_HOME}/.config" \
           "${PLAYWRIGHT_LOCAL_HOME}/Library/Application Support/Chromium/Crashpad"
  export HOME="${PLAYWRIGHT_LOCAL_HOME}"
  export CFFIXED_USER_HOME="${PLAYWRIGHT_LOCAL_HOME}"
  export XDG_CONFIG_HOME="${PLAYWRIGHT_LOCAL_HOME}/.config"
  using_local_home=true
fi

ensure_playwright_chromium() {
  local chromium_path=""
  chromium_path="$(
    corepack pnpm exec node -e "const { chromium } = require('@playwright/test'); process.stdout.write(chromium.executablePath() || '');" 2>/dev/null || true
  )"
  if [[ -n "${chromium_path}" && -x "${chromium_path}" ]]; then
    return 0
  fi
  if [[ -n "${PLAYWRIGHT_BROWSERS_PATH:-}" ]]; then
    echo "Installing Playwright Chromium runtime into ${PLAYWRIGHT_BROWSERS_PATH}..."
  else
    echo "Installing Playwright Chromium runtime into system Playwright browser cache..."
  fi
  corepack pnpm exec playwright install chromium
}

cleanup_loopback_bans() {
  if ! command -v make >/dev/null 2>&1; then
    return 0
  fi
  if [[ -z "${SHUMA_API_KEY:-}" ]]; then
    return 0
  fi
  make --no-print-directory clear-dev-loopback-bans >/dev/null 2>&1 || true
}

run_preflight() {
  local status=0
  corepack pnpm exec node scripts/tests/verify_playwright_launch.mjs || status=$?
  return "${status}"
}

ensure_playwright_chromium

cleanup_loopback_bans
trap cleanup_loopback_bans EXIT

status=0
run_preflight || status=$?
if [[ "$status" -eq 42 && "${using_local_home}" == "true" && -n "${ORIGINAL_HOME}" ]]; then
  echo "Playwright launch failed under repo-local HOME; retrying preflight with system HOME..."
  using_local_home=false
  export HOME="${ORIGINAL_HOME}"
  unset CFFIXED_USER_HOME
  unset XDG_CONFIG_HOME
  status=0
  run_preflight || status=$?
fi

if [[ "$status" -eq 42 && "${PLAYWRIGHT_BROWSERS_PATH_USER_SET}" == "0" && -n "${ORIGINAL_HOME}" ]]; then
  echo "Playwright launch failed with repo-local browser cache; retrying preflight with system browser cache..."
  using_repo_browser_cache=false
  unset PLAYWRIGHT_BROWSERS_PATH
  ensure_playwright_chromium
  status=0
  run_preflight || status=$?
fi

if [[ "${status:-0}" -ne 0 ]]; then
  if [[ "${PLAYWRIGHT_SANDBOX_ALLOW_SKIP:-0}" == "1" ]]; then
    if [[ -n "${CI:-}" && "${CI}" != "0" ]]; then
      echo "PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1 is not allowed in CI; dashboard e2e must run."
      exit "${status}"
    fi
    echo "Playwright Chromium launch is blocked in this environment; skipping dashboard e2e because PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1."
    exit 0
  fi
  exit "$status"
fi

if [[ "${using_local_home}" == "true" ]]; then
  echo "Playwright preflight succeeded with repo-local HOME (${PLAYWRIGHT_LOCAL_HOME}) and repo-local browser cache (${PLAYWRIGHT_BROWSER_CACHE})."
elif [[ "${using_repo_browser_cache}" == "true" ]]; then
  echo "Playwright preflight succeeded with system HOME (${HOME}) and repo-local browser cache (${PLAYWRIGHT_BROWSER_CACHE})."
else
  echo "Playwright preflight succeeded with system HOME (${HOME}) and system Playwright browser cache."
fi

corepack pnpm run test:dashboard:e2e:raw "$@"
