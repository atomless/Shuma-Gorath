#!/usr/bin/env python3

"""
Frontier artifact governance guard.

Fails when forbidden field names or leaked frontier secrets are detected in
machine-readable artifacts (`latest_report.json`, `attack_plan.json`).
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Any, Dict, Iterable, List


FRONTIER_SECRET_ENV_KEYS = [
    "SHUMA_FRONTIER_OPENAI_API_KEY",
    "SHUMA_FRONTIER_ANTHROPIC_API_KEY",
    "SHUMA_FRONTIER_GOOGLE_API_KEY",
    "SHUMA_FRONTIER_XAI_API_KEY",
]


def load_json(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise ValueError(f"missing artifact: {path}")
    try:
        loaded = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise ValueError(f"invalid JSON artifact: {path}") from exc
    if not isinstance(loaded, dict):
        raise ValueError(f"artifact must be top-level JSON object: {path}")
    return loaded


def load_schema(schema_path: Path) -> Dict[str, Any]:
    schema = load_json(schema_path)
    if str(schema.get("schema_version") or "").strip() != "frontier_payload_schema.v1":
        raise ValueError(f"unexpected frontier payload schema version: {schema.get('schema_version')}")
    return schema


def normalize_tokens(values: Iterable[Any]) -> List[str]:
    tokens: List[str] = []
    for value in values:
        token = str(value or "").strip().lower()
        if token:
            tokens.append(token)
    return sorted(set(tokens))


def collect_forbidden_key_paths(value: Any, forbidden_tokens: List[str], prefix: str = "$") -> List[str]:
    violations: List[str] = []
    if isinstance(value, dict):
        for key, nested in value.items():
            key_text = str(key)
            key_normalized = key_text.strip().lower()
            if any(token in key_normalized for token in forbidden_tokens):
                violations.append(f"{prefix}.{key_text}")
            violations.extend(
                collect_forbidden_key_paths(
                    value=nested,
                    forbidden_tokens=forbidden_tokens,
                    prefix=f"{prefix}.{key_text}",
                )
            )
    elif isinstance(value, list):
        for index, item in enumerate(value):
            violations.extend(
                collect_forbidden_key_paths(
                    value=item,
                    forbidden_tokens=forbidden_tokens,
                    prefix=f"{prefix}[{index}]",
                )
            )
    return violations


def validate_attack_plan_payloads(
    attack_plan: Dict[str, Any], allowed_top_level_keys: List[str]
) -> List[str]:
    errors: List[str] = []
    attack_generation_contract = dict(attack_plan.get("attack_generation_contract") or {})
    if (
        str(attack_generation_contract.get("schema_version") or "").strip()
        != "frontier-attack-generation-contract.v1"
    ):
        errors.append(
            "attack_plan.attack_generation_contract.schema_version must be "
            "frontier-attack-generation-contract.v1"
        )
    if not str(attack_generation_contract.get("contract_path") or "").strip():
        errors.append("attack_plan.attack_generation_contract.contract_path must be non-empty")

    generation_summary = dict(attack_plan.get("generation_summary") or {})
    for key in (
        "seed_candidate_count",
        "generated_candidate_count",
        "accepted_candidate_count",
        "rejected_candidate_count",
    ):
        value = generation_summary.get(key)
        if isinstance(value, bool) or not isinstance(value, int) or int(value) < 0:
            errors.append(f"attack_plan.generation_summary.{key} must be integer >= 0")

    candidates = attack_plan.get("candidates")
    if not isinstance(candidates, list):
        return ["attack_plan.candidates must be an array"]
    allowed_keys_set = set(allowed_top_level_keys)
    for index, candidate in enumerate(candidates):
        if not isinstance(candidate, dict):
            errors.append(f"attack_plan.candidates[{index}] must be an object")
            continue
        required_candidate_keys = (
            "candidate_id",
            "source_scenario_id",
            "generation_kind",
            "mutation_class",
            "behavioral_class",
            "novelty_score",
            "governance_passed",
        )
        for key in required_candidate_keys:
            if key not in candidate:
                errors.append(f"attack_plan.candidates[{index}].{key} is required")
        candidate_id = str(candidate.get("candidate_id") or "").strip()
        if not candidate_id:
            errors.append(f"attack_plan.candidates[{index}].candidate_id must be non-empty")
        generation_kind = str(candidate.get("generation_kind") or "").strip()
        if generation_kind not in {"seed", "mutation"}:
            errors.append(
                f"attack_plan.candidates[{index}].generation_kind must be seed|mutation"
            )
        novelty_score = candidate.get("novelty_score")
        if isinstance(novelty_score, bool) or not isinstance(
            novelty_score, (int, float)
        ):
            errors.append(
                f"attack_plan.candidates[{index}].novelty_score must be numeric within [0,1]"
            )
        elif float(novelty_score) < 0.0 or float(novelty_score) > 1.0:
            errors.append(
                f"attack_plan.candidates[{index}].novelty_score must be numeric within [0,1]"
            )
        if not isinstance(candidate.get("governance_passed"), bool):
            errors.append(
                f"attack_plan.candidates[{index}].governance_passed must be boolean"
            )
        payload = candidate.get("payload")
        if not isinstance(payload, dict):
            errors.append(f"attack_plan.candidates[{index}].payload must be an object")
            continue
        payload_keys = set(str(key) for key in payload.keys())
        unknown_keys = sorted(payload_keys.difference(allowed_keys_set))
        if unknown_keys:
            errors.append(
                "attack_plan.candidates[{}].payload contains unknown top-level keys: {}".format(
                    index, ", ".join(unknown_keys)
                )
            )
    return errors


def collect_frontier_secret_values() -> List[str]:
    secrets: List[str] = []
    for env_key in FRONTIER_SECRET_ENV_KEYS:
        value = str(os.environ.get(env_key, "")).strip()
        if value:
            secrets.append(value)
    return secrets


def detect_leaked_secret_values(serialized: str, secret_values: List[str], label: str) -> List[str]:
    errors: List[str] = []
    for secret_value in secret_values:
        if secret_value and secret_value in serialized:
            errors.append(f"{label} contains literal frontier secret value")
    return errors


def validate_artifacts(
    report: Dict[str, Any],
    attack_plan: Dict[str, Any],
    schema: Dict[str, Any],
    frontier_secret_values: List[str],
) -> List[str]:
    errors: List[str] = []
    forbidden_tokens = normalize_tokens(schema.get("forbidden_field_examples") or [])
    allowed_top_level_keys = [str(value) for value in (schema.get("allowed_top_level_keys") or [])]
    report_frontier = dict(report.get("frontier") or {})

    if not forbidden_tokens:
        errors.append("frontier schema has no forbidden_field_examples tokens")
    if not allowed_top_level_keys:
        errors.append("frontier schema has no allowed_top_level_keys")

    errors.extend(validate_attack_plan_payloads(attack_plan, allowed_top_level_keys))
    errors.extend(
        f"report forbidden key path: {path}"
        for path in collect_forbidden_key_paths(
            report_frontier,
            forbidden_tokens,
            prefix="$.frontier",
        )
    )
    errors.extend(
        f"attack_plan forbidden key path: {path}"
        for path in collect_forbidden_key_paths(attack_plan, forbidden_tokens)
    )

    report_serialized = json.dumps(report, sort_keys=True)
    attack_plan_serialized = json.dumps(attack_plan, sort_keys=True)
    errors.extend(
        detect_leaked_secret_values(
            serialized=report_serialized,
            secret_values=frontier_secret_values,
            label="report",
        )
    )
    errors.extend(
        detect_leaked_secret_values(
            serialized=attack_plan_serialized,
            secret_values=frontier_secret_values,
            label="attack_plan",
        )
    )
    return sorted(set(errors))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Fail when forbidden fields or frontier secret values leak into frontier artifacts."
    )
    parser.add_argument(
        "--report",
        default="scripts/tests/adversarial/latest_report.json",
        help="Path to adversarial report JSON",
    )
    parser.add_argument(
        "--attack-plan",
        default="scripts/tests/adversarial/attack_plan.json",
        help="Path to attack plan JSON",
    )
    parser.add_argument(
        "--schema",
        default="scripts/tests/adversarial/frontier_payload_schema.v1.json",
        help="Path to frontier payload schema JSON",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report_path = Path(args.report)
    attack_plan_path = Path(args.attack_plan)
    schema_path = Path(args.schema)

    report = load_json(report_path)
    attack_plan = load_json(attack_plan_path)
    schema = load_schema(schema_path)
    errors = validate_artifacts(
        report=report,
        attack_plan=attack_plan,
        schema=schema,
        frontier_secret_values=collect_frontier_secret_values(),
    )

    if errors:
        print("[frontier-governance] FAIL")
        for error in errors:
            print(f"[frontier-governance] {error}")
        return 1

    print("[frontier-governance] PASS")
    print(f"[frontier-governance] report={report_path}")
    print(f"[frontier-governance] attack_plan={attack_plan_path}")
    print(f"[frontier-governance] schema={schema_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
