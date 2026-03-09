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

DEFAULT_REPORT_PATH = REPO_ROOT / ".spin" / "remote_edge_signal_smoke.json"
FINGERPRINT_FIXTURE_DIR = REPO_ROOT / "scripts" / "tests" / "fixtures" / "akamai"
ADDITIVE_FIXTURE_PATH = FINGERPRINT_FIXTURE_DIR / "fingerprint_additive_deny_signal.json"
AUTHORITATIVE_FIXTURE_PATH = FINGERPRINT_FIXTURE_DIR / "fingerprint_authoritative_deny_signal.json"

FINGERPRINT_ADDITIVE_IP = "10.0.0.230"
FINGERPRINT_AUTHORITATIVE_IP = "10.0.0.231"
GEO_CHALLENGE_IP = "10.0.0.210"
GEO_MAZE_IP = "10.0.0.211"
GEO_BLOCK_IP = "10.0.0.212"


class SmokeFailure(RuntimeError):
    pass


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


def merge_nested_dicts(base: dict[str, Any], patch: dict[str, Any]) -> dict[str, Any]:
    merged = json.loads(json.dumps(base))
    for key, value in patch.items():
        if isinstance(value, dict) and isinstance(merged.get(key), dict):
            merged[key] = merge_nested_dicts(merged[key], value)
        else:
            merged[key] = value
    return merged


def nested_restore_payload(config: dict[str, Any]) -> dict[str, Any]:
    return {
        "provider_backends": {
            "fingerprint_signal": config.get("provider_backends", {}).get(
                "fingerprint_signal", "internal"
            )
        },
        "edge_integration_mode": config.get("edge_integration_mode", "off"),
        "cdp_detection_enabled": config.get("cdp_detection_enabled", True),
        "cdp_auto_ban": config.get("cdp_auto_ban", True),
        "geo_edge_headers_enabled": config.get("geo_edge_headers_enabled", False),
        "geo_risk": config.get("geo_risk", []),
        "geo_allow": config.get("geo_allow", []),
        "geo_challenge": config.get("geo_challenge", []),
        "geo_maze": config.get("geo_maze", []),
        "geo_block": config.get("geo_block", []),
        "maze_enabled": config.get("maze_enabled", True),
        "maze_auto_ban": config.get("maze_auto_ban", True),
    }


