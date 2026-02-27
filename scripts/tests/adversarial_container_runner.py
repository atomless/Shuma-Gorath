#!/usr/bin/env python3
"""Host-side orchestrator for containerized black-box adversary runs."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Dict, List, Tuple


DEFAULT_IMAGE_TAG = "shuma-adversary-blackbox:local"
DEFAULT_WORKER_PATH = "scripts/tests/adversarial_container/worker.py"
DEFAULT_DOCKERFILE_PATH = "scripts/tests/adversarial_container/Dockerfile"
DEFAULT_BLACKBOX_REPORT = "scripts/tests/adversarial/container_blackbox_report.json"
DEFAULT_ISOLATION_REPORT = "scripts/tests/adversarial/container_isolation_report.json"
FORBIDDEN_ENV_PREFIXES = ("SHUMA_",)
FORBIDDEN_ENV_KEYS = {
    "SHUMA_API_KEY",
    "SHUMA_ADMIN_READONLY_API_KEY",
    "SHUMA_JS_SECRET",
    "SHUMA_CHALLENGE_SECRET",
    "SHUMA_HEALTH_SECRET",
    "SHUMA_FORWARDED_IP_SECRET",
}


def normalize_container_base_url(base_url: str) -> str:
    parsed = urllib.parse.urlparse(base_url)
    if parsed.hostname in {"127.0.0.1", "localhost"}:
        replacement = "host.docker.internal"
        netloc = replacement
        if parsed.port:
            netloc = f"{replacement}:{parsed.port}"
        return urllib.parse.urlunparse(
            (parsed.scheme, netloc, parsed.path, parsed.params, parsed.query, parsed.fragment)
        )
    return base_url


def target_origin(url: str) -> str:
    parsed = urllib.parse.urlparse(url)
    if not parsed.scheme or not parsed.netloc:
        raise RuntimeError(f"target base URL is invalid: {url}")
    return f"{parsed.scheme}://{parsed.netloc}"


def docker_available() -> bool:
    try:
        result = subprocess.run(
            ["docker", "version", "--format", "{{.Server.Version}}"],
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        return False
    return result.returncode == 0


def run_cmd(command: List[str], *, env: Dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, capture_output=True, text=True, check=False, env=env)


def ensure_image_built(image_tag: str, dockerfile_path: str) -> None:
    result = run_cmd(
        ["docker", "build", "-f", dockerfile_path, "-t", image_tag, "."]
    )
    if result.returncode != 0:
        raise RuntimeError(
            "failed to build adversary container image:\n"
            f"stdout:\n{result.stdout}\n\nstderr:\n{result.stderr}"
        )


def forwarded_headers(forwarded_secret: str, health_secret: str) -> Dict[str, str]:
    headers: Dict[str, str] = {"X-Forwarded-For": "127.0.0.1"}
    if forwarded_secret:
        headers["X-Shuma-Forwarded-Secret"] = forwarded_secret
    if health_secret:
        headers["X-Shuma-Health-Secret"] = health_secret
    return headers


def wait_ready(base_url: str, forwarded_secret: str, health_secret: str, timeout_seconds: int = 30) -> None:
    deadline = time.monotonic() + timeout_seconds
    health_url = base_url.rstrip("/") + "/health"
    headers = forwarded_headers(forwarded_secret, health_secret)
    while time.monotonic() < deadline:
        request = urllib.request.Request(health_url, method="GET")
        for key, value in headers.items():
            request.add_header(key, value)
        try:
            with urllib.request.urlopen(request, timeout=5.0) as response:
                body = response.read().decode("utf-8", errors="replace")
                if response.status == 200 and "OK" in body:
                    return
        except Exception:
            time.sleep(1)
            continue
        time.sleep(1)
    raise RuntimeError(f"Spin server was not ready at {health_url} within {timeout_seconds}s")


def orchestrator_reset_hook(base_url: str, api_key: str, forwarded_secret: str) -> Dict[str, Any]:
    if not api_key:
        return {
            "hook": "orchestrator_reset",
            "performed": False,
            "reason": "missing_api_key",
        }
    config_url = base_url.rstrip("/") + "/admin/config"
    payload = {
        "test_mode": False,
        "adversary_sim_enabled": False,
    }
    body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
    request = urllib.request.Request(config_url, method="POST", data=body)
    request.add_header("Authorization", f"Bearer {api_key}")
    request.add_header("Content-Type", "application/json")
    if forwarded_secret:
        request.add_header("X-Shuma-Forwarded-Secret", forwarded_secret)
    try:
        with urllib.request.urlopen(request, timeout=10.0) as response:
            return {
                "hook": "orchestrator_reset",
                "performed": response.status == 200,
                "status": response.status,
            }
    except urllib.error.HTTPError as exc:
        return {
            "hook": "orchestrator_reset",
            "performed": False,
            "status": exc.code,
            "error": f"http_error_{exc.code}",
        }
    except Exception as exc:
        return {
            "hook": "orchestrator_reset",
            "performed": False,
            "error": str(exc),
        }


def parse_worker_json(stdout_text: str) -> Dict[str, Any]:
    for line in reversed(stdout_text.splitlines()):
        candidate = line.strip()
        if not candidate:
            continue
        if not candidate.startswith("{"):
            continue
        try:
            parsed = json.loads(candidate)
        except Exception:
            continue
        if isinstance(parsed, dict):
            return parsed
    raise RuntimeError("container worker did not emit JSON payload")


def container_command(
    image_tag: str,
    mode: str,
    base_url: str,
    allowed_origin: str,
    run_id: str,
    request_budget: int,
    time_budget_seconds: int,
) -> List[str]:
    command = [
        "docker",
        "run",
        "--rm",
        "--read-only",
        "--cap-drop=ALL",
        "--security-opt=no-new-privileges",
        "--pids-limit=128",
        "--memory=256m",
        "--cpus=1.0",
        "--tmpfs=/tmp:rw,nosuid,nodev,size=64m",
        "--add-host=host.docker.internal:host-gateway",
        "--network=bridge",
        "-e",
        f"BLACKBOX_MODE={mode}",
        "-e",
        f"BLACKBOX_BASE_URL={base_url}",
        "-e",
        f"BLACKBOX_ALLOWED_ORIGINS={allowed_origin}",
        "-e",
        f"BLACKBOX_RUN_ID={run_id}",
        "-e",
        f"BLACKBOX_REQUEST_BUDGET={request_budget}",
        "-e",
        f"BLACKBOX_TIME_BUDGET_SECONDS={time_budget_seconds}",
        image_tag,
    ]
    return command


def run_container_worker(
    image_tag: str,
    mode: str,
    base_url: str,
    allowed_origin: str,
    run_id: str,
    request_budget: int,
    time_budget_seconds: int,
) -> Tuple[Dict[str, Any], subprocess.CompletedProcess[str], List[str]]:
    command = container_command(
        image_tag=image_tag,
        mode=mode,
        base_url=base_url,
        allowed_origin=allowed_origin,
        run_id=run_id,
        request_budget=request_budget,
        time_budget_seconds=time_budget_seconds,
    )
    result = run_cmd(command)
    parsed = parse_worker_json(result.stdout)
    return parsed, result, command


def report_path_for_mode(mode: str, custom_report: str) -> Path:
    if custom_report:
        return Path(custom_report)
    if mode == "isolation":
        return Path(DEFAULT_ISOLATION_REPORT)
    return Path(DEFAULT_BLACKBOX_REPORT)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run containerized black-box adversary worker and isolation checks"
    )
    parser.add_argument("--mode", choices=["isolation", "blackbox"], required=True)
    parser.add_argument(
        "--base-url",
        default=os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000"),
        help="Target base URL (host runtime).",
    )
    parser.add_argument("--image-tag", default=DEFAULT_IMAGE_TAG, help="Container image tag")
    parser.add_argument("--dockerfile", default=DEFAULT_DOCKERFILE_PATH, help="Dockerfile path")
    parser.add_argument("--worker-path", default=DEFAULT_WORKER_PATH, help="Worker script path")
    parser.add_argument("--report", default="", help="Optional report output path")
    parser.add_argument(
        "--request-budget",
        default=os.environ.get("SHUMA_BLACKBOX_REQUEST_BUDGET", "24"),
        help="Maximum worker requests",
    )
    parser.add_argument(
        "--time-budget-seconds",
        default=os.environ.get("SHUMA_BLACKBOX_TIME_BUDGET_SECONDS", "120"),
        help="Maximum worker runtime",
    )
    args = parser.parse_args()

    request_budget = int(str(args.request_budget).strip())
    time_budget_seconds = int(str(args.time_budget_seconds).strip())
    if request_budget < 1:
        print("request budget must be >= 1", file=sys.stderr)
        return 2
    if time_budget_seconds < 10:
        print("time budget must be >= 10 seconds", file=sys.stderr)
        return 2

    if not docker_available():
        print("docker is required for container adversary targets", file=sys.stderr)
        return 2
    if not Path(args.dockerfile).exists():
        print(f"Dockerfile not found: {args.dockerfile}", file=sys.stderr)
        return 2
    if not Path(args.worker_path).exists():
        print(f"worker path not found: {args.worker_path}", file=sys.stderr)
        return 2

    host_base_url = args.base_url.rstrip("/")
    container_base_url = normalize_container_base_url(host_base_url)
    allowed_origin = target_origin(container_base_url)
    run_id = f"container-{int(time.time())}"

    try:
        ensure_image_built(args.image_tag, args.dockerfile)
    except Exception as exc:
        print(f"[adversarial-container] {exc}", file=sys.stderr)
        return 1

    forwarded_secret = os.environ.get("SHUMA_FORWARDED_IP_SECRET", "").strip()
    health_secret = os.environ.get("SHUMA_HEALTH_SECRET", "").strip()
    api_key = os.environ.get("SHUMA_API_KEY", "").strip()
    orchestrator_hook = {"hook": "orchestrator_reset", "performed": False, "reason": "not_applicable"}

    if args.mode == "blackbox":
        try:
            wait_ready(host_base_url, forwarded_secret, health_secret, timeout_seconds=30)
        except Exception as exc:
            print(f"[adversarial-container] {exc}", file=sys.stderr)
            return 1
        orchestrator_hook = orchestrator_reset_hook(host_base_url, api_key, forwarded_secret)

    try:
        worker_payload, worker_result, command = run_container_worker(
            image_tag=args.image_tag,
            mode=args.mode,
            base_url=container_base_url,
            allowed_origin=allowed_origin,
            run_id=run_id,
            request_budget=request_budget,
            time_budget_seconds=time_budget_seconds,
        )
    except Exception as exc:
        print(f"[adversarial-container] worker execution failed: {exc}", file=sys.stderr)
        return 1

    has_forbidden_env = not bool(worker_payload.get("admin_credentials_absent"))
    isolation_contract = {
        "container_process_boundary": True,
        "workspace_mount_absent": bool(worker_payload.get("workspace_mount_absent")),
        "shuma_env_absent": bool(worker_payload.get("admin_credentials_absent")),
        "admin_credentials_absent": bool(worker_payload.get("admin_credentials_absent")),
        "egress_allowlist_enforced": bool(worker_payload.get("egress_allowlist_enforced")),
        "tooling_limited_http_browser_class": bool(worker_payload.get("tooling_limited")),
        "ephemeral_run_identity": bool(worker_payload.get("ephemeral_run_identity")),
        "orchestrator_reset_hook_outside_container": bool(orchestrator_hook.get("performed"))
        if args.mode == "blackbox"
        else True,
        "runtime_hardening_non_root": bool(worker_payload.get("runtime_hardening_non_root")),
        "runtime_hardening_no_new_privileges": True,
        "runtime_hardening_cap_drop_all": True,
        "runtime_hardening_read_only_rootfs": True,
    }
    contract_pass = all(isolation_contract.values())

    passed = bool(worker_payload.get("passed")) and worker_result.returncode == 0 and contract_pass
    report = {
        "schema_version": "adversarial-container-report.v1",
        "mode": args.mode,
        "passed": passed,
        "run_id": run_id,
        "host_base_url": host_base_url,
        "container_base_url": container_base_url,
        "allowed_origin": allowed_origin,
        "request_budget": request_budget,
        "time_budget_seconds": time_budget_seconds,
        "isolation_contract": isolation_contract,
        "orchestrator_hook": orchestrator_hook,
        "worker_result": {
            "exit_code": worker_result.returncode,
            "stdout_tail": worker_result.stdout.splitlines()[-20:],
            "stderr_tail": worker_result.stderr.splitlines()[-20:],
        },
        "worker_payload": worker_payload,
        "runtime_flags": {
            "docker_command": command,
        },
        "generated_at_unix": int(time.time()),
    }

    report_path = report_path_for_mode(args.mode, args.report)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"[adversarial-container] report={report_path}")
    print(
        "[adversarial-container] mode={} passed={} contract_pass={} exit_code={}".format(
            args.mode, passed, contract_pass, worker_result.returncode
        )
    )
    if not passed:
        print("[adversarial-container] isolation_contract=" + json.dumps(isolation_contract, sort_keys=True))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
