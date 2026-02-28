#!/usr/bin/env python3
"""Validate SIM2 ADR conformance guardrails for core architecture domains."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List


DEFAULT_OUTPUT_PATH = Path("scripts/tests/adversarial/sim2_adr_conformance_report.json")

ADR_REQUIREMENTS = [
    {
        "id": "adr_0007",
        "path": "docs/adr/0007-adversary-sim-toggle-command-controller.md",
        "markers": [
            "Trust-Boundary",
            "adversary-sim/control",
            "SIM2-GC-11",
        ],
    },
    {
        "id": "adr_0008",
        "path": "docs/adr/0008-realtime-monitoring-cursor-sse-hybrid.md",
        "markers": [
            "cursor",
            "SSE",
            "SIM2-GC-11",
        ],
    },
    {
        "id": "adr_0009",
        "path": "docs/adr/0009-telemetry-lifecycle-retention-cost-security.md",
        "markers": [
            "retention",
            "cost",
            "security/privacy",
            "SIM2-GC-11",
        ],
    },
]

IMPLEMENTATION_REQUIREMENTS = [
    {
        "id": "api_control_endpoint",
        "path": "src/admin/api.rs",
        "markers": [
            "/admin/adversary-sim/control",
            "validate_origin_and_fetch_metadata",
        ],
    },
    {
        "id": "api_realtime_endpoints",
        "path": "src/admin/api.rs",
        "markers": [
            "/admin/monitoring/delta",
            "/admin/monitoring/stream",
            "strict_monotonic_cursor_ascending",
        ],
    },
    {
        "id": "promotion_hybrid_policy",
        "path": "scripts/tests/adversarial_promote_candidates.py",
        "markers": [
            "HYBRID_CONFIRMATION_MIN_PERCENT",
            "HYBRID_FALSE_DISCOVERY_MAX_PERCENT",
            "HYBRID_OWNER_DISPOSITION_SLA_HOURS",
        ],
    },
]


def check_markers(text: str, markers: List[str]) -> List[str]:
    missing: List[str] = []
    for marker in markers:
        if marker not in text:
            missing.append(marker)
    return missing


def read_text(path: Path) -> str:
    if not path.exists():
        raise FileNotFoundError(str(path))
    return path.read_text(encoding="utf-8")


def evaluate_requirements(repo_root: Path) -> Dict[str, Any]:
    checks: List[Dict[str, Any]] = []
    failures: List[str] = []
    for requirement in ADR_REQUIREMENTS + IMPLEMENTATION_REQUIREMENTS:
        relative_path = Path(requirement["path"])
        absolute_path = repo_root / relative_path
        marker_list = list(requirement["markers"])
        check_id = str(requirement["id"])
        try:
            text = read_text(absolute_path)
            missing = check_markers(text, marker_list)
            passed = len(missing) == 0
            detail = (
                "all markers present"
                if passed
                else "missing markers: " + ", ".join(missing)
            )
        except Exception as exc:
            passed = False
            detail = f"read_error:{exc}"
            missing = marker_list
        checks.append(
            {
                "id": check_id,
                "path": str(relative_path),
                "passed": passed,
                "detail": detail,
                "required_markers": marker_list,
            }
        )
        if not passed:
            failures.append(f"{check_id}:{detail}")

    return {
        "schema_version": "sim2-adr-conformance.v1",
        "checks": checks,
        "status": {
            "passed": len(failures) == 0,
            "failures": failures,
        },
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check SIM2 ADR conformance markers and emit diagnostics report."
    )
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT_PATH))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    output_path = Path(args.output)
    repo_root = Path(__file__).resolve().parents[2]
    payload = evaluate_requirements(repo_root)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"[sim2-adr-conformance] report={output_path}")
    if not bool(dict(payload.get("status") or {}).get("passed")):
        print("[sim2-adr-conformance] FAIL")
        for failure in list(dict(payload.get("status") or {}).get("failures") or []):
            print(f"- {failure}")
        return 1
    print("[sim2-adr-conformance] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
