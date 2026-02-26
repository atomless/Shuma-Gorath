#!/usr/bin/env python3
"""Deterministic adversarial simulation runner for Shuma-Gorath.

This runner executes manifest-defined simulation profiles (fast smoke, abuse, Akamai)
with bounded runtime and quantitative gate assertions.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import hmac
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple


ALLOWED_OUTCOMES = {"allow", "monitor", "not-a-bot", "challenge", "maze", "deny_temp"}
ALLOWED_TIERS = {"SIM-T0", "SIM-T1", "SIM-T2", "SIM-T3", "SIM-T4"}
ALLOWED_DRIVERS = {
    "allow_browser_allowlist",
    "not_a_bot_pass",
    "geo_challenge",
    "geo_maze",
    "honeypot_deny_temp",
    "not_a_bot_replay_abuse",
    "not_a_bot_stale_token_abuse",
    "not_a_bot_ordering_cadence_abuse",
    "akamai_additive_report",
    "akamai_authoritative_deny",
}

GOOD_NOT_A_BOT_TELEMETRY = {
    "has_pointer": True,
    "pointer_move_count": 42,
    "pointer_path_length": 560.0,
    "pointer_direction_changes": 18,
    "down_up_ms": 220,
    "focus_changes": 1,
    "visibility_changes": 0,
    "interaction_elapsed_ms": 1800,
    "keyboard_used": False,
    "touch_used": False,
    "events_order_valid": True,
    "activation_method": "pointer",
    "activation_trusted": True,
    "activation_count": 1,
    "control_focused": True,
}

BAD_ORDERING_NOT_A_BOT_TELEMETRY = {
    "has_pointer": True,
    "pointer_move_count": 6,
    "pointer_path_length": 35.0,
    "pointer_direction_changes": 1,
    "down_up_ms": 15,
    "focus_changes": 0,
    "visibility_changes": 0,
    "interaction_elapsed_ms": 120,
    "keyboard_used": False,
    "touch_used": False,
    "events_order_valid": False,
    "activation_method": "pointer",
    "activation_trusted": False,
    "activation_count": 4,
    "control_focused": False,
}


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


class SimulationError(Exception):
    pass


class Runner:
    def __init__(
        self,
        manifest_path: Path,
        manifest: Dict[str, Any],
        profile_name: str,
        base_url: str,
        request_timeout_seconds: float,
        report_path: Path,
    ):
        self.manifest_path = manifest_path
        self.manifest = manifest
        self.profile_name = profile_name
        self.profile = manifest["profiles"][profile_name]
        self.base_url = base_url.rstrip("/")
        self.request_timeout_seconds = request_timeout_seconds
        self.report_path = report_path
        self.opener = urllib.request.build_opener(NoRedirectHandler())
        self.request_count = 0
        self.forwarded_secret = env_or_local("SHUMA_FORWARDED_IP_SECRET")
        self.health_secret = env_or_local("SHUMA_HEALTH_SECRET")
        self.api_key = env_or_local("SHUMA_API_KEY")
        self.challenge_secret = env_or_local("SHUMA_CHALLENGE_SECRET") or env_or_local("SHUMA_JS_SECRET")
        self.session_nonce = f"{int(time.time())}-{os.getpid()}"
        self.honeypot_path = "/instaban"

        if not self.api_key:
            raise SimulationError(
                "Missing SHUMA_API_KEY. Run make setup (or export SHUMA_API_KEY) before adversarial tests."
            )
        if self.api_key in {
            "changeme-dev-only-api-key",
            "changeme-supersecret",
            "changeme-prod-api-key",
        }:
            raise SimulationError(
                "SHUMA_API_KEY is a placeholder. Run make setup or make api-key-generate first."
            )

        self.scenarios = scenario_map(self.manifest)
        self.selected_scenarios = [self.scenarios[sid] for sid in self.profile["scenario_ids"]]

    def run(self) -> int:
        self.wait_ready(timeout_seconds=30)
        self.reset_baseline_config()
        self.honeypot_path = self.resolve_honeypot_path()

        all_ips = sorted({scenario["ip"] for scenario in self.selected_scenarios})
        self.cleanup_ips(all_ips)

        monitoring_before = self.monitoring_snapshot()
        suite_start = time.monotonic()
        results: List[ScenarioResult] = []

        for scenario in self.selected_scenarios:
            elapsed = time.monotonic() - suite_start
            if elapsed > self.profile["max_runtime_seconds"]:
                results.append(
                    ScenarioResult(
                        id=scenario["id"],
                        tier=scenario["tier"],
                        driver=scenario["driver"],
                        expected_outcome=scenario["expected_outcome"],
                        observed_outcome=None,
                        passed=False,
                        latency_ms=0,
                        runtime_budget_ms=scenario["runtime_budget_ms"],
                        detail=(
                            f"Suite runtime budget exceeded before scenario start "
                            f"({elapsed:.2f}s > {self.profile['max_runtime_seconds']}s)"
                        ),
                    )
                )
                break

            result = self.run_scenario(scenario)
            results.append(result)

        monitoring_after = self.monitoring_snapshot()
        suite_runtime_ms = int((time.monotonic() - suite_start) * 1000)
        gate_results = self.evaluate_gates(results, monitoring_before, monitoring_after, suite_runtime_ms)

        report = {
            "schema_version": "sim-report.v1",
            "suite_id": self.manifest["suite_id"],
            "profile": self.profile_name,
            "base_url": self.base_url,
            "request_count": self.request_count,
            "suite_runtime_ms": suite_runtime_ms,
            "monitoring_before": monitoring_before,
            "monitoring_after": monitoring_after,
            "results": [result.__dict__ for result in results],
            "gates": gate_results,
            "passed": all(result.passed for result in results) and gate_results["all_passed"],
            "generated_at_unix": int(time.time()),
        }

        self.report_path.parent.mkdir(parents=True, exist_ok=True)
        self.report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")

        print(f"[adversarial] report: {self.report_path}")
        for result in results:
            status = "PASS" if result.passed else "FAIL"
            print(
                f"[{status}] {result.id} tier={result.tier} driver={result.driver} "
                f"expected={result.expected_outcome} observed={result.observed_outcome or 'n/a'} "
                f"latency_ms={result.latency_ms} detail={result.detail}"
            )

        if gate_results["all_passed"]:
            print("[adversarial] quantitative gates: PASS")
        else:
            print("[adversarial] quantitative gates: FAIL")
            for gate in gate_results["checks"]:
                if not gate["passed"]:
                    print(f"  - {gate['name']}: {gate['detail']}")

        if report["passed"]:
            print("[adversarial] profile PASS")
            return 0

        print("[adversarial] profile FAIL")
        return 1

    def wait_ready(self, timeout_seconds: int) -> None:
        deadline = time.monotonic() + timeout_seconds
        while time.monotonic() < deadline:
            try:
                result = self.request(
                    "GET",
                    "/health",
                    headers=self.forwarded_headers("127.0.0.1", include_health_secret=True),
                    count_request=False,
                )
                if result.status == 200 and "OK" in result.body:
                    return
            except Exception:
                pass
            time.sleep(1)
        raise SimulationError(
            f"Spin server was not ready at {self.base_url}/health within {timeout_seconds}s"
        )

    def resolve_honeypot_path(self) -> str:
        config = self.admin_get_config()
        candidate = str(config.get("honeypot_path") or "").strip()
        if candidate.startswith("/") and len(candidate) > 1:
            return candidate
        return "/instaban"

    def reset_baseline_config(self) -> None:
        self.admin_patch(
            {
                "test_mode": False,
                "honeypot_enabled": True,
                "maze_enabled": True,
                "maze_auto_ban": False,
                "not_a_bot_enabled": True,
                "challenge_puzzle_enabled": True,
                "not_a_bot_nonce_ttl_seconds": 300,
                "not_a_bot_pass_score": 6,
                "not_a_bot_fail_score": 3,
                "not_a_bot_attempt_limit_per_window": 100,
                "not_a_bot_attempt_window_seconds": 300,
                "geo_edge_headers_enabled": False,
                "geo_risk": [],
                "geo_allow": [],
                "geo_challenge": [],
                "geo_maze": [],
                "geo_block": [],
                "allowlist": [],
                "path_allowlist": [],
                "browser_policy_enabled": True,
                "browser_allowlist": [],
                "provider_backends": {
                    "rate_limiter": "internal",
                    "fingerprint_signal": "internal",
                },
                "edge_integration_mode": "off",
                "cdp_detection_enabled": True,
                "cdp_auto_ban": True,
            }
        )

    def cleanup_ips(self, ips: List[str]) -> None:
        for ip in ips:
            self.admin_unban(ip)

    def monitoring_snapshot(self) -> Dict[str, Any]:
        result = self.admin_request("GET", "/admin/monitoring?hours=24&limit=5")
        data = parse_json_or_raise(result.body, "Failed to parse /admin/monitoring response")
        summary = data.get("summary") or {}
        details = data.get("details") or {}
        cdp = details.get("cdp") or {}
        fingerprint_stats = cdp.get("fingerprint_stats") or {}

        honeypot_hits = int_or_zero((summary.get("honeypot") or {}).get("total_hits"))
        challenge_failures = int_or_zero((summary.get("challenge") or {}).get("total_failures"))
        not_a_bot_submitted = int_or_zero((summary.get("not_a_bot") or {}).get("submitted"))
        pow_attempts = int_or_zero((summary.get("pow") or {}).get("total_attempts"))
        rate_violations = int_or_zero((summary.get("rate") or {}).get("total_violations"))
        geo_violations = int_or_zero((summary.get("geo") or {}).get("total_violations"))

        return {
            "fingerprint_events": int_or_zero(fingerprint_stats.get("events")),
            "monitoring_total": (
                honeypot_hits
                + challenge_failures
                + not_a_bot_submitted
                + pow_attempts
                + rate_violations
                + geo_violations
            ),
            "components": {
                "honeypot_hits": honeypot_hits,
                "challenge_failures": challenge_failures,
                "not_a_bot_submitted": not_a_bot_submitted,
                "pow_attempts": pow_attempts,
                "rate_violations": rate_violations,
                "geo_violations": geo_violations,
            },
        }

    def evaluate_gates(
        self,
        results: List[ScenarioResult],
        monitoring_before: Dict[str, Any],
        monitoring_after: Dict[str, Any],
        suite_runtime_ms: int,
    ) -> Dict[str, Any]:
        checks: List[Dict[str, Any]] = []

        latency_values = [result.latency_ms for result in results if result.passed and result.latency_ms > 0]
        p95 = percentile(latency_values, 95)
        p95_limit = int(self.profile["gates"]["latency"]["p95_max_ms"])
        checks.append(
            {
                "name": "latency_p95",
                "passed": p95 <= p95_limit,
                "detail": f"p95={p95}ms limit={p95_limit}ms",
                "observed": p95,
                "limit": p95_limit,
            }
        )

        successful_results = [result for result in results if result.passed and result.observed_outcome]
        outcome_counts: Dict[str, int] = {}
        for result in successful_results:
            outcome_counts[result.observed_outcome] = outcome_counts.get(result.observed_outcome, 0) + 1

        total_successful = len(successful_results)
        ratio_bounds = self.profile["gates"]["outcome_ratio_bounds"]
        for outcome, bounds in ratio_bounds.items():
            ratio = (outcome_counts.get(outcome, 0) / total_successful) if total_successful else 0.0
            minimum = float(bounds["min"])
            maximum = float(bounds["max"])
            checks.append(
                {
                    "name": f"ratio_{outcome}",
                    "passed": minimum <= ratio <= maximum,
                    "detail": (
                        f"ratio={ratio:.3f} bounds=[{minimum:.3f},{maximum:.3f}] "
                        f"count={outcome_counts.get(outcome, 0)} total={total_successful}"
                    ),
                    "observed": ratio,
                    "min": minimum,
                    "max": maximum,
                }
            )

        fp_delta = max(0, monitoring_after["fingerprint_events"] - monitoring_before["fingerprint_events"])
        monitoring_delta = max(0, monitoring_after["monitoring_total"] - monitoring_before["monitoring_total"])

        req_count = max(1, self.request_count)
        fp_amp = fp_delta / req_count
        mon_amp = monitoring_delta / req_count

        fp_limit = float(self.profile["gates"]["telemetry_amplification"]["max_fingerprint_events_per_request"])
        mon_limit = float(self.profile["gates"]["telemetry_amplification"]["max_monitoring_events_per_request"])

        checks.append(
            {
                "name": "telemetry_fingerprint_amplification",
                "passed": fp_amp <= fp_limit,
                "detail": f"amp={fp_amp:.3f} limit={fp_limit:.3f} delta={fp_delta} requests={req_count}",
                "observed": fp_amp,
                "limit": fp_limit,
            }
        )
        checks.append(
            {
                "name": "telemetry_monitoring_amplification",
                "passed": mon_amp <= mon_limit,
                "detail": f"amp={mon_amp:.3f} limit={mon_limit:.3f} delta={monitoring_delta} requests={req_count}",
                "observed": mon_amp,
                "limit": mon_limit,
            }
        )

        runtime_limit_ms = int(self.profile["max_runtime_seconds"]) * 1000
        checks.append(
            {
                "name": "suite_runtime_budget",
                "passed": suite_runtime_ms <= runtime_limit_ms,
                "detail": f"runtime={suite_runtime_ms}ms limit={runtime_limit_ms}ms",
                "observed": suite_runtime_ms,
                "limit": runtime_limit_ms,
            }
        )

        all_passed = all(check["passed"] for check in checks)
        return {
            "all_passed": all_passed,
            "checks": checks,
            "outcome_counts": outcome_counts,
            "request_count": self.request_count,
        }

    def run_scenario(self, scenario: Dict[str, Any]) -> ScenarioResult:
        scenario_id = scenario["id"]
        start = time.monotonic()
        observed_outcome: Optional[str] = None

        try:
            self.reset_baseline_config()
            self.admin_unban(scenario["ip"])

            driver = scenario["driver"]
            if driver == "allow_browser_allowlist":
                observed_outcome = self.driver_allow_browser_allowlist(scenario)
            elif driver == "not_a_bot_pass":
                observed_outcome = self.driver_not_a_bot_pass(scenario)
            elif driver == "geo_challenge":
                observed_outcome = self.driver_geo_challenge(scenario)
            elif driver == "geo_maze":
                observed_outcome = self.driver_geo_maze(scenario)
            elif driver == "honeypot_deny_temp":
                observed_outcome = self.driver_honeypot_deny_temp(scenario)
            elif driver == "not_a_bot_replay_abuse":
                observed_outcome = self.driver_not_a_bot_replay_abuse(scenario)
            elif driver == "not_a_bot_stale_token_abuse":
                observed_outcome = self.driver_not_a_bot_stale_token_abuse(scenario)
            elif driver == "not_a_bot_ordering_cadence_abuse":
                observed_outcome = self.driver_not_a_bot_ordering_cadence_abuse(scenario)
            elif driver == "akamai_additive_report":
                observed_outcome = self.driver_akamai_additive_report(scenario)
            elif driver == "akamai_authoritative_deny":
                observed_outcome = self.driver_akamai_authoritative_deny(scenario)
            else:
                raise SimulationError(f"Unsupported scenario driver: {driver}")

            latency_ms = int((time.monotonic() - start) * 1000)

            if observed_outcome != scenario["expected_outcome"]:
                return ScenarioResult(
                    id=scenario_id,
                    tier=scenario["tier"],
                    driver=scenario["driver"],
                    expected_outcome=scenario["expected_outcome"],
                    observed_outcome=observed_outcome,
                    passed=False,
                    latency_ms=latency_ms,
                    runtime_budget_ms=scenario["runtime_budget_ms"],
                    detail=(
                        f"Outcome mismatch: expected={scenario['expected_outcome']} "
                        f"observed={observed_outcome}"
                    ),
                )

            max_latency_ms = int(scenario["assertions"]["max_latency_ms"])
            if latency_ms > max_latency_ms:
                return ScenarioResult(
                    id=scenario_id,
                    tier=scenario["tier"],
                    driver=scenario["driver"],
                    expected_outcome=scenario["expected_outcome"],
                    observed_outcome=observed_outcome,
                    passed=False,
                    latency_ms=latency_ms,
                    runtime_budget_ms=scenario["runtime_budget_ms"],
                    detail=f"Scenario latency exceeded: {latency_ms}ms > {max_latency_ms}ms",
                )

            if latency_ms > int(scenario["runtime_budget_ms"]):
                return ScenarioResult(
                    id=scenario_id,
                    tier=scenario["tier"],
                    driver=scenario["driver"],
                    expected_outcome=scenario["expected_outcome"],
                    observed_outcome=observed_outcome,
                    passed=False,
                    latency_ms=latency_ms,
                    runtime_budget_ms=scenario["runtime_budget_ms"],
                    detail=(
                        f"Scenario runtime budget exceeded: {latency_ms}ms "
                        f"> {scenario['runtime_budget_ms']}ms"
                    ),
                )

            return ScenarioResult(
                id=scenario_id,
                tier=scenario["tier"],
                driver=scenario["driver"],
                expected_outcome=scenario["expected_outcome"],
                observed_outcome=observed_outcome,
                passed=True,
                latency_ms=latency_ms,
                runtime_budget_ms=scenario["runtime_budget_ms"],
                detail="ok",
            )
        except Exception as exc:
            latency_ms = int((time.monotonic() - start) * 1000)
            return ScenarioResult(
                id=scenario_id,
                tier=scenario["tier"],
                driver=scenario["driver"],
                expected_outcome=scenario["expected_outcome"],
                observed_outcome=observed_outcome,
                passed=False,
                latency_ms=latency_ms,
                runtime_budget_ms=scenario["runtime_budget_ms"],
                detail=f"exception: {exc}",
            )

    def driver_allow_browser_allowlist(self, scenario: Dict[str, Any]) -> str:
        self.admin_patch(
            {
                "test_mode": False,
                "browser_policy_enabled": False,
                "browser_allowlist": [["Chrome", 120]],
            }
        )
        result = self.request(
            "GET",
            "/",
            headers=self.forwarded_headers(scenario["ip"], user_agent=scenario["user_agent"]),
            count_request=True,
        )
        if result.status == 200 and "OK (passed bot defence)" in result.body:
            return "allow"
        raise SimulationError(f"Expected allow response, got status={result.status}")

    def driver_not_a_bot_pass(self, scenario: Dict[str, Any]) -> str:
        self.admin_patch({"test_mode": True, "not_a_bot_enabled": True, "challenge_puzzle_enabled": True})
        seed, _ = self.fetch_not_a_bot_seed(scenario)
        submit = self.submit_not_a_bot(seed, scenario, GOOD_NOT_A_BOT_TELEMETRY)
        headers_lower = lower_headers(submit.headers)
        if submit.status == 303 and "shuma_not_a_bot=" in headers_lower.get("set-cookie", ""):
            return "not-a-bot"
        raise SimulationError(f"Expected 303 + marker cookie, got status={submit.status}")

    def driver_geo_challenge(self, scenario: Dict[str, Any]) -> str:
        self.admin_patch(
            {
                "test_mode": False,
                "geo_edge_headers_enabled": True,
                "geo_risk": [],
                "geo_allow": [],
                "geo_challenge": [scenario["geo_country"]],
                "geo_maze": [],
                "geo_block": [],
            }
        )
        headers = self.forwarded_headers(scenario["ip"], user_agent=scenario["user_agent"])
        headers["X-Geo-Country"] = scenario["geo_country"]
        result = self.request("GET", "/", headers=headers, count_request=True)
        if result.status == 200 and ("Puzzle" in result.body or "I am not a bot" in result.body):
            return "challenge"
        raise SimulationError(f"Expected challenge body, got status={result.status}")

    def driver_geo_maze(self, scenario: Dict[str, Any]) -> str:
        self.admin_patch(
            {
                "test_mode": False,
                "maze_enabled": True,
                "maze_auto_ban": False,
                "geo_edge_headers_enabled": True,
                "geo_risk": [],
                "geo_allow": [],
                "geo_challenge": [],
                "geo_maze": [scenario["geo_country"]],
                "geo_block": [],
            }
        )
        headers = self.forwarded_headers(scenario["ip"], user_agent=scenario["user_agent"])
        headers["X-Geo-Country"] = scenario["geo_country"]
        result = self.request("GET", "/", headers=headers, count_request=True)
        if result.status == 200 and 'data-link-kind="maze"' in result.body:
            return "maze"
        raise SimulationError(f"Expected maze response, got status={result.status}")

    def driver_honeypot_deny_temp(self, scenario: Dict[str, Any]) -> str:
        self.admin_patch({"test_mode": False, "honeypot_enabled": True})

        self.request(
            "GET",
            self.honeypot_path,
            headers=self.forwarded_headers(scenario["ip"], user_agent=scenario["user_agent"]),
            count_request=True,
        )
        result = self.request(
            "GET",
            "/",
            headers=self.forwarded_headers(scenario["ip"], user_agent=scenario["user_agent"]),
            count_request=True,
        )
        if result.status in {403, 429} and (
            "Access Blocked" in result.body or "Access Restricted" in result.body
        ):
            self.admin_unban(scenario["ip"])
            return "deny_temp"
        raise SimulationError(f"Expected deny response, got status={result.status}")

    def driver_not_a_bot_replay_abuse(self, scenario: Dict[str, Any]) -> str:
        self.admin_patch({"test_mode": True, "maze_enabled": True, "not_a_bot_nonce_ttl_seconds": 300})
        seed, _ = self.fetch_not_a_bot_seed(scenario)
        first_submit = self.submit_not_a_bot(seed, scenario, GOOD_NOT_A_BOT_TELEMETRY)
        if first_submit.status != 303:
            detail = collapse_whitespace(first_submit.body)[:160]
            raise SimulationError(
                f"Unable to prime not-a-bot replay scenario: first submit status={first_submit.status} body={detail}"
            )
        replay_submit = self.submit_not_a_bot(seed, scenario, GOOD_NOT_A_BOT_TELEMETRY)
        if replay_submit.status == 200 and 'data-link-kind="maze"' in replay_submit.body:
            return "maze"
        raise SimulationError(f"Expected maze replay response, got status={replay_submit.status}")

    def driver_not_a_bot_stale_token_abuse(self, scenario: Dict[str, Any]) -> str:
        self.admin_patch({"test_mode": True, "maze_enabled": True, "not_a_bot_nonce_ttl_seconds": 300})
        seed, _ = self.fetch_not_a_bot_seed(scenario)
        expired_seed = self.make_expired_seed(seed)
        expired_submit = self.submit_not_a_bot(expired_seed, scenario, GOOD_NOT_A_BOT_TELEMETRY)
        if expired_submit.status == 200 and 'data-link-kind="maze"' in expired_submit.body:
            return "maze"
        raise SimulationError(f"Expected stale-token maze response, got status={expired_submit.status}")

    def driver_not_a_bot_ordering_cadence_abuse(self, scenario: Dict[str, Any]) -> str:
        self.admin_patch({"test_mode": True, "maze_enabled": True})
        seed, _ = self.fetch_not_a_bot_seed(scenario)
        abuse_submit = self.submit_not_a_bot(seed, scenario, BAD_ORDERING_NOT_A_BOT_TELEMETRY)
        if abuse_submit.status == 200 and 'data-link-kind="maze"' in abuse_submit.body:
            return "maze"
        raise SimulationError(f"Expected ordering/cadence maze response, got status={abuse_submit.status}")

    def driver_akamai_additive_report(self, scenario: Dict[str, Any]) -> str:
        self.admin_patch(
            {
                "test_mode": False,
                "provider_backends": {"fingerprint_signal": "external"},
                "edge_integration_mode": "additive",
                "cdp_detection_enabled": True,
                "cdp_auto_ban": True,
            }
        )
        payload = read_fixture_json(Path(scenario["payload_fixture"]))
        report = self.request(
            "POST",
            "/fingerprint-report",
            headers=self.forwarded_headers(scenario["ip"], user_agent=scenario["user_agent"]),
            json_body=payload,
            count_request=True,
        )
        if report.status != 200 or "additive" not in report.body.lower():
            raise SimulationError(
                f"Expected additive fingerprint acknowledgement, got status={report.status} body={report.body[:120]}"
            )

        followup = self.request(
            "GET",
            "/",
            headers=self.forwarded_headers(scenario["ip"], user_agent=scenario["user_agent"]),
            count_request=True,
        )
        if "Access Blocked" in followup.body or "Access Restricted" in followup.body:
            raise SimulationError("Additive mode unexpectedly blocked follow-up request")
        return "monitor"

    def driver_akamai_authoritative_deny(self, scenario: Dict[str, Any]) -> str:
        self.admin_patch(
            {
                "test_mode": False,
                "provider_backends": {"fingerprint_signal": "external"},
                "edge_integration_mode": "authoritative",
                "cdp_detection_enabled": True,
                "cdp_auto_ban": True,
            }
        )
        payload = read_fixture_json(Path(scenario["payload_fixture"]))
        report = self.request(
            "POST",
            "/fingerprint-report",
            headers=self.forwarded_headers(scenario["ip"], user_agent=scenario["user_agent"]),
            json_body=payload,
            count_request=True,
        )
        if report.status != 200 or "banned" not in report.body.lower():
            raise SimulationError(
                f"Expected authoritative ban acknowledgement, got status={report.status} body={report.body[:120]}"
            )

        followup = self.request(
            "GET",
            "/",
            headers=self.forwarded_headers(scenario["ip"], user_agent=scenario["user_agent"]),
            count_request=True,
        )
        if followup.status in {403, 429} and (
            "Access Blocked" in followup.body or "Access Restricted" in followup.body
        ):
            self.admin_unban(scenario["ip"])
            return "deny_temp"
        raise SimulationError(f"Expected blocked follow-up after authoritative signal, got {followup.status}")

    def fetch_not_a_bot_seed(self, scenario: Dict[str, Any]) -> Tuple[str, HttpResult]:
        page = self.request(
            "GET",
            "/challenge/not-a-bot-checkbox",
            headers=self.forwarded_headers(scenario["ip"], user_agent=self.not_a_bot_user_agent(scenario)),
            count_request=True,
        )
        if page.status != 200 or "I am not a bot" not in page.body:
            raise SimulationError(f"Not-a-Bot page unavailable (status={page.status})")
        match = re.search(r'name="seed" value="([^"]+)"', page.body)
        if not match:
            raise SimulationError("Unable to parse not-a-bot seed")
        return match.group(1), page

    def submit_not_a_bot(self, seed: str, scenario: Dict[str, Any], telemetry: Dict[str, Any]) -> HttpResult:
        form_body = {
            "seed": seed,
            "checked": "on",
            "telemetry": json.dumps(telemetry, separators=(",", ":")),
        }
        return self.request(
            "POST",
            "/challenge/not-a-bot-checkbox",
            headers=self.forwarded_headers(scenario["ip"], user_agent=self.not_a_bot_user_agent(scenario)),
            form_body=form_body,
            count_request=True,
        )

    def not_a_bot_user_agent(self, scenario: Dict[str, Any]) -> str:
        base = str(scenario.get("user_agent") or "ShumaAdversarial/1.0").strip()
        if not base:
            base = "ShumaAdversarial/1.0"
        # Isolate cadence buckets per run so repeated local executions do not poison replay tests.
        return f"{base} sim-run/{self.session_nonce}"

    def make_expired_seed(self, seed_token: str) -> str:
        if not self.challenge_secret:
            raise SimulationError(
                "Missing SHUMA_CHALLENGE_SECRET/SHUMA_JS_SECRET required for stale-token simulation"
            )

        payload = parse_seed_payload(seed_token)
        now = int(time.time())
        payload["issued_at"] = max(1, now - 120)
        payload["expires_at"] = max(2, now - 30)
        if payload["issued_at"] >= payload["expires_at"]:
            payload["issued_at"] = max(1, payload["expires_at"] - 1)
        payload_json = json.dumps(payload, separators=(",", ":"))
        signature = hmac.new(
            self.challenge_secret.encode("utf-8"),
            payload_json.encode("utf-8"),
            hashlib.sha256,
        ).digest()
        payload_b64 = base64.b64encode(payload_json.encode("utf-8")).decode("ascii")
        sig_b64 = base64.b64encode(signature).decode("ascii")
        return f"{payload_b64}.{sig_b64}"

    def admin_get_config(self) -> Dict[str, Any]:
        result = self.admin_request("GET", "/admin/config")
        if result.status != 200:
            detail = collapse_whitespace(result.body)[:160]
            raise SimulationError(f"Failed to read /admin/config: status={result.status} body={detail}")
        data = parse_json_or_raise(result.body, "Failed to parse /admin/config response")
        return data.get("config") if isinstance(data.get("config"), dict) else data

    def admin_patch(self, payload: Dict[str, Any]) -> None:
        result = self.admin_request("POST", "/admin/config", json_body=payload)
        if result.status != 200:
            detail = collapse_whitespace(result.body)[:160]
            raise SimulationError(f"Failed to apply /admin/config patch: status={result.status} body={detail}")
        data = parse_json_or_raise(result.body, "Failed to parse /admin/config patch response")
        if data.get("status") != "updated":
            detail = collapse_whitespace(result.body)[:160]
            raise SimulationError(
                f"Failed to apply /admin/config patch: expected status=updated body={detail}"
            )

    def admin_unban(self, ip: str) -> None:
        query = urllib.parse.urlencode({"ip": ip})
        self.admin_request("POST", f"/admin/unban?{query}")

    def admin_request(
        self,
        method: str,
        path: str,
        json_body: Optional[Dict[str, Any]] = None,
    ) -> HttpResult:
        return self.request(
            method,
            path,
            headers=self.admin_headers(),
            json_body=json_body,
            count_request=False,
        )

    def admin_headers(self) -> Dict[str, str]:
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "X-Forwarded-For": "127.0.0.1",
        }
        if self.forwarded_secret:
            headers["X-Shuma-Forwarded-Secret"] = self.forwarded_secret
        return headers

    def forwarded_headers(
        self,
        ip: str,
        user_agent: Optional[str] = None,
        include_health_secret: bool = False,
    ) -> Dict[str, str]:
        headers = {
            "X-Forwarded-For": ip,
        }
        if user_agent:
            headers["User-Agent"] = user_agent
        if self.forwarded_secret:
            headers["X-Shuma-Forwarded-Secret"] = self.forwarded_secret
        if include_health_secret and self.health_secret:
            headers["X-Shuma-Health-Secret"] = self.health_secret
        return headers

    def request(
        self,
        method: str,
        path: str,
        headers: Optional[Dict[str, str]] = None,
        json_body: Optional[Dict[str, Any]] = None,
        form_body: Optional[Dict[str, str]] = None,
        count_request: bool = False,
    ) -> HttpResult:
        url = path if path.startswith("http://") or path.startswith("https://") else f"{self.base_url}{path}"

        data: Optional[bytes] = None
        request_headers = dict(headers or {})
        if json_body is not None:
            data = json.dumps(json_body, separators=(",", ":")).encode("utf-8")
            request_headers["Content-Type"] = "application/json"
        elif form_body is not None:
            data = urllib.parse.urlencode(form_body).encode("utf-8")
            request_headers["Content-Type"] = "application/x-www-form-urlencoded"

        req = urllib.request.Request(url=url, method=method, data=data)
        for key, value in request_headers.items():
            req.add_header(key, value)

        start = time.monotonic()
        try:
            with self.opener.open(req, timeout=self.request_timeout_seconds) as resp:
                body = resp.read().decode("utf-8", errors="replace")
                headers_map = {k.lower(): v for k, v in resp.headers.items()}
                status = int(resp.getcode() or 0)
        except urllib.error.HTTPError as err:
            body = err.read().decode("utf-8", errors="replace")
            headers_map = {k.lower(): v for k, v in (err.headers.items() if err.headers else [])}
            status = int(err.code)
        except Exception as exc:
            raise SimulationError(f"HTTP request failed for {method} {url}: {exc}") from exc

        latency_ms = int((time.monotonic() - start) * 1000)
        if count_request:
            self.request_count += 1

        return HttpResult(status=status, body=body, headers=headers_map, latency_ms=latency_ms)


def parse_json_or_raise(raw: str, error_message: str) -> Dict[str, Any]:
    try:
        parsed = json.loads(raw)
    except Exception as exc:
        detail = collapse_whitespace(raw)[:160] or "<empty>"
        raise SimulationError(f"{error_message}: {detail}") from exc
    if not isinstance(parsed, dict):
        raise SimulationError(f"{error_message}: response was not a JSON object")
    return parsed


def parse_seed_payload(seed_token: str) -> Dict[str, Any]:
    payload_b64 = seed_token.split(".", 1)[0]
    padded = payload_b64 + "=" * (-len(payload_b64) % 4)
    try:
        payload_json = base64.b64decode(padded.encode("ascii")).decode("utf-8")
        parsed = json.loads(payload_json)
    except Exception as exc:
        raise SimulationError("Failed to decode not-a-bot seed payload") from exc
    if not isinstance(parsed, dict):
        raise SimulationError("Failed to decode not-a-bot seed payload")
    return parsed


def collapse_whitespace(raw: str) -> str:
    return re.sub(r"\s+", " ", raw).strip()


def int_or_zero(value: Any) -> int:
    try:
        if value is None:
            return 0
        return int(value)
    except Exception:
        return 0


def percentile(values: List[int], pct: int) -> int:
    if not values:
        return 0
    sorted_values = sorted(values)
    index = int(round((pct / 100.0) * (len(sorted_values) - 1)))
    index = max(0, min(len(sorted_values) - 1, index))
    return sorted_values[index]


def env_or_local(key: str) -> str:
    value = os.environ.get(key)
    if value is not None and value.strip():
        return value.strip()
    return read_env_local_value(key)


def read_env_local_value(key: str) -> str:
    env_local = Path(".env.local")
    if not env_local.exists():
        return ""
    for line in env_local.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.startswith(f"{key}="):
            continue
        value = line.split("=", 1)[1].strip()
        value = value.strip('"').strip("'")
        if value:
            return value
    return ""


def scenario_map(manifest: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    mapping: Dict[str, Dict[str, Any]] = {}
    for scenario in manifest["scenarios"]:
        mapping[scenario["id"]] = scenario
    return mapping


def lower_headers(headers: Dict[str, str]) -> Dict[str, str]:
    return {key.lower(): value for key, value in headers.items()}


def read_fixture_json(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise SimulationError(f"Fixture file not found: {path}")
    raw = path.read_text(encoding="utf-8")
    try:
        parsed = json.loads(raw)
    except Exception as exc:
        raise SimulationError(f"Fixture JSON invalid: {path}") from exc
    if not isinstance(parsed, dict):
        raise SimulationError(f"Fixture JSON must be object: {path}")
    return parsed


def validate_manifest(manifest_path: Path, manifest: Dict[str, Any], profile_name: str) -> None:
    if manifest.get("schema_version") != "sim-manifest.v1":
        raise SimulationError("schema_version must be sim-manifest.v1")

    profiles = manifest.get("profiles")
    if not isinstance(profiles, dict) or not profiles:
        raise SimulationError("profiles must be a non-empty object")
    if profile_name not in profiles:
        raise SimulationError(f"profile not found in manifest: {profile_name}")

    scenarios = manifest.get("scenarios")
    if not isinstance(scenarios, list) or not scenarios:
        raise SimulationError("scenarios must be a non-empty array")

    scenario_ids = set()
    for scenario in scenarios:
        if not isinstance(scenario, dict):
            raise SimulationError("each scenario must be an object")
        required = [
            "id",
            "description",
            "tier",
            "driver",
            "expected_outcome",
            "ip",
            "user_agent",
            "seed",
            "runtime_budget_ms",
            "assertions",
        ]
        for key in required:
            if key not in scenario:
                raise SimulationError(f"scenario missing required key: {key}")
        sid = scenario["id"]
        if sid in scenario_ids:
            raise SimulationError(f"duplicate scenario id: {sid}")
        scenario_ids.add(sid)

        if scenario["tier"] not in ALLOWED_TIERS:
            raise SimulationError(f"scenario {sid} has invalid tier: {scenario['tier']}")
        if scenario["driver"] not in ALLOWED_DRIVERS:
            raise SimulationError(f"scenario {sid} has invalid driver: {scenario['driver']}")
        if scenario["expected_outcome"] not in ALLOWED_OUTCOMES:
            raise SimulationError(
                f"scenario {sid} has invalid expected_outcome: {scenario['expected_outcome']}"
            )

        assertions = scenario.get("assertions")
        if not isinstance(assertions, dict) or "max_latency_ms" not in assertions:
            raise SimulationError(f"scenario {sid} assertions.max_latency_ms is required")

        payload_fixture = scenario.get("payload_fixture")
        if payload_fixture:
            fixture_path = (manifest_path.parents[3] / payload_fixture).resolve()
            if not fixture_path.exists():
                # Also allow direct relative-to-repo path.
                fixture_path = Path(payload_fixture)
            if not fixture_path.exists():
                raise SimulationError(f"scenario {sid} references missing payload_fixture: {payload_fixture}")

    profile = profiles[profile_name]
    profile_required = ["description", "max_runtime_seconds", "scenario_ids", "gates"]
    for key in profile_required:
        if key not in profile:
            raise SimulationError(f"profile {profile_name} missing key: {key}")

    if not isinstance(profile["scenario_ids"], list) or not profile["scenario_ids"]:
        raise SimulationError(f"profile {profile_name} scenario_ids must be non-empty array")
    for sid in profile["scenario_ids"]:
        if sid not in scenario_ids:
            raise SimulationError(f"profile {profile_name} references unknown scenario: {sid}")

    gates = profile.get("gates")
    if not isinstance(gates, dict):
        raise SimulationError(f"profile {profile_name} gates must be an object")
    if "latency" not in gates or "p95_max_ms" not in (gates.get("latency") or {}):
        raise SimulationError(f"profile {profile_name} must include gates.latency.p95_max_ms")

    ratio_bounds = gates.get("outcome_ratio_bounds")
    if not isinstance(ratio_bounds, dict) or not ratio_bounds:
        raise SimulationError(f"profile {profile_name} must include at least one outcome ratio bound")
    for outcome, bounds in ratio_bounds.items():
        if outcome not in ALLOWED_OUTCOMES:
            raise SimulationError(
                f"profile {profile_name} has unsupported outcome ratio key: {outcome}"
            )
        if not isinstance(bounds, dict) or "min" not in bounds or "max" not in bounds:
            raise SimulationError(f"profile {profile_name} outcome {outcome} must define min and max")
        minimum = float(bounds["min"])
        maximum = float(bounds["max"])
        if minimum < 0.0 or maximum > 1.0 or minimum > maximum:
            raise SimulationError(
                f"profile {profile_name} outcome {outcome} has invalid bounds [{minimum},{maximum}]"
            )

    telemetry = gates.get("telemetry_amplification")
    if not isinstance(telemetry, dict):
        raise SimulationError(f"profile {profile_name} must include telemetry_amplification")
    if "max_fingerprint_events_per_request" not in telemetry or "max_monitoring_events_per_request" not in telemetry:
        raise SimulationError(
            f"profile {profile_name} telemetry_amplification must include fingerprint and monitoring limits"
        )


def main() -> int:
    parser = argparse.ArgumentParser(description="Run deterministic adversarial simulation profiles")
    parser.add_argument(
        "--manifest",
        default="scripts/tests/adversarial/scenario_manifest.v1.json",
        help="Path to adversarial scenario manifest JSON",
    )
    parser.add_argument(
        "--profile",
        default="fast_smoke",
        help="Profile name from manifest profiles object",
    )
    parser.add_argument(
        "--base-url",
        default=os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000"),
        help="Base URL for Shuma server",
    )
    parser.add_argument(
        "--request-timeout-seconds",
        type=float,
        default=10.0,
        help="Per-request timeout in seconds",
    )
    parser.add_argument(
        "--report",
        default="scripts/tests/adversarial/latest_report.json",
        help="Path to write simulation report JSON",
    )
    parser.add_argument(
        "--validate-only",
        action="store_true",
        help="Validate manifest/profile/fixtures and exit",
    )

    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    if not manifest_path.exists():
        print(f"Manifest not found: {manifest_path}", file=sys.stderr)
        return 2

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"Failed to parse manifest JSON: {exc}", file=sys.stderr)
        return 2

    try:
        validate_manifest(manifest_path, manifest, args.profile)
    except Exception as exc:
        print(f"Manifest validation failed: {exc}", file=sys.stderr)
        return 2

    if args.validate_only:
        scenario_count = len(manifest["profiles"][args.profile]["scenario_ids"])
        print(
            f"Manifest validation PASS: profile={args.profile} scenarios={scenario_count} file={manifest_path}"
        )
        return 0

    try:
        runner = Runner(
            manifest_path=manifest_path,
            manifest=manifest,
            profile_name=args.profile,
            base_url=args.base_url,
            request_timeout_seconds=args.request_timeout_seconds,
            report_path=Path(args.report),
        )
        return runner.run()
    except SimulationError as exc:
        print(f"Adversarial simulation failed: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
