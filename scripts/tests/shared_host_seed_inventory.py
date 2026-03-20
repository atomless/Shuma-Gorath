#!/usr/bin/env python3
"""Minimal shared-host seed inventory builder."""

from __future__ import annotations

import argparse
from collections.abc import Callable, Mapping, Sequence
import json
from pathlib import Path
import sys
from typing import Any
from urllib.parse import urlsplit, urlunsplit
from urllib.request import urlopen

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tests import shared_host_scope


SCHEMA_VERSION = "shared-host-seed-contract.v1"
SOURCE_LABELS = (
    "primary_start_url",
    "robots",
    "manual_extra_seed",
)
INVENTORY_SECTIONS = (
    "accepted_start_urls",
    "accepted_hint_documents",
    "rejected_inputs",
)
SEED_REJECTION_REASONS = (
    *shared_host_scope.REJECTION_REASONS,
    "robots_fetch_failed",
    "robots_parse_failed",
)


class SharedHostSeedError(ValueError):
    """Raised when required shared-host seed inputs are invalid."""


def load_scope_descriptor(path: Path) -> shared_host_scope.SharedHostScopeDescriptor:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise SharedHostSeedError(f"scope descriptor not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise SharedHostSeedError(f"invalid scope descriptor JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise SharedHostSeedError("scope descriptor must be a JSON object")
    return shared_host_scope.descriptor_from_payload(payload)


def parse_robots_sitemap_urls(robots_text: str) -> list[str]:
    urls: list[str] = []
    for line in robots_text.splitlines():
        stripped = line.strip()
        if not stripped or ":" not in stripped:
            continue
        key, value = stripped.split(":", 1)
        if key.strip().lower() != "sitemap":
            continue
        candidate = value.strip()
        if candidate:
            urls.append(candidate)
    return urls


def _merge_sources(entries: list[dict[str, Any]], url: str, source: str) -> None:
    for entry in entries:
        if entry["url"] != url:
            continue
        sources = entry["sources"]
        if source not in sources:
            sources.append(source)
        return
    entries.append({"url": url, "sources": [source]})


def _append_rejection(
    rejected_inputs: list[dict[str, str]],
    *,
    source: str,
    raw_value: str,
    reason: str,
) -> None:
    rejected_inputs.append(
        {
            "source": source,
            "raw_value": raw_value,
            "reason": reason,
        }
    )


def _derive_default_robots_url(primary_start_url: str) -> str:
    parts = urlsplit(primary_start_url)
    path = "/robots.txt"
    return urlunsplit((parts.scheme, parts.netloc, path, "", ""))


def _read_robots_file(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _fetch_robots_text(
    robots_url: str, fetcher: Callable[[str], str] | None = None
) -> str:
    if fetcher is not None:
        return fetcher(robots_url)
    with urlopen(robots_url) as response:  # nosec B310 - operator-supplied workflow helper
        return response.read().decode("utf-8")


def build_seed_inventory(
    descriptor: shared_host_scope.SharedHostScopeDescriptor,
    *,
    primary_start_url: str,
    extra_seed_urls: Sequence[str] | None = None,
    robots_url: str | None = None,
    robots_text: str | None = None,
    robots_fetcher: Callable[[str], str] | None = None,
) -> dict[str, Any]:
    if not str(primary_start_url).strip():
        raise SharedHostSeedError("primary_start_url is required")

    accepted_start_urls: list[dict[str, Any]] = []
    accepted_hint_documents: list[dict[str, Any]] = []
    rejected_inputs: list[dict[str, str]] = []

    primary_decision = shared_host_scope.evaluate_url_candidate(
        primary_start_url,
        descriptor,
    )
    if not primary_decision.allowed or primary_decision.normalized_url is None:
        reason = primary_decision.rejection_reason or "malformed_url"
        raise SharedHostSeedError(f"primary_start_url rejected: {reason}")
    normalized_primary_start_url = primary_decision.normalized_url
    _merge_sources(
        accepted_start_urls,
        normalized_primary_start_url,
        "primary_start_url",
    )

    for raw_value in extra_seed_urls or []:
        decision = shared_host_scope.evaluate_url_candidate(raw_value, descriptor)
        if decision.allowed and decision.normalized_url is not None:
            _merge_sources(
                accepted_start_urls,
                decision.normalized_url,
                "manual_extra_seed",
            )
            continue
        _append_rejection(
            rejected_inputs,
            source="manual_extra_seed",
            raw_value=str(raw_value),
            reason=decision.rejection_reason or "malformed_url",
        )

    effective_robots_url = robots_url.strip() if robots_url else None
    if robots_text is None and effective_robots_url:
        decision = shared_host_scope.evaluate_url_candidate(effective_robots_url, descriptor)
        if not decision.allowed:
            _append_rejection(
                rejected_inputs,
                source="robots",
                raw_value=effective_robots_url,
                reason=decision.rejection_reason or "malformed_url",
            )
        else:
            try:
                robots_text = _fetch_robots_text(effective_robots_url, robots_fetcher)
            except Exception:
                _append_rejection(
                    rejected_inputs,
                    source="robots",
                    raw_value=effective_robots_url,
                    reason="robots_fetch_failed",
                )

    if robots_text is not None:
        try:
            sitemap_urls = parse_robots_sitemap_urls(robots_text)
        except Exception:
            _append_rejection(
                rejected_inputs,
                source="robots",
                raw_value=effective_robots_url or _derive_default_robots_url(normalized_primary_start_url),
                reason="robots_parse_failed",
            )
        else:
            for sitemap_url in sitemap_urls:
                decision = shared_host_scope.evaluate_url_candidate(sitemap_url, descriptor)
                if decision.allowed and decision.normalized_url is not None:
                    _merge_sources(
                        accepted_hint_documents,
                        decision.normalized_url,
                        "robots",
                    )
                    continue
                _append_rejection(
                    rejected_inputs,
                    source="robots",
                    raw_value=sitemap_url,
                    reason=decision.rejection_reason or "malformed_url",
                )

    return {
        "schema_version": SCHEMA_VERSION,
        "primary_start_url": normalized_primary_start_url,
        "accepted_start_urls": accepted_start_urls,
        "accepted_hint_documents": accepted_hint_documents,
        "rejected_inputs": rejected_inputs,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a minimal shared-host seed inventory under the scope contract."
    )
    parser.add_argument(
        "--scope-descriptor",
        required=True,
        help="Path to the shared-host scope descriptor JSON",
    )
    parser.add_argument(
        "--primary-start-url",
        required=True,
        help="Required primary public start URL",
    )
    parser.add_argument(
        "--extra-seed-url",
        action="append",
        default=[],
        help="Optional extra crawl start URL (repeatable)",
    )
    parser.add_argument(
        "--robots-file",
        help="Optional local robots.txt file to ingest as hint input",
    )
    parser.add_argument(
        "--robots-url",
        help="Optional robots.txt URL to fetch; defaults to primary host /robots.txt when --fetch-robots is used",
    )
    parser.add_argument(
        "--fetch-robots",
        action="store_true",
        help="Fetch robots.txt from the derived or explicit robots URL",
    )
    parser.add_argument(
        "--output",
        help="Write JSON output to this path instead of stdout",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    descriptor = load_scope_descriptor(Path(args.scope_descriptor).expanduser().resolve())

    robots_text = None
    robots_url = args.robots_url
    if args.robots_file:
        robots_text = _read_robots_file(Path(args.robots_file).expanduser().resolve())
    elif args.fetch_robots:
        robots_url = robots_url or _derive_default_robots_url(args.primary_start_url)

    payload = build_seed_inventory(
        descriptor,
        primary_start_url=args.primary_start_url,
        extra_seed_urls=args.extra_seed_url,
        robots_url=robots_url,
        robots_text=robots_text,
    )
    rendered = json.dumps(payload, indent=2, sort_keys=True)
    if args.output:
        output_path = Path(args.output).expanduser().resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(f"{rendered}\n", encoding="utf-8")
    else:
        print(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
