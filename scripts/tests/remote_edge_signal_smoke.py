#!/usr/bin/env python3
"""Live smoke for the implemented trusted-edge signal surfaces on the active remote."""

from __future__ import annotations

import argparse
import base64
import ipaddress
import json
import shlex
import ssl
import subprocess
import sys
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
    first_ip_from_allowlist,
    select_remote,
    ssh_command_for_operation,
)
from scripts.tests.edge_signal_smoke_common import EdgeSignalSmokeBase, SmokeFailure

DEFAULT_REPORT_PATH = REPO_ROOT / ".spin" / "remote_edge_signal_smoke.json"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run live trusted-edge signal smoke against the selected ssh_systemd remote "
            "(Akamai fingerprint fixtures + trusted GEO header routing)."
        )
    )
    parser.add_argument("--env-file", default=str(DEFAULT_ENV_FILE))
    parser.add_argument("--receipts-dir", default=str(DEFAULT_REMOTE_RECEIPTS_DIR))
    parser.add_argument("--name", help="Override the active remote target")
    parser.add_argument("--report-path", default=str(DEFAULT_REPORT_PATH))
    return parser.parse_args(argv)


class RemoteEdgeSignalSmoke(EdgeSignalSmokeBase):
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
        self.transport_mode = self._select_transport_mode()
        self.local_env = read_env_file(env_file)
        self.remote_env: dict[str, str] | None = None
        env_values = self.local_env
        api_key = env_values.get("SHUMA_API_KEY", "").strip()
        if not api_key:
            raise SmokeFailure("SHUMA_API_KEY must be present in the active smoke transport env.")
        forwarded_ip_secret = env_values.get("SHUMA_FORWARDED_IP_SECRET", "").strip()
        admin_forwarded_ip = first_ip_from_allowlist(
            env_values.get("SHUMA_ADMIN_IP_ALLOWLIST", "").strip()
        ) or "127.0.0.1"
        self.ssl_context = self._build_ssl_context(self.receipt["runtime"]["public_base_url"])
        super().__init__(
            base_url=self.receipt["runtime"]["public_base_url"],
            report_path=report_path,
            api_key=api_key,
            forwarded_ip_secret=forwarded_ip_secret,
            admin_forwarded_ip=admin_forwarded_ip,
            synthetic_forwarding=True,
        )

    def _target_report_key(self) -> str:
        return "remote"

    def _target_report_metadata(self) -> dict[str, Any]:
        return {
            "name": self.receipt["identity"]["name"],
            "base_url": self.base_url,
            "transport_mode": self.transport_mode,
        }

    def _select_transport_mode(self) -> str:
        ssh = self.receipt.get("ssh", {})
        host = str(ssh.get("host", "")).strip().lower()
        if not host:
            return "direct_http"
        if host == "localhost":
            return "direct_http"
        try:
            if ipaddress.ip_address(host).is_loopback:
                return "direct_http"
        except ValueError:
            pass
        private_key_path = Path(str(ssh.get("private_key_path", ""))).expanduser()
        if private_key_path.exists():
            return "ssh_loopback"
        return "direct_http"

    def _build_ssl_context(self, base_url: str):
        hostname = urlparse(base_url).hostname or ""
        if hostname.endswith(".sslip.io"):
            return ssl._create_unverified_context()
        return None

    def _read_remote_env(self) -> dict[str, str]:
        runtime = self.receipt["runtime"]
        remote_env_path = f"{runtime['app_dir']}/.env.local"
        result = subprocess.run(
            ssh_command_for_operation(self.receipt, f"cat {shlex.quote(remote_env_path)}"),
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            stderr = (result.stderr or "").strip()
            raise SmokeFailure(
                f"Failed to read remote env for edge signal smoke: {stderr or 'unknown SSH error'}"
            )
        return parse_env_text(result.stdout)

    def _ensure_transport_env_loaded(self) -> None:
        if self.transport_mode != "ssh_loopback" or self.remote_env is not None:
            return
        self.remote_env = self._read_remote_env()
        env_values = self.remote_env
        self.api_key = env_values.get("SHUMA_API_KEY", "").strip() or self.api_key
        self.forwarded_ip_secret = (
            env_values.get("SHUMA_FORWARDED_IP_SECRET", "").strip() or self.forwarded_ip_secret
        )
        self.admin_forwarded_ip = first_ip_from_allowlist(
            env_values.get("SHUMA_ADMIN_IP_ALLOWLIST", "").strip()
        ) or self.admin_forwarded_ip

    def _ssh_request(
        self,
        method: str,
        path: str,
        *,
        body: bytes | None = None,
        headers: dict[str, str] | None = None,
        expected_statuses: tuple[int, ...] = (200,),
    ) -> tuple[int, str]:
        self._ensure_transport_env_loaded()
        remote_script = """python3 - <<'PY'
import base64
import json
import os
import urllib.error
import urllib.request

headers = json.loads(os.environ["SHUMA_SMOKE_HEADERS_JSON"])
body_b64 = os.environ.get("SHUMA_SMOKE_BODY_B64", "")
data = base64.b64decode(body_b64) if body_b64 else None
request = urllib.request.Request(
    "http://127.0.0.1:3000" + os.environ["SHUMA_SMOKE_PATH"],
    data=data,
    method=os.environ["SHUMA_SMOKE_METHOD"],
)
for key, value in headers.items():
    request.add_header(key, value)
try:
    with urllib.request.urlopen(request, timeout=15) as response:
        payload = response.read().decode("utf-8", errors="replace")
        status = int(response.status)
except urllib.error.HTTPError as exc:
    payload = exc.read().decode("utf-8", errors="replace")
    status = int(exc.code)
print(json.dumps({"status": status, "body": payload}))
PY"""
        env_assignments = {
            "SHUMA_SMOKE_METHOD": method.upper(),
            "SHUMA_SMOKE_PATH": path,
            "SHUMA_SMOKE_HEADERS_JSON": json.dumps(headers or {}, sort_keys=True),
        }
        if body:
            env_assignments["SHUMA_SMOKE_BODY_B64"] = base64.b64encode(body).decode("ascii")
        result = subprocess.run(
            ssh_command_for_operation(
                self.receipt,
                f"{' '.join(f'{key}={shlex.quote(value)}' for key, value in env_assignments.items())} bash -c {shlex.quote(remote_script)}",
            ),
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            stderr = (result.stderr or "").strip()
            raise SmokeFailure(
                f"SSH loopback request failed for {method} {path}: {stderr or 'unknown SSH error'}"
            )
        try:
            payload = json.loads((result.stdout or "").strip())
        except json.JSONDecodeError as exc:
            raise SmokeFailure(
                f"SSH loopback request returned invalid JSON for {method} {path}: {exc}"
            ) from exc
        status = int(payload.get("status", 0))
        response_body = str(payload.get("body", ""))
        if status not in expected_statuses:
            raise SmokeFailure(f"{method} {path} returned {status}: {response_body}")
        return status, response_body

    def _request(
        self,
        method: str,
        path: str,
        *,
        body: bytes | None = None,
        headers: dict[str, str] | None = None,
        expected_statuses: tuple[int, ...] = (200,),
    ) -> tuple[int, str]:
        if self.transport_mode == "ssh_loopback":
            return self._ssh_request(
                method,
                path,
                body=body,
                headers=headers,
                expected_statuses=expected_statuses,
            )
        url = f"{self.base_url}{path}"
        request = urllib.request.Request(url, data=body, method=method.upper())
        for key, value in (headers or {}).items():
            request.add_header(key, value)
        try:
            with urllib.request.urlopen(
                request,
                timeout=15,
                context=self.ssl_context,
            ) as response:
                payload = response.read().decode("utf-8", errors="replace")
                status = int(response.status)
        except urllib.error.HTTPError as exc:
            payload = exc.read().decode("utf-8", errors="replace")
            status = int(exc.code)
        if status not in expected_statuses:
            raise SmokeFailure(f"{method} {path} returned {status}: {payload}")
        return status, payload


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    runner = RemoteEdgeSignalSmoke(
        env_file=Path(args.env_file).expanduser().resolve(),
        receipts_dir=Path(args.receipts_dir).expanduser().resolve(),
        remote_name=args.name,
        report_path=Path(args.report_path).expanduser().resolve(),
    )
    return runner.run()


if __name__ == "__main__":
    raise SystemExit(main())
