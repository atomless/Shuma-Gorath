#!/usr/bin/env python3
"""Shared-host scope descriptor normalization and fail-closed URL validation."""

from __future__ import annotations

from dataclasses import dataclass
import ipaddress
from typing import Any, Mapping
from urllib.parse import SplitResult, urljoin, urlsplit, urlunsplit


SCHEMA_VERSION = "shared-host-scope-contract.v1"
REQUIRED_DESCRIPTOR_FIELDS = (
    "allowed_hosts",
    "denied_path_prefixes",
    "require_https",
    "deny_ip_literals",
)
DEFAULT_REQUIRE_HTTPS = True
DEFAULT_DENY_IP_LITERALS = True
BASELINE_DENIED_PATH_PREFIXES = (
    "/admin",
    "/internal",
    "/dashboard",
    "/session",
    "/auth",
    "/login",
)
REJECTION_REASONS = (
    "malformed_url",
    "missing_host",
    "non_https",
    "ip_literal_host",
    "host_not_allowed",
    "denied_path_prefix",
    "redirect_target_out_of_scope",
)


class SharedHostScopeError(ValueError):
    """Raised when a shared-host scope descriptor is invalid."""


@dataclass(frozen=True)
class SharedHostScopeDescriptor:
    allowed_hosts: tuple[str, ...]
    denied_path_prefixes: tuple[str, ...]
    require_https: bool = DEFAULT_REQUIRE_HTTPS
    deny_ip_literals: bool = DEFAULT_DENY_IP_LITERALS


@dataclass(frozen=True)
class ScopeDecision:
    allowed: bool
    normalized_url: str | None
    rejection_reason: str | None


def normalize_allowed_host_entry(raw_value: str) -> str:
    value = str(raw_value).strip().lower().rstrip(".")
    if not value:
        raise SharedHostScopeError("allowed_hosts entries must not be empty")
    if any(ch.isspace() for ch in value):
        raise SharedHostScopeError("allowed_hosts entries must not contain whitespace")
    if any(token in value for token in ("://", "/", "?", "#", "@")):
        raise SharedHostScopeError(
            "allowed_hosts entries must be host or authority values only"
        )
    return value


def normalize_denied_path_prefix(raw_value: str) -> str:
    value = str(raw_value).strip()
    if not value:
        raise SharedHostScopeError("denied_path_prefixes entries must not be empty")
    if not value.startswith("/"):
        raise SharedHostScopeError("denied_path_prefixes entries must start with /")
    if "://" in value or "?" in value or "#" in value:
        raise SharedHostScopeError(
            "denied_path_prefixes entries must be path prefixes only"
        )
    if len(value) > 1:
        value = value.rstrip("/")
    return value or "/"


def _dedupe_preserve_order(values: list[str]) -> tuple[str, ...]:
    seen: set[str] = set()
    ordered: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        ordered.append(value)
    return tuple(ordered)


def descriptor_from_payload(payload: Mapping[str, Any]) -> SharedHostScopeDescriptor:
    missing = [field for field in REQUIRED_DESCRIPTOR_FIELDS if field not in payload]
    if missing:
        raise SharedHostScopeError(
            f"shared-host scope descriptor missing fields: {', '.join(missing)}"
        )

    allowed_hosts_value = payload.get("allowed_hosts")
    if not isinstance(allowed_hosts_value, list):
        raise SharedHostScopeError("allowed_hosts must be a list")
    allowed_hosts = _dedupe_preserve_order(
        [normalize_allowed_host_entry(item) for item in allowed_hosts_value]
    )
    if not allowed_hosts:
        raise SharedHostScopeError("allowed_hosts must contain at least one entry")

    denied_prefixes_value = payload.get("denied_path_prefixes")
    if not isinstance(denied_prefixes_value, list):
        raise SharedHostScopeError("denied_path_prefixes must be a list")
    denied_path_prefixes = _dedupe_preserve_order(
        [
            *[
                normalize_denied_path_prefix(item)
                for item in denied_prefixes_value
            ],
            *BASELINE_DENIED_PATH_PREFIXES,
        ]
    )

    require_https = payload.get("require_https")
    if not isinstance(require_https, bool):
        raise SharedHostScopeError("require_https must be a boolean")
    deny_ip_literals = payload.get("deny_ip_literals")
    if not isinstance(deny_ip_literals, bool):
        raise SharedHostScopeError("deny_ip_literals must be a boolean")

    return SharedHostScopeDescriptor(
        allowed_hosts=allowed_hosts,
        denied_path_prefixes=denied_path_prefixes,
        require_https=require_https,
        deny_ip_literals=deny_ip_literals,
    )


