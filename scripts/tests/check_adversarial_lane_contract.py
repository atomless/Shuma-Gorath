#!/usr/bin/env python3
"""Validate black-box lane contract parity across adversarial tooling."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Dict, List

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

import scripts.tests.adversarial_container.worker as container_worker
import scripts.tests.adversarial_container_runner as container_runner
import scripts.tests.adversarial_simulation_runner as sim_runner


LANE_CONTRACT_PATH = Path("scripts/tests/adversarial/lane_contract.v1.json")


class LaneContractError(Exception):
    pass


def load_lane_contract(path: Path = LANE_CONTRACT_PATH) -> Dict[str, Any]:
    if not path.exists():
        raise LaneContractError(f"lane contract not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise LaneContractError(f"invalid lane contract JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise LaneContractError("lane contract must be a JSON object")
    return payload


def compare_sets(name: str, expected: set[str], observed: set[str], errors: List[str]) -> None:
    missing = sorted(expected - observed)
    extra = sorted(observed - expected)
    if missing:
        errors.append(f"{name}: missing entries: {', '.join(missing)}")
    if extra:
        errors.append(f"{name}: unexpected entries: {', '.join(extra)}")


def validate_lane_contract() -> List[str]:
    errors: List[str] = []
    payload = load_lane_contract()

    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != "sim-lane-contract.v1":
        errors.append(
            f"lane contract schema_version must be sim-lane-contract.v1 (got {schema_version})"
        )

    execution_lane = str(payload.get("execution_lane") or "").strip()
    if execution_lane != "black_box":
        errors.append(f"lane contract execution_lane must be black_box (got {execution_lane})")

    attacker = payload.get("attacker")
    if not isinstance(attacker, dict):
        errors.append("lane contract attacker section must be an object")
        return errors

    forbidden_headers = {
        str(item).strip().lower()
        for item in attacker.get("forbidden_headers", [])
        if str(item).strip()
    }
    required_sim_headers = {
        str(item).strip().lower()
        for item in attacker.get("required_sim_headers", [])
        if str(item).strip()
    }
    forbidden_paths = {
        str(item).strip() for item in attacker.get("forbidden_path_prefixes", []) if str(item).strip()
    }

    if "x-shuma-forwarded-secret" not in forbidden_headers:
        errors.append("lane contract must forbid x-shuma-forwarded-secret for attacker plane")

    compare_sets(
        "attacker.forbidden_headers parity",
        forbidden_headers,
        set(sim_runner.ATTACKER_FORBIDDEN_HEADERS),
        errors,
    )
    compare_sets(
        "attacker.required_sim_headers parity",
        required_sim_headers,
        set(sim_runner.ATTACKER_REQUIRED_SIM_HEADERS),
        errors,
    )
    compare_sets(
        "attacker.required_sim_headers parity (sim-tag contract)",
        required_sim_headers,
        set(sim_runner.SIM_TAG_REQUIRED_SIM_HEADERS),
        errors,
    )
    compare_sets(
        "attacker.forbidden_path_prefixes parity",
        forbidden_paths,
        set(sim_runner.ATTACKER_FORBIDDEN_PATH_PREFIXES),
        errors,
    )

    required_secret_keys = {
        "SHUMA_API_KEY",
        "SHUMA_ADMIN_READONLY_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_CHALLENGE_SECRET",
        "SHUMA_HEALTH_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_SIM_TELEMETRY_SECRET",
    }
    compare_sets(
        "container worker forbidden env keys",
        required_secret_keys,
        set(container_worker.FORBIDDEN_ENV_KEYS),
        errors,
    )
    compare_sets(
        "container runner forbidden env keys",
        required_secret_keys,
        set(container_runner.FORBIDDEN_ENV_KEYS),
        errors,
    )

    return errors


def main() -> int:
    try:
        errors = validate_lane_contract()
    except LaneContractError as exc:
        print(f"lane-contract validation failed: {exc}")
        return 1
    if errors:
        print("lane-contract validation failed:")
        for item in errors:
            print(f"- {item}")
        return 1
    print("lane-contract validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
