#!/usr/bin/env python3
"""Adversarial runner preflight checks for required secret posture."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tests.playwright_runtime import (
    DEFAULT_PLAYWRIGHT_BROWSER_CACHE,
    ensure_playwright_chromium,
)

DEFAULT_OUTPUT_PATH = Path("scripts/tests/adversarial/preflight_report.json")
ENV_LOCAL_PATH = Path(".env.local")
PLACEHOLDER_VALUES = {
    "changeme-dev-only-api-key",
    "changeme-supersecret",
    "changeme-prod-api-key",
    "changeme-dev-only-sim-telemetry-secret",
}
HEX_PATTERN = re.compile(r"^[0-9a-fA-F]+$")


def read_env_local_value(key: str) -> str:
    if not ENV_LOCAL_PATH.exists():
        return ""
    for raw_line in ENV_LOCAL_PATH.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line.startswith(f"{key}="):
            continue
        value = line.split("=", 1)[1].strip().strip('"').strip("'")
        return value
    return ""


def env_or_local(key: str) -> str:
    value = str(os.environ.get(key) or "").strip()
    if value:
        return value
    return read_env_local_value(key).strip()


def check_api_key(values: Dict[str, str]) -> List[str]:
    failures: List[str] = []
    api_key = values.get("SHUMA_API_KEY", "")
    if not api_key:
        failures.append("preflight_missing_secret:SHUMA_API_KEY")
    elif api_key in PLACEHOLDER_VALUES:
        failures.append("preflight_placeholder_secret:SHUMA_API_KEY")
    return failures


def check_sim_telemetry_secret(values: Dict[str, str]) -> List[str]:
    failures: List[str] = []
    secret = values.get("SHUMA_SIM_TELEMETRY_SECRET", "")
    if not secret:
        failures.append("preflight_missing_secret:SHUMA_SIM_TELEMETRY_SECRET")
        return failures
    if secret in PLACEHOLDER_VALUES:
        failures.append("preflight_placeholder_secret:SHUMA_SIM_TELEMETRY_SECRET")
    if len(secret) < 64 or not HEX_PATTERN.match(secret):
        failures.append("preflight_invalid_secret_format:SHUMA_SIM_TELEMETRY_SECRET")
    return failures


def evaluate(
    values: Dict[str, str], browser_runtime_status: Dict[str, Any] | None = None
) -> Dict[str, Any]:
    failures: List[str] = []
    failures.extend(check_api_key(values))
    failures.extend(check_sim_telemetry_secret(values))
    browser_runtime_status = dict(browser_runtime_status or {})
    browser_runtime_failure = str(browser_runtime_status.get("failure") or "").strip()
    if browser_runtime_failure:
        failures.append(browser_runtime_failure)
    guidance: List[str] = []
    if failures:
        guidance.append("Run make setup to generate local dev secrets and normalize .env.local.")
        guidance.append(
            "Or export SHUMA_API_KEY and SHUMA_SIM_TELEMETRY_SECRET with non-placeholder values before adversarial runs."
        )
        guidance.append(
            "If run output shows sim-tag signature mismatch, rotate SHUMA_SIM_TELEMETRY_SECRET and restart runtime before retry."
        )
        guidance.append(
            "If run output shows sim-tag nonce replay, reset runtime-dev state and rerun with fresh sim-tag envelopes."
        )
    if browser_runtime_failure:
        guidance.append(
            "Ensure Node/pnpm dependencies are installed, then rerun make test-adversarial-preflight "
            "so the repo-local Playwright Chromium runtime can be provisioned."
        )
    return {
        "schema_version": "adversarial-preflight.v1",
        "status": {
            "passed": len(failures) == 0,
            "failure_count": len(failures),
            "failures": failures,
        },
        "checked_values": {
            "SHUMA_API_KEY_present": bool(values.get("SHUMA_API_KEY")),
            "SHUMA_SIM_TELEMETRY_SECRET_present": bool(
                values.get("SHUMA_SIM_TELEMETRY_SECRET")
            ),
            "PLAYWRIGHT_BROWSER_CACHE": str(
                browser_runtime_status.get("browser_cache")
                or DEFAULT_PLAYWRIGHT_BROWSER_CACHE
            ),
            "PLAYWRIGHT_CHROMIUM_present": bool(
                browser_runtime_status.get("chromium_executable")
            ),
        },
        "guidance": guidance,
    }


def gather_browser_runtime_status() -> Dict[str, Any]:
    try:
        status = ensure_playwright_chromium()
    except RuntimeError as exc:
        return {
            "passed": False,
            "failure": "preflight_missing_playwright_chromium",
            "browser_cache": str(DEFAULT_PLAYWRIGHT_BROWSER_CACHE),
            "chromium_executable": "",
            "detail": str(exc),
        }
    return {
        "passed": True,
        "failure": "",
        "browser_cache": status.browser_cache,
        "chromium_executable": status.chromium_executable,
        "installed_now": bool(status.installed_now),
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate required secrets before adversarial simulation targets run."
    )
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT_PATH))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    values = {
        "SHUMA_API_KEY": env_or_local("SHUMA_API_KEY"),
        "SHUMA_SIM_TELEMETRY_SECRET": env_or_local("SHUMA_SIM_TELEMETRY_SECRET"),
    }
    browser_runtime_status = gather_browser_runtime_status()
    payload = evaluate(values, browser_runtime_status=browser_runtime_status)
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"[adversarial-preflight] report={output_path}")
    if browser_runtime_status.get("passed"):
        state = "installed" if browser_runtime_status.get("installed_now") else "ready"
        print(
            "[adversarial-preflight] Playwright Chromium "
            f"{state} at {browser_runtime_status.get('chromium_executable')}"
        )
    if not bool(dict(payload.get("status") or {}).get("passed")):
        print("[adversarial-preflight] FAIL")
        for failure in list(dict(payload.get("status") or {}).get("failures") or []):
            print(f"- {failure}")
        detail = str(browser_runtime_status.get("detail") or "").strip()
        if detail:
            print(f"- detail: {detail}")
        for line in list(payload.get("guidance") or []):
            print(f"- guidance: {line}")
        return 1
    print("[adversarial-preflight] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
