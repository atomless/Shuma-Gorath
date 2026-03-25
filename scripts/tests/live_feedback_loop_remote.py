#!/usr/bin/env python3
"""Live shared-host feedback-loop proof against the active ssh-managed remote."""

from __future__ import annotations

import argparse
import base64
import json
import os
import shlex
import socket
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
HTTP_REQUEST_RETRY_ATTEMPTS = 3
HTTP_REQUEST_RETRY_DELAY_SECONDS = 2.0
SCRAPLING_PUBLIC_NETWORK_IDENTITIES_ENV = "ADVERSARY_SIM_SCRAPLING_PUBLIC_NETWORK_IDENTITIES"
ALLOWED_OVERSIGHT_APPLY_STAGES = {
    "eligible",
    "canary_applied",
    "watch_window_open",
    "improved",
    "refused",
    "rollback_applied",
}
SCRAPLING_COVERAGE_SCHEMA_VERSION = "scrapling_owned_defense_surface_coverage_v1"


def _looks_like_timeout(exc: Exception) -> bool:
    message = str(exc).lower()
    return "timed out" in message or "timeout" in message


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

    def _loopback_admin_forwarded_ip(self) -> str:
        env_values = self.remote_env or self.local_env
        raw_allowlist = str(env_values.get("SHUMA_ADMIN_IP_ALLOWLIST", "")).strip()
        if raw_allowlist:
            first_entry = raw_allowlist.split(",")[0].strip()
            if first_entry:
                return first_entry.split("/")[0].strip() or "127.0.0.1"
        return "127.0.0.1"

    def _public_host_header(self) -> str:
        parsed = urlparse(self.base_url)
        return parsed.netloc or "127.0.0.1"

    def _request_json(self, method: str, path: str, payload: dict[str, Any] | None = None) -> dict[str, Any]:
        body = None
        headers = self._admin_headers(include_json=payload is not None)
        if payload is not None:
            body = json.dumps(payload).encode("utf-8")
            headers["Idempotency-Key"] = _idempotency_key("live-feedback-loop")
        last_transport_error: Exception | None = None
        for attempt in range(1, HTTP_REQUEST_RETRY_ATTEMPTS + 1):
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
                break
            except urllib.error.HTTPError as exc:
                raw = exc.read().decode("utf-8", errors="replace")
                status = int(exc.code)
                break
            except (urllib.error.URLError, socket.timeout, TimeoutError) as exc:
                last_transport_error = exc
                if attempt >= HTTP_REQUEST_RETRY_ATTEMPTS:
                    raise
                time.sleep(HTTP_REQUEST_RETRY_DELAY_SECONDS)
        else:
            raise SmokeFailure(
                f"{method} {path} exhausted retry budget without a response: {last_transport_error}"
            )
        if status != 200:
            raise SmokeFailure(f"{method} {path} returned {status}: {raw[:240]}")
        try:
            return json.loads(raw or "{}")
        except json.JSONDecodeError as exc:
            raise SmokeFailure(f"{method} {path} returned invalid JSON: {exc}") from exc

    def _loopback_request_json(
        self,
        method: str,
        path: str,
        headers: dict[str, str],
        payload: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        if self.transport_mode != "ssh_loopback":
            raise SmokeFailure("Shared-host loopback proof requires SSH transport.")
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

    def _internal_request_json(
        self,
        method: str,
        path: str,
        payload: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
            "X-Forwarded-For": "127.0.0.1",
            "X-Forwarded-Proto": "https",
            "X-Shuma-Internal-Supervisor": "oversight-agent",
        }
        if self.forwarded_ip_secret:
            headers["X-Shuma-Forwarded-Secret"] = self.forwarded_ip_secret
        return self._loopback_request_json(method, path, headers, payload)

    def _verify_service_wrapper(self) -> str:
        if self.transport_mode != "ssh_loopback":
            raise SmokeFailure("Live shared-host feedback-loop proof requires SSH loopback transport.")
        service_name = self.receipt["runtime"]["service_name"]
        exec_start = self._run_ssh_command(
            f"systemctl show {shlex.quote(service_name)} --property=ExecStart --no-page"
        ).strip()
        if "run_with_oversight_supervisor.sh" in exec_start:
            return exec_start
        status_output = self._run_ssh_command(
            f"systemctl status {shlex.quote(service_name)} --no-pager"
        ).strip()
        if "run_with_oversight_supervisor.sh" not in status_output:
            raise SmokeFailure(
                "Remote service is not using scripts/run_with_oversight_supervisor.sh: "
                f"{exec_start or 'missing ExecStart output'}"
            )
        return status_output

    def _scrapling_public_network_identity_summary(self) -> dict[str, Any]:
        env_values = self.remote_env or self.local_env
        raw_value = str(env_values.get(SCRAPLING_PUBLIC_NETWORK_IDENTITIES_ENV, "") or "").strip()
        if not raw_value:
            raise SmokeFailure(
                f"Remote transport environment does not configure {SCRAPLING_PUBLIC_NETWORK_IDENTITIES_ENV}; "
                "live Scrapling loop proof requires at least one bounded http_proxy identity so "
                "geo_ip_policy coverage can be proven attacker-faithfully."
            )
        try:
            payload = json.loads(raw_value)
        except json.JSONDecodeError as exc:
            raise SmokeFailure(
                f"Remote transport environment has invalid {SCRAPLING_PUBLIC_NETWORK_IDENTITIES_ENV} JSON: {exc}"
            ) from exc
        if not isinstance(payload, list) or not payload:
            raise SmokeFailure(
                f"Remote transport environment must configure at least one bounded identity in "
                f"{SCRAPLING_PUBLIC_NETWORK_IDENTITIES_ENV} for live Scrapling proof."
            )
        valid_identities = []
        for item in payload:
            if not isinstance(item, dict):
                continue
            identity_id = str(item.get("identity_id") or "").strip()
            identity_class = str(item.get("identity_class") or "").strip()
            proxy_url = str(item.get("proxy_url") or "").strip()
            if not identity_id or identity_class != "http_proxy":
                continue
            if not proxy_url.startswith(("http://", "https://")):
                continue
            valid_identities.append(
                {
                    "identity_id": identity_id,
                    "identity_class": identity_class,
                    "expected_geo_country": str(item.get("expected_geo_country") or "").strip() or None,
                }
            )
        if not valid_identities:
            raise SmokeFailure(
                f"Remote transport environment {SCRAPLING_PUBLIC_NETWORK_IDENTITIES_ENV} does not contain a usable "
                "bounded http_proxy identity for live Scrapling proof."
            )
        return {
            "configured_count": len(valid_identities),
            "identity_ids": [item["identity_id"] for item in valid_identities],
            "expected_geo_countries": [
                item["expected_geo_country"]
                for item in valid_identities
                if item["expected_geo_country"] is not None
            ],
        }

    def _oversight_status_summary(self, payload: dict[str, Any]) -> dict[str, Any]:
        latest_run = payload.get("latest_run") or {}
        recent_runs = payload.get("recent_runs") or []
        return {
            "schema_version": payload.get("schema_version"),
            "execution_boundary": payload.get("execution_boundary"),
            "latest_run_id": latest_run.get("run_id"),
            "latest_trigger_kind": latest_run.get("trigger_kind"),
            "latest_apply_stage": (((latest_run.get("execution") or {}).get("apply") or {}).get("stage")),
            "recent_run_count": len(recent_runs),
        }

    def _validated_apply_stage(self, run: dict[str, Any], *, context: str) -> str:
        execution = run.get("execution") or {}
        apply = execution.get("apply") or {}
        stage = str(apply.get("stage") or "").strip()
        if stage not in ALLOWED_OVERSIGHT_APPLY_STAGES:
            raise SmokeFailure(
                f"{context} did not expose a valid oversight apply stage: {stage!r}"
            )
        return stage

    def _validate_episode_row_against_apply_stage(
        self,
        episode_row: dict[str, Any],
        *,
        apply_stage: str,
        sim_run_id: str,
    ) -> None:
        acceptance_status = str(episode_row.get("acceptance_status") or "").strip()
        completion_status = str(episode_row.get("completion_status") or "").strip()
        retention_status = str(episode_row.get("retention_status") or "").strip()

        if apply_stage in {"canary_applied", "watch_window_open"}:
            if acceptance_status != "accepted_canary":
                raise SmokeFailure(
                    f"Episode archive row for {sim_run_id} did not mark accepted_canary for stage {apply_stage}."
                )
            if completion_status != "open":
                raise SmokeFailure(
                    f"Episode archive row for {sim_run_id} did not remain open for stage {apply_stage}."
                )
            if retention_status != "pending":
                raise SmokeFailure(
                    f"Episode archive row for {sim_run_id} did not remain pending for stage {apply_stage}."
                )
        elif apply_stage == "improved":
            if acceptance_status != "accepted_canary" or completion_status != "completed" or retention_status != "retained":
                raise SmokeFailure(
                    f"Episode archive row for {sim_run_id} was not retained coherently for improved apply stage."
                )
        elif apply_stage == "rollback_applied":
            if acceptance_status != "accepted_canary" or completion_status != "completed" or retention_status != "rolled_back":
                raise SmokeFailure(
                    f"Episode archive row for {sim_run_id} was not rolled back coherently."
                )
        elif apply_stage in {"eligible", "refused"}:
            if acceptance_status not in {"not_accepted", "preview_only"}:
                raise SmokeFailure(
                    f"Episode archive row for {sim_run_id} exposed invalid acceptance status {acceptance_status!r} for stage {apply_stage}."
                )

    def _adversary_status_summary(self, payload: dict[str, Any]) -> dict[str, Any]:
        generation = payload.get("generation") or {}
        lane_diagnostics = payload.get("lane_diagnostics") or {}
        persisted_event_evidence = payload.get("persisted_event_evidence") or {}
        return {
            "phase": payload.get("phase"),
            "enabled": payload.get("adversary_sim_enabled"),
            "generation_active": payload.get("generation_active"),
            "run_id": payload.get("run_id"),
            "last_run_id": payload.get("last_run_id"),
            "tick_count": generation.get("tick_count"),
            "request_count": generation.get("request_count"),
            "generation_truth_basis": generation.get("truth_basis"),
            "lane_diagnostics_truth_basis": lane_diagnostics.get("truth_basis"),
            "persisted_event_run_id": persisted_event_evidence.get("run_id"),
            "persisted_event_count": persisted_event_evidence.get("monitoring_event_count"),
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

    def _wait_for_operator_snapshot_sim_run(self, sim_run_id: str) -> dict[str, Any]:
        def _predicate():
            payload = self._fetch_operator_snapshot()
            recent_runs = ((payload.get("adversary_sim") or {}).get("recent_runs") or [])
            for run in recent_runs:
                if run.get("run_id") == sim_run_id:
                    return payload
            return None

        return self._wait_for(
            _predicate,
            timeout_seconds=POST_SIM_AGENT_TIMEOUT_SECONDS,
            description=f"operator snapshot recent sim run {sim_run_id}",
        )

    def _operator_snapshot_summary(self, payload: dict[str, Any], *, sim_run_id: str) -> dict[str, Any]:
        adversary_sim = payload.get("adversary_sim") or {}
        coverage = adversary_sim.get("scrapling_owned_surface_coverage") or {}
        if coverage.get("schema_version") != SCRAPLING_COVERAGE_SCHEMA_VERSION:
            raise SmokeFailure(
                "Operator snapshot did not expose the Scrapling owned-surface coverage schema."
            )
        coverage_status = str(coverage.get("overall_status") or "").strip()
        if coverage_status != "covered":
            raise SmokeFailure(
                f"Scrapling owned-surface coverage was not covered for sim run {sim_run_id}: {coverage_status or 'missing'}"
            )
        if int(coverage.get("recent_scrapling_run_count") or 0) <= 0:
            raise SmokeFailure(
                f"Operator snapshot did not expose recent Scrapling run receipts for {sim_run_id}."
            )

        matching_run = None
        for run in adversary_sim.get("recent_runs") or []:
            if run.get("run_id") == sim_run_id:
                matching_run = run
                break
        if matching_run is None:
            raise SmokeFailure(
                f"Operator snapshot did not expose the completed Scrapling sim run {sim_run_id}."
            )
        if matching_run.get("lane") != "scrapling_traffic":
            raise SmokeFailure(
                f"Operator snapshot recent sim run {sim_run_id} was not Scrapling-backed."
            )
        observed_defense_keys = matching_run.get("observed_defense_keys") or []
        if not observed_defense_keys:
            raise SmokeFailure(
                f"Operator snapshot did not expose observed defense receipts for Scrapling sim run {sim_run_id}."
            )

        return {
            "schema_version": payload.get("schema_version"),
            "adversary_sim": {
                "coverage_status": coverage_status,
                "covered_surface_count": int(coverage.get("covered_surface_count") or 0),
                "uncovered_surface_count": int(coverage.get("uncovered_surface_count") or 0),
                "recent_run_id": matching_run.get("run_id"),
                "recent_run_lane": matching_run.get("lane"),
                "observed_defense_keys": observed_defense_keys,
            },
        }

    def _fetch_adversary_sim_status(self) -> dict[str, Any]:
        return self._request_json("GET", "/admin/adversary-sim/status")

    def _fetch_oversight_history(self) -> dict[str, Any]:
        payload = self._request_json("GET", "/admin/oversight/history")
        if payload.get("schema_version") != "oversight_history_v1":
            raise SmokeFailure(
                f"Unexpected oversight history schema_version: {payload.get('schema_version')!r}"
            )
        return payload

    def _fetch_recent_events(self) -> dict[str, Any]:
        payload = self._request_json("GET", "/admin/events?hours=2&limit=200")
        if "recent_events" not in payload:
            raise SmokeFailure("Recent events payload did not include recent_events.")
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
        payload = {"enabled": True, "lane": "scrapling_traffic"}
        try:
            if self.transport_mode == "ssh_loopback":
                response = self._loopback_request_json(
                    "POST",
                    "/admin/adversary-sim/control",
                    {
                        "Authorization": f"Bearer {self.api_key}",
                        "Accept": "application/json",
                        "Content-Type": "application/json",
                        "Idempotency-Key": _idempotency_key("live-feedback-loop"),
                        "Host": self._public_host_header(),
                        "Origin": self.base_url,
                        "Referer": f"{self.base_url}/dashboard",
                        "X-Forwarded-For": self._loopback_admin_forwarded_ip(),
                        "X-Forwarded-Proto": "https",
                        "X-Shuma-Forwarded-Secret": self.forwarded_ip_secret,
                    },
                    payload,
                )
            else:
                response = self._request_json("POST", "/admin/adversary-sim/control", payload)
        except Exception as exc:
            if not _looks_like_timeout(exc):
                raise
            status = self._fetch_adversary_sim_status()
            return {
                "operation_id": ((status.get("controller_lease") or {}).get("operation_id"))
                or "transport_timeout_pending",
                "decision": "accepted_via_timeout_fallback",
                "request_error": str(exc),
            }
        if not response.get("operation_id"):
            raise SmokeFailure("Adversary-sim enable response did not include operation_id.")
        return response

    def _disable_adversary_sim(self) -> dict[str, Any]:
        try:
            if self.transport_mode == "ssh_loopback":
                payload = self._loopback_request_json(
                    "POST",
                    "/admin/adversary-sim/control",
                    {
                        "Authorization": f"Bearer {self.api_key}",
                        "Accept": "application/json",
                        "Content-Type": "application/json",
                        "Idempotency-Key": _idempotency_key("live-feedback-loop"),
                        "Host": self._public_host_header(),
                        "Origin": self.base_url,
                        "Referer": f"{self.base_url}/dashboard",
                        "X-Forwarded-For": self._loopback_admin_forwarded_ip(),
                        "X-Forwarded-Proto": "https",
                        "X-Shuma-Forwarded-Secret": self.forwarded_ip_secret,
                    },
                    {"enabled": False},
                )
            else:
                payload = self._request_json("POST", "/admin/adversary-sim/control", {"enabled": False})
        except Exception as exc:
            if not _looks_like_timeout(exc):
                raise
            status = self._fetch_adversary_sim_status()
            return {
                "operation_id": ((status.get("controller_lease") or {}).get("operation_id"))
                or "transport_timeout_pending",
                "decision": "accepted_via_timeout_fallback",
                "request_error": str(exc),
            }
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

    def _status_has_lane_generation(self, payload: dict[str, Any]) -> bool:
        lanes = ((payload.get("lane_diagnostics") or {}).get("lanes") or {})
        for lane_payload in lanes.values():
            generated_requests = int((lane_payload or {}).get("generated_requests") or 0)
            beat_successes = int((lane_payload or {}).get("beat_successes") or 0)
            if generated_requests > 0 or beat_successes > 0:
                return True
        return False

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
            report["scrapling_public_network_identities"] = (
                self._scrapling_public_network_identity_summary()
            )
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
            periodic_apply_stage = self._validated_apply_stage(
                periodic_run,
                context="Periodic internal oversight trigger",
            )
            periodic_status = self._wait_for_agent_run(periodic_run_id)
            report["periodic_trigger"] = {
                "run_id": periodic_run_id,
                "decision_id": (
                    ((periodic_run.get("execution") or {}).get("decision") or {}).get("decision_id")
                ),
                "apply_stage": periodic_apply_stage,
                "status": periodic.get("status"),
                "latest_status": self._oversight_status_summary(periodic_status),
            }

            enable_response = self._enable_adversary_sim()
            running_observed = True
            try:
                running_status = self._wait_for_adversary_sim_running()
            except SmokeFailure:
                running_observed = False
                running_status = {
                    "phase": "not_observed",
                    "adversary_sim_enabled": True,
                    "generation_active": None,
                    "run_id": None,
                    "last_run_id": None,
                    "generation": {"tick_count": None, "request_count": None},
                }
            completion_status = self._wait_for_adversary_sim_completion()
            sim_run_id = str(completion_status.get("last_run_id") or "").strip()
            recent_events = self._fetch_recent_events()
            matching_event_count = sum(
                1
                for row in recent_events.get("recent_events") or []
                if row.get("sim_run_id") == sim_run_id and row.get("is_simulation") is True
            )
            if matching_event_count <= 0:
                raise SmokeFailure(
                    f"Adversary-sim run {sim_run_id} completed, but recent event surfaces did not expose persisted simulated traffic."
                )
            completed_generation = completion_status.get("generation") or {}
            completed_tick_count = int(completed_generation.get("tick_count") or 0)
            completed_request_count = int(completed_generation.get("request_count") or 0)
            persisted_event_evidence = completion_status.get("persisted_event_evidence") or {}
            persisted_event_evidence_count = int(
                persisted_event_evidence.get("monitoring_event_count") or 0
            )
            if completed_tick_count <= 0 or completed_request_count <= 0:
                raise SmokeFailure(
                    f"Adversary-sim status still under-reported completion counters for {sim_run_id}: "
                    f"tick_count={completed_tick_count} request_count={completed_request_count}"
                )
            if persisted_event_evidence.get("run_id") != sim_run_id or persisted_event_evidence_count <= 0:
                raise SmokeFailure(
                    f"Adversary-sim status did not surface persisted event evidence for completed run {sim_run_id}."
                )
            if not self._status_has_lane_generation(completion_status):
                raise SmokeFailure(
                    f"Adversary-sim status did not recover lane diagnostics for completed run {sim_run_id}."
                )
            operator_snapshot = self._wait_for_operator_snapshot_sim_run(sim_run_id)
            report["operator_snapshot"] = self._operator_snapshot_summary(
                operator_snapshot,
                sim_run_id=sim_run_id,
            )
            post_sim_status = self._wait_for_post_sim_agent_run(sim_run_id)
            history = self._fetch_oversight_history()
            matching_post_run = None
            for run in post_sim_status.get("recent_runs") or []:
                if run.get("trigger_kind") == "post_adversary_sim" and run.get("sim_run_id") == sim_run_id:
                    matching_post_run = run
                    break
            if matching_post_run is None:
                raise SmokeFailure(f"Post-sim oversight run for {sim_run_id} was not found in status history.")
            post_sim_apply_stage = self._validated_apply_stage(
                matching_post_run,
                context=f"Post-sim oversight run for {sim_run_id}",
            )
            history_rows = history.get("rows") or []
            latest_history_row = history_rows[0] if history_rows else {}
            history_apply_stage = (((latest_history_row.get("apply") or {}).get("stage")) or "")
            if history_apply_stage and history_apply_stage not in ALLOWED_OVERSIGHT_APPLY_STAGES:
                raise SmokeFailure(
                    f"Oversight history exposed an invalid apply stage: {history_apply_stage!r}"
                )
            episode_rows = ((history.get("episode_archive") or {}).get("rows")) or []
            matching_episode = None
            expected_decision_id = (
                ((matching_post_run.get("execution") or {}).get("decision") or {}).get("decision_id")
            )
            for row in episode_rows:
                if (
                    row.get("latest_decision_id") == expected_decision_id
                    or row.get("latest_sim_run_id") == sim_run_id
                ):
                    matching_episode = row
                    break
            if matching_episode is None:
                raise SmokeFailure(
                    f"Oversight history did not expose an episode-archive row for post-sim run {sim_run_id}."
                )
            if matching_episode.get("latest_sim_run_id") != sim_run_id:
                raise SmokeFailure(
                    f"Episode archive row for {sim_run_id} did not preserve latest_sim_run_id."
                )
            self._validate_episode_row_against_apply_stage(
                matching_episode,
                apply_stage=post_sim_apply_stage,
                sim_run_id=sim_run_id,
            )
            report["adversary_sim"] = {
                "enable_operation_id": enable_response.get("operation_id"),
                "running_observed": running_observed,
                "running": self._adversary_status_summary(running_status),
                "completed": self._adversary_status_summary(completion_status),
                "persisted_event_count": matching_event_count,
            }
            report["post_sim_trigger"] = {
                "sim_run_id": sim_run_id,
                "run_id": matching_post_run.get("run_id"),
                "decision_id": (
                    ((matching_post_run.get("execution") or {}).get("decision") or {}).get("decision_id")
                ),
                "apply_stage": post_sim_apply_stage,
                "history_latest_decision_id": latest_history_row.get("decision_id"),
                "history_latest_apply_stage": history_apply_stage or None,
                "episode_latest_sim_run_id": matching_episode.get("latest_sim_run_id"),
                "episode_acceptance_status": matching_episode.get("acceptance_status"),
                "episode_completion_status": matching_episode.get("completion_status"),
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
