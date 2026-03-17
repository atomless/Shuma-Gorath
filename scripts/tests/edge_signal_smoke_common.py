#!/usr/bin/env python3
"""Shared live trusted-edge signal smoke helpers."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

FINGERPRINT_FIXTURE_DIR = Path(__file__).resolve().parents[0] / "fixtures" / "akamai"
ADDITIVE_FIXTURE_PATH = FINGERPRINT_FIXTURE_DIR / "fingerprint_additive_deny_signal.json"
AUTHORITATIVE_FIXTURE_PATH = FINGERPRINT_FIXTURE_DIR / "fingerprint_authoritative_deny_signal.json"

FINGERPRINT_ADDITIVE_IP = "10.0.0.230"
FINGERPRINT_AUTHORITATIVE_IP = "10.0.0.231"
GEO_CHALLENGE_IP = "10.0.0.210"
GEO_MAZE_IP = "10.0.0.211"
GEO_BLOCK_IP = "10.0.0.212"


class SmokeFailure(RuntimeError):
    pass


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


class EdgeSignalSmokeBase:
    def __init__(
        self,
        *,
        base_url: str,
        report_path: Path,
        api_key: str,
        forwarded_ip_secret: str,
        admin_forwarded_ip: str | None,
        synthetic_forwarding: bool,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.report_path = report_path
        self.api_key = api_key
        self.forwarded_ip_secret = forwarded_ip_secret
        self.admin_forwarded_ip = (admin_forwarded_ip or "").strip() or None
        self.synthetic_forwarding = synthetic_forwarding
        self.original_config: dict[str, Any] | None = None
        self.checks: list[dict[str, Any]] = []
        self.restore_error: str = ""
        self.cleanup_ban_ips: set[str] = set()

    def _request(
        self,
        method: str,
        path: str,
        *,
        body: bytes | None = None,
        headers: dict[str, str] | None = None,
        expected_statuses: tuple[int, ...] = (200,),
    ) -> tuple[int, str]:
        raise NotImplementedError

    def _target_report_key(self) -> str:
        raise NotImplementedError

    def _target_report_metadata(self) -> dict[str, Any]:
        raise NotImplementedError

    def _authoritative_guardrail_details(self, status: int, body: str) -> str | None:
        return None

    def _trusted_headers(
        self,
        *,
        forwarded_ip: str | None = None,
        extra_headers: dict[str, str] | None = None,
    ) -> dict[str, str]:
        headers: dict[str, str] = {}
        if self.synthetic_forwarding and forwarded_ip:
            headers["X-Forwarded-For"] = forwarded_ip
            headers["X-Forwarded-Proto"] = "https"
        if self.forwarded_ip_secret:
            headers["X-Shuma-Forwarded-Secret"] = self.forwarded_ip_secret
        if extra_headers:
            headers.update(extra_headers)
        return headers

    def _admin_headers(self, extra_headers: dict[str, str] | None = None) -> dict[str, str]:
        return self._trusted_headers(
            forwarded_ip=self.admin_forwarded_ip if self.synthetic_forwarding else None,
            extra_headers={
                "Authorization": f"Bearer {self.api_key}",
                **(extra_headers or {}),
            },
        )

    def _get_config(self) -> dict[str, Any]:
        _, body = self._request("GET", "/admin/config", headers=self._admin_headers())
        try:
            payload = json.loads(body)
        except json.JSONDecodeError as exc:
            raise SmokeFailure(f"/admin/config returned invalid JSON: {exc}") from exc
        if not isinstance(payload, dict):
            raise SmokeFailure("/admin/config returned a non-object payload.")
        config = payload.get("config")
        if not isinstance(config, dict):
            raise SmokeFailure("/admin/config returned a payload without a config object.")
        return config

    def _patch_config(self, patch: dict[str, Any]) -> dict[str, Any]:
        body = json.dumps(patch).encode("utf-8")
        _, payload = self._request(
            "POST",
            "/admin/config",
            body=body,
            headers=self._admin_headers({"Content-Type": "application/json"}),
        )
        try:
            parsed = json.loads(payload)
        except json.JSONDecodeError as exc:
            raise SmokeFailure(f"/admin/config update returned invalid JSON: {exc}") from exc
        if not isinstance(parsed, dict):
            raise SmokeFailure("/admin/config update returned a non-object payload.")
        return parsed

    def _list_bans(self) -> list[dict[str, Any]]:
        _, body = self._request("GET", "/admin/ban", headers=self._admin_headers())
        try:
            payload = json.loads(body)
        except json.JSONDecodeError as exc:
            raise SmokeFailure(f"/admin/ban returned invalid JSON: {exc}") from exc
        bans = payload.get("bans") if isinstance(payload, dict) else None
        if not isinstance(bans, list):
            raise SmokeFailure("/admin/ban returned a non-list ban payload.")
        return [ban for ban in bans if isinstance(ban, dict)]

    def _unban(self, ip: str) -> None:
        self._request("POST", f"/admin/unban?ip={ip}", headers=self._admin_headers())

    def _root_request(
        self,
        *,
        forwarded_ip: str | None,
        geo_country: str = "",
    ) -> tuple[int, str]:
        headers = self._trusted_headers(forwarded_ip=forwarded_ip)
        if geo_country:
            headers["X-Geo-Country"] = geo_country
        return self._request("GET", "/", headers=headers, expected_statuses=(200, 403))

    def _post_fingerprint_fixture(
        self,
        fixture_path: Path,
        *,
        forwarded_ip: str | None,
        expected_statuses: tuple[int, ...] = (200,),
    ) -> tuple[int, str]:
        payload = fixture_path.read_bytes()
        return self._request(
            "POST",
            "/fingerprint-report",
            body=payload,
            headers=self._trusted_headers(
                forwarded_ip=forwarded_ip,
                extra_headers={"Content-Type": "application/json"},
            ),
            expected_statuses=expected_statuses,
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

    def _run_additive_fingerprint_check(self, *, forwarded_ip: str | None) -> None:
        patch = {
            "provider_backends": {"fingerprint_signal": "external"},
            "edge_integration_mode": "additive",
            "cdp_detection_enabled": True,
            "cdp_auto_ban": True,
        }
        self._patch_config(patch)
        _, body = self._post_fingerprint_fixture(
            ADDITIVE_FIXTURE_PATH,
            forwarded_ip=forwarded_ip,
        )
        self._assert_contains(
            body,
            "External fingerprint report received (additive)",
            context="additive fingerprint report",
        )
        status, followup = self._root_request(forwarded_ip=forwarded_ip)
        if status == 403 or "Access Blocked" in followup:
            raise SmokeFailure(
                f"additive fingerprint follow-up unexpectedly blocked: status={status} body={followup!r}"
            )
        self._record_check(
            "akamai_fingerprint_additive",
            True,
            "strong Akamai fixture is accepted without an immediate ban",
        )

    def _run_authoritative_fingerprint_check(self, *, forwarded_ip: str | None) -> None:
        patch = {
            "provider_backends": {"fingerprint_signal": "external"},
            "edge_integration_mode": "authoritative",
            "cdp_detection_enabled": True,
            "cdp_auto_ban": True,
        }
        self._patch_config(patch)
        before_bans = {str(item.get("ip") or "") for item in self._list_bans()}
        status, body = self._post_fingerprint_fixture(
            AUTHORITATIVE_FIXTURE_PATH,
            forwarded_ip=forwarded_ip,
            expected_statuses=(200, 503),
        )
        if status == 200:
            self._assert_contains(
                body,
                "External fingerprint automation detected - banned",
                context="authoritative fingerprint report",
            )
            follow_status, followup = self._root_request(forwarded_ip=forwarded_ip)
            if follow_status != 403 or "Access Blocked" not in followup:
                raise SmokeFailure(
                    f"authoritative fingerprint follow-up did not block: status={follow_status} body={followup!r}"
                )
            after_bans = {str(item.get("ip") or "") for item in self._list_bans()}
            new_bans = {ip for ip in after_bans - before_bans if ip}
            if not new_bans:
                raise SmokeFailure(
                    "authoritative fingerprint follow-up blocked, but no new ban entry was visible via /admin/ban"
                )
            self.cleanup_ban_ips.update(new_bans)
            self._record_check(
                "akamai_fingerprint_authoritative",
                True,
                "strong Akamai fixture triggers immediate authoritative ban",
            )
            return

        guardrail_details = self._authoritative_guardrail_details(status, body)
        if guardrail_details is None:
            raise SmokeFailure(
                f"authoritative fingerprint report returned unexpected status={status} body={body!r}"
            )
        after_bans = {str(item.get("ip") or "") for item in self._list_bans()}
        unexpected_new_bans = {ip for ip in after_bans - before_bans if ip}
        if unexpected_new_bans:
            raise SmokeFailure(
                "authoritative fingerprint guardrail response must not create new bans"
            )
        self._record_check("akamai_fingerprint_authoritative", True, guardrail_details)

    def _run_geo_check(
        self,
        *,
        name: str,
        patch: dict[str, Any],
        forwarded_ip: str | None,
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
            for ip in sorted(self.cleanup_ban_ips):
                self._unban(ip)
        except Exception as exc:  # pragma: no cover - reported in the final JSON
            self.restore_error = str(exc)

    def write_report(self) -> None:
        self.report_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            self._target_report_key(): self._target_report_metadata(),
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
            self._run_additive_fingerprint_check(
                forwarded_ip=FINGERPRINT_ADDITIVE_IP if self.synthetic_forwarding else None
            )
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
                forwarded_ip=GEO_CHALLENGE_IP if self.synthetic_forwarding else None,
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
                forwarded_ip=GEO_MAZE_IP if self.synthetic_forwarding else None,
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
                forwarded_ip=GEO_BLOCK_IP if self.synthetic_forwarding else None,
                country="KP",
                expect_status=403,
                expect_fragments=("Access Blocked", "Access Restricted"),
                details="trusted country header routes to block tier",
            )
            self._run_authoritative_fingerprint_check(
                forwarded_ip=FINGERPRINT_AUTHORITATIVE_IP if self.synthetic_forwarding else None
            )
        except SmokeFailure as exc:
            failure = exc
        finally:
            self._restore_original_state()
            if failure is not None:
                self._record_check(self._target_report_key() + "_edge_signal_smoke", False, str(failure))
            self.write_report()

        if self.restore_error:
            print(f"FAIL restore: {self.restore_error}")
            return 1
        if failure is not None:
            print(f"FAIL edge signal smoke: {failure}")
            return 1

        print(f"Report written: {self.report_path}")
        return 0