class RemoteEdgeSignalSmoke:
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
        self.report_path = report_path
        self.receipt = select_remote(remote_name, env_file, receipts_dir)
        self.base_url = self.receipt["runtime"]["public_base_url"].rstrip("/")
        self.transport_mode = self._select_transport_mode()
        self.local_env = read_env_file(env_file)
        self.remote_env: dict[str, str] | None = None
        env_values = self.local_env
        self.api_key = env_values.get("SHUMA_API_KEY", "").strip()
        if not self.api_key:
            raise SmokeFailure("SHUMA_API_KEY must be present in the active smoke transport env.")
        self.forwarded_ip_secret = env_values.get("SHUMA_FORWARDED_IP_SECRET", "").strip()
        self.admin_forwarded_ip = first_ip_from_allowlist(
            env_values.get("SHUMA_ADMIN_IP_ALLOWLIST", "").strip()
        ) or "127.0.0.1"
        self.ssl_context = self._build_ssl_context()
        self.original_config: dict[str, Any] | None = None
        self.checks: list[dict[str, Any]] = []
        self.restore_error: str = ""

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

    def _build_ssl_context(self):
        hostname = urlparse(self.base_url).hostname or ""
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

    def _trusted_headers(
        self,
        *,
        forwarded_ip: str,
        extra_headers: dict[str, str] | None = None,
    ) -> dict[str, str]:
        headers = {
            "X-Forwarded-For": forwarded_ip,
            "X-Forwarded-Proto": "https",
        }
        if self.forwarded_ip_secret:
            headers["X-Shuma-Forwarded-Secret"] = self.forwarded_ip_secret
        if extra_headers:
            headers.update(extra_headers)
        return headers

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
        request_headers = dict(headers or {})
        request = urllib.request.Request(url, data=body, method=method.upper())
        for key, value in request_headers.items():
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

    def _get_config(self) -> dict[str, Any]:
        _, body = self._request(
            "GET",
            "/admin/config",
            headers=self._trusted_headers(
                forwarded_ip=self.admin_forwarded_ip,
                extra_headers={"Authorization": f"Bearer {self.api_key}"},
            ),
        )
        try:
            payload = json.loads(body)
        except json.JSONDecodeError as exc:
            raise SmokeFailure(f"/admin/config returned invalid JSON: {exc}") from exc
        if not isinstance(payload, dict):
            raise SmokeFailure("/admin/config returned a non-object payload.")
        return payload

    def _patch_config(self, patch: dict[str, Any]) -> dict[str, Any]:
        body = json.dumps(patch).encode("utf-8")
        _, payload = self._request(
            "POST",
            "/admin/config",
            body=body,
            headers=self._trusted_headers(
                forwarded_ip=self.admin_forwarded_ip,
                extra_headers={
                    "Authorization": f"Bearer {self.api_key}",
                    "Content-Type": "application/json",
                },
            ),
        )
        try:
            parsed = json.loads(payload)
        except json.JSONDecodeError as exc:
            raise SmokeFailure(f"/admin/config update returned invalid JSON: {exc}") from exc
        if not isinstance(parsed, dict):
            raise SmokeFailure("/admin/config update returned a non-object payload.")
        return parsed

    def _unban(self, ip: str) -> None:
        self._request(
            "POST",
            f"/admin/unban?ip={ip}",
            headers=self._trusted_headers(
                forwarded_ip=self.admin_forwarded_ip,
                extra_headers={"Authorization": f"Bearer {self.api_key}"},
            ),
        )

    def _root_request(self, *, forwarded_ip: str, geo_country: str = "") -> tuple[int, str]:
        headers = self._trusted_headers(forwarded_ip=forwarded_ip)
        if geo_country:
            headers["X-Geo-Country"] = geo_country
        return self._request("GET", "/", headers=headers, expected_statuses=(200, 403))

    def _post_fingerprint_fixture(self, fixture_path: Path, *, forwarded_ip: str) -> tuple[int, str]:
        payload = fixture_path.read_bytes()
        return self._request(
            "POST",
            "/fingerprint-report",
            body=payload,
            headers=self._trusted_headers(
                forwarded_ip=forwarded_ip,
                extra_headers={"Content-Type": "application/json"},
            ),
        )

    def _record_check(self, name: str, ok: bool, details: str) -> None:
        self.checks.append({"name": name, "ok": ok, "details": details})
        prefix = "PASS" if ok else "FAIL"
        print(f"{prefix} {name}: {details}")

    def _assert_contains(self, body: str, needle: str, *, context: str) -> None:
        if needle not in body:
            raise SmokeFailure(f"{context}: expected {needle!r} in response body {body!r}")

    def _assert_contains_any(
        self,
        body: str,
        needles: tuple[str, ...],
        *,
        context: str,
    ) -> None:
        if any(needle in body for needle in needles):
            return
        raise SmokeFailure(
            f"{context}: expected one of {needles!r} in response body {body!r}"
        )

    def _run_additive_fingerprint_check(self) -> None:
        patch = {
            "provider_backends": {"fingerprint_signal": "external"},
            "edge_integration_mode": "additive",
            "cdp_detection_enabled": True,
            "cdp_auto_ban": True,
        }
        self._patch_config(patch)
        self._unban(FINGERPRINT_ADDITIVE_IP)
        _, body = self._post_fingerprint_fixture(
            ADDITIVE_FIXTURE_PATH,
            forwarded_ip=FINGERPRINT_ADDITIVE_IP,
        )
        self._assert_contains(
            body,
            "External fingerprint report received (additive)",
            context="additive fingerprint report",
        )
        status, followup = self._root_request(forwarded_ip=FINGERPRINT_ADDITIVE_IP)
        if status == 403 or "Access Blocked" in followup:
            raise SmokeFailure(
                f"additive fingerprint follow-up unexpectedly blocked: status={status} body={followup!r}"
            )
        self._record_check(
            "akamai_fingerprint_additive",
            True,
            "strong Akamai fixture is accepted without an immediate ban",
        )

    def _run_authoritative_fingerprint_check(self) -> None:
        patch = {
            "provider_backends": {"fingerprint_signal": "external"},
            "edge_integration_mode": "authoritative",
            "cdp_detection_enabled": True,
            "cdp_auto_ban": True,
        }
        self._patch_config(patch)
        self._unban(FINGERPRINT_AUTHORITATIVE_IP)
        _, body = self._post_fingerprint_fixture(
            AUTHORITATIVE_FIXTURE_PATH,
            forwarded_ip=FINGERPRINT_AUTHORITATIVE_IP,
        )
        self._assert_contains(
            body,
            "External fingerprint automation detected - banned",
            context="authoritative fingerprint report",
        )
        status, followup = self._root_request(forwarded_ip=FINGERPRINT_AUTHORITATIVE_IP)
        if status != 403 or "Access Blocked" not in followup:
            raise SmokeFailure(
                f"authoritative fingerprint follow-up did not block: status={status} body={followup!r}"
            )
        self._record_check(
            "akamai_fingerprint_authoritative",
            True,
            "strong Akamai fixture triggers immediate authoritative ban",
        )

    def _run_geo_check(
        self,
        *,
        name: str,
        patch: dict[str, Any],
        forwarded_ip: str,
        country: str,
        expect_status: int,
        expect_fragments: tuple[str, ...],
        details: str,
    ) -> None:
        self._patch_config(
            merge_nested_dicts(
                {
                    "provider_backends": {"fingerprint_signal": "internal"},
                    "edge_integration_mode": "off",
                },
                patch,
            )
        )
        status, body = self._root_request(forwarded_ip=forwarded_ip, geo_country=country)
        if status != expect_status:
            raise SmokeFailure(f"{name} returned {status}, expected {expect_status}: {body!r}")
        self._assert_contains_any(body, expect_fragments, context=name)
        self._record_check(name, True, details)

    def _restore_original_state(self) -> None:
        if self.original_config is None:
            return
        try:
            self._patch_config(nested_restore_payload(self.original_config))
            for ip in (FINGERPRINT_ADDITIVE_IP, FINGERPRINT_AUTHORITATIVE_IP):
                self._unban(ip)
        except Exception as exc:  # pragma: no cover - reported in the final JSON
            self.restore_error = str(exc)

    def write_report(self) -> None:
        self.report_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "remote": {
                "name": self.receipt["identity"]["name"],
                "base_url": self.base_url,
                "transport_mode": self.transport_mode,
            },
            "checks": self.checks,
            "restore_error": self.restore_error,
        }
        self.report_path.write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def run(self) -> int:
        failure: SmokeFailure | None = None
        try:
            self.original_config = self._get_config()
            self._run_additive_fingerprint_check()
            self._run_geo_check(
                name="trusted_geo_challenge",
                patch={
                    "geo_edge_headers_enabled": True,
                    "geo_risk": [],
                    "geo_allow": [],
                    "geo_challenge": ["BR"],
                    "geo_maze": [],
                    "geo_block": [],
                },
                forwarded_ip=GEO_CHALLENGE_IP,
                country="BR",
                expect_status=200,
                expect_fragments=("Puzzle",),
                details="trusted country header routes to challenge tier",
            )
            self._run_geo_check(
                name="trusted_geo_maze",
                patch={
                    "geo_edge_headers_enabled": True,
                    "geo_risk": [],
                    "geo_allow": [],
                    "geo_challenge": [],
                    "geo_maze": ["RU"],
                    "geo_block": [],
                    "maze_enabled": True,
                    "maze_auto_ban": False,
                },
                forwarded_ip=GEO_MAZE_IP,
                country="RU",
                expect_status=200,
                expect_fragments=('data-link-kind="maze"',),
                details="trusted country header routes to maze tier",
            )
            self._run_geo_check(
                name="trusted_geo_block",
                patch={
                    "geo_edge_headers_enabled": True,
                    "geo_risk": [],
                    "geo_allow": [],
                    "geo_challenge": [],
                    "geo_maze": [],
                    "geo_block": ["KP"],
                },
                forwarded_ip=GEO_BLOCK_IP,
                country="KP",
                expect_status=403,
                expect_fragments=("Access Blocked", "Access Restricted"),
                details="trusted country header routes to block tier",
            )
            self._run_authoritative_fingerprint_check()
        except SmokeFailure as exc:
            failure = exc
        finally:
            self._restore_original_state()
            if failure is not None:
                self._record_check("remote_edge_signal_smoke", False, str(failure))
            self.write_report()

        if self.restore_error:
            print(f"FAIL restore: {self.restore_error}", file=sys.stderr)
            return 1
        if failure is not None:
            print(f"FAIL remote edge signal smoke: {failure}", file=sys.stderr)
            return 1

        print(f"Report written: {self.report_path}")
        return 0


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
