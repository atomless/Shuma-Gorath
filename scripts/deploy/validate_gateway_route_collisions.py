#!/usr/bin/env python3
"""Validate origin surface catalog does not collide with Shuma-owned routes."""

from __future__ import annotations

from datetime import datetime, timezone
import json
import os
from pathlib import Path
import sys
REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.gateway_surface_catalog import RESERVED_ROUTES, ReservedRoute, extract_catalog_paths, matches_reserved


def fail(message: str) -> int:
    print(f"❌ {message}", file=sys.stderr)
    return 1


def parse_bool(raw: str) -> bool:
    return raw.strip().lower() in {"1", "true", "yes", "on"}

def remediation_hint(route: ReservedRoute) -> str:
    return (
        f"Origin route must move away from reserved {route.owner} namespace "
        f"({route.pattern}) before gateway cutover."
    )


def build_report(
    runtime_env: str,
    catalog_path: Path,
    report_path: Path,
    catalog_paths: list[str],
    collisions: list[dict[str, str]],
) -> dict[str, object]:
    return {
        "schema": "shuma.gateway.route_collision_report.v1",
        "runtime_env": runtime_env,
        "catalog_path": str(catalog_path),
        "report_path": str(report_path),
        "checked_at_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "reserved_routes_checked": [
            {"kind": route.kind, "pattern": route.pattern, "owner": route.owner}
            for route in RESERVED_ROUTES
        ],
        "catalog_path_count": len(catalog_paths),
        "collision_count": len(collisions),
        "collisions": collisions,
        "passed": len(collisions) == 0,
    }


def main() -> int:
    runtime_env = os.getenv("SHUMA_RUNTIME_ENV", "runtime-prod").strip().lower()
    if runtime_env != "runtime-prod":
        print(
            "ℹ️  Gateway route-collision preflight skipped because SHUMA_RUNTIME_ENV is not runtime-prod."
        )
        return 0

    catalog_path_raw = os.getenv("GATEWAY_SURFACE_CATALOG_PATH", "").strip()
    if not catalog_path_raw:
        return fail(
            "Missing GATEWAY_SURFACE_CATALOG_PATH; provide the discovered origin surface catalog JSON for reserved-route collision preflight"
        )
    catalog_path = Path(catalog_path_raw)
    if not catalog_path.exists():
        return fail(f"Gateway surface catalog not found: {catalog_path}")

    report_path_raw = os.getenv("GATEWAY_ROUTE_COLLISION_REPORT_PATH", "").strip()
    if not report_path_raw:
        report_path_raw = ".spin/deploy/gateway_reserved_route_collision_report.json"
    report_path = Path(report_path_raw)
    if not report_path.is_absolute():
        report_path = Path.cwd() / report_path

    try:
        payload = json.loads(catalog_path.read_text(encoding="utf-8"))
        catalog_paths = extract_catalog_paths(payload)
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        return fail(f"Failed to parse gateway surface catalog {catalog_path}: {exc}")

    collisions: list[dict[str, str]] = []
    for origin_path in catalog_paths:
        for route in RESERVED_ROUTES:
            if matches_reserved(origin_path, route):
                collisions.append(
                    {
                        "origin_path": origin_path,
                        "reserved_kind": route.kind,
                        "reserved_pattern": route.pattern,
                        "reserved_owner": route.owner,
                        "remediation": remediation_hint(route),
                    }
                )
                break

    report = build_report(runtime_env, catalog_path, report_path, catalog_paths, collisions)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True), encoding="utf-8")

    if collisions:
        return fail(
            "Reserved-route collision preflight failed; unresolved collisions found between origin surface and Shuma-owned routes. "
            f"See report: {report_path}"
        )

    if not parse_bool(os.getenv("SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED", "false")):
        return fail(
            "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED must be true after a clean reserved-route collision preflight"
        )

    print(
        "✅ Gateway reserved-route collision preflight passed "
        f"(catalog_paths={len(catalog_paths)}, report={report_path})."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
