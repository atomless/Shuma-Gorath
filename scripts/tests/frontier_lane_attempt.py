#!/usr/bin/env python3

"""
Protected-lane frontier attempt probe.

This script is intentionally advisory: degraded frontier status does not fail
deterministic release gates. It emits machine-readable status for CI artifacts.
"""

from __future__ import annotations

import argparse
import json
import os
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Dict, List, Tuple


FRONTIER_PROVIDER_SPECS: List[Dict[str, str]] = [
    {
        "provider": "openai",
        "api_key_env": "SHUMA_FRONTIER_OPENAI_API_KEY",
        "model_env": "SHUMA_FRONTIER_OPENAI_MODEL",
        "default_model": "gpt-5-mini",
        "probe_url": "https://api.openai.com/v1/models?limit=1",
    },
    {
        "provider": "anthropic",
        "api_key_env": "SHUMA_FRONTIER_ANTHROPIC_API_KEY",
        "model_env": "SHUMA_FRONTIER_ANTHROPIC_MODEL",
        "default_model": "claude-3-5-haiku-latest",
        "probe_url": "https://api.anthropic.com/v1/models",
    },
    {
        "provider": "google",
        "api_key_env": "SHUMA_FRONTIER_GOOGLE_API_KEY",
        "model_env": "SHUMA_FRONTIER_GOOGLE_MODEL",
        "default_model": "gemini-2.0-flash-lite",
        "probe_url": "https://generativelanguage.googleapis.com/v1beta/models",
    },
    {
        "provider": "xai",
        "api_key_env": "SHUMA_FRONTIER_XAI_API_KEY",
        "model_env": "SHUMA_FRONTIER_XAI_MODEL",
        "default_model": "grok-3-mini",
        "probe_url": "https://api.x.ai/v1/models?limit=1",
    },
]


def env_trimmed(name: str) -> str:
    return str(os.environ.get(name, "")).strip()


def classify_http_status(http_status: int) -> str:
    if 200 <= http_status < 300:
        return "ok"
    if http_status in (401, 403):
        return "auth_error"
    if http_status == 429:
        return "rate_limited"
    if 500 <= http_status < 600:
        return "provider_error"
    return f"http_{http_status}"


def build_probe_request(provider: str, api_key: str, base_url: str) -> urllib.request.Request:
    if provider == "openai":
        return urllib.request.Request(
            base_url,
            headers={"Authorization": f"Bearer {api_key}"},
            method="GET",
        )
    if provider == "anthropic":
        return urllib.request.Request(
            base_url,
            headers={
                "x-api-key": api_key,
                "anthropic-version": "2023-06-01",
            },
            method="GET",
        )
    if provider == "google":
        parsed = urllib.parse.urlparse(base_url)
        query = urllib.parse.parse_qs(parsed.query, keep_blank_values=True)
        query["key"] = [api_key]
        encoded_query = urllib.parse.urlencode(query, doseq=True)
        url = urllib.parse.urlunparse(
            (
                parsed.scheme,
                parsed.netloc,
                parsed.path,
                parsed.params,
                encoded_query,
                parsed.fragment,
            )
        )
        return urllib.request.Request(url, method="GET")
    if provider == "xai":
        return urllib.request.Request(
            base_url,
            headers={"Authorization": f"Bearer {api_key}"},
            method="GET",
        )
    raise ValueError(f"unsupported provider for probe request: {provider}")


def probe_provider(provider: str, api_key: str, base_url: str, timeout_seconds: float) -> Dict[str, Any]:
    started = time.time()
    request = build_probe_request(provider, api_key, base_url)
    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            status_code = int(getattr(response, "status", 0) or 0)
            status = classify_http_status(status_code)
    except urllib.error.HTTPError as exc:
        status_code = int(getattr(exc, "code", 0) or 0)
        status = classify_http_status(status_code)
    except TimeoutError:
        status_code = 0
        status = "timeout"
    except urllib.error.URLError:
        status_code = 0
        status = "network_error"
    elapsed_ms = int(max(0.0, (time.time() - started) * 1000.0))
    return {
        "probe_status": status,
        "http_status": status_code,
        "probe_latency_ms": elapsed_ms,
    }


