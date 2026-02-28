#!/usr/bin/env python3
"""Adversarial runner preflight checks for required secret posture."""

from __future__ import annotations

import argparse
import json
import os
import re
from pathlib import Path
from typing import Any, Dict, List


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


def evaluate(values: Dict[str, str]) -> Dict[str, Any]:
    failures: List[str] = []
    failures.extend(check_api_key(values))
    failures.extend(check_sim_telemetry_secret(values))
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
        },
        "guidance": guidance,
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
    payload = evaluate(values)
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"[adversarial-preflight] report={output_path}")
    if not bool(dict(payload.get("status") or {}).get("passed")):
        print("[adversarial-preflight] FAIL")
        for failure in list(dict(payload.get("status") or {}).get("failures") or []):
            print(f"- {failure}")
        for line in list(payload.get("guidance") or []):
            print(f"- guidance: {line}")
        return 1
    print("[adversarial-preflight] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