def _normalize_candidate_parts(parts: SplitResult) -> tuple[str, str, str, str]:
    scheme = parts.scheme.lower()
    hostname = (parts.hostname or "").lower().rstrip(".")
    port = parts.port
    path = parts.path or "/"
    query = parts.query

    if ":" in hostname and not hostname.startswith("["):
        normalized_host = f"[{hostname}]"
    else:
        normalized_host = hostname

    netloc = normalized_host
    if port is not None:
        netloc = f"{netloc}:{port}"
    return scheme, hostname, netloc, path if path.startswith("/") else f"/{path}"


def _is_ip_literal(hostname: str) -> bool:
    candidate = hostname.strip().strip("[]")
    if not candidate:
        return False
    try:
        ipaddress.ip_address(candidate)
    except ValueError:
        return False
    return True


def _host_is_allowed(hostname: str, netloc: str, allowed_hosts: tuple[str, ...]) -> bool:
    return hostname in allowed_hosts or netloc in allowed_hosts


def _path_matches_prefix(path: str, prefix: str) -> bool:
    normalized_path = path or "/"
    if prefix == "/":
        return True
    if normalized_path == prefix:
        return True
    return normalized_path.startswith(f"{prefix}/")


def evaluate_url_candidate(
    raw_url: str, descriptor: SharedHostScopeDescriptor
) -> ScopeDecision:
    candidate = str(raw_url).strip()
    if not candidate:
        return ScopeDecision(False, None, "malformed_url")

    try:
        parts = urlsplit(candidate)
    except ValueError:
        return ScopeDecision(False, None, "malformed_url")

    if not parts.scheme or not parts.netloc:
        return ScopeDecision(False, None, "missing_host")

    try:
        scheme, hostname, netloc, path = _normalize_candidate_parts(parts)
    except ValueError:
        return ScopeDecision(False, None, "malformed_url")
    if not hostname:
        return ScopeDecision(False, None, "missing_host")

    if scheme not in ("http", "https"):
        return ScopeDecision(False, None, "malformed_url")

    if descriptor.require_https and scheme != "https":
        return ScopeDecision(False, None, "non_https")

    if descriptor.deny_ip_literals and _is_ip_literal(hostname):
        return ScopeDecision(False, None, "ip_literal_host")

    if not _host_is_allowed(hostname, netloc, descriptor.allowed_hosts):
        return ScopeDecision(False, None, "host_not_allowed")

    for prefix in descriptor.denied_path_prefixes:
        if _path_matches_prefix(path, prefix):
            return ScopeDecision(False, None, "denied_path_prefix")

    normalized_url = urlunsplit((scheme, netloc, path, parts.query, ""))
    return ScopeDecision(True, normalized_url, None)


def evaluate_redirect_target(
    current_url: str, redirect_target: str, descriptor: SharedHostScopeDescriptor
) -> ScopeDecision:
    base = str(current_url).strip()
    target = str(redirect_target).strip()
    if not base or not target:
        return ScopeDecision(False, None, "redirect_target_out_of_scope")

    try:
        resolved = urljoin(base, target)
    except ValueError:
        return ScopeDecision(False, None, "redirect_target_out_of_scope")

    decision = evaluate_url_candidate(resolved, descriptor)
    if decision.allowed:
        return decision
    return ScopeDecision(False, None, "redirect_target_out_of_scope")