def build_provider_probe_results(timeout_seconds: float) -> List[Dict[str, Any]]:
    results: List[Dict[str, Any]] = []
    for spec in FRONTIER_PROVIDER_SPECS:
        provider = str(spec["provider"])
        api_key_env = str(spec["api_key_env"])
        model_env = str(spec["model_env"])
        default_model = str(spec["default_model"])
        probe_url = str(spec["probe_url"])
        api_key = env_trimmed(api_key_env)
        model_id = env_trimmed(model_env) or default_model
        configured = bool(api_key)

        provider_result: Dict[str, Any] = {
            "provider": provider,
            "model_id": model_id,
            "configured": configured,
            "probe_status": "not_configured",
            "http_status": 0,
            "probe_latency_ms": 0,
        }
        if configured:
            provider_result.update(
                probe_provider(
                    provider=provider,
                    api_key=api_key,
                    base_url=probe_url,
                    timeout_seconds=timeout_seconds,
                )
            )
        results.append(provider_result)
    return results


def summarize_frontier_lane(provider_results: List[Dict[str, Any]]) -> Tuple[str, str]:
    configured = [result for result in provider_results if result.get("configured") is True]
    if not configured:
        return (
            "degraded_missing_keys",
            "No frontier provider keys configured in protected lane; frontier attempt is degraded and advisory.",
        )

    healthy = [result for result in configured if result.get("probe_status") == "ok"]
    if len(healthy) == len(configured):
        return (
            "ok",
            "Frontier provider probe attempt succeeded for all configured providers.",
        )
    if healthy:
        return (
            "degraded_partial_provider_failure",
            "Frontier provider probe attempt partially succeeded; deterministic gates remain authoritative blockers.",
        )
    return (
        "degraded_provider_unavailable",
        "Frontier provider probe attempt could not confirm any configured provider; deterministic gates remain authoritative blockers.",
    )


def build_frontier_lane_status(timeout_seconds: float) -> Dict[str, Any]:
    provider_results = build_provider_probe_results(timeout_seconds=timeout_seconds)
    lane_status, advisory = summarize_frontier_lane(provider_results)
    configured_count = len([result for result in provider_results if result.get("configured") is True])
    healthy_count = len([result for result in provider_results if result.get("probe_status") == "ok"])
    return {
        "schema_version": "frontier-lane-status.v1",
        "generated_at_unix": int(time.time()),
        "frontier_required_on_protected_lane": True,
        "blocking": False,
        "deterministic_oracle_authoritative": True,
        "status": lane_status,
        "advisory": advisory,
        "provider_count_configured": configured_count,
        "provider_count_healthy": healthy_count,
        "providers": provider_results,
    }


def print_summary(status: Dict[str, Any]) -> None:
    lane_status = str(status.get("status", "unknown"))
    configured = int(status.get("provider_count_configured", 0) or 0)
    healthy = int(status.get("provider_count_healthy", 0) or 0)
    print(
        f"[frontier-lane] status={lane_status} configured={configured} healthy={healthy} blocking=false"
    )
    print(f"[frontier-lane] advisory={status.get('advisory', '')}")
    for provider in status.get("providers", []):
        print(
            "[frontier-lane] provider="
            f"{provider.get('provider')} model={provider.get('model_id')} configured={provider.get('configured')} "
            f"probe_status={provider.get('probe_status')} http_status={provider.get('http_status')} "
            f"latency_ms={provider.get('probe_latency_ms')}"
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Attempt frontier provider probes for protected CI lanes (advisory/non-blocking)."
    )
    parser.add_argument(
        "--output",
        default="scripts/tests/adversarial/frontier_lane_status.json",
        help="Output path for frontier lane status JSON",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=8.0,
        help="Per-provider probe timeout in seconds",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    status = build_frontier_lane_status(timeout_seconds=float(args.timeout_seconds))
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(status, indent=2), encoding="utf-8")
    print_summary(status)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
