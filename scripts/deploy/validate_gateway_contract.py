#!/usr/bin/env python3
"""Validate gateway env + Spin outbound host alignment for deployment."""

from __future__ import annotations

import os
import sys
import ast
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.spin_manifest import normalize_origin

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover - fallback for Python <3.11
    tomllib = None  # type: ignore[assignment]


def fail(message: str) -> int:
    print(f"❌ {message}", file=sys.stderr)
    return 1

def load_allowed_outbound_hosts(manifest_path: Path) -> list[str]:
    if not manifest_path.exists():
        raise ValueError(f"spin manifest not found: {manifest_path}")
    if tomllib is not None:
        with manifest_path.open("rb") as handle:
            manifest = tomllib.load(handle)
        component = (manifest.get("component") or {}).get("bot-defence")
        if not isinstance(component, dict):
            raise ValueError("missing [component.bot-defence] in spin manifest")
        hosts = component.get("allowed_outbound_hosts")
        if not isinstance(hosts, list):
            raise ValueError("component.bot-defence.allowed_outbound_hosts must be a list")
        return [str(raw or "").strip() for raw in hosts if str(raw or "").strip()]

    in_component = False
    hosts_literal = None
    for raw_line in manifest_path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            in_component = line == "[component.bot-defence]"
            continue
        if in_component and line.startswith("allowed_outbound_hosts"):
            _, rhs = line.split("=", 1)
            hosts_literal = rhs.strip()
            break

    if hosts_literal is None:
        raise ValueError("missing component.bot-defence.allowed_outbound_hosts in spin manifest")
    try:
        hosts = ast.literal_eval(hosts_literal)
    except (ValueError, SyntaxError) as exc:
        raise ValueError(
            "component.bot-defence.allowed_outbound_hosts must be a string list literal"
        ) from exc
    if not isinstance(hosts, list):
        raise ValueError("component.bot-defence.allowed_outbound_hosts must be a list")
    return [str(raw or "").strip() for raw in hosts if str(raw or "").strip()]


def main() -> int:
    runtime_env = os.getenv("SHUMA_RUNTIME_ENV", "runtime-prod").strip().lower()
    if runtime_env != "runtime-prod":
        print("ℹ️  Gateway deploy guardrails: runtime is not runtime-prod; strict outbound alignment check skipped.")
        return 0

    profile = os.getenv("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server").strip().lower()
    if profile not in {"shared-server", "edge-fermyon"}:
        return fail(
            "Invalid SHUMA_GATEWAY_DEPLOYMENT_PROFILE; expected shared-server or edge-fermyon"
        )

    upstream_raw = os.getenv("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "")
    try:
        normalized_upstream, upstream_scheme = normalize_origin(upstream_raw)
    except ValueError as exc:
        return fail(f"Invalid SHUMA_GATEWAY_UPSTREAM_ORIGIN: {exc}")

    if profile == "edge-fermyon" and upstream_scheme != "https":
        return fail(
            "SHUMA_GATEWAY_DEPLOYMENT_PROFILE=edge-fermyon requires SHUMA_GATEWAY_UPSTREAM_ORIGIN with https://"
        )

    manifest_path = Path(os.getenv("SHUMA_SPIN_MANIFEST", "spin.toml"))
    try:
        raw_hosts = load_allowed_outbound_hosts(manifest_path)
    except ValueError as exc:
        return fail(str(exc))

    normalized_hosts: set[str] = set()
    for host in raw_hosts:
        if "*" in host:
            return fail(
                "Wildcard entries in component.bot-defence.allowed_outbound_hosts are forbidden for runtime-prod"
            )
        if profile == "edge-fermyon" and ("${" in host or "$(" in host):
            return fail(
                "Variable-templated allowed_outbound_hosts entries are forbidden for SHUMA_GATEWAY_DEPLOYMENT_PROFILE=edge-fermyon"
            )
        try:
            normalized, _ = normalize_origin(host)
        except ValueError:
            return fail(
                f"Invalid allowed_outbound_hosts entry {host!r}; expected explicit scheme://host[:port]"
            )
        normalized_hosts.add(normalized)

    if normalized_upstream not in normalized_hosts:
        sorted_hosts = ", ".join(sorted(normalized_hosts)) if normalized_hosts else "<empty>"
        return fail(
            "SHUMA_GATEWAY_UPSTREAM_ORIGIN is not present in component.bot-defence.allowed_outbound_hosts "
            f"(expected {normalized_upstream}; allowed={sorted_hosts})"
        )

    print(
        "✅ Gateway outbound contract passed "
        f"(runtime_env={runtime_env}, profile={profile}, upstream={normalized_upstream})."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
