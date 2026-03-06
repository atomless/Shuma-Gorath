"""Helpers for deployment-specific Spin manifest rendering."""

from __future__ import annotations

import ast
import re
from urllib.parse import urlsplit

ALLOWED_OUTBOUND_HOSTS_PATTERN = re.compile(
    r"^\s*allowed_outbound_hosts\s*=\s*(\[[^\n]*\])\s*$",
    re.MULTILINE,
)


def normalize_origin(raw: str) -> tuple[str, str]:
    value = (raw or "").strip()
    if not value:
        raise ValueError("origin is empty")
    parsed = urlsplit(value)
    if parsed.scheme not in {"http", "https"}:
        raise ValueError("scheme must be http or https")
    if not parsed.hostname:
        raise ValueError("hostname is missing")
    if parsed.path not in {"", "/"} or parsed.query or parsed.fragment or parsed.username or parsed.password:
        raise ValueError("must not include path, query, fragment, or userinfo")
    port = parsed.port or (443 if parsed.scheme == "https" else 80)
    return f"{parsed.scheme}://{parsed.hostname.lower()}:{port}", parsed.scheme


def extract_allowed_outbound_hosts(manifest_text: str) -> list[str]:
    match = ALLOWED_OUTBOUND_HOSTS_PATTERN.search(manifest_text)
    if match is None:
        raise ValueError("spin manifest must define component.bot-defence allowed_outbound_hosts")
    try:
        hosts = ast.literal_eval(match.group(1))
    except (ValueError, SyntaxError) as exc:
        raise ValueError(
            "component.bot-defence.allowed_outbound_hosts must be a string list literal"
        ) from exc
    if not isinstance(hosts, list):
        raise ValueError("component.bot-defence.allowed_outbound_hosts must be a list")
    return [str(raw or "").strip() for raw in hosts if str(raw or "").strip()]


def build_manifest_with_allowed_outbound_hosts(manifest_text: str, allowed_hosts: list[str]) -> str:
    serialized_hosts = ", ".join(f'"{host}"' for host in allowed_hosts)
    replacement = f"allowed_outbound_hosts = [{serialized_hosts}]"
    rewritten, count = ALLOWED_OUTBOUND_HOSTS_PATTERN.subn(replacement, manifest_text, count=1)
    if count != 1:
        raise ValueError("spin manifest must define component.bot-defence allowed_outbound_hosts")
    return rewritten


def render_gateway_manifest(manifest_text: str, upstream_origin: str) -> str:
    normalized_upstream, _ = normalize_origin(upstream_origin)
    ordered_hosts: list[str] = []
    seen_hosts: set[str] = set()

    for host in [*extract_allowed_outbound_hosts(manifest_text), normalized_upstream]:
        normalized_host, _ = normalize_origin(host)
        if normalized_host in seen_hosts:
            continue
        seen_hosts.add(normalized_host)
        ordered_hosts.append(normalized_host)

    return build_manifest_with_allowed_outbound_hosts(manifest_text, ordered_hosts)
