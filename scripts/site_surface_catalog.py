"""Shared site-surface catalog helpers for local docroot-backed sites."""

from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import sys
import xml.etree.ElementTree as ET

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.gateway_surface_catalog import normalize_path

SUPPORTED_MODES = {"auto", "static-html-docroot", "php-docroot"}
INDEX_FILENAMES = {
    "static-html-docroot": {"index.html", "index.htm"},
    "php-docroot": {"index.php"},
}


def is_hidden(path: Path) -> bool:
    return any(part.startswith(".") for part in path.parts)


def detect_mode(docroot: Path) -> str:
    for file_path in docroot.rglob("*"):
        if not file_path.is_file():
            continue
        relative_path = file_path.relative_to(docroot)
        if is_hidden(relative_path):
            continue
        if file_path.suffix.lower() == ".php":
            return "php-docroot"
    return "static-html-docroot"


def file_route(relative_path: Path, mode: str) -> tuple[str, str]:
    posix_relative = relative_path.as_posix()
    if relative_path.name.lower() in INDEX_FILENAMES[mode]:
        if posix_relative == relative_path.name:
            return "/", "docroot:index"
        return f"/{relative_path.parent.as_posix()}/", "docroot:index"
    return f"/{posix_relative}", "docroot:file"


def add_inventory_entry(
    inventory: dict[str, dict[str, object]],
    path: str,
    source: str,
    relative_file: str | None = None,
) -> None:
    entry = inventory.setdefault(path, {"path": path, "sources": set()})
    sources = entry["sources"]
    assert isinstance(sources, set)
    sources.add(source)
    if relative_file and "relative_file" not in entry:
        entry["relative_file"] = relative_file


def collect_docroot_inventory(docroot: Path, mode: str) -> dict[str, dict[str, object]]:
    inventory: dict[str, dict[str, object]] = {}
    for file_path in sorted(docroot.rglob("*")):
        if not file_path.is_file():
            continue
        relative_path = file_path.relative_to(docroot)
        if is_hidden(relative_path):
            continue
        route, source = file_route(relative_path, mode)
        add_inventory_entry(inventory, route, source, relative_path.as_posix())
    return inventory


def strip_namespace(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def loc_to_docroot_path(docroot: Path, loc_value: str) -> Path | None:
    normalized = normalize_path(loc_value)
    if not normalized:
        return None
    candidate = (docroot / normalized.lstrip("/")).resolve()
    try:
        candidate.relative_to(docroot.resolve())
    except ValueError:
        return None
    return candidate


def collect_sitemap_candidates(docroot: Path) -> list[Path]:
    candidates: list[Path] = []
    direct = docroot / "sitemap.xml"
    if direct.is_file():
        candidates.append(direct)
    robots_path = docroot / "robots.txt"
    if robots_path.is_file():
        for line in robots_path.read_text(encoding="utf-8").splitlines():
            raw = line.strip()
            if not raw or ":" not in raw:
                continue
            key, value = raw.split(":", 1)
            if key.strip().lower() != "sitemap":
                continue
            candidate = loc_to_docroot_path(docroot, value.strip())
            if candidate and candidate.is_file() and candidate not in candidates:
                candidates.append(candidate)
    return candidates


def collect_sitemap_paths(docroot: Path) -> tuple[set[str], list[str]]:
    paths: set[str] = set()
    diagnostics: list[str] = []
    queue = collect_sitemap_candidates(docroot)
    visited: set[Path] = set()

    while queue:
        sitemap_path = queue.pop(0).resolve()
        if sitemap_path in visited:
            continue
        visited.add(sitemap_path)
        try:
            root = ET.fromstring(sitemap_path.read_text(encoding="utf-8"))
        except (OSError, ET.ParseError) as exc:
            diagnostics.append(f"ignored sitemap {sitemap_path.name}: {exc}")
            continue

        root_tag = strip_namespace(root.tag)
        if root_tag == "urlset":
            for url_node in root:
                if strip_namespace(url_node.tag) != "url":
                    continue
                for child in url_node:
                    if strip_namespace(child.tag) != "loc" or not child.text:
                        continue
                    normalized = normalize_path(child.text)
                    if normalized:
                        paths.add(normalized)
        elif root_tag == "sitemapindex":
            for sitemap_node in root:
                if strip_namespace(sitemap_node.tag) != "sitemap":
                    continue
                for child in sitemap_node:
                    if strip_namespace(child.tag) != "loc" or not child.text:
                        continue
                    nested = loc_to_docroot_path(docroot, child.text)
                    if nested and nested.is_file():
                        queue.append(nested)
                    else:
                        diagnostics.append(
                            f"ignored non-local sitemap reference from {sitemap_path.name}: {child.text.strip()}"
                        )
        else:
            diagnostics.append(f"ignored unsupported sitemap root {root_tag} in {sitemap_path.name}")

    return paths, diagnostics


def serialize_inventory(entries: dict[str, dict[str, object]]) -> list[dict[str, object]]:
    serialized: list[dict[str, object]] = []
    for path in sorted(entries):
        entry = dict(entries[path])
        sources = entry.get("sources", set())
        if isinstance(sources, set):
            entry["sources"] = sorted(sources)
        serialized.append(entry)
    return serialized


def build_payload(docroot: Path, requested_mode: str) -> dict[str, object]:
    mode = requested_mode if requested_mode != "auto" else detect_mode(docroot)
    inventory = collect_docroot_inventory(docroot, mode)
    sitemap_paths, diagnostics = collect_sitemap_paths(docroot)
    for path in sorted(sitemap_paths):
        add_inventory_entry(inventory, path, "sitemap")
    return {
        "schema": "shuma.gateway.surface_catalog.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "mode": mode,
        "docroot": str(docroot),
        "inventory": serialize_inventory(inventory),
        "diagnostics": diagnostics,
    }
