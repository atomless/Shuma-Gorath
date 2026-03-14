#!/usr/bin/env python3
"""Live trusted-edge signal smoke for the current Fermyon / Akamai deployment receipt."""

from __future__ import annotations

import argparse
import json
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

from scripts.deploy.fermyon_akamai_edge_setup import DEFAULT_DEPLOY_RECEIPT_PATH
from scripts.deploy.local_env import read_env_file
from scripts.tests.edge_signal_smoke_common import EdgeSignalSmokeBase, SmokeFailure

DEFAULT_ENV_FILE = REPO_ROOT / ".env.local"
DEFAULT_REPORT_PATH = REPO_ROOT / ".spin" / "fermyon_edge_signal_smoke.json"
AUTHORITATIVE_GUARDRAIL_FRAGMENT = (
    "enterprise multi-instance rollout cannot run with local-only rate/ban state in authoritative mode"
)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run live trusted-edge signal smoke against the current Fermyon / Akamai deploy receipt "
            "using real edge client identity semantics."
        )
    )
    parser.add_argument("--env-file", default=str(DEFAULT_ENV_FILE))
    parser.add_argument("--deploy-receipt", default=str(DEFAULT_DEPLOY_RECEIPT_PATH))
    parser.add_argument("--report-path", default=str(DEFAULT_REPORT_PATH))
    return parser.parse_args(argv)


def load_deploy_receipt(path: Path) -> dict[str, Any]:
    if not path.exists():
        raise SmokeFailure(f"Fermyon deploy receipt does not exist: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SmokeFailure(f"Fermyon deploy receipt is not valid JSON: {path} ({exc})") from exc
    if payload.get("schema") != "shuma.fermyon.akamai_edge_deploy.v1":
        raise SmokeFailure(
            f"Unexpected Fermyon deploy receipt schema for {path}: {payload.get('schema')!r}"
        )
    fermyon = payload.get("fermyon")
    if not isinstance(fermyon, dict):
        raise SmokeFailure("Fermyon deploy receipt is missing fermyon metadata.")
    primary_url = str(fermyon.get("primary_url") or "").strip()
    if not primary_url:
        raise SmokeFailure("Fermyon deploy receipt does not include fermyon.primary_url.")
    return payload


class FermyonEdgeSignalSmoke(EdgeSignalSmokeBase):
    def __init__(
        self,
        *,
        env_file: Path,
        deploy_receipt_path: Path,
        report_path: Path,
    ) -> None:
        self.env_file = env_file
        self.deploy_receipt_path = deploy_receipt_path
        self.deploy_receipt = load_deploy_receipt(deploy_receipt_path)
        env_values = read_env_file(env_file)
        api_key = env_values.get("SHUMA_API_KEY", "").strip()
        if not api_key:
            raise SmokeFailure("SHUMA_API_KEY must be present in the env file for Fermyon smoke.")
        forwarded_ip_secret = env_values.get("SHUMA_FORWARDED_IP_SECRET", "").strip()
        if not forwarded_ip_secret:
            raise SmokeFailure(
                "SHUMA_FORWARDED_IP_SECRET must be present in the env file for Fermyon smoke."
            )
        self.fermyon = self.deploy_receipt["fermyon"]
        self.ssl_context = self._build_ssl_context(str(self.fermyon["primary_url"]))
        super().__init__(
            base_url=str(self.fermyon["primary_url"]),
            report_path=report_path,
            api_key=api_key,
            forwarded_ip_secret=forwarded_ip_secret,
            admin_forwarded_ip=None,
            synthetic_forwarding=False,
        )

    def _build_ssl_context(self, base_url: str):
        hostname = urlparse(base_url).hostname or ""
        if hostname.endswith(".sslip.io"):
            return ssl._create_unverified_context()
        return None

    def _target_report_key(self) -> str:
        return "fermyon"

    def _target_report_metadata(self) -> dict[str, Any]:
        return {
            "app_name": str(self.fermyon.get("app_name") or ""),
            "app_id": str(self.fermyon.get("app_id") or ""),
            "base_url": self.base_url,
            "deploy_receipt_path": str(self.deploy_receipt_path),
            "setup_receipt_path": str(self.deploy_receipt.get("setup_receipt_path") or ""),
        }

    def _request(
        self,
        method: str,
        path: str,
        *,
        body: bytes | None = None,
        headers: dict[str, str] | None = None,
        expected_statuses: tuple[int, ...] = (200,),
    ) -> tuple[int, str]:
        url = f"{self.base_url}{path}"
        request = urllib.request.Request(url, data=body, method=method.upper())
        for key, value in (headers or {}).items():
            request.add_header(key, value)
        try:
            with urllib.request.urlopen(
                request,
                timeout=30,
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

    def _fetch_recent_logs(self) -> str:
        command = ["spin", "aka", "logs", "--since", "30m", "-n", "200"]
        app_id = str(self.fermyon.get("app_id") or "").strip()
        app_name = str(self.fermyon.get("app_name") or "").strip()
        account_id = str(self.fermyon.get("account_id") or "").strip()
        account_name = str(self.fermyon.get("account_name") or "").strip()
        if app_id:
            command.extend(["--app-id", app_id])
        elif app_name:
            command.extend(["--app-name", app_name])
        if account_id:
            command.extend(["--account-id", account_id])
        elif account_name:
            command.extend(["--account-name", account_name])
        result = subprocess.run(command, capture_output=True, text=True, check=False)
        if result.returncode != 0:
            stderr = (result.stderr or "").strip()
            stdout = (result.stdout or "").strip()
            raise SmokeFailure(
                f"spin aka logs failed while verifying Fermyon authoritative guardrail: {stderr or stdout or 'unknown error'}"
            )
        return result.stdout or ""

    def _authoritative_guardrail_details(self, status: int, body: str) -> str | None:
        if status != 503 or "Server configuration error" not in body:
            return None
        logs = self._fetch_recent_logs()
        if AUTHORITATIVE_GUARDRAIL_FRAGMENT not in logs:
            raise SmokeFailure(
                "authoritative fingerprint returned a configuration error on Fermyon, but the app logs did not show the expected enterprise-state guardrail"
            )
        return (
            "authoritative fingerprint is correctly guardrailed on enterprise Fermyon until "
            "distributed rate/ban state is enabled"
        )


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    runner = FermyonEdgeSignalSmoke(
        env_file=Path(args.env_file).expanduser().resolve(),
        deploy_receipt_path=Path(args.deploy_receipt).expanduser().resolve(),
        report_path=Path(args.report_path).expanduser().resolve(),
    )
    return runner.run()


if __name__ == "__main__":
    raise SystemExit(main())
