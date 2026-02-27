#!/usr/bin/env python3
"""Validate simulation tag signing contract parity across tooling."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Dict, List

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

import scripts.tests.adversarial_container.worker as container_worker
import scripts.tests.adversarial_simulation_runner as sim_runner


SIM_TAG_CONTRACT_PATH = Path("scripts/tests/adversarial/sim_tag_contract.v1.json")
LANE_CONTRACT_PATH = Path("scripts/tests/adversarial/lane_contract.v1.json")


class SimTagContractError(Exception):
    pass


def load_json_object(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise SimTagContractError(f"missing file: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise SimTagContractError(f"invalid JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise SimTagContractError(f"expected JSON object: {path}")
    return payload


def compare_sets(name: str, expected: set[str], observed: set[str], errors: List[str]) -> None:
    missing = sorted(expected - observed)
    extra = sorted(observed - expected)
    if missing:
        errors.append(f"{name}: missing entries: {', '.join(missing)}")
    if extra:
        errors.append(f"{name}: unexpected entries: {', '.join(extra)}")


def validate_sim_tag_contract() -> List[str]:
    errors: List[str] = []
    contract = load_json_object(SIM_TAG_CONTRACT_PATH)
    lane_contract = load_json_object(LANE_CONTRACT_PATH)

    schema_version = str(contract.get("schema_version") or "").strip()
    if schema_version != "sim-tag-contract.v1":
        errors.append(
            f"sim tag contract schema_version must be sim-tag-contract.v1 (got {schema_version})"
        )

    required_headers = {
        str(value).strip().lower()
        for value in list(contract.get("required_sim_headers") or [])
        if str(value).strip()
    }
    if not required_headers:
        errors.append("sim tag contract required_sim_headers must be non-empty")

    compare_sets(
        "required sim headers parity (runner)",
        required_headers,
        set(sim_runner.SIM_TAG_REQUIRED_SIM_HEADERS),
        errors,
    )
    compare_sets(
        "required sim headers parity (container worker)",
        required_headers,
        {
            container_worker.SIM_TAG_HEADER_RUN_ID,
            container_worker.SIM_TAG_HEADER_PROFILE,
            container_worker.SIM_TAG_HEADER_LANE,
            container_worker.SIM_TAG_HEADER_TIMESTAMP,
            container_worker.SIM_TAG_HEADER_NONCE,
            container_worker.SIM_TAG_HEADER_SIGNATURE,
        },
        errors,
    )

    lane_required_headers = {
        str(value).strip().lower()
        for value in list(
            dict(lane_contract.get("attacker") or {}).get("required_sim_headers") or []
        )
        if str(value).strip()
    }
    compare_sets(
        "required sim headers parity (lane contract)",
        required_headers,
        lane_required_headers,
        errors,
    )

    header_map = {
        str(key).strip(): str(value).strip().lower()
        for key, value in dict(contract.get("headers") or {}).items()
        if str(key).strip()
    }
    expected_header_map = {
        "sim_run_id": sim_runner.SIM_TAG_HEADER_RUN_ID,
        "sim_profile": sim_runner.SIM_TAG_HEADER_PROFILE,
        "sim_lane": sim_runner.SIM_TAG_HEADER_LANE,
        "sim_timestamp": sim_runner.SIM_TAG_HEADER_TIMESTAMP,
        "sim_nonce": sim_runner.SIM_TAG_HEADER_NONCE,
        "sim_signature": sim_runner.SIM_TAG_HEADER_SIGNATURE,
    }
    for key, runner_value in expected_header_map.items():
        contract_value = header_map.get(key)
        if contract_value != runner_value:
            errors.append(
                f"sim-tag contract header drift for {key}: contract={contract_value} runner={runner_value}"
            )
    worker_header_map = {
        "sim_run_id": container_worker.SIM_TAG_HEADER_RUN_ID,
        "sim_profile": container_worker.SIM_TAG_HEADER_PROFILE,
        "sim_lane": container_worker.SIM_TAG_HEADER_LANE,
        "sim_timestamp": container_worker.SIM_TAG_HEADER_TIMESTAMP,
        "sim_nonce": container_worker.SIM_TAG_HEADER_NONCE,
        "sim_signature": container_worker.SIM_TAG_HEADER_SIGNATURE,
    }
    for key, worker_value in worker_header_map.items():
        contract_value = header_map.get(key)
        if contract_value != worker_value:
            errors.append(
                f"sim-tag contract header drift for {key}: contract={contract_value} worker={worker_value}"
            )

    canonical = dict(contract.get("canonical") or {})
    contract_prefix = str(canonical.get("prefix") or "").strip()
    contract_separator = str(canonical.get("separator") or "")
    if contract_prefix != sim_runner.SIM_TAG_CANONICAL_PREFIX:
        errors.append("canonical.prefix drift between sim-tag contract and runner")
    if contract_separator != sim_runner.SIM_TAG_CANONICAL_SEPARATOR:
        errors.append("canonical.separator drift between sim-tag contract and runner")
    if contract_prefix != container_worker.SIM_TAG_CANONICAL_PREFIX:
        errors.append("canonical.prefix drift between sim-tag contract and container worker")
    if contract_separator != container_worker.SIM_TAG_CANONICAL_SEPARATOR:
        errors.append("canonical.separator drift between sim-tag contract and container worker")

    if int(contract.get("timestamp_max_skew_seconds") or 0) != sim_runner.SIM_TAG_TIMESTAMP_MAX_SKEW_SECONDS:
        errors.append(
            "timestamp_max_skew_seconds drift between sim-tag contract and runner"
        )
    if int(contract.get("nonce_ttl_seconds") or 0) != sim_runner.SIM_TAG_NONCE_TTL_SECONDS:
        errors.append("nonce_ttl_seconds drift between sim-tag contract and runner")
    if int(contract.get("nonce_max_entries") or 0) != sim_runner.SIM_TAG_NONCE_MAX_ENTRIES:
        errors.append("nonce_max_entries drift between sim-tag contract and runner")

    return errors


def main() -> int:
    try:
        errors = validate_sim_tag_contract()
    except SimTagContractError as exc:
        print(f"sim-tag-contract validation failed: {exc}")
        return 1
    if errors:
        print("sim-tag-contract validation failed:")
        for item in errors:
            print(f"- {item}")
        return 1
    print("sim-tag-contract validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
