"""Shared runtime-state and request-plane helpers for the adversarial runner."""

from __future__ import annotations

import urllib.request
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any, Dict, Optional

from scripts.tests.adversarial_runner.contracts import (
    SIM_TAG_HEADER_LANE,
    SIM_TAG_HEADER_PROFILE,
    SIM_TAG_HEADER_RUN_ID,
)

if TYPE_CHECKING:
    from scripts.tests.adversarial_simulation_runner import Runner


class NoRedirectHandler(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


@dataclass
class HttpResult:
    status: int
    body: str
    headers: Dict[str, str]
    latency_ms: int


@dataclass
class ScenarioResult:
    id: str
    tier: str
    driver: str
    expected_outcome: str
    observed_outcome: Optional[str]
    passed: bool
    latency_ms: int
    runtime_budget_ms: int
    detail: str
    realism: Optional[Dict[str, Any]] = None
    execution_evidence: Optional[Dict[str, Any]] = None


class SimulationError(Exception):
    pass


class AttackerPlaneClient:
    def __init__(self, owner: "Runner"):
        self.owner = owner

    def headers(self, ip: str, user_agent: Optional[str] = None) -> Dict[str, str]:
        headers = {"X-Forwarded-For": ip}
        if user_agent:
            headers["User-Agent"] = user_agent
        headers[SIM_TAG_HEADER_RUN_ID] = self.owner.sim_run_id
        headers[SIM_TAG_HEADER_PROFILE] = self.owner.sim_profile
        headers[SIM_TAG_HEADER_LANE] = self.owner.sim_lane
        headers.update(self.owner.signed_sim_tag_headers())
        return headers

    def request(
        self,
        method: str,
        path: str,
        headers: Optional[Dict[str, str]] = None,
        json_body: Optional[Dict[str, Any]] = None,
        form_body: Optional[Dict[str, str]] = None,
        count_request: bool = False,
        trusted_forwarded: bool = False,
    ) -> HttpResult:
        return self.owner.attacker_request(
            method,
            path,
            headers=headers,
            json_body=json_body,
            form_body=form_body,
            count_request=count_request,
            trusted_forwarded=trusted_forwarded,
        )


class ControlPlaneClient:
    def __init__(self, owner: "Runner"):
        self.owner = owner

    def admin_headers(self) -> Dict[str, str]:
        headers = {
            "Authorization": f"Bearer {self.owner.api_key}",
            "X-Forwarded-For": self.owner.next_control_plane_ip(),
        }
        if self.owner.forwarded_secret:
            headers["X-Shuma-Forwarded-Secret"] = self.owner.forwarded_secret
        return headers

    def health_headers(self) -> Dict[str, str]:
        # /health trust-boundary checks only allow exact loopback identities.
        headers = {"X-Forwarded-For": "127.0.0.1"}
        if self.owner.forwarded_secret:
            headers["X-Shuma-Forwarded-Secret"] = self.owner.forwarded_secret
        if self.owner.health_secret:
            headers["X-Shuma-Health-Secret"] = self.owner.health_secret
        return headers

    def request(
        self,
        method: str,
        path: str,
        headers: Optional[Dict[str, str]] = None,
        json_body: Optional[Dict[str, Any]] = None,
    ) -> HttpResult:
        merged_headers = self.admin_headers()
        if headers:
            merged_headers.update(headers)
        return self.owner.request(
            method,
            path,
            headers=merged_headers,
            plane="control",
            json_body=json_body,
            count_request=False,
        )
