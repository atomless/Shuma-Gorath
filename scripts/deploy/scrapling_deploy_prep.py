"""Shared helper for deploy-time Scrapling scope/seed/runtime receipts."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Sequence
from urllib.parse import SplitResult, urlsplit, urlunsplit

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.setup_common import utc_now_iso, write_json
from scripts.tests import shared_host_scope
from scripts.tests import shared_host_seed_inventory


DEFAULT_RECEIPTS_DIR = REPO_ROOT / ".shuma" / "scrapling"
DEFAULT_REMOTE_STATE_DIR = "/opt/shuma-gorath/.shuma/adversary-sim"
DEFAULT_REMOTE_SCOPE_PATH = f"{DEFAULT_REMOTE_STATE_DIR}/scrapling-scope.json"
DEFAULT_REMOTE_SEED_PATH = f"{DEFAULT_REMOTE_STATE_DIR}/scrapling-seed-inventory.json"
DEFAULT_REMOTE_CRAWLDIR = f"{DEFAULT_REMOTE_STATE_DIR}/scrapling-crawldir"
DEFAULT_LOCAL_CRAWLDIR = REPO_ROOT / ".shuma" / "adversary-sim" / "scrapling-crawldir"
RECEIPT_SCHEMA = "shuma.scrapling.deploy_prep.v1"
SUPPORTED_RUNTIME_MODES = ("ssh_systemd", "external_supervisor")


class ScraplingDeployPrepError(ValueError):
    """Raised when deploy-time Scrapling preparation inputs are invalid."""


def _normalize_slug(value: str) -> str:
    normalized = re.sub(r"[^a-z0-9]+", "-", value.strip().lower()).strip("-")
    return normalized or "scrapling"


def _port_for_scheme(parts: SplitResult) -> int:
    if parts.port is not None:
        return parts.port
    if parts.scheme == "https":
        return 443
    if parts.scheme == "http":
        return 80
    raise ScraplingDeployPrepError(
        "public base URL must use http or https"
    )


def normalize_public_base_url(raw_value: str) -> str:
    candidate = str(raw_value).strip()
    if not candidate:
        raise ScraplingDeployPrepError("public_base_url is required")
    try:
        parts = urlsplit(candidate)
    except ValueError as exc:
        raise ScraplingDeployPrepError(
            f"public_base_url is not a valid URL: {candidate}"
        ) from exc
    if parts.scheme.lower() not in {"http", "https"}:
        raise ScraplingDeployPrepError("public_base_url must use http or https")
    hostname = (parts.hostname or "").strip().lower().rstrip(".")
    if not hostname:
        raise ScraplingDeployPrepError("public_base_url must include a host")
    port = parts.port
    default_port = 443 if parts.scheme.lower() == "https" else 80
    netloc = hostname if port in {None, default_port} else f"{hostname}:{port}"
    return urlunsplit((parts.scheme.lower(), netloc, "/", "", ""))


def _default_output_paths(public_base_url: str) -> tuple[Path, Path, Path]:
    hostname = urlsplit(public_base_url).hostname or "scrapling"
    slug = _normalize_slug(hostname)
    return (
        DEFAULT_RECEIPTS_DIR / f"{slug}.deploy-prep.json",
        DEFAULT_RECEIPTS_DIR / f"{slug}.scope.json",
        DEFAULT_RECEIPTS_DIR / f"{slug}.seed.json",
    )


def _scope_payload(allowed_hosts: list[str], *, require_https: bool) -> dict[str, Any]:
    return {
        "schema_version": shared_host_scope.SCHEMA_VERSION,
        "allowed_hosts": allowed_hosts,
        "denied_path_prefixes": list(shared_host_scope.BASELINE_DENIED_PATH_PREFIXES),
        "require_https": require_https,
        "deny_ip_literals": True,
    }


def _receipt_support_tier(runtime_mode: str) -> str:
    if runtime_mode == "ssh_systemd":
        return "supported_shared_host_runtime"
    return "deferred_edge_runtime"


def prepare_scrapling_deploy(
    *,
    public_base_url: str,
    runtime_mode: str,
    require_https: bool = True,
    receipt_output: Path | None = None,
    scope_output: Path | None = None,
    seed_output: Path | None = None,
    extra_seed_urls: Sequence[str] | None = None,
    remote_scope_path: str = DEFAULT_REMOTE_SCOPE_PATH,
    remote_seed_path: str = DEFAULT_REMOTE_SEED_PATH,
    remote_crawldir_path: str = DEFAULT_REMOTE_CRAWLDIR,
) -> dict[str, Any]:
    normalized_public_base_url = normalize_public_base_url(public_base_url)
    if runtime_mode not in SUPPORTED_RUNTIME_MODES:
        raise ScraplingDeployPrepError(
            f"runtime_mode must be one of: {', '.join(SUPPORTED_RUNTIME_MODES)}"
        )
    default_receipt, default_scope, default_seed = _default_output_paths(
        normalized_public_base_url
    )
    receipt_path = Path(receipt_output or default_receipt).expanduser().resolve()
    scope_path = Path(scope_output or default_scope).expanduser().resolve()
    seed_path = Path(seed_output or default_seed).expanduser().resolve()
    hostname = urlsplit(normalized_public_base_url).hostname or ""
    allowed_hosts = [hostname]
    scope_payload = _scope_payload(allowed_hosts, require_https=require_https)
    scope_descriptor = shared_host_scope.descriptor_from_payload(scope_payload)
    seed_payload = shared_host_seed_inventory.build_seed_inventory(
        scope_descriptor,
        primary_start_url=normalized_public_base_url,
        extra_seed_urls=list(extra_seed_urls or []),
    )

    write_json(scope_path, scope_payload)
    write_json(seed_path, seed_payload)

    parts = urlsplit(normalized_public_base_url)
    required_outbound_host = (
        f"{parts.scheme}://{hostname}:{_port_for_scheme(parts)}"
    )
    receipt = {
        "schema": RECEIPT_SCHEMA,
        "generated_at_utc": utc_now_iso(),
        "runtime_mode": runtime_mode,
        "support_tier": _receipt_support_tier(runtime_mode),
        "public_base_url": normalized_public_base_url,
        "scope": {
            "allowed_hosts": allowed_hosts,
            "descriptor_path": str(scope_path),
        },
        "seed": {
            "primary_start_url": normalized_public_base_url,
            "extra_seed_urls": list(extra_seed_urls or []),
            "robots_fetch_enabled": False,
            "inventory_path": str(seed_path),
        },
        "artifacts": {
            "scope_descriptor_path": str(scope_path),
            "seed_inventory_path": str(seed_path),
            "receipt_path": str(receipt_path),
        },
        "environment": {
            "local": {
                "ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH": str(scope_path),
                "ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH": str(seed_path),
                "ADVERSARY_SIM_SCRAPLING_CRAWLDIR": str(
                    DEFAULT_LOCAL_CRAWLDIR.resolve()
                ),
            },
            "remote": {
                "ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH": remote_scope_path,
                "ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH": remote_seed_path,
                "ADVERSARY_SIM_SCRAPLING_CRAWLDIR": remote_crawldir_path,
            },
        },
        "egress": {
            "required_outbound_hosts": [required_outbound_host],
            "notes": [
                "Constrain outbound to the approved public host plus DNS.",
                "Do not use deploy-time catalogs as the runtime reachable-surface map.",
            ],
        },
        "verification": {
            "commands": [
                "make test-shared-host-scope-contract",
                "make test-shared-host-seed-contract",
                "make verify-runtime",
            ]
        },
        "notes": [
            "shared-host-first supported runtime: full Scrapling adversary-sim automation currently targets shared-host deployments.",
            "The default deploy-time seed is just the normalized public root URL.",
        ],
    }
    if not require_https:
        receipt["notes"].append(
            "HTTP allowance is for controlled local or test runtimes only; deployment posture must continue to require HTTPS."
        )
    if runtime_mode == "external_supervisor":
        receipt["notes"].append(
            "Edge/external-supervisor runtime productization is deferred until there is a concrete deployment target worth supporting end to end."
        )
    write_json(receipt_path, receipt)
    return receipt


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Prepare deploy-time Scrapling scope/seed/runtime artifacts."
    )
    parser.add_argument(
        "--public-base-url",
        required=True,
        help="Canonical public base URL for the deploy target",
    )
    parser.add_argument(
        "--runtime-mode",
        required=True,
        choices=SUPPORTED_RUNTIME_MODES,
        help="Runtime adapter kind for the deploy target",
    )
    parser.add_argument(
        "--receipt-output",
        help="Where to write the Scrapling deploy-prep receipt",
    )
    parser.add_argument(
        "--scope-output",
        help="Where to write the shared-host scope descriptor",
    )
    parser.add_argument(
        "--seed-output",
        help="Where to write the minimal seed inventory",
    )
    parser.add_argument(
        "--extra-seed-url",
        action="append",
        default=[],
        help="Optional extra seed URL (repeatable)",
    )
    parser.add_argument(
        "--remote-scope-path",
        default=DEFAULT_REMOTE_SCOPE_PATH,
        help="Runtime remote path for the scope descriptor",
    )
    parser.add_argument(
        "--remote-seed-path",
        default=DEFAULT_REMOTE_SEED_PATH,
        help="Runtime remote path for the seed inventory",
    )
    parser.add_argument(
        "--remote-crawldir-path",
        default=DEFAULT_REMOTE_CRAWLDIR,
        help="Runtime remote path for the Scrapling crawldir",
    )
    parser.add_argument(
        "--allow-http",
        action="store_true",
        help="Allow HTTP primary URLs for controlled local or test runtimes",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    prepare_scrapling_deploy(
        public_base_url=args.public_base_url,
        runtime_mode=args.runtime_mode,
        require_https=not args.allow_http,
        receipt_output=Path(args.receipt_output).expanduser().resolve()
        if args.receipt_output
        else None,
        scope_output=Path(args.scope_output).expanduser().resolve()
        if args.scope_output
        else None,
        seed_output=Path(args.seed_output).expanduser().resolve()
        if args.seed_output
        else None,
        extra_seed_urls=args.extra_seed_url,
        remote_scope_path=args.remote_scope_path,
        remote_seed_path=args.remote_seed_path,
        remote_crawldir_path=args.remote_crawldir_path,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
