#!/usr/bin/env python3
"""Validate origin surface catalog does not collide with Shuma-owned routes."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import sys
from urllib.parse import urlsplit


@dataclass(frozen=True)
class ReservedRoute:
    kind: str
    pattern: str
    owner: str


RESERVED_ROUTES: list[ReservedRoute] = [
    ReservedRoute(kind="exact", pattern="/.well-known/spin", owner="spin_runtime"),
    ReservedRoute(kind="prefix", pattern="/.well-known/spin/", owner="spin_runtime"),
    ReservedRoute(kind="exact", pattern="/dashboard", owner="shuma_dashboard"),
    ReservedRoute(kind="prefix", pattern="/dashboard/", owner="shuma_dashboard"),
    ReservedRoute(kind="exact", pattern="/health", owner="shuma_control_plane"),
    ReservedRoute(kind="exact", pattern="/metrics", owner="shuma_control_plane"),
    ReservedRoute(kind="exact", pattern="/robots.txt", owner="shuma_control_plane"),
    ReservedRoute(kind="prefix", pattern="/admin", owner="shuma_admin_api"),
    ReservedRoute(kind="prefix", pattern="/internal/", owner="shuma_internal_api"),
    ReservedRoute(kind="exact", pattern="/challenge/puzzle", owner="shuma_challenge"),
    ReservedRoute(kind="exact", pattern="/challenge/not-a-bot-checkbox", owner="shuma_challenge"),
    ReservedRoute(kind="exact", pattern="/pow", owner="shuma_challenge"),
    ReservedRoute(kind="exact", pattern="/pow/verify", owner="shuma_challenge"),
    ReservedRoute(kind="exact", pattern="/tarpit/progress", owner="shuma_tarpit"),
    ReservedRoute(kind="prefix", pattern="/sim/public", owner="shuma_sim_public"),
    ReservedRoute(kind="prefix", pattern="/_/", owner="shuma_maze_namespace"),
]


def fail(message: str) -> int:
    print(f"❌ {message}", file=sys.stderr)
    return 1


def parse_bool(raw: str) -> bool:
    return raw.strip().lower() in {"1", "true", "yes", "on"}


def normalize_path(raw: str) -> str:
    value = str(raw or "").strip()
    if not value:
        return ""
    if "://" in value:
        parsed = urlsplit(value)
        path = parsed.path or "/"
    elif value.startswith("//"):
        parsed = urlsplit(f"https:{value}")
        path = parsed.path or "/"
    else:
        path = value
    path = path.split("#", 1)[0].split("?", 1)[0].strip()
    if not path:
        return "/"
    if not path.startswith("/"):
        return f"/{path}"
    return path


def collect_catalog_entries(value: object, out: list[str]) -> None:
    if isinstance(value, str):
        out.append(value)
        return
    if isinstance(value, list):
        for item in value:
            collect_catalog_entries(item, out)
        return
    if isinstance(value, dict):
        for key in ("paths", "urls", "inventory", "entries", "items", "routes"):
            if key in value:
                collect_catalog_entries(value[key], out)
        for key in ("path", "url", "href", "loc"):
            entry = value.get(key)
            if isinstance(entry, str):
                out.append(entry)


def extract_catalog_paths(payload: object) -> list[str]:
    raw_entries: list[str] = []
    collect_catalog_entries(payload, raw_entries)
    normalized = sorted(
        {
            candidate
            for candidate in (normalize_path(entry) for entry in raw_entries)
            if candidate
        }
    )
    if not normalized:
        raise ValueError(
            "surface catalog does not contain any discoverable paths/urls (expected keys like paths, urls, inventory, or path/url/href/loc entries)"
        )
    return normalized


def matches_reserved(path: str, route: ReservedRoute) -> bool:
    if route.kind == "exact":
        return path == route.pattern
    if route.pattern.endswith("/"):
        return path.startswith(route.pattern)
    return path == route.pattern or path.startswith(f"{route.pattern}/")


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

    report_path = Path(
        os.getenv(
            "GATEWAY_ROUTE_COLLISION_REPORT_PATH",
            "scripts/tests/adversarial/gateway_reserved_route_collision_report.json",
        ).strip()
    )
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
