#!/usr/bin/env python3
"""Live shared-host feedback-loop proof against the active ssh-managed remote."""

from __future__ import annotations

import argparse
import base64
import json
import os
import shlex
import ssl
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.local_env import parse_env_text, read_env_file
from scripts.deploy.remote_target import (
    DEFAULT_ENV_FILE,
    DEFAULT_REMOTE_RECEIPTS_DIR,
    select_remote,
    ssh_command_for_operation,
)

DEFAULT_REPORT_PATH = REPO_ROOT / ".spin" / "live_feedback_loop_remote.json"
REPORT_SCHEMA_VERSION = "shuma.live_feedback_loop_remote.v1"
POLL_INTERVAL_SECONDS = 0.5
AGENT_RUN_TIMEOUT_SECONDS = 20
SIM_START_TIMEOUT_SECONDS = 60
SIM_COMPLETION_TIMEOUT_SECONDS = 420
POST_SIM_AGENT_TIMEOUT_SECONDS = 120


class SmokeFailure(RuntimeError):
    """Raised when the live feedback-loop proof fails."""


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run live shared-host feedback-loop proof against the selected ssh_systemd remote "
            "(wrapper contract, public oversight status, internal trigger execution, and post-sim linkage)."
        )
    )
    parser.add_argument("--env-file", default=str(DEFAULT_ENV_FILE))
    parser.add_argument("--receipts-dir", default=str(DEFAULT_REMOTE_RECEIPTS_DIR))
    parser.add_argument("--name", help="Override the active remote target")
    parser.add_argument("--report-path", default=str(DEFAULT_REPORT_PATH))
    return parser.parse_args(argv)


def _idempotency_key(prefix: str) -> str:
    return f"{prefix}-{int(time.time() * 1000):x}"


