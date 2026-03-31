"""Shared helpers for gateway surface-catalog parsing and path selection."""

from __future__ import annotations

from dataclasses import dataclass
import json
from pathlib import PurePosixPath
from urllib.parse import urlsplit


@dataclass(frozen=True)
class ReservedRoute:
    kind: str
    pattern: str
    owner: str


RESERVED_ROUTES: list[ReservedRoute] = [
    ReservedRoute(kind="exact", pattern="/.well-known/spin", owner="spin_runtime"),
    ReservedRoute(kind="prefix", pattern="/.well-known/spin/", owner="spin_runtime"),
    ReservedRoute(kind="exact", pattern="/shuma/dashboard", owner="shuma_dashboard"),
    ReservedRoute(kind="prefix", pattern="/shuma/dashboard/", owner="shuma_dashboard"),
    ReservedRoute(kind="exact", pattern="/shuma/health", owner="shuma_control_plane"),
    ReservedRoute(kind="exact", pattern="/shuma/metrics", owner="shuma_control_plane"),
    ReservedRoute(kind="exact", pattern="/robots.txt", owner="shuma_control_plane"),
    ReservedRoute(kind="prefix", pattern="/shuma/admin", owner="shuma_admin_api"),
    ReservedRoute(kind="prefix", pattern="/internal/", owner="shuma_internal_api"),
    ReservedRoute(kind="exact", pattern="/challenge/puzzle", owner="shuma_challenge"),
    ReservedRoute(kind="exact", pattern="/challenge/not-a-bot-checkbox", owner="shuma_challenge"),
    ReservedRoute(kind="exact", pattern="/pow", owner="shuma_challenge"),
    ReservedRoute(kind="exact", pattern="/pow/verify", owner="shuma_challenge"),
    ReservedRoute(kind="exact", pattern="/tarpit/progress", owner="shuma_tarpit"),
    ReservedRoute(kind="prefix", pattern="/_/", owner="shuma_maze_namespace"),
]

PREFERRED_TEXT_SUFFIXES = {
    ".css",
    ".csv",
    ".htm",
    ".html",
    ".js",
    ".json",
    ".mjs",
    ".svg",
    ".txt",
    ".xml",
}

STATIC_BYPASS_PREFIXES = (
    "/assets/",
    "/static/",
    "/images/",
    "/img/",
    "/js/",
    "/css/",
    "/fonts/",
    "/_next/static/",
)
STATIC_BYPASS_EXACT_PATHS = {
    "/favicon.ico",
    "/favicon.svg",
    "/apple-touch-icon.png",
    "/manifest.json",
    "/site.webmanifest",
    "/sitemap.xml",
    "/browserconfig.xml",
}
STATIC_BYPASS_SUFFIXES = {
    ".css",
    ".js",
    ".mjs",
    ".map",
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".webp",
    ".svg",
    ".ico",
    ".woff",
    ".woff2",
    ".ttf",
    ".otf",
    ".eot",
    ".webmanifest",
    ".xml",
}


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


def load_catalog_paths(catalog_path: str) -> list[str]:
    with open(catalog_path, "r", encoding="utf-8") as handle:
        payload = json.load(handle)
    return extract_catalog_paths(payload)


def matches_reserved(path: str, route: ReservedRoute) -> bool:
    if route.kind == "exact":
        return path == route.pattern
    if route.pattern.endswith("/"):
        return path.startswith(route.pattern)
    return path == route.pattern or path.startswith(f"{route.pattern}/")


def is_reserved_path(path: str) -> bool:
    return any(matches_reserved(path, route) for route in RESERVED_ROUTES)


def _probe_priority(path: str) -> tuple[int, int, str]:
    suffix = PurePosixPath(path).suffix.lower()
    if (
        path in STATIC_BYPASS_EXACT_PATHS
        or any(path.startswith(prefix) for prefix in STATIC_BYPASS_PREFIXES)
        or suffix in STATIC_BYPASS_SUFFIXES
    ):
        return (0, len(path), path)
    if suffix in PREFERRED_TEXT_SUFFIXES:
        return (1, len(path), path)
    if path != "/":
        return (2, len(path), path)
    return (3, len(path), path)


def select_forward_probe_path(catalog_paths: list[str]) -> str:
    candidates = [path for path in catalog_paths if not is_reserved_path(path)]
    if not candidates:
        raise ValueError(
            "surface catalog does not contain a non-reserved public path suitable for gateway smoke forwarding checks"
        )
    return min(candidates, key=_probe_priority)
