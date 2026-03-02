#!/usr/bin/env python3
"""Validate shared deterministic attack corpus parity across runtime + CI lanes."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

import scripts.tests.adversarial_simulation_runner as sim_runner


CORPUS_PATH = Path("scripts/tests/adversarial/deterministic_attack_corpus.v1.json")
RUNTIME_PATH = Path("src/admin/adversary_sim.rs")


class DeterministicCorpusError(Exception):
    pass


def load_json_object(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise DeterministicCorpusError(f"missing file: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise DeterministicCorpusError(f"invalid JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise DeterministicCorpusError(f"expected JSON object: {path}")
    return payload


def compare_sets(name: str, expected: set[str], observed: set[str], errors: List[str]) -> None:
    missing = sorted(expected - observed)
    extra = sorted(observed - expected)
    if missing:
        errors.append(f"{name}: missing entries: {', '.join(missing)}")
    if extra:
        errors.append(f"{name}: unexpected entries: {', '.join(extra)}")


def extract_rust_constant(source: str, name: str) -> str:
    pattern = re.compile(rf"{re.escape(name)}\s*:\s*&str\s*=\s*\"([^\"]+)\"", re.MULTILINE)
    match = pattern.search(source)
    if not match:
        raise DeterministicCorpusError(f"missing Rust constant: {name}")
    return str(match.group(1)).strip()


def validate_deterministic_corpus() -> List[str]:
    errors: List[str] = []
    corpus = load_json_object(CORPUS_PATH)

    schema_version = str(corpus.get("schema_version") or "").strip()
    if schema_version != "sim-deterministic-attack-corpus.v1":
        errors.append(
            "deterministic attack corpus schema_version must be sim-deterministic-attack-corpus.v1 "
            f"(got {schema_version})"
        )

    revision = str(corpus.get("corpus_revision") or "").strip()
    if not revision:
        errors.append("deterministic attack corpus corpus_revision must be non-empty")

    taxonomy_version = str(corpus.get("taxonomy_version") or "").strip()
    if not taxonomy_version:
        errors.append("deterministic attack corpus taxonomy_version must be non-empty")

    ci_profile_name = str(corpus.get("ci_profile") or "").strip()
    ci_profile = corpus.get(ci_profile_name)
    drivers = {}
    if not isinstance(ci_profile, dict):
        errors.append(f"deterministic attack corpus missing ci profile object: {ci_profile_name}")
    else:
        drivers = dict(ci_profile.get("drivers") or {})
        if not drivers:
            errors.append("deterministic attack corpus ci_oracle.drivers must be a non-empty object")

    compare_sets(
        "driver coverage parity",
        set(sim_runner.ALLOWED_DRIVERS),
        {str(key).strip() for key in drivers.keys() if str(key).strip()},
        errors,
    )

    for driver_name in sorted(sim_runner.ALLOWED_DRIVERS):
        row = dict(drivers.get(driver_name) or {})
        expected_class = str(row.get("driver_class") or "").strip()
        expected_path_hint = str(row.get("path_hint") or "").strip()
        if not expected_class:
            errors.append(f"driver row missing driver_class: {driver_name}")
            continue
        observed_class = sim_runner.scenario_driver_class({"driver": driver_name})
        if observed_class != expected_class:
            errors.append(
                "driver_class drift "
                f"driver={driver_name} expected={expected_class} observed={observed_class}"
            )
        observed_path_hint = sim_runner.frontier_path_hint_for_scenario({"driver": driver_name})
        if observed_path_hint != expected_path_hint:
            errors.append(
                "path_hint drift "
                f"driver={driver_name} expected={expected_path_hint} observed={observed_path_hint}"
            )

    if sim_runner.DETERMINISTIC_ATTACK_CORPUS_REVISION != revision:
        errors.append(
            "runner corpus_revision drift "
            f"expected={revision} observed={sim_runner.DETERMINISTIC_ATTACK_CORPUS_REVISION}"
        )

    if sim_runner.DETERMINISTIC_ATTACK_CORPUS_TAXONOMY_VERSION != taxonomy_version:
        errors.append(
            "runner taxonomy_version drift "
            f"expected={taxonomy_version} observed={sim_runner.DETERMINISTIC_ATTACK_CORPUS_TAXONOMY_VERSION}"
        )

    rust_source = RUNTIME_PATH.read_text(encoding="utf-8")
    rust_schema = extract_rust_constant(rust_source, "DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION")
    rust_path = extract_rust_constant(rust_source, "DETERMINISTIC_ATTACK_CORPUS_PATH")

    if rust_schema != schema_version:
        errors.append(
            "runtime corpus schema constant drift "
            f"expected={schema_version} observed={rust_schema}"
        )

    if rust_path != str(CORPUS_PATH):
        errors.append(
            "runtime corpus path constant drift "
            f"expected={CORPUS_PATH} observed={rust_path}"
        )

    if "load_deterministic_attack_corpus" not in rust_source:
        errors.append("runtime loader missing: load_deterministic_attack_corpus")

    return errors


def main() -> int:
    try:
        errors = validate_deterministic_corpus()
    except DeterministicCorpusError as exc:
        print(f"deterministic-corpus validation failed: {exc}")
        return 1
    if errors:
        print("deterministic-corpus validation failed:")
        for item in errors:
            print(f"- {item}")
        return 1
    print("deterministic-corpus validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
