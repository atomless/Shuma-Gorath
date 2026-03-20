#!/usr/bin/env python3
"""Validate shared-host seed contract parity across tooling."""

from __future__ import annotations

import json
from pathlib import Path
import sys
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

import scripts.tests.shared_host_seed_inventory as shared_host_seed_inventory


CONTRACT_PATH = Path("scripts/tests/adversarial/shared_host_seed_contract.v1.json")


class SharedHostSeedContractError(Exception):
    pass


def load_contract(path: Path = CONTRACT_PATH) -> dict[str, Any]:
    if not path.exists():
        raise SharedHostSeedContractError(f"shared-host seed contract not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise SharedHostSeedContractError(
            f"invalid shared-host seed contract JSON: {path}"
        ) from exc
    if not isinstance(payload, dict):
        raise SharedHostSeedContractError("shared-host seed contract must be a JSON object")
    return payload


def validate_shared_host_seed_contract() -> list[str]:
    errors: list[str] = []
    payload = load_contract()

    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != shared_host_seed_inventory.SCHEMA_VERSION:
        errors.append(
            "shared-host seed contract schema_version must be "
            f"{shared_host_seed_inventory.SCHEMA_VERSION} (got {schema_version})"
        )

    source_labels = tuple(
        str(item).strip() for item in payload.get("source_labels", []) if str(item).strip()
    )
    if source_labels != shared_host_seed_inventory.SOURCE_LABELS:
        errors.append(
            "shared-host seed contract source_labels mismatch: expected "
            f"{shared_host_seed_inventory.SOURCE_LABELS}, got {source_labels}"
        )

    inventory_sections = tuple(
        str(item).strip()
        for item in payload.get("inventory_sections", [])
        if str(item).strip()
    )
    if inventory_sections != shared_host_seed_inventory.INVENTORY_SECTIONS:
        errors.append(
            "shared-host seed contract inventory_sections mismatch: expected "
            f"{shared_host_seed_inventory.INVENTORY_SECTIONS}, got {inventory_sections}"
        )

    validation = payload.get("validation")
    if not isinstance(validation, dict):
        errors.append("shared-host seed contract validation section must be an object")
    else:
        rejection_reasons = tuple(
            str(item).strip()
            for item in validation.get("rejection_reasons", [])
            if str(item).strip()
        )
        if rejection_reasons != shared_host_seed_inventory.SEED_REJECTION_REASONS:
            errors.append(
                "validation.rejection_reasons mismatch: expected "
                f"{shared_host_seed_inventory.SEED_REJECTION_REASONS}, "
                f"got {rejection_reasons}"
            )

    return errors


def main() -> int:
    try:
        errors = validate_shared_host_seed_contract()
    except SharedHostSeedContractError as exc:
        print(f"shared-host-seed-contract validation failed: {exc}")
        return 1
    if errors:
        print("shared-host-seed-contract validation failed:")
        for item in errors:
            print(f"- {item}")
        return 1
    print("shared-host-seed-contract validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
