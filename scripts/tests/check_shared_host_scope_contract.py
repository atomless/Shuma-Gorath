#!/usr/bin/env python3
"""Validate shared-host scope contract parity across tooling."""

from __future__ import annotations

import json
from pathlib import Path
import sys
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

import scripts.tests.shared_host_scope as shared_host_scope


CONTRACT_PATH = Path("scripts/tests/adversarial/shared_host_scope_contract.v1.json")


class SharedHostScopeContractError(Exception):
    pass


def load_contract(path: Path = CONTRACT_PATH) -> dict[str, Any]:
    if not path.exists():
        raise SharedHostScopeContractError(f"shared-host scope contract not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise SharedHostScopeContractError(
            f"invalid shared-host scope contract JSON: {path}"
        ) from exc
    if not isinstance(payload, dict):
        raise SharedHostScopeContractError(
            "shared-host scope contract must be a JSON object"
        )
    return payload


def validate_shared_host_scope_contract() -> list[str]:
    errors: list[str] = []
    payload = load_contract()

    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != shared_host_scope.SCHEMA_VERSION:
        errors.append(
            "shared-host scope contract schema_version must be "
            f"{shared_host_scope.SCHEMA_VERSION} (got {schema_version})"
        )

    descriptor = payload.get("descriptor")
    if not isinstance(descriptor, dict):
        errors.append("shared-host scope contract descriptor section must be an object")
        return errors

    required_fields = tuple(
        str(item).strip()
        for item in descriptor.get("required_fields", [])
        if str(item).strip()
    )
    if required_fields != shared_host_scope.REQUIRED_DESCRIPTOR_FIELDS:
        errors.append(
            "descriptor.required_fields mismatch: expected "
            f"{shared_host_scope.REQUIRED_DESCRIPTOR_FIELDS}, got {required_fields}"
        )

    defaults = descriptor.get("defaults")
    if not isinstance(defaults, dict):
        errors.append("descriptor.defaults must be an object")
    else:
        if defaults.get("require_https") is not shared_host_scope.DEFAULT_REQUIRE_HTTPS:
            errors.append(
                "descriptor.defaults.require_https must match shared_host_scope default"
            )
        if defaults.get("deny_ip_literals") is not shared_host_scope.DEFAULT_DENY_IP_LITERALS:
            errors.append(
                "descriptor.defaults.deny_ip_literals must match shared_host_scope default"
            )

    baseline_denied_path_prefixes = tuple(
        str(item).strip()
        for item in descriptor.get("baseline_denied_path_prefixes", [])
        if str(item).strip()
    )
    if baseline_denied_path_prefixes != shared_host_scope.BASELINE_DENIED_PATH_PREFIXES:
        errors.append(
            "descriptor.baseline_denied_path_prefixes mismatch: expected "
            f"{shared_host_scope.BASELINE_DENIED_PATH_PREFIXES}, "
            f"got {baseline_denied_path_prefixes}"
        )

    validation = payload.get("validation")
    if not isinstance(validation, dict):
        errors.append("shared-host scope contract validation section must be an object")
    else:
        rejection_reasons = tuple(
            str(item).strip()
            for item in validation.get("rejection_reasons", [])
            if str(item).strip()
        )
        if rejection_reasons != shared_host_scope.REJECTION_REASONS:
            errors.append(
                "validation.rejection_reasons mismatch: expected "
                f"{shared_host_scope.REJECTION_REASONS}, got {rejection_reasons}"
            )

    return errors


def main() -> int:
    try:
        errors = validate_shared_host_scope_contract()
    except SharedHostScopeContractError as exc:
        print(f"shared-host-scope-contract validation failed: {exc}")
        return 1
    if errors:
        print("shared-host-scope-contract validation failed:")
        for item in errors:
            print(f"- {item}")
        return 1
    print("shared-host-scope-contract validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