class LiveFeedbackLoopRemote:
    def __init__(
        self,
        *,
        env_file: Path,
        receipts_dir: Path,
        remote_name: str | None,
        report_path: Path,
    ) -> None:
        self.env_file = env_file
        self.receipts_dir = receipts_dir
        self.receipt = select_remote(remote_name, env_file, receipts_dir)
        self.report_path = report_path
        self.transport_mode = self._select_transport_mode()
        self.local_env = read_env_file(env_file)
        self.remote_env: dict[str, str] | None = None
        self.base_url = self.receipt["runtime"]["public_base_url"].rstrip("/")
        self.ssl_context = self._build_ssl_context(self.base_url)
        self.api_key = ""
        self.forwarded_ip_secret = ""
        self._load_transport_credentials()

    def _select_transport_mode(self) -> str:
        ssh = self.receipt.get("ssh", {})
        host = str(ssh.get("host", "")).strip().lower()
        if not host:
            return "direct_http"
        private_key_path = Path(str(ssh.get("private_key_path", ""))).expanduser()
        if private_key_path.exists():
            return "ssh_loopback"
        return "direct_http"

    def _build_ssl_context(self, base_url: str):
        hostname = urlparse(base_url).hostname or ""
        if hostname.endswith(".sslip.io"):
            return ssl._create_unverified_context()
        return None

    def _load_transport_credentials(self) -> None:
        env_values = self.local_env
        if self.transport_mode == "ssh_loopback":
            env_values = self._read_remote_env()
            self.remote_env = env_values
        self.api_key = env_values.get("SHUMA_API_KEY", "").strip()
        self.forwarded_ip_secret = env_values.get("SHUMA_FORWARDED_IP_SECRET", "").strip()
        if not self.api_key:
            raise SmokeFailure("SHUMA_API_KEY must be present in the selected transport environment.")

    def _read_remote_env(self) -> dict[str, str]:
        runtime = self.receipt["runtime"]
        remote_env_path = f"{runtime['app_dir']}/.env.local"
        output = self._run_ssh_command(f"cat {shlex.quote(remote_env_path)}")
        return parse_env_text(output)

    def _run_ssh_command(self, command: str) -> str:
        result = subprocess.run(
            ssh_command_for_operation(self.receipt, command),
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            stderr = (result.stderr or "").strip()
            raise SmokeFailure(stderr or f"SSH command failed: {command}")
        return result.stdout

    def _admin_headers(self, *, include_json: bool = False) -> dict[str, str]:
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Accept": "application/json",
            "Origin": self.base_url,
            "Cache-Control": "no-store",
            "Pragma": "no-cache",
        }
        if include_json:
            headers["Content-Type"] = "application/json"
        return headers

    def _request_json(self, method: str, path: str, payload: dict[str, Any] | None = None) -> dict[str, Any]:
        body = None
        headers = self._admin_headers(include_json=payload is not None)
        if payload is not None:
            body = json.dumps(payload).encode("utf-8")
            headers["Idempotency-Key"] = _idempotency_key("live-feedback-loop")
        request = urllib.request.Request(
            f"{self.base_url}{path}",
            data=body,
            method=method.upper(),
        )
        for key, value in headers.items():
            request.add_header(key, value)
        try:
            with urllib.request.urlopen(request, timeout=20, context=self.ssl_context) as response:
                raw = response.read().decode("utf-8", errors="replace")
                status = int(response.status)
        except urllib.error.HTTPError as exc:
            raw = exc.read().decode("utf-8", errors="replace")
            status = int(exc.code)
        if status != 200:
            raise SmokeFailure(f"{method} {path} returned {status}: {raw[:240]}")
        try:
            return json.loads(raw or "{}")
        except json.JSONDecodeError as exc:
            raise SmokeFailure(f"{method} {path} returned invalid JSON: {exc}") from exc

    def _internal_request_json(
        self,
        method: str,
        path: str,
        payload: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        if self.transport_mode != "ssh_loopback":
            raise SmokeFailure("Internal shared-host proof requires SSH loopback transport.")
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
            "X-Forwarded-For": "127.0.0.1",
            "X-Forwarded-Proto": "https",
            "X-Shuma-Internal-Supervisor": "oversight-agent",
        }
        if self.forwarded_ip_secret:
            headers["X-Shuma-Forwarded-Secret"] = self.forwarded_ip_secret
        body = json.dumps(payload or {}).encode("utf-8")
        remote_script = """python3 - <<'PY'
import base64
import json
import os
import urllib.error
import urllib.request

headers = json.loads(os.environ["SHUMA_HEADERS_JSON"])
body_b64 = os.environ["SHUMA_BODY_B64"]
data = base64.b64decode(body_b64)
request = urllib.request.Request(
    "http://127.0.0.1:3000" + os.environ["SHUMA_PATH"],
    data=data,
    method=os.environ["SHUMA_METHOD"],
)
for key, value in headers.items():
    request.add_header(key, value)
try:
    with urllib.request.urlopen(request, timeout=20) as response:
        payload = response.read().decode("utf-8", errors="replace")
        status = int(response.status)
except urllib.error.HTTPError as exc:
    payload = exc.read().decode("utf-8", errors="replace")
    status = int(exc.code)
print(json.dumps({"status": status, "body": payload}))
PY"""
        env_assignments = {
            "SHUMA_METHOD": method.upper(),
            "SHUMA_PATH": path,
            "SHUMA_HEADERS_JSON": json.dumps(headers, sort_keys=True),
            "SHUMA_BODY_B64": base64.b64encode(body).decode("ascii"),
        }
        quoted_assignments = " ".join(
            f"{key}={shlex.quote(value)}" for key, value in env_assignments.items()
        )
        raw = self._run_ssh_command(f"{quoted_assignments} bash -c {shlex.quote(remote_script)}")
        try:
            envelope = json.loads(raw.strip() or "{}")
        except json.JSONDecodeError as exc:
            raise SmokeFailure(
                f"Internal {method} {path} returned invalid JSON envelope: {exc}"
            ) from exc
        status = int(envelope.get("status", 0))
        response_body = str(envelope.get("body", ""))
        if status != 200:
            raise SmokeFailure(f"Internal {method} {path} returned {status}: {response_body[:240]}")
        try:
            return json.loads(response_body or "{}")
        except json.JSONDecodeError as exc:
            raise SmokeFailure(
                f"Internal {method} {path} returned invalid JSON payload: {exc}"
            ) from exc

    def _verify_service_wrapper(self) -> str:
        if self.transport_mode != "ssh_loopback":
            raise SmokeFailure("Live shared-host feedback-loop proof requires SSH loopback transport.")
        service_name = self.receipt["runtime"]["service_name"]
        exec_start = self._run_ssh_command(
            f"systemctl show {shlex.quote(service_name)} --property=ExecStart --no-page"
        ).strip()
        if "run_with_oversight_supervisor.sh" not in exec_start:
            raise SmokeFailure(
                "Remote service is not using scripts/run_with_oversight_supervisor.sh: "
                f"{exec_start or 'missing ExecStart output'}"
            )
        return exec_start

    def _oversight_status_summary(self, payload: dict[str, Any]) -> dict[str, Any]:
        latest_run = payload.get("latest_run") or {}
        recent_runs = payload.get("recent_runs") or []
        return {
            "schema_version": payload.get("schema_version"),
            "execution_boundary": payload.get("execution_boundary"),
            "latest_run_id": latest_run.get("run_id"),
            "latest_trigger_kind": latest_run.get("trigger_kind"),
            "recent_run_count": len(recent_runs),
        }

    def _adversary_status_summary(self, payload: dict[str, Any]) -> dict[str, Any]:
        generation = payload.get("generation") or {}
        return {
            "phase": payload.get("phase"),
            "enabled": payload.get("adversary_sim_enabled"),
            "generation_active": payload.get("generation_active"),
            "run_id": payload.get("run_id"),
            "last_run_id": payload.get("last_run_id"),
            "tick_count": generation.get("tick_count"),
            "request_count": generation.get("request_count"),
        }

    def _status_is_off(self, payload: dict[str, Any]) -> bool:
        return (
            payload.get("adversary_sim_enabled") is not True
            and payload.get("generation_active") is not True
            and str(payload.get("phase", "")).strip().lower() == "off"
        )

    def _status_is_running(self, payload: dict[str, Any]) -> bool:
        return payload.get("generation_active") is True or str(payload.get("phase", "")).strip().lower() == "running"

    def _wait_for(self, predicate, *, timeout_seconds: float, description: str):
        deadline = time.monotonic() + timeout_seconds
        last_value = None
        while time.monotonic() < deadline:
            last_value = predicate()
            if last_value:
                return last_value
            time.sleep(POLL_INTERVAL_SECONDS)
        raise SmokeFailure(f"Timed out waiting for {description}.")

    def _fetch_oversight_status(self) -> dict[str, Any]:
        payload = self._request_json("GET", "/admin/oversight/agent/status")
        if payload.get("schema_version") != "oversight_agent_status_v1":
            raise SmokeFailure(
                f"Unexpected oversight status schema_version: {payload.get('schema_version')!r}"
            )
        if payload.get("execution_boundary") != "shared_host_only":
            raise SmokeFailure(
                f"Unexpected oversight execution boundary: {payload.get('execution_boundary')!r}"
            )
        wrapper_command = (
            (payload.get("periodic_trigger") or {}).get("wrapper_command") or ""
        )
        if wrapper_command != "scripts/run_with_oversight_supervisor.sh":
            raise SmokeFailure(
                f"Unexpected periodic wrapper command in oversight status: {wrapper_command!r}"
            )
        return payload

    def _fetch_operator_snapshot(self) -> dict[str, Any]:
        payload = self._request_json("GET", "/admin/operator-snapshot")
        if payload.get("schema_version") != "operator_snapshot_v1":
            raise SmokeFailure(
                f"Unexpected operator snapshot schema_version: {payload.get('schema_version')!r}"
            )
        return payload

    def _fetch_adversary_sim_status(self) -> dict[str, Any]:
        return self._request_json("GET", "/admin/adversary-sim/status")

    def _fetch_oversight_history(self) -> dict[str, Any]:
        payload = self._request_json("GET", "/admin/oversight/history")
        if payload.get("schema_version") != "oversight_history_v1":
            raise SmokeFailure(
                f"Unexpected oversight history schema_version: {payload.get('schema_version')!r}"
            )
        return payload

    def _trigger_periodic_agent_run(self) -> dict[str, Any]:
        payload = self._internal_request_json(
            "POST",
            "/internal/oversight/agent/run",
            {"trigger_kind": "periodic_supervisor"},
        )
        if payload.get("schema_version") != "oversight_agent_execution_v1":
            raise SmokeFailure(
                f"Unexpected oversight execution schema_version: {payload.get('schema_version')!r}"
            )
        return payload

    def _wait_for_agent_run(self, run_id: str) -> dict[str, Any]:
        def _predicate():
            payload = self._fetch_oversight_status()
            for run in payload.get("recent_runs") or []:
                if run.get("run_id") == run_id:
                    return payload
            return None

        return self._wait_for(
            _predicate,
            timeout_seconds=AGENT_RUN_TIMEOUT_SECONDS,
            description=f"oversight agent run {run_id}",
        )

    def _enable_adversary_sim(self) -> dict[str, Any]:
        payload = self._request_json("POST", "/admin/adversary-sim/control", {"enabled": True})
        if not payload.get("operation_id"):
            raise SmokeFailure("Adversary-sim enable response did not include operation_id.")
        return payload

    def _disable_adversary_sim(self) -> dict[str, Any]:
        payload = self._request_json("POST", "/admin/adversary-sim/control", {"enabled": False})
        if not payload.get("operation_id"):
            raise SmokeFailure("Adversary-sim disable response did not include operation_id.")
        return payload

    def _ensure_adversary_sim_disabled(self) -> None:
        status = self._fetch_adversary_sim_status()
        if self._status_is_off(status):
            return
        self._disable_adversary_sim()

        def _predicate():
            current = self._fetch_adversary_sim_status()
            if self._status_is_off(current):
                return current
            return None

        self._wait_for(
            _predicate,
            timeout_seconds=SIM_START_TIMEOUT_SECONDS,
            description="adversary-sim off state",
        )

    def _wait_for_adversary_sim_running(self) -> dict[str, Any]:
        def _predicate():
            payload = self._fetch_adversary_sim_status()
            if self._status_is_running(payload):
                return payload
            return None

        return self._wait_for(
            _predicate,
            timeout_seconds=SIM_START_TIMEOUT_SECONDS,
            description="adversary-sim running state",
        )

    def _wait_for_adversary_sim_completion(self) -> dict[str, Any]:
        def _predicate():
            payload = self._fetch_adversary_sim_status()
            if self._status_is_off(payload):
                return payload
            return None

        payload = self._wait_for(
            _predicate,
            timeout_seconds=SIM_COMPLETION_TIMEOUT_SECONDS,
            description="adversary-sim completion",
        )
        generation = payload.get("generation") or {}
        if int(generation.get("tick_count") or 0) <= 0 or int(generation.get("request_count") or 0) <= 0:
            raise SmokeFailure(
                "Adversary-sim completed without generated traffic; expected non-zero tick_count and request_count."
            )
        if not payload.get("last_run_id"):
            raise SmokeFailure("Adversary-sim completion did not expose last_run_id.")
        return payload

    def _wait_for_post_sim_agent_run(self, sim_run_id: str) -> dict[str, Any]:
        def _predicate():
            payload = self._fetch_oversight_status()
            for run in payload.get("recent_runs") or []:
                if (
                    run.get("trigger_kind") == "post_adversary_sim"
                    and run.get("sim_run_id") == sim_run_id
                ):
                    return payload
            return None

        return self._wait_for(
            _predicate,
            timeout_seconds=POST_SIM_AGENT_TIMEOUT_SECONDS,
            description=f"post-sim agent run for {sim_run_id}",
        )

    def _write_report(self, report: dict[str, Any]) -> None:
        self.report_path.parent.mkdir(parents=True, exist_ok=True)
        self.report_path.write_text(
            json.dumps(report, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def run(self) -> int:
        report: dict[str, Any] = {
            "schema_version": REPORT_SCHEMA_VERSION,
            "result": "fail",
            "target": {
                "name": self.receipt["identity"]["name"],
                "provider_kind": self.receipt["identity"]["provider_kind"],
                "transport_mode": self.transport_mode,
                "base_url": self.base_url,
                "last_deployed_commit": self.receipt["metadata"].get("last_deployed_commit"),
                "last_deployed_at_utc": self.receipt["metadata"].get("last_deployed_at_utc"),
            },
        }
        try:
            report["service_exec"] = self._verify_service_wrapper()
            report["operator_snapshot"] = {
                "schema_version": self._fetch_operator_snapshot().get("schema_version")
            }
            initial_oversight = self._fetch_oversight_status()
            report["initial_oversight"] = self._oversight_status_summary(initial_oversight)

            self._ensure_adversary_sim_disabled()
            periodic = self._trigger_periodic_agent_run()
            periodic_run = periodic.get("run") or {}
            periodic_run_id = str(periodic_run.get("run_id") or "").strip()
            if not periodic_run_id:
                raise SmokeFailure("Periodic internal oversight trigger did not return run_id.")
            periodic_status = self._wait_for_agent_run(periodic_run_id)
            report["periodic_trigger"] = {
                "run_id": periodic_run_id,
                "decision_id": (
                    ((periodic_run.get("execution") or {}).get("decision") or {}).get("decision_id")
                ),
                "status": periodic.get("status"),
                "latest_status": self._oversight_status_summary(periodic_status),
            }

            enable_response = self._enable_adversary_sim()
            running_status = self._wait_for_adversary_sim_running()
            completion_status = self._wait_for_adversary_sim_completion()
            sim_run_id = str(completion_status.get("last_run_id") or "").strip()
            post_sim_status = self._wait_for_post_sim_agent_run(sim_run_id)
            history = self._fetch_oversight_history()
            matching_post_run = None
            for run in post_sim_status.get("recent_runs") or []:
                if run.get("trigger_kind") == "post_adversary_sim" and run.get("sim_run_id") == sim_run_id:
                    matching_post_run = run
                    break
            if matching_post_run is None:
                raise SmokeFailure(f"Post-sim oversight run for {sim_run_id} was not found in status history.")
            report["adversary_sim"] = {
                "enable_operation_id": enable_response.get("operation_id"),
                "running": self._adversary_status_summary(running_status),
                "completed": self._adversary_status_summary(completion_status),
            }
            report["post_sim_trigger"] = {
                "sim_run_id": sim_run_id,
                "run_id": matching_post_run.get("run_id"),
                "decision_id": (
                    ((matching_post_run.get("execution") or {}).get("decision") or {}).get("decision_id")
                ),
                "history_latest_decision_id": (
                    ((history.get("rows") or [{}])[0]).get("decision_id")
                ),
                "latest_status": self._oversight_status_summary(post_sim_status),
            }
            report["result"] = "pass"
            self._write_report(report)
            print(
                "[live-feedback-loop-remote] PASS "
                f"target={report['target']['name']} "
                f"periodic_run={report['periodic_trigger']['run_id']} "
                f"post_sim_run={report['post_sim_trigger']['run_id']} "
                f"sim_run={report['post_sim_trigger']['sim_run_id']}"
            )
            return 0
        except SmokeFailure as exc:
            report["error"] = str(exc)
            self._write_report(report)
            print(f"[live-feedback-loop-remote] FAIL {exc}", file=sys.stderr)
            return 1
        finally:
            try:
                self._ensure_adversary_sim_disabled()
            except Exception:
                pass


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    runner = LiveFeedbackLoopRemote(
        env_file=Path(args.env_file).expanduser().resolve(),
        receipts_dir=Path(args.receipts_dir).expanduser().resolve(),
        remote_name=args.name,
        report_path=Path(args.report_path).expanduser().resolve(),
    )
    return runner.run()


if __name__ == "__main__":
    raise SystemExit(main())
