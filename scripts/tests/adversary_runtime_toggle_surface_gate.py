#!/usr/bin/env python3
"""Runtime-toggle Scrapling owned-surface coverage and no-impact gate.

This validates the dashboard-toggle execution lane (control endpoint + autonomous supervisor)
produces a real recent Scrapling run with covered owned-surface receipts in operator snapshot
without polluting the live-only monitoring summary paths.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Dict, Optional

RUNTIME_SURFACE_LANE = "scrapling_traffic"
RUNTIME_SURFACE_IP_RESET_TICK_HORIZON = 40
RUNTIME_SURFACE_ATTACK_CORPUS_PATH = (
    Path(__file__).resolve().parent / "adversarial" / "deterministic_attack_corpus.v1.json"
)
REQUIRED_RUNTIME_SURFACE_MODES = {
    "crawler",
    "bulk_scraper",
    "browser_automation",
    "stealth_browser",
    "http_agent",
}
REQUIRED_RUNTIME_SURFACE_IDS = {
    "public_path_traversal",
    "challenge_routing",
    "rate_pressure",
    "not_a_bot_submit",
    "puzzle_submit_or_escalation",
    "pow_verify_abuse",
    "tarpit_progress_abuse",
    "maze_navigation",
    "js_verification_execution",
    "browser_automation_detection",
}
MIN_RUNTIME_SURFACE_DEFENSE_DELTAS = 3
MEANINGFUL_RUNTIME_EVENT_FAMILIES = {
    "challenge",
    "not_a_bot",
    "pow",
    "maze",
    "tarpit",
    "rate_limit",
    "geo",
    "ban_path",
}
NON_TARPIT_MEANINGFUL_RUNTIME_EVENT_FAMILIES = MEANINGFUL_RUNTIME_EVENT_FAMILIES - {"tarpit"}


def runtime_surface_coverage_meets_gate(coverage: Dict[str, Any]) -> bool:
    required_surface_ids = {
        str(value).strip()
        for value in list(coverage.get("required_surface_ids") or [])
        if str(value).strip()
    }
    satisfied_surface_ids = {
        str(value).strip()
        for value in list(coverage.get("satisfied_surface_ids") or [])
        if str(value).strip()
    }
    blocking_surface_ids = {
        str(value).strip()
        for value in list(coverage.get("blocking_surface_ids") or [])
        if str(value).strip()
    }
    observed_fulfillment_modes = {
        str(value).strip()
        for value in list(coverage.get("observed_fulfillment_modes") or [])
        if str(value).strip()
    }
    if not str(coverage.get("run_id") or "").strip():
        return False
    if not REQUIRED_RUNTIME_SURFACE_IDS.issubset(required_surface_ids):
        return False
    if not REQUIRED_RUNTIME_SURFACE_IDS.issubset(satisfied_surface_ids):
        return False
    if REQUIRED_RUNTIME_SURFACE_IDS.intersection(blocking_surface_ids):
        return False
    if not REQUIRED_RUNTIME_SURFACE_MODES.issubset(observed_fulfillment_modes):
        return False
    return int(coverage.get("defense_delta_count") or 0) >= MIN_RUNTIME_SURFACE_DEFENSE_DELTAS


def parse_bool(value: str, default: bool) -> bool:
    lowered = str(value or "").strip().lower()
    if lowered in {"1", "true", "yes", "on"}:
        return True
    if lowered in {"0", "false", "no", "off"}:
        return False
    return default


def live_summary_leaks(current: Dict[str, int], baseline: Dict[str, int]) -> Dict[str, int]:
    leaked: Dict[str, int] = {}
    for name, value in current.items():
        delta = int(value) - int(baseline.get(name, 0))
        if delta > 0:
            leaked[name] = delta
    return leaked


def normalize_event_token(value: Any) -> str:
    return str(value or "").strip().lower().replace("-", "_")


def classify_runtime_surface_event_family(row: Dict[str, Any]) -> str:
    combined = " ".join(
        [
            normalize_event_token(row.get("event")),
            normalize_event_token(row.get("reason")),
            normalize_event_token(row.get("outcome_code")),
            normalize_event_token(row.get("outcome")),
        ]
    )
    if "honeypot" in combined:
        return "honeypot"
    if "tarpit" in combined:
        return "tarpit"
    if "maze" in combined:
        return "maze"
    if "not_a_bot" in combined or "not-a-bot" in combined:
        return "not_a_bot"
    if "pow" in combined or "proof" in combined:
        return "pow"
    if "rate" in combined:
        return "rate_limit"
    if "geo" in combined:
        return "geo"
    if "cdp" in combined:
        return "cdp"
    if "fingerprint" in combined:
        return "fingerprint"
    if "challenge" in combined:
        return "challenge"
    if "ban" in combined or "deny_temp" in combined or "block" in combined:
        return "ban_path"
    if str(row.get("sim_run_id") or "").strip():
        return "event_stream"
    return "other"


def runtime_surface_event_mentions_tarpit_progress(row: Dict[str, Any]) -> bool:
    combined = " ".join(
        [
            normalize_event_token(row.get("event")),
            normalize_event_token(row.get("reason")),
            normalize_event_token(row.get("outcome_code")),
            normalize_event_token(row.get("outcome")),
        ]
    )
    return "tarpit_progress" in combined


def runtime_surface_event_has_hostile_outcome(row: Dict[str, Any]) -> bool:
    if normalize_event_token(row.get("event")) == "ban":
        return True
    combined = " ".join(
        [
            normalize_event_token(row.get("outcome_code")),
            normalize_event_token(row.get("outcome")),
            normalize_event_token(row.get("reason")),
        ]
    )
    return any(
        token in combined
        for token in ("fail", "reject", "deny", "block", "banned", "limited")
    )


def runtime_surface_event_evidence_meets_gate(evidence: Dict[str, Any]) -> bool:
    defense_families = {
        str(value).strip()
        for value in list(evidence.get("defense_families") or [])
        if str(value).strip()
    }
    return (
        int(evidence.get("matching_event_count") or 0) > 0
        and int(evidence.get("challenge_event_count") or 0) > 0
        and int(evidence.get("hostile_outcome_event_count") or 0) > 0
        and int(evidence.get("tarpit_progress_event_count") or 0) > 0
        and bool(
            NON_TARPIT_MEANINGFUL_RUNTIME_EVENT_FAMILIES.intersection(defense_families)
        )
    )


def load_runtime_surface_corpus_profile() -> Dict[str, Any]:
    raw = json.loads(RUNTIME_SURFACE_ATTACK_CORPUS_PATH.read_text(encoding="utf-8"))
    runtime_toggle = raw.get("runtime_toggle")
    return runtime_toggle if isinstance(runtime_toggle, dict) else {}


def runtime_surface_primary_request_ip(
    generation_batch_size_max: int,
    tick_count: int,
    index: int,
) -> str:
    offset = tick_count * generation_batch_size_max + index
    third = ((offset // 254) % 254) + 1
    fourth = (offset % 254) + 1
    return f"198.51.{third}.{fourth}"


def runtime_surface_lane_actor_ip(
    third_octet: int,
    tick_count: int,
    rotate_every_ticks: int,
    lane_salt: int,
) -> str:
    rotate_every_ticks = max(1, int(rotate_every_ticks))
    bucket = ((tick_count // rotate_every_ticks) + lane_salt) % 254 + 1
    return f"198.51.{int(third_octet)}.{bucket}"


def runtime_surface_candidate_ips(
    corpus_profile: Dict[str, Any],
    tick_horizon: int = RUNTIME_SURFACE_IP_RESET_TICK_HORIZON,
) -> list[str]:
    primary_request_count = int(corpus_profile.get("primary_request_count") or 0)
    supplemental_request_count = int(corpus_profile.get("supplemental_request_count") or 0)
    rate_burst = corpus_profile.get("rate_burst") or {}
    lane_ip_octets = corpus_profile.get("lane_ip_octets") or {}
    lane_ip_rotation_ticks = corpus_profile.get("lane_ip_rotation_ticks") or {}
    lane_ip_entropy_salts = corpus_profile.get("lane_ip_entropy_salts") or {}
    generation_batch_size_max = (
        primary_request_count
        + supplemental_request_count
        + int(rate_burst.get("high") or 0)
    )
    ips: set[str] = set()
    for tick_count in range(max(0, int(tick_horizon))):
        for index in range(primary_request_count):
            ips.add(runtime_surface_primary_request_ip(generation_batch_size_max, tick_count, index))
        for lane_name, third_octet in lane_ip_octets.items():
            ips.add(
                runtime_surface_lane_actor_ip(
                    int(third_octet or 0),
                    tick_count,
                    int(lane_ip_rotation_ticks.get(lane_name) or 1),
                    int(lane_ip_entropy_salts.get(lane_name) or 0),
                )
            )
    return sorted(ips)


class RuntimeToggleSurfaceGate:
    def __init__(
        self,
        base_url: str,
        api_key: str,
        forwarded_secret: str,
        health_secret: str,
        timeout_seconds: int,
    ):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.forwarded_secret = forwarded_secret.strip()
        self.health_secret = health_secret.strip()
        self.timeout_seconds = timeout_seconds
        self.opener = urllib.request.build_opener()

    @staticmethod
    def log(message: str) -> None:
        print(f"[runtime-surface-gate] {message}", file=sys.stderr, flush=True)

    def _headers(self, include_json: bool = False) -> Dict[str, str]:
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "X-Forwarded-For": "127.0.0.1",
        }
        if self.forwarded_secret:
            headers["X-Shuma-Forwarded-Secret"] = self.forwarded_secret
        if include_json:
            headers["Content-Type"] = "application/json"
        return headers

    def _health_headers(self) -> Dict[str, str]:
        headers = self._headers()
        if self.health_secret:
            headers["X-Shuma-Health-Secret"] = self.health_secret
        return headers

    def request(
        self,
        method: str,
        path: str,
        payload: Optional[Dict[str, Any]] = None,
        extra_headers: Optional[Dict[str, str]] = None,
    ) -> Dict[str, Any]:
        url = f"{self.base_url}{path}"
        body = None
        headers = self._headers(include_json=payload is not None)
        if extra_headers:
            headers.update(extra_headers)
        if payload is not None:
            body = json.dumps(payload).encode("utf-8")

        req = urllib.request.Request(url, data=body, method=method.upper(), headers=headers)
        try:
            with self.opener.open(req, timeout=5) as response:
                text = response.read().decode("utf-8", errors="replace")
                try:
                    parsed = json.loads(text) if text else {}
                except json.JSONDecodeError:
                    parsed = {"raw": text}
                return {
                    "status": int(getattr(response, "status", 0) or 0),
                    "body": parsed,
                    "raw": text,
                }
        except urllib.error.HTTPError as err:
            text = err.read().decode("utf-8", errors="replace")
            try:
                parsed = json.loads(text) if text else {}
            except json.JSONDecodeError:
                parsed = {"raw": text}
            return {"status": int(err.code), "body": parsed, "raw": text}

    @staticmethod
    def _as_obj(value: Any) -> Dict[str, Any]:
        return value if isinstance(value, dict) else {}

    @staticmethod
    def _as_list(value: Any) -> list[Any]:
        return value if isinstance(value, list) else []

    @staticmethod
    def _as_int(value: Any) -> int:
        try:
            return int(value)
        except (TypeError, ValueError):
            return 0

    def live_summary_counts(self, monitoring_body: Dict[str, Any]) -> Dict[str, int]:
        summary = self._as_obj(monitoring_body.get("summary"))
        return {
            "challenge_failures": self._as_int(
                self._as_obj(summary.get("challenge")).get("total_failures")
            ),
            "pow_attempts": self._as_int(self._as_obj(summary.get("pow")).get("total_attempts")),
            "rate_violations": self._as_int(
                self._as_obj(summary.get("rate")).get("total_violations")
            ),
            "geo_violations": self._as_int(self._as_obj(summary.get("geo")).get("total_violations")),
        }

    def read_live_summary_counts(self) -> Dict[str, int]:
        deadline = time.time() + float(self.timeout_seconds)
        counts = {
            "challenge_failures": 0,
            "pow_attempts": 0,
            "rate_violations": 0,
            "geo_violations": 0,
        }

        while time.time() < deadline:
            monitoring = self.request("GET", "/shuma/admin/monitoring?hours=24&limit=200")
            if monitoring["status"] != 200:
                time.sleep(1)
                continue
            return self.live_summary_counts(self._as_obj(monitoring["body"]))

        return counts

    def ensure_health(self) -> None:
        response = self.request("GET", "/shuma/health", extra_headers=self._health_headers())
        if response["status"] != 200:
            raise RuntimeError(f"health check failed: status={response['status']} body={response['raw'][:200]}")

    def clear_loopback_bans(self) -> None:
        for ip in ("127.0.0.1", "::1", "unknown"):
            response = self.request("POST", f"/shuma/admin/unban?ip={ip}")
            if response["status"] != 200:
                raise RuntimeError(
                    f"failed to clear loopback ban for {ip}: status={response['status']} body={response['raw'][:200]}"
                )

    def active_ban_ips(self) -> set[str]:
        response = self.request("GET", "/shuma/admin/ban?active=true")
        if response["status"] != 200:
            raise RuntimeError(
                f"failed to read active bans: status={response['status']} body={response['raw'][:200]}"
            )
        body = response["body"]
        entries = []
        if isinstance(body, dict):
            entries = self._as_list(body.get("bans"))
        elif isinstance(body, list):
            entries = body
        ips: set[str] = set()
        for entry in entries:
            if isinstance(entry, dict):
                ip = str(entry.get("ip") or "").strip()
            else:
                ip = str(entry or "").strip()
            if ip:
                ips.add(ip)
        return ips

    def clear_runtime_surface_bans(self) -> None:
        candidate_ips = set(runtime_surface_candidate_ips(load_runtime_surface_corpus_profile()))
        active_runtime_bans = sorted(self.active_ban_ips().intersection(candidate_ips))
        self.log(
            f"clearing runtime-surface bans active={len(active_runtime_bans)} candidate_pool={len(candidate_ips)}"
        )
        for index, ip in enumerate(active_runtime_bans, start=1):
            if index == 1 or index == len(active_runtime_bans) or index % 25 == 0:
                self.log(f"clearing runtime-surface ban {index}/{len(active_runtime_bans)} ip={ip}")
            response = self.request("POST", f"/shuma/admin/unban?ip={ip}")
            if response["status"] != 200:
                raise RuntimeError(
                    f"failed to clear runtime-surface ban for {ip}: status={response['status']} body={response['raw'][:200]}"
                )

    def configure_runtime_surface_profile(self) -> None:
        payload = {
            "defence_modes": {"rate": "both", "geo": "both", "js": "both"},
            # Keep the proof lane confrontational so request-native Scrapling must
            # provoke live rate pressure inside a normal watch window.
            "rate_limit": 12,
            "js_required_enforced": True,
            "pow_enabled": True,
            "cdp_auto_ban": False,
            "challenge_puzzle_enabled": True,
            "challenge_puzzle_risk_threshold": 4,
            "not_a_bot_enabled": True,
            "not_a_bot_risk_threshold": 2,
            "maze_auto_ban": False,
            "geo_edge_headers_enabled": True,
            "geo_challenge": [],
            # Preserve crawler root traversal while still proving geo-served
            # defence confrontation from its deterministic local country.
            "geo_maze": ["RU"],
            "geo_block": [],
            "ban_durations": {
                "rate_limit": 1,
                "tarpit_persistence": 1,
                "not_a_bot_abuse": 1,
                "challenge_puzzle_abuse": 1,
            },
        }
        response = self.request("POST", "/shuma/admin/config", payload)
        if response["status"] != 200:
            raise RuntimeError(
                f"failed to apply runtime surface config profile: status={response['status']} body={response['raw'][:200]}"
            )

    def toggle(self, enabled: bool, suffix: str) -> None:
        max_attempts = 1 if enabled else 10
        for attempt in range(1, max_attempts + 1):
            operation_id = f"runtime-surface-{int(time.time())}-{suffix}-a{attempt}"
            response = self.request(
                "POST",
                "/shuma/admin/adversary-sim/control",
                {
                    "enabled": bool(enabled),
                    "lane": RUNTIME_SURFACE_LANE,
                    "reason": "runtime_surface_gate",
                },
                extra_headers={
                    "Idempotency-Key": operation_id,
                    "Origin": self.base_url,
                    "Sec-Fetch-Site": "same-origin",
                },
            )
            if response["status"] == 200:
                return
            if not enabled and response["status"] == 429 and attempt < max_attempts:
                time.sleep(1)
                continue
            raise RuntimeError(
                f"toggle {enabled} failed: status={response['status']} body={response['raw'][:200]}"
            )

    def recent_scrapling_run_coverage(
        self,
        operator_snapshot_body: Dict[str, Any],
        existing_run_ids: Optional[set[str]] = None,
        minimum_started_at: int = 0,
    ) -> Dict[str, Any]:
        objectives = self._as_obj(operator_snapshot_body.get("objectives"))
        verified_identity = self._as_obj(operator_snapshot_body.get("verified_identity"))
        effective_policy = self._as_obj(verified_identity.get("effective_non_human_policy"))
        budget_rows = self._as_list(self._as_obj(operator_snapshot_body.get("budget_distance")).get("rows"))
        adversary_sim = self._as_obj(operator_snapshot_body.get("adversary_sim"))
        recent_runs = self._as_list(adversary_sim.get("recent_runs"))
        existing_run_ids = existing_run_ids or set()
        for row in recent_runs:
            run = self._as_obj(row)
            if str(run.get("lane") or "").strip() != "scrapling_traffic":
                continue
            run_id = str(run.get("run_id") or "").strip()
            if run_id in existing_run_ids:
                continue
            run_started_at = self._as_int(run.get("first_ts"))
            if minimum_started_at > 0 and run_started_at < minimum_started_at:
                continue
            coverage = self._as_obj(run.get("owned_surface_coverage"))
            if not coverage:
                continue
            return {
                "run_id": run_id,
                "overall_status": str(coverage.get("overall_status") or "").strip(),
                "profile_id": str(objectives.get("profile_id") or "").strip(),
                "verified_identity_override_mode": str(
                    effective_policy.get("verified_identity_override_mode") or ""
                ).strip(),
                "suspicious_forwarded_request_target": self._budget_target(
                    budget_rows,
                    "suspicious_forwarded_request_rate",
                ),
                "suspicious_forwarded_byte_target": self._budget_target(
                    budget_rows,
                    "suspicious_forwarded_byte_rate",
                ),
                "suspicious_forwarded_latency_target": self._budget_target(
                    budget_rows,
                    "suspicious_forwarded_latency_share",
                ),
                "required_surface_ids": [
                    str(value).strip()
                    for value in self._as_list(coverage.get("required_surface_ids"))
                    if str(value).strip()
                ],
                "satisfied_surface_ids": [
                    str(value).strip()
                    for value in self._as_list(coverage.get("satisfied_surface_ids"))
                    if str(value).strip()
                ],
                "blocking_surface_ids": [
                    str(value).strip()
                    for value in self._as_list(coverage.get("blocking_surface_ids"))
                    if str(value).strip()
                ],
                "observed_fulfillment_modes": [
                    str(value).strip()
                    for value in self._as_list(run.get("observed_fulfillment_modes"))
                    if str(value).strip()
                ],
                "defense_delta_count": self._as_int(run.get("defense_delta_count")),
                "ban_outcome_count": self._as_int(run.get("ban_outcome_count")),
            }
        return {
            "run_id": "",
            "overall_status": "",
            "profile_id": "",
            "verified_identity_override_mode": "",
            "suspicious_forwarded_request_target": None,
            "suspicious_forwarded_byte_target": None,
            "suspicious_forwarded_latency_target": None,
            "required_surface_ids": [],
            "satisfied_surface_ids": [],
            "blocking_surface_ids": [],
            "observed_fulfillment_modes": [],
            "defense_delta_count": 0,
            "ban_outcome_count": 0,
        }

    def current_recent_scrapling_run_ids(self) -> set[str]:
        operator_snapshot = self.request("GET", "/shuma/admin/operator-snapshot")
        if operator_snapshot["status"] != 200:
            return set()
        recent_runs = self._as_list(
            self._as_obj(self._as_obj(operator_snapshot["body"]).get("adversary_sim")).get("recent_runs")
        )
        run_ids: set[str] = set()
        for row in recent_runs:
            run = self._as_obj(row)
            if str(run.get("lane") or "").strip() != "scrapling_traffic":
                continue
            run_id = str(run.get("run_id") or "").strip()
            if run_id:
                run_ids.add(run_id)
        return run_ids

    def poll_recent_scrapling_run_coverage(
        self,
        existing_run_ids: Optional[set[str]] = None,
        minimum_started_at: int = 0,
    ) -> Dict[str, Any]:
        deadline = time.time() + float(self.timeout_seconds)
        last_seen = {
            "run_id": "",
            "overall_status": "",
            "profile_id": "",
            "verified_identity_override_mode": "",
            "suspicious_forwarded_request_target": None,
            "suspicious_forwarded_byte_target": None,
            "suspicious_forwarded_latency_target": None,
            "required_surface_ids": [],
            "satisfied_surface_ids": [],
            "blocking_surface_ids": [],
            "observed_fulfillment_modes": [],
            "defense_delta_count": 0,
            "ban_outcome_count": 0,
        }

        while time.time() < deadline:
            operator_snapshot = self.request("GET", "/shuma/admin/operator-snapshot")
            if operator_snapshot["status"] != 200:
                time.sleep(1)
                continue
            last_seen = self.recent_scrapling_run_coverage(
                self._as_obj(operator_snapshot["body"]),
                existing_run_ids=existing_run_ids,
                minimum_started_at=minimum_started_at,
            )
            if runtime_surface_coverage_meets_gate(last_seen):
                return last_seen
            time.sleep(1)

        return last_seen

    def recent_sim_run_event_evidence(
        self,
        events_body: Dict[str, Any],
        sim_run_id: str,
    ) -> Dict[str, Any]:
        matching_events: list[Dict[str, Any]] = []
        for row in self._as_list(events_body.get("recent_events")):
            event = self._as_obj(row)
            if event.get("is_simulation") is not True:
                continue
            if str(event.get("sim_run_id") or "").strip() != sim_run_id:
                continue
            matching_events.append(event)
        defense_families = sorted(
            {
                classify_runtime_surface_event_family(event)
                for event in matching_events
                if classify_runtime_surface_event_family(event) != "other"
            }
        )
        challenge_event_count = sum(
            1
            for event in matching_events
            if normalize_event_token(event.get("event")) == "challenge"
        )
        hostile_outcome_event_count = sum(
            1 for event in matching_events if runtime_surface_event_has_hostile_outcome(event)
        )
        tarpit_progress_event_count = sum(
            1 for event in matching_events if runtime_surface_event_mentions_tarpit_progress(event)
        )
        return {
            "sim_run_id": sim_run_id,
            "matching_event_count": len(matching_events),
            "challenge_event_count": challenge_event_count,
            "hostile_outcome_event_count": hostile_outcome_event_count,
            "tarpit_progress_event_count": tarpit_progress_event_count,
            "defense_families": defense_families,
        }

    def poll_recent_sim_run_event_evidence(self, sim_run_id: str) -> Dict[str, Any]:
        deadline = time.time() + float(self.timeout_seconds)
        last_seen = {
            "sim_run_id": sim_run_id,
            "matching_event_count": 0,
            "challenge_event_count": 0,
            "hostile_outcome_event_count": 0,
            "tarpit_progress_event_count": 0,
            "defense_families": [],
        }

        while time.time() < deadline:
            events = self.request("GET", "/shuma/admin/events?hours=2&limit=200")
            if events["status"] != 200:
                time.sleep(1)
                continue
            last_seen = self.recent_sim_run_event_evidence(
                self._as_obj(events["body"]),
                sim_run_id,
            )
            if runtime_surface_event_evidence_meets_gate(last_seen):
                return last_seen
            time.sleep(1)

        return last_seen

    def poll_post_sim_oversight_run(self, sim_run_id: str) -> Dict[str, Any]:
        deadline = time.time() + float(self.timeout_seconds)
        last_seen = {
            "run_id": "",
            "trigger_kind": "",
            "sim_run_id": "",
            "apply_stage": "",
        }

        while time.time() < deadline:
            status = self.request("GET", "/shuma/admin/oversight/agent/status")
            if status["status"] != 200:
                time.sleep(1)
                continue
            status_body = self._as_obj(status["body"])
            latest_run = self._as_obj(status_body.get("latest_run"))
            if (
                str(latest_run.get("trigger_kind") or "").strip() == "post_adversary_sim"
                and str(latest_run.get("sim_run_id") or "").strip() == sim_run_id
            ):
                execution = self._as_obj(latest_run.get("execution"))
                apply = self._as_obj(execution.get("apply"))
                return {
                    "run_id": str(latest_run.get("run_id") or "").strip(),
                    "trigger_kind": str(latest_run.get("trigger_kind") or "").strip(),
                    "sim_run_id": str(latest_run.get("sim_run_id") or "").strip(),
                    "apply_stage": str(apply.get("stage") or "").strip(),
                }
            recent_runs = self._as_list(status_body.get("recent_runs"))
            for row in recent_runs:
                run = self._as_obj(row)
                if str(run.get("trigger_kind") or "").strip() != "post_adversary_sim":
                    continue
                if str(run.get("sim_run_id") or "").strip() != sim_run_id:
                    continue
                execution = self._as_obj(run.get("execution"))
                apply = self._as_obj(execution.get("apply"))
                return {
                    "run_id": str(run.get("run_id") or "").strip(),
                    "trigger_kind": str(run.get("trigger_kind") or "").strip(),
                    "sim_run_id": str(run.get("sim_run_id") or "").strip(),
                    "apply_stage": str(apply.get("stage") or "").strip(),
                }
            time.sleep(1)

        return last_seen

    def poll_live_summary_matches_baseline(self, baseline: Dict[str, int]) -> Dict[str, int]:
        deadline = time.time() + float(self.timeout_seconds)
        counts = dict(baseline)

        while time.time() < deadline:
            monitoring = self.request("GET", "/shuma/admin/monitoring?hours=24&limit=200")
            if monitoring["status"] != 200:
                time.sleep(1)
                continue
            counts = self.live_summary_counts(self._as_obj(monitoring["body"]))
            if all(
                counts.get(name, 0) == baseline.get(name, 0) for name in baseline.keys()
            ):
                return counts
            time.sleep(1)

        return counts

    @staticmethod
    def _budget_target(rows: list[Any], metric: str) -> Optional[float]:
        for row in rows:
            budget_row = row if isinstance(row, dict) else {}
            if str(budget_row.get("metric") or "").strip() != metric:
                continue
            target = budget_row.get("target")
            try:
                return float(target)
            except (TypeError, ValueError):
                return None
        return None


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Runtime-toggle Scrapling owned-surface coverage and no-impact gate"
    )
    parser.add_argument("--base-url", default=os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000"))
    parser.add_argument("--timeout-seconds", type=int, default=120)
    args = parser.parse_args()

    api_key = os.environ.get("SHUMA_API_KEY", "").strip()
    if not api_key:
        print("[runtime-surface-gate] SHUMA_API_KEY is required", file=sys.stderr)
        return 2

    forwarded_secret = os.environ.get("SHUMA_FORWARDED_IP_SECRET", "")
    health_secret = os.environ.get("SHUMA_HEALTH_SECRET", "")
    gate = RuntimeToggleSurfaceGate(
        base_url=args.base_url,
        api_key=api_key,
        forwarded_secret=forwarded_secret,
        health_secret=health_secret,
        timeout_seconds=max(10, int(args.timeout_seconds)),
    )

    try:
        gate.log("checking health")
        gate.ensure_health()
        gate.log("clearing loopback bans")
        gate.clear_loopback_bans()
        gate.log("clearing runtime-surface candidate bans")
        gate.clear_runtime_surface_bans()
        gate.log("configuring runtime-surface defence profile")
        gate.configure_runtime_surface_profile()
        gate.log("reading live-summary baseline")
        live_summary_baseline = gate.read_live_summary_counts()
        existing_run_ids = gate.current_recent_scrapling_run_ids()
        minimum_started_at = max(0, int(time.time()) - 1)
        gate.log("toggling runtime surface run on")
        gate.toggle(True, "on")
        gate.log("waiting for recent Scrapling run coverage")
        coverage = gate.poll_recent_scrapling_run_coverage(
            existing_run_ids=existing_run_ids,
            minimum_started_at=minimum_started_at,
        )
        gate.log("waiting for live-summary baseline restoration")
        live_summary_counts = gate.poll_live_summary_matches_baseline(live_summary_baseline)
    except Exception as exc:  # noqa: BLE001
        print(f"[runtime-surface-gate] error: {exc}", file=sys.stderr)
        try:
            gate.toggle(False, "off-error")
        except Exception:
            pass
        try:
            gate.clear_loopback_bans()
        except Exception:
            pass
        return 1

    try:
        gate.log("toggling runtime surface run off")
        gate.toggle(False, "off")
    except Exception as exc:  # noqa: BLE001
        print(f"[runtime-surface-gate] warning: failed to toggle off: {exc}", file=sys.stderr)
    try:
        gate.log("clearing loopback bans after run")
        gate.clear_loopback_bans()
    except Exception as exc:  # noqa: BLE001
        print(f"[runtime-surface-gate] warning: failed to clear loopback bans: {exc}", file=sys.stderr)

    if not runtime_surface_coverage_meets_gate(coverage):
        print(
            "[runtime-surface-gate] runtime surface coverage did not satisfy the hostile subset gate",
            file=sys.stderr,
        )
        print(
            f"[runtime-surface-gate] coverage={json.dumps(coverage, sort_keys=True)}",
            file=sys.stderr,
        )
        return 1

    gate.log("waiting for event evidence")
    event_evidence = gate.poll_recent_sim_run_event_evidence(str(coverage.get("run_id") or ""))
    if not runtime_surface_event_evidence_meets_gate(event_evidence):
        print(
            "[runtime-surface-gate] server-observed runtime event evidence did not prove root-served defense confrontation: "
            + json.dumps({"coverage": coverage, "event_evidence": event_evidence}, sort_keys=True),
            file=sys.stderr,
        )
        return 1

    gate.log("waiting for oversight run")
    oversight_run = gate.poll_post_sim_oversight_run(str(coverage.get("run_id") or ""))

    if coverage.get("profile_id") != "human_only_private":
        print(
            "[runtime-surface-gate] strict operator-objectives profile was not active: "
            + json.dumps(coverage, sort_keys=True),
            file=sys.stderr,
        )
        return 1

    if coverage.get("verified_identity_override_mode") != "strict_human_only":
        print(
            "[runtime-surface-gate] verified identity was not locked to strict human-only mode: "
            + json.dumps(coverage, sort_keys=True),
            file=sys.stderr,
        )
        return 1

    for field in (
        "suspicious_forwarded_request_target",
        "suspicious_forwarded_byte_target",
        "suspicious_forwarded_latency_target",
    ):
        if coverage.get(field) != 0.0:
            print(
                "[runtime-surface-gate] strict suspicious leakage target was not zero: "
                + json.dumps(coverage, sort_keys=True),
                file=sys.stderr,
            )
            return 1

    if oversight_run.get("sim_run_id") != coverage.get("run_id"):
        print(
            "[runtime-surface-gate] post-sim oversight trigger did not materialize for the completed Scrapling run: "
            + json.dumps({"coverage": coverage, "oversight_run": oversight_run}, sort_keys=True),
            file=sys.stderr,
        )
        return 1

    leaked = live_summary_leaks(live_summary_counts, live_summary_baseline)
    if leaked:
        print(
            "[runtime-surface-gate] live-only monitoring summary was polluted by sim traffic: "
            + json.dumps(leaked, sort_keys=True),
            file=sys.stderr,
        )
        print(
            f"[runtime-surface-gate] coverage={json.dumps(coverage, sort_keys=True)}",
            file=sys.stderr,
        )
        return 1

    print(
        "[runtime-surface-gate] PASS coverage="
        + json.dumps(coverage, sort_keys=True)
        + " event_evidence="
        + json.dumps(event_evidence, sort_keys=True)
        + " oversight="
        + json.dumps(oversight_run, sort_keys=True)
        + " live_summary="
        + json.dumps(live_summary_counts, sort_keys=True)
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
