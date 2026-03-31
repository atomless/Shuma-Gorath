#!/usr/bin/env python3
"""Real Scrapling worker for the adversary-sim Scrapling lane."""

from __future__ import annotations

import argparse
from collections import Counter
from collections.abc import AsyncGenerator
import json
import os
from pathlib import Path
import socket
import sys
import time
from typing import Any
from urllib.parse import urljoin, urlsplit


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tests import shared_host_scope
from scripts.tests import sim_tag_helpers
from scripts.tests.adversarial_runner.contracts import (
    normalize_lane_realism_profile,
    resolve_lane_realism_profile,
)
from scripts.tests.adversarial_runner.identity_envelope import (
    normalize_identity_pool_entries,
    summarize_identity_realism,
)
from scripts.tests.adversarial_runner.transport_envelope import (
    resolve_browser_transport_observation,
    resolve_request_transport_observation,
)
from scripts.tests.adversarial_runner.realism import (
    realism_range_value as _realism_range_value,
    stable_bucket as _stable_bucket,
)


def _import_scrapling() -> tuple[Any, Any, Any, Any, Any]:
    from scrapling.fetchers import DynamicSession, FetcherSession, StealthySession
    from scrapling.spiders import Request, Spider

    return DynamicSession, FetcherSession, StealthySession, Request, Spider


class WorkerConfigError(ValueError):
    """Raised when required worker inputs are missing or invalid."""


SCRAPLING_FULFILLMENT_MODES = {
    "crawler",
    "bulk_scraper",
    "browser_automation",
    "stealth_browser",
    "http_agent",
}


def _sleep_ms(delay_ms: int) -> None:
    if delay_ms > 0:
        time.sleep(delay_ms / 1000.0)


class _ScraplingRealismTracker:
    def __init__(
        self,
        *,
        plan: dict[str, Any],
        browser_session: bool,
        proxy_configured: bool,
    ) -> None:
        self.plan = plan
        self.profile = normalize_lane_realism_profile(
            plan.get("realism_profile"),
            field_name="worker_plan.realism_profile",
        )
        self.recurrence_context = dict(plan.get("recurrence_context") or {})
        self.browser_session = browser_session
        self.proxy_configured = proxy_configured
        self.request_identity_pool = normalize_identity_pool_entries(
            plan.get("request_identity_pool"),
            field_name="worker_plan.request_identity_pool",
        )
        self.browser_identity_pool = normalize_identity_pool_entries(
            plan.get("browser_identity_pool"),
            field_name="worker_plan.browser_identity_pool",
        )
        self.run_id = str(plan.get("run_id") or "")
        self.tick_id = str(plan.get("tick_id") or "")
        self.fulfillment_mode = str(plan.get("fulfillment_mode") or "")
        self.max_requests = max(1, int(plan.get("max_requests") or 1))
        self.max_bytes = max(1, int(plan.get("max_bytes") or 1))
        self.max_ms = max(1, int(plan.get("max_ms") or 1))
        self.planned_activity_budget = _realism_range_value(
            dict(self.profile.get("activity_budget") or {}),
            self.run_id,
            self.tick_id,
            self.fulfillment_mode,
            "activity_budget",
        )
        self.effective_activity_budget = max(
            1,
            min(self.max_requests, self.planned_activity_budget),
        )
        self.planned_burst_size = _realism_range_value(
            dict(self.profile.get("burst_size") or {}),
            self.run_id,
            self.tick_id,
            self.fulfillment_mode,
            "burst_size",
        )
        self.effective_burst_size = max(
            1,
            min(self.effective_activity_budget, self.planned_burst_size),
        )
        rotation = dict(self.profile.get("identity_rotation") or {})
        self.rotation_strategy = str(rotation.get("strategy") or "none")
        if int(rotation.get("max_every_n_activities") or 0) > 0:
            self.rotation_every_n_activities = _realism_range_value(
                {
                    "min": int(rotation.get("min_every_n_activities") or 0),
                    "max": int(rotation.get("max_every_n_activities") or 0),
                },
                self.run_id,
                self.tick_id,
                self.fulfillment_mode,
                "identity_rotation",
            )
        else:
            self.rotation_every_n_activities = 0
        self.download_delay_ms = _realism_range_value(
            dict(self.profile.get("intra_burst_jitter_ms") or {}),
            self.run_id,
            self.tick_id,
            self.fulfillment_mode,
            "crawler_download_delay",
        )
        self.activity_count = 0
        self.current_burst_size = 0
        self.burst_sizes: list[int] = []
        self.inter_activity_gaps_ms: list[int] = []
        self.dwell_intervals_ms: list[int] = []
        self.identity_handles: list[str] = []
        self.session_handles: list[str] = []
        self.observed_country_codes: list[str] = []
        self.transport_profile = ""
        self.observed_user_agent_families: list[str] = []
        self.observed_accept_languages: list[str] = []
        self.observed_browser_locales: list[str] = []
        self.secondary_capture_mode = "xhr_capture" if browser_session else ""
        self.secondary_request_count = 0
        self.background_request_count = 0
        self.subresource_request_count = 0
        self.identity_rotation_count = 0
        self.visited_targets: set[str] = set()
        self.discovered_targets: set[str] = set()
        self.sitemap_documents: set[str] = set()
        self.canonical_public_pages: set[str] = set()
        self.deepest_depth_reached = 0
        self._last_identity_handle: str | None = None
        self._last_session_handle: str | None = None
        self._activities_since_rotation = 0

    def _identity_summary(self) -> dict[str, Any]:
        relevant_pool = self.browser_identity_pool if self.browser_session else self.request_identity_pool
        fixed_proxy_url = (
            _normalize_optional_proxy_url(self.plan.get("browser_proxy_url"))
            if self.browser_session
            else _normalize_optional_proxy_url(self.plan.get("request_proxy_url"))
        )
        return summarize_identity_realism(
            self.profile,
            pool_entries=relevant_pool,
            fixed_proxy_url=fixed_proxy_url,
            observed_country_codes=self.observed_country_codes,
        )

    def activity_limit_reached(self) -> bool:
        return self.activity_count >= self.effective_activity_budget

    def crawler_download_delay_seconds(self) -> float:
        return self.download_delay_ms / 1000.0

    def _range_value(self, field_name: str, ordinal: int) -> int:
        return _realism_range_value(
            dict(self.profile.get(field_name) or {}),
            self.run_id,
            self.tick_id,
            self.fulfillment_mode,
            field_name,
            ordinal,
        )

    def _cap_gap_to_remaining_window(self, gap_ms: int, remaining_ms: int) -> int:
        if gap_ms <= 0 or remaining_ms <= 0:
            return 0
        remaining_activities = max(1, self.effective_activity_budget - self.activity_count)
        return min(gap_ms, max(0, remaining_ms // remaining_activities))

    def _record_handle(
        self,
        handle: str,
        *,
        browser_session: bool,
        country_code: str | None = None,
    ) -> None:
        normalized = str(handle or "").strip()
        if not normalized:
            return
        normalized_country = str(country_code or "").strip().upper()
        if normalized_country and normalized_country not in self.observed_country_codes:
            self.observed_country_codes.append(normalized_country)
        if browser_session:
            if normalized not in self.session_handles:
                self.session_handles.append(normalized)
            if self._last_session_handle and self._last_session_handle != normalized:
                self.identity_rotation_count += 1
            self._last_session_handle = normalized
            return
        if normalized not in self.identity_handles:
            self.identity_handles.append(normalized)
        if self._last_identity_handle and self._last_identity_handle != normalized:
            self.identity_rotation_count += 1
        self._last_identity_handle = normalized

    def observe_transport(
        self,
        *,
        transport_profile: str,
        user_agent_family: str,
        accept_language: str,
        browser_locale: str | None = None,
    ) -> None:
        normalized_transport = str(transport_profile or "").strip()
        if normalized_transport:
            self.transport_profile = normalized_transport
        normalized_family = str(user_agent_family or "").strip()
        if normalized_family and normalized_family not in self.observed_user_agent_families:
            self.observed_user_agent_families.append(normalized_family)
        normalized_language = str(accept_language or "").strip()
        if normalized_language and normalized_language not in self.observed_accept_languages:
            self.observed_accept_languages.append(normalized_language)
        normalized_locale = str(browser_locale or "").strip()
        if normalized_locale and normalized_locale not in self.observed_browser_locales:
            self.observed_browser_locales.append(normalized_locale)

    def observe_discovered_target(self, target: str) -> None:
        normalized = str(target or "").strip()
        if normalized:
            self.discovered_targets.add(normalized)

    def observe_exploration_visit(
        self,
        *,
        target: str,
        depth: int,
        content_type: str = "",
    ) -> None:
        normalized = str(target or "").strip()
        if not normalized:
            return
        self.visited_targets.add(normalized)
        self.deepest_depth_reached = max(self.deepest_depth_reached, max(0, int(depth)))
        if _looks_canonical_public_page(normalized, content_type):
            self.canonical_public_pages.add(normalized)

    def observe_sitemap_document(self, target: str) -> None:
        normalized = str(target or "").strip()
        if normalized:
            self.sitemap_documents.add(normalized)

    def _finalize_burst(self) -> None:
        if self.current_burst_size > 0:
            self.burst_sizes.append(self.current_burst_size)
            self.current_burst_size = 0

    def crawler_observe_activity(
        self,
        identity_handle: str,
        *,
        country_code: str | None = None,
    ) -> None:
        if self.activity_count > 0 and self.download_delay_ms > 0:
            self.inter_activity_gaps_ms.append(self.download_delay_ms)
        self.activity_count += 1
        self.current_burst_size += 1
        if self.current_burst_size >= self.effective_burst_size:
            self._finalize_burst()
        self._record_handle(identity_handle, browser_session=False, country_code=country_code)

    def prepare_request_attempt(self, *, remaining_ms: int) -> tuple[int, bool]:
        rotate_identity = False
        gap_ms = 0
        if self.activity_count > 0:
            if self.current_burst_size >= self.effective_burst_size:
                self._finalize_burst()
                gap_ms = self._range_value(
                    "between_burst_pause_ms",
                    len(self.burst_sizes),
                )
                if (
                    self.proxy_configured
                    and self.rotation_strategy == "per_burst_when_proxy_available"
                ):
                    rotate_identity = True
            else:
                gap_ms = self._range_value(
                    "intra_burst_jitter_ms",
                    self.activity_count,
                )
            if (
                self.proxy_configured
                and self.rotation_strategy == "per_n_activities_when_proxy_available"
                and self.rotation_every_n_activities > 0
                and self._activities_since_rotation >= self.rotation_every_n_activities
            ):
                rotate_identity = True
        gap_ms = self._cap_gap_to_remaining_window(gap_ms, remaining_ms)
        if gap_ms > 0:
            self.inter_activity_gaps_ms.append(gap_ms)
            _sleep_ms(gap_ms)
        return gap_ms, rotate_identity

    def mark_request_attempt(
        self,
        identity_handle: str,
        *,
        country_code: str | None = None,
    ) -> None:
        self.activity_count += 1
        self.current_burst_size += 1
        self._activities_since_rotation += 1
        self._record_handle(identity_handle, browser_session=False, country_code=country_code)

    def note_rotation(self) -> None:
        self._activities_since_rotation = 0

    def observe_browser_secondary_traffic(
        self,
        *,
        capture_mode: str,
        background_paths: list[str] | None = None,
        subresource_count: int = 0,
    ) -> None:
        if not self.browser_session:
            return
        normalized_capture_mode = str(capture_mode or "").strip()
        if normalized_capture_mode:
            self.secondary_capture_mode = normalized_capture_mode
        background_count = len(list(background_paths or []))
        subresource_total = max(0, int(subresource_count))
        self.background_request_count += background_count
        self.subresource_request_count += subresource_total
        self.secondary_request_count += background_count + subresource_total

    def prepare_browser_action(
        self,
        session_handle: str,
        *,
        remaining_ms: int,
        country_code: str | None = None,
    ) -> None:
        if self.activity_count > 0:
            dwell_ms = self._range_value(
                "navigation_dwell_ms",
                self.activity_count,
            )
            dwell_ms = self._cap_gap_to_remaining_window(dwell_ms, remaining_ms)
            if dwell_ms > 0:
                self.dwell_intervals_ms.append(dwell_ms)
                _sleep_ms(dwell_ms)
        self.activity_count += 1
        self._record_handle(
            session_handle,
            browser_session=True,
            country_code=country_code,
        )

    def stop_reason(
        self,
        *,
        bytes_observed: int,
        deadline_reached: bool,
        activity_sequence_exhausted: bool,
        transport_failure: bool,
    ) -> str:
        if self.activity_count >= self.effective_activity_budget:
            if self.max_requests <= self.planned_activity_budget:
                return "max_requests_exhausted"
            return "activity_budget_reached"
        if bytes_observed >= self.max_bytes:
            return "byte_budget_exhausted"
        if deadline_reached:
            return "time_budget_exhausted"
        if transport_failure and self.activity_count == 0:
            return "transport_error"
        if activity_sequence_exhausted:
            return "activity_sequence_exhausted"
        return "completed"

    def render_receipt(
        self,
        *,
        bytes_observed: int,
        deadline_reached: bool,
        activity_sequence_exhausted: bool,
        transport_failure: bool,
    ) -> dict[str, Any]:
        self._finalize_burst()
        identity_summary = self._identity_summary()
        receipt = {
            "schema_version": str(
                dict(self.profile.get("receipt_contract") or {}).get("schema_version")
                or "sim-lane-realism-receipt.v1"
            ),
            "profile_id": str(self.profile.get("profile_id") or ""),
            "activity_unit": str(self.profile.get("activity_unit") or ""),
            "planned_activity_budget": self.planned_activity_budget,
            "effective_activity_budget": self.effective_activity_budget,
            "planned_burst_size": self.planned_burst_size,
            "effective_burst_size": self.effective_burst_size,
            "activity_count": self.activity_count,
            "transport_profile": self.transport_profile,
            "observed_user_agent_families": list(self.observed_user_agent_families),
            "observed_accept_languages": list(self.observed_accept_languages),
            "identity_realism_status": identity_summary["identity_realism_status"],
            "identity_envelope_classes": list(
                identity_summary["identity_envelope_classes"]
            ),
            "geo_affinity_mode": identity_summary["geo_affinity_mode"],
            "session_stickiness": identity_summary["session_stickiness"],
            "observed_country_codes": list(identity_summary["observed_country_codes"]),
            "identity_rotation_count": self.identity_rotation_count,
            "recurrence_strategy": str(
                self.recurrence_context.get("strategy") or ""
            ),
            "session_index": int(self.recurrence_context.get("session_index") or 0),
            "reentry_count": int(self.recurrence_context.get("reentry_count") or 0),
            "max_reentries_per_run": int(
                self.recurrence_context.get("max_reentries_per_run") or 0
            ),
            "planned_dormant_gap_seconds": int(
                self.recurrence_context.get("planned_dormant_gap_seconds") or 0
            ),
            "visited_url_count": len(self.visited_targets),
            "discovered_url_count": len(self.discovered_targets),
            "deepest_depth_reached": self.deepest_depth_reached,
            "sitemap_documents_seen": len(self.sitemap_documents),
            "frontier_remaining_count": max(
                0,
                len(self.discovered_targets.difference(self.visited_targets)),
            ),
            "canonical_public_pages_reached": len(self.canonical_public_pages),
            "stop_reason": self.stop_reason(
                bytes_observed=bytes_observed,
                deadline_reached=deadline_reached,
                activity_sequence_exhausted=activity_sequence_exhausted,
                transport_failure=transport_failure,
            ),
        }
        if self.browser_session:
            receipt.update(
                {
                    "top_level_action_count": self.activity_count,
                    "dwell_intervals_ms": list(self.dwell_intervals_ms),
                    "observed_browser_locales": list(self.observed_browser_locales),
                    "secondary_capture_mode": self.secondary_capture_mode,
                    "secondary_request_count": self.secondary_request_count,
                    "background_request_count": self.background_request_count,
                    "subresource_request_count": self.subresource_request_count,
                    "session_handles": list(self.session_handles),
                }
            )
        else:
            receipt.update(
                {
                    "burst_count": len(self.burst_sizes),
                    "burst_sizes": list(self.burst_sizes),
                    "inter_activity_gaps_ms": list(self.inter_activity_gaps_ms),
                    "identity_handles": list(self.identity_handles),
                }
            )
        required_fields = list(
            dict(self.profile.get("receipt_contract") or {}).get("required_fields") or []
        )
        missing = [field for field in required_fields if field not in receipt]
        if missing:
            raise WorkerConfigError(
                "worker_plan realism_profile receipt contract is missing required fields: "
                + ", ".join(missing)
            )
        return receipt


class _PacedRequestNativeSession:
    def __init__(
        self,
        session_cls: Any,
        *,
        tracker: "_DirectPersonaTracker",
        timeout_seconds: float,
        accept_header: str,
        proxy_url: str | None,
        request_transport: dict[str, str],
        identity_pool: list[dict[str, str]] | None = None,
    ) -> None:
        self.session_cls = session_cls
        self.tracker = tracker
        self.timeout_seconds = timeout_seconds
        self.accept_header = accept_header
        self.proxy_url = proxy_url
        self.request_transport = dict(request_transport)
        self.identity_pool = list(identity_pool or [])
        self._session_cm: Any | None = None
        self._session: Any | None = None
        self._session_index = 0
        self._current_identity_handle = ""
        self._current_country_code: str | None = None
        self._identity_pool_index = -1

    def __enter__(self) -> "_PacedRequestNativeSession":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        if self._session_cm is not None:
            self._session_cm.__exit__(exc_type, exc, tb)
        self._session_cm = None
        self._session = None

    def _open_session(self) -> None:
        if self._session_cm is not None:
            self._session_cm.__exit__(None, None, None)
        current_proxy_url = self.proxy_url
        current_identity_handle = ""
        current_country_code: str | None = None
        if self.identity_pool:
            self._identity_pool_index = (self._identity_pool_index + 1) % len(self.identity_pool)
            entry = dict(self.identity_pool[self._identity_pool_index])
            current_proxy_url = str(entry.get("proxy_url") or "").strip() or None
            current_identity_handle = f"request-session-{str(entry.get('label') or '').strip()}"
            current_country_code = str(entry.get("country_code") or "").strip().upper() or None
            request_transport = resolve_request_transport_observation(
                self.tracker.realism_tracker.profile,
                country_code=current_country_code,
            )
        else:
            request_transport = dict(self.request_transport)
        self._session_cm = self.session_cls(
            **_request_native_session_kwargs(
                timeout_seconds=self.timeout_seconds,
                accept_header=self.accept_header,
                proxy_url=current_proxy_url,
                request_impersonate=str(request_transport.get("request_impersonate") or "chrome"),
                accept_language=str(request_transport.get("accept_language") or "en-US,en;q=0.9"),
            ),
        )
        self._session = self._session_cm.__enter__()
        self._session_index += 1
        self._current_identity_handle = (
            current_identity_handle or f"request-session-{self._session_index}"
        )
        self._current_country_code = current_country_code
        self.tracker.realism_tracker.observe_transport(
            transport_profile=str(request_transport.get("transport_profile") or ""),
            user_agent_family=str(request_transport.get("user_agent_family") or ""),
            accept_language=str(request_transport.get("accept_language") or ""),
        )
        if self._session_index > 1:
            self.tracker.realism_tracker.note_rotation()

    def fetch(self, target: str, **kwargs) -> Any:
        if self.tracker.should_stop():
            raise RuntimeError("request budget exhausted before next persona fetch")
        _, rotate_identity = self.tracker.realism_tracker.prepare_request_attempt(
            remaining_ms=self.tracker.remaining_ms(),
        )
        if self._session is None or rotate_identity:
            self._open_session()
        self.tracker.realism_tracker.mark_request_attempt(
            self._current_identity_handle,
            country_code=self._current_country_code,
        )
        return self._session.fetch(target, **kwargs)

    def _call_method(self, method_name: str, target: str, **kwargs) -> Any:
        if self.tracker.should_stop():
            raise RuntimeError("request budget exhausted before next persona fetch")
        _, rotate_identity = self.tracker.realism_tracker.prepare_request_attempt(
            remaining_ms=self.tracker.remaining_ms(),
        )
        if self._session is None or rotate_identity:
            self._open_session()
        self.tracker.realism_tracker.mark_request_attempt(
            self._current_identity_handle,
            country_code=self._current_country_code,
        )
        return getattr(self._session, method_name)(target, **kwargs)

    def get(self, target: str, **kwargs) -> Any:
        return self._call_method("get", target, **kwargs)

    def post(self, target: str, **kwargs) -> Any:
        return self._call_method("post", target, **kwargs)


def _load_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise WorkerConfigError(f"JSON payload at {path} must be an object")
    return payload


def _normalize_allowed_domains(descriptor: shared_host_scope.SharedHostScopeDescriptor) -> set[str]:
    normalized: set[str] = set()
    for host in descriptor.allowed_hosts:
        raw_host = host.strip().lower()
        if not raw_host:
            continue
        normalized.add(raw_host)
        normalized.add(raw_host.split(":", 1)[0].strip())
    return normalized


def _normalized_start_urls(seed_inventory: dict[str, Any]) -> list[str]:
    ordered: list[str] = []
    for section in ("accepted_start_urls", "accepted_hint_documents"):
        entries = seed_inventory.get(section) or []
        if not isinstance(entries, list):
            continue
        for entry in entries:
            if not isinstance(entry, dict):
                continue
            url = str(entry.get("url") or "").strip()
            if url and url not in ordered:
                ordered.append(url)
    return ordered


def _env_or_arg(value: str | None, env_name: str) -> str | None:
    if value and str(value).strip():
        return str(value).strip()
    env_value = os.environ.get(env_name, "").strip()
    return env_value or None


def _normalize_optional_proxy_url(raw_proxy: Any) -> str | None:
    value = str(raw_proxy or "").strip()
    if not value:
        return None
    if "\r" in value or "\n" in value:
        raise WorkerConfigError("worker_plan proxy URLs must not contain newline characters")
    return value


def _normalize_category_targets(raw_targets: Any) -> list[str]:
    if not isinstance(raw_targets, list):
        return []
    normalized: list[str] = []
    for value in raw_targets:
        item = str(value or "").strip()
        if item and item not in normalized:
            normalized.append(item)
    return normalized


def _expected_category_targets_for_mode(fulfillment_mode: str) -> list[str]:
    return {
        "crawler": ["indexing_bot"],
        "bulk_scraper": ["ai_scraper_bot"],
        "browser_automation": ["automated_browser"],
        "stealth_browser": ["automated_browser"],
        "http_agent": ["http_agent"],
    }.get(fulfillment_mode, [])


def _normalize_surface_targets(raw_targets: Any) -> list[str]:
    if not isinstance(raw_targets, list):
        return []
    normalized: list[str] = []
    for value in raw_targets:
        item = str(value or "").strip()
        if item and item not in normalized:
            normalized.append(item)
    return normalized


def _expected_surface_targets_for_mode(fulfillment_mode: str) -> list[str]:
    return {
        "crawler": [
            "public_path_traversal",
            "challenge_routing",
            "rate_pressure",
            "geo_ip_policy",
        ],
        "bulk_scraper": [
            "public_path_traversal",
            "challenge_routing",
            "rate_pressure",
            "geo_ip_policy",
            "not_a_bot_submit",
            "puzzle_submit_or_escalation",
        ],
        "browser_automation": [
            "challenge_routing",
            "maze_navigation",
            "js_verification_execution",
            "browser_automation_detection",
        ],
        "stealth_browser": [
            "challenge_routing",
            "maze_navigation",
            "js_verification_execution",
            "browser_automation_detection",
        ],
        "http_agent": [
            "challenge_routing",
            "rate_pressure",
            "geo_ip_policy",
            "not_a_bot_submit",
            "puzzle_submit_or_escalation",
            "pow_verify_abuse",
            "tarpit_progress_abuse",
        ],
    }.get(fulfillment_mode, [])


def _invalid_not_a_bot_body() -> str:
    telemetry = json.dumps(
        {
            "has_pointer": False,
            "pointer_move_count": 0,
            "pointer_path_length": 0.0,
            "pointer_direction_changes": 0,
            "down_up_ms": 70,
            "focus_changes": 4,
            "visibility_changes": 1,
            "interaction_elapsed_ms": 700,
            "keyboard_used": False,
            "touch_used": False,
            "activation_method": "unknown",
            "activation_trusted": False,
            "activation_count": 1,
            "control_focused": False,
        },
        separators=(",", ":"),
    )
    return f"seed=invalid-seed&checked=1&telemetry={telemetry}"


def _request_spec(
    method: str,
    target: str,
    *,
    surface_ids: list[str] | None = None,
    headers: dict[str, str] | None = None,
    cookies: dict[str, str] | None = None,
    data: str | bytes | None = None,
    json_body: dict[str, Any] | list[Any] | None = None,
    follow_redirect: bool = False,
) -> dict[str, Any]:
    spec: dict[str, Any] = {
        "method": method,
        "target": target,
        "headers": dict(headers or {}),
        "follow_redirect": follow_redirect,
    }
    if surface_ids:
        spec["surface_ids"] = list(surface_ids)
    if cookies:
        spec["cookies"] = dict(cookies)
    if data is not None:
        spec["data"] = data
    if json_body is not None:
        spec["json"] = json_body
    return spec


def _request_path_value(raw_target: str) -> str:
    parsed = urlsplit(str(raw_target))
    path = parsed.path or "/"
    if parsed.query:
        return f"{path}?{parsed.query}"
    return path


def _response_content_type(response: Any) -> str:
    headers = getattr(response, "headers", None)
    if headers is None:
        return ""
    return str(headers.get("content-type") or headers.get("Content-Type") or "").strip().lower()


def _looks_static_asset_path(path: str) -> bool:
    normalized = str(path or "").strip().lower()
    return normalized.endswith(
        (
            ".css",
            ".js",
            ".png",
            ".jpg",
            ".jpeg",
            ".gif",
            ".svg",
            ".ico",
            ".webp",
            ".woff",
            ".woff2",
            ".ttf",
            ".map",
            ".xml",
        )
    )


def _looks_canonical_public_page(target: str, content_type: str) -> bool:
    normalized_target = str(target or "").strip()
    if not normalized_target:
        return False
    request_path = _request_path_value(normalized_target)
    if _looks_static_asset_path(request_path):
        return False
    normalized_content_type = str(content_type or "").strip().lower()
    if normalized_content_type.startswith("text/html"):
        return True
    if normalized_content_type.startswith("application/xhtml+xml"):
        return True
    if normalized_content_type.startswith("application/json"):
        return True
    return not normalized_content_type


def _ordered_unique(values: list[str]) -> list[str]:
    ordered: list[str] = []
    for value in values:
        item = str(value or "").strip()
        if item and item not in ordered:
            ordered.append(item)
    return ordered


def _response_css_values(response: Any, selector: str) -> list[str]:
    try:
        values = response.css(selector).getall()
    except Exception:
        return []
    return [str(value or "").strip() for value in values if str(value or "").strip()]


def _response_anchor_targets(response: Any) -> list[str]:
    base_url = str(getattr(response, "url", "") or "").strip()
    targets = [
        urljoin(base_url, raw_target)
        for raw_target in _response_css_values(response, "a::attr(href)")
    ]
    return _ordered_unique(targets)


def _response_form_targets(response: Any) -> list[str]:
    base_url = str(getattr(response, "url", "") or "").strip()
    targets = [
        urljoin(base_url, raw_target)
        for raw_target in _response_css_values(response, "form::attr(action)")
    ]
    return _ordered_unique(targets)


def _public_discovery_surface_ids(surface_targets: set[str]) -> list[str]:
    return [
        surface_id
        for surface_id in (
            "challenge_routing",
            "rate_pressure",
            "geo_ip_policy",
        )
        if surface_id in surface_targets
    ]


def _path_contains(raw_target: str, *needles: str) -> bool:
    candidate = _request_path_value(raw_target).lower()
    return any(str(needle or "").lower() in candidate for needle in needles)


def _looks_bulk_scraper_public_target(raw_target: str) -> bool:
    candidate = _request_path_value(raw_target).lower()
    return (
        candidate == "/"
        or "catalog" in candidate
        or "/detail/" in candidate
    )


def _bulk_scraper_priority(raw_target: str) -> tuple[int, str]:
    candidate = _request_path_value(raw_target).lower()
    if "catalog" in candidate:
        return (0, candidate)
    if "/detail/" in candidate:
        return (1, candidate)
    if candidate == "/":
        return (2, candidate)
    return (3, candidate)


def _crawler_link_priority(raw_target: str) -> tuple[int, str]:
    candidate = _request_path_value(raw_target).lower()
    if "redirect" in candidate:
        return (0, candidate)
    if candidate == "/page":
        return (1, candidate)
    if "catalog" in candidate:
        return (2, candidate)
    if "challenge" in candidate:
        return (3, candidate)
    if candidate == "/pow":
        return (4, candidate)
    if "/maze/" in candidate:
        return (5, candidate)
    return (6, candidate)


def _first_matching_target(candidates: list[str], predicate) -> str | None:
    for candidate in candidates:
        if predicate(candidate):
            return candidate
    return None


def _normalize_browser_discovery_targets(base_url: str, raw_targets: list[Any]) -> list[str]:
    targets: list[str] = []
    for raw_target in raw_targets:
        value = str(raw_target or "").strip()
        if not value:
            continue
        targets.append(urljoin(base_url, value))
    return _ordered_unique(targets)


def _coverage_status_for_http_status(status: int) -> str:
    return "pass_observed" if 200 <= int(status) < 400 else "fail_observed"


def _record_surface_receipt(
    receipts: dict[str, dict[str, Any]],
    *,
    surface_ids: list[str],
    coverage_status: str,
    request_method: str,
    request_target: str,
    response_status: int | None,
) -> None:
    sample_request_method = str(request_method or "").upper()
    sample_request_path = _request_path_value(request_target)
    for surface_id in surface_ids:
        key = str(surface_id or "").strip()
        if not key:
            continue
        receipt_key = f"{key}:{coverage_status}"
        existing = receipts.get(receipt_key)
        if existing is None:
            receipts[receipt_key] = {
                "surface_id": key,
                "coverage_status": coverage_status,
                "attempt_count": 1,
                "sample_request_method": sample_request_method,
                "sample_request_path": sample_request_path,
                "sample_response_status": response_status,
            }
            continue
        existing["attempt_count"] = int(existing.get("attempt_count") or 0) + 1
        existing["sample_request_method"] = sample_request_method
        existing["sample_request_path"] = sample_request_path
        existing["sample_response_status"] = response_status


def _render_surface_receipts(
    receipts: dict[str, dict[str, Any]],
) -> list[dict[str, Any]]:
    rendered: list[dict[str, Any]] = []
    for receipt_key in sorted(
        receipts,
        key=lambda key: (
            str(receipts[key].get("surface_id") or ""),
            str(receipts[key].get("coverage_status") or ""),
            str(receipts[key].get("sample_request_path") or ""),
        ),
    ):
        entry = dict(receipts[receipt_key])
        if entry.get("sample_response_status") is None:
            entry.pop("sample_response_status", None)
        rendered.append(entry)
    return rendered


def _build_failure_result(
    beat_payload: dict[str, Any],
    *,
    failure_class: str,
    error: str,
) -> dict[str, Any]:
    plan = beat_payload.get("worker_plan") if isinstance(beat_payload.get("worker_plan"), dict) else {}
    now = int(time.time())
    return {
        "schema_version": "adversary-sim-scrapling-worker-result.v1",
        "run_id": str(plan.get("run_id") or ""),
        "tick_id": str(plan.get("tick_id") or ""),
        "lane": str(plan.get("lane") or "scrapling_traffic"),
        "fulfillment_mode": str(plan.get("fulfillment_mode") or ""),
        "category_targets": _normalize_category_targets(plan.get("category_targets")),
        "worker_id": socket.gethostname(),
        "tick_started_at": int(plan.get("tick_started_at") or now),
        "tick_completed_at": now,
        "generated_requests": 0,
        "failed_requests": 0,
        "last_response_status": None,
        "failure_class": failure_class,
        "error": error,
        "crawl_stats": {
            "requests_count": 0,
            "offsite_requests_count": 0,
            "blocked_requests_count": 0,
            "response_status_count": {},
            "response_bytes": 0,
        },
        "scope_rejections": {},
    }


def _request_native_session_kwargs(
    *,
    timeout_seconds: float,
    accept_header: str,
    request_impersonate: str,
    accept_language: str,
    proxy_url: str | None = None,
) -> dict[str, Any]:
    kwargs = {
        "impersonate": request_impersonate,
        "stealthy_headers": True,
        "follow_redirects": False,
        "timeout": timeout_seconds,
        "retries": 1,
        "headers": {
            "accept": accept_header,
            "accept-language": accept_language,
        },
    }
    if proxy_url:
        kwargs["proxy"] = proxy_url
    return kwargs


def _browser_session_kwargs(
    *,
    fulfillment_mode: str,
    timeout_ms: int,
    locale: str,
    useragent: str,
    proxy_url: str | None = None,
) -> dict[str, Any]:
    kwargs: dict[str, Any] = {
        "headless": True,
        "disable_resources": False,
        "google_search": False,
        "network_idle": False,
        "load_dom": True,
        "capture_xhr": ".*",
        "timeout": timeout_ms,
        "wait": min(500, max(100, timeout_ms // 12)),
        "retries": 1,
        "retry_delay": 0,
        "locale": locale,
        "useragent": useragent,
    }
    if fulfillment_mode == "stealth_browser":
        kwargs.update(
            {
                "hide_canvas": True,
                "block_webrtc": True,
                "allow_webgl": True,
            }
        )
    if proxy_url:
        kwargs["proxy"] = proxy_url
    return kwargs


def _signed_headers(
    secret: str,
    *,
    run_id: str,
    profile: str,
    lane: str,
    fulfillment_mode: str,
    seq: int,
    extra_headers: dict[str, str] | None = None,
) -> dict[str, str]:
    sim_profile = f"{profile}.{fulfillment_mode}" if fulfillment_mode else profile
    timestamp = str(int(time.time()))
    nonce = f"{run_id}:{sim_profile}:{lane}:{seq}:{timestamp}"
    signature = sim_tag_helpers.sign_sim_tag(
        secret=secret,
        run_id=run_id,
        profile=sim_profile,
        lane=lane,
        timestamp=timestamp,
        nonce=nonce,
    )
    headers = {
        sim_tag_helpers.SIM_TAG_HEADER_RUN_ID: run_id,
        sim_tag_helpers.SIM_TAG_HEADER_PROFILE: sim_profile,
        sim_tag_helpers.SIM_TAG_HEADER_LANE: lane,
        sim_tag_helpers.SIM_TAG_HEADER_TIMESTAMP: timestamp,
        sim_tag_helpers.SIM_TAG_HEADER_NONCE: nonce,
        sim_tag_helpers.SIM_TAG_HEADER_SIGNATURE: signature,
    }
    if extra_headers:
        headers.update(extra_headers)
    return headers


def _build_spider_class(fetcher_session_cls: Any, request_cls: Any, spider_base: Any):
    class ShumaScraplingSpider(spider_base):  # type: ignore[misc]
        name = "shuma_scrapling_lane"
        concurrent_requests = 1
        concurrent_requests_per_domain = 1
        download_delay = 0.0
        max_blocked_retries = 0

        def __init__(
            self,
            *,
            plan: dict[str, Any],
            descriptor: shared_host_scope.SharedHostScopeDescriptor,
            seed_inventory: dict[str, Any],
            crawldir: Path,
            sim_telemetry_secret: str,
        ) -> None:
            self.plan = plan
            self.descriptor = descriptor
            self.seed_inventory = seed_inventory
            self.max_requests = max(1, int(plan.get("max_requests") or 1))
            self.max_depth = max(0, int(plan.get("max_depth") or 0))
            self.max_bytes = max(1, int(plan.get("max_bytes") or 1))
            self.max_ms = max(1, int(plan.get("max_ms") or 1))
            self.run_id = str(plan.get("run_id") or "")
            self.tick_id = str(plan.get("tick_id") or "")
            self.lane = str(plan.get("lane") or "scrapling_traffic")
            self.sim_profile = str(plan.get("sim_profile") or "scrapling_runtime_lane")
            self.fulfillment_mode = str(plan.get("fulfillment_mode") or "crawler")
            self.surface_targets = set(_normalize_surface_targets(plan.get("surface_targets")))
            self.request_identity_pool = normalize_identity_pool_entries(
                plan.get("request_identity_pool"),
                field_name="worker_plan.request_identity_pool",
            )
            self.realism_tracker = _ScraplingRealismTracker(
                plan=plan,
                browser_session=False,
                proxy_configured=bool(
                    _normalize_optional_proxy_url(plan.get("request_proxy_url"))
                    or self.request_identity_pool
                ),
            )
            self.request_transport = resolve_request_transport_observation(
                self.realism_tracker.profile,
                country_code=(
                    str(self.request_identity_pool[0].get("country_code") or "").strip().upper()
                    if self.request_identity_pool
                    else None
                ),
            )
            self.realism_tracker.observe_transport(
                transport_profile=str(self.request_transport.get("transport_profile") or ""),
                user_agent_family=str(self.request_transport.get("user_agent_family") or ""),
                accept_language=str(self.request_transport.get("accept_language") or ""),
            )
            self.deadline = time.monotonic() + (self.max_ms / 1000.0)
            self.sim_telemetry_secret = sim_telemetry_secret
            self.request_sequence = 0
            self.requests_observed = 0
            self.bytes_observed = 0
            self.last_response_status: int | None = None
            self.scope_rejections: Counter[str] = Counter()
            self.last_transport_error: str | None = None
            self.surface_receipts: dict[str, dict[str, Any]] = {}
            self.allowed_domains = _normalize_allowed_domains(descriptor)
            self.start_urls = _normalized_start_urls(seed_inventory)
            self.discovery_surfaces_recorded = False
            self.activity_sequence_exhausted = False
            super().__init__(crawldir=str(crawldir), interval=0.0)
            self.download_delay = self.realism_tracker.crawler_download_delay_seconds()

        def configure_sessions(self, manager) -> None:
            timeout_seconds = max(1.0, min(30.0, self.max_ms / 1000.0))
            request_proxy_url = _normalize_optional_proxy_url(self.plan.get("request_proxy_url"))
            if self.request_identity_pool:
                request_proxy_url = (
                    str(self.request_identity_pool[0].get("proxy_url") or "").strip()
                    or request_proxy_url
                )
            manager.add(
                "default",
                fetcher_session_cls(**_request_native_session_kwargs(
                    timeout_seconds=timeout_seconds,
                    accept_header="*/*",
                    request_impersonate=str(self.request_transport.get("request_impersonate") or "chrome"),
                    accept_language=str(self.request_transport.get("accept_language") or "en-US,en;q=0.9"),
                    proxy_url=request_proxy_url,
                )),
            )

        def _should_stop(self) -> bool:
            return (
                self.realism_tracker.activity_limit_reached()
                or self.bytes_observed >= self.max_bytes
                or time.monotonic() >= self.deadline
            )

        def _next_headers(self, extra_headers: dict[str, str] | None = None) -> dict[str, str]:
            self.request_sequence += 1
            return _signed_headers(
                self.sim_telemetry_secret,
                run_id=self.run_id,
                profile=self.sim_profile,
                lane=self.lane,
                fulfillment_mode=self.fulfillment_mode,
                seq=self.request_sequence,
                extra_headers=extra_headers,
            )

        def _record_rejection(self, reason: str | None) -> None:
            reason_key = str(reason or "malformed_url").strip() or "malformed_url"
            self.scope_rejections[reason_key] += 1

        async def on_error(self, request, error: Exception) -> None:
            self.last_transport_error = f"{type(error).__name__}: {error}"

        def _allowed_request(
            self,
            current_url: str,
            raw_target: str,
            *,
            is_redirect: bool,
        ) -> tuple[bool, str | None]:
            if is_redirect:
                decision = shared_host_scope.evaluate_redirect_target(
                    current_url, raw_target, self.descriptor
                )
            else:
                decision = shared_host_scope.evaluate_url_candidate(raw_target, self.descriptor)
            if not decision.allowed or not decision.normalized_url:
                self._record_rejection(decision.rejection_reason)
                return False, None
            return True, decision.normalized_url

        async def start_requests(self) -> AsyncGenerator[Any, None]:
            for url in self.start_urls:
                self.realism_tracker.observe_discovered_target(url)
                yield request_cls(
                    url,
                    sid="default",
                    meta={"depth": 0},
                    headers=self._next_headers(
                        {"accept-language": str(self.request_transport.get("accept_language") or "")}
                    ),
                )

        async def parse(self, response) -> AsyncGenerator[Any, None]:
            depth = int((response.meta or {}).get("depth") or 0)
            self.realism_tracker.observe_exploration_visit(
                target=str(getattr(response, "url", "") or ""),
                depth=depth,
                content_type=_response_content_type(response),
            )
            self.realism_tracker.crawler_observe_activity(
                (
                    f"crawl-session-{str(self.request_identity_pool[0].get('label') or '').strip()}"
                    if self.request_identity_pool
                    else "crawl-session-1"
                ),
                country_code=(
                    str(self.request_identity_pool[0].get("country_code") or "").strip().upper()
                    if self.request_identity_pool
                    else None
                ),
            )
            self.requests_observed += 1
            self.bytes_observed += len(response.body)
            self.last_response_status = int(response.status)
            surface_ids: list[str] = []
            if "public_path_traversal" in self.surface_targets:
                surface_ids.append("public_path_traversal")
            if not self.discovery_surfaces_recorded:
                discovery_surface_ids = _public_discovery_surface_ids(self.surface_targets)
                if discovery_surface_ids:
                    surface_ids.extend(discovery_surface_ids)
                    self.discovery_surfaces_recorded = True
            if surface_ids:
                _record_surface_receipt(
                    self.surface_receipts,
                    surface_ids=surface_ids,
                    coverage_status=_coverage_status_for_http_status(int(response.status)),
                    request_method=str(getattr(response.request, "method", "GET")),
                    request_target=str(getattr(response, "url", "")),
                    response_status=int(response.status),
                )

            if self._should_stop():
                self.pause()
                return

            next_depth = depth + 1

            if 300 <= int(response.status) < 400:
                location = str(response.headers.get("location") or "").strip()
                if location and next_depth <= self.max_depth:
                    allowed, normalized_url = self._allowed_request(
                        response.url,
                        location,
                        is_redirect=True,
                    )
                    if allowed and normalized_url:
                        self.realism_tracker.observe_discovered_target(normalized_url)
                        yield response.follow(
                            normalized_url,
                            meta={"depth": next_depth},
                            headers=self._next_headers(
                                {
                                    "accept-language": str(
                                        self.request_transport.get("accept_language") or ""
                                    )
                                }
                            ),
                        )
                return

            sitemap_targets = [value.strip() for value in response.css("loc::text").getall()]
            if sitemap_targets:
                self.realism_tracker.observe_sitemap_document(
                    str(getattr(response, "url", "") or "")
                )
                for raw_target in sitemap_targets:
                    if not raw_target or next_depth > self.max_depth:
                        continue
                    allowed, normalized_url = self._allowed_request(
                        response.url,
                        raw_target,
                        is_redirect=False,
                    )
                    if allowed and normalized_url:
                        self.realism_tracker.observe_discovered_target(normalized_url)
                        yield response.follow(
                            normalized_url,
                            meta={"depth": next_depth},
                            headers=self._next_headers(
                                {
                                    "accept-language": str(
                                        self.request_transport.get("accept_language") or ""
                                    )
                                }
                            ),
                        )
                        if self._should_stop():
                            self.pause()
                            return
                return

            link_targets = sorted(
                [urljoin(response.url, href) for href in response.css("a::attr(href)").getall()],
                key=_crawler_link_priority,
            )
            yielded_next_target = False
            for candidate in link_targets:
                if next_depth > self.max_depth:
                    continue
                allowed, normalized_url = self._allowed_request(
                    response.url,
                    candidate,
                    is_redirect=False,
                )
                if allowed and normalized_url:
                    self.realism_tracker.observe_discovered_target(normalized_url)
                    yield response.follow(
                        normalized_url,
                        meta={"depth": next_depth},
                        headers=self._next_headers(),
                    )
                    yielded_next_target = True
                    if self._should_stop():
                        self.pause()
                        return
            if not yielded_next_target:
                self.activity_sequence_exhausted = True

    return ShumaScraplingSpider


class _DirectPersonaTracker:
    def __init__(
        self,
        *,
        plan: dict[str, Any],
        descriptor: shared_host_scope.SharedHostScopeDescriptor,
        sim_telemetry_secret: str,
    ) -> None:
        self.plan = plan
        self.descriptor = descriptor
        self.max_requests = max(1, int(plan.get("max_requests") or 1))
        self.max_bytes = max(1, int(plan.get("max_bytes") or 1))
        self.max_ms = max(1, int(plan.get("max_ms") or 1))
        self.run_id = str(plan.get("run_id") or "")
        self.tick_id = str(plan.get("tick_id") or "")
        self.lane = str(plan.get("lane") or "scrapling_traffic")
        self.sim_profile = str(plan.get("sim_profile") or "scrapling_runtime_lane")
        self.fulfillment_mode = str(plan.get("fulfillment_mode") or "")
        self.realism_tracker = _ScraplingRealismTracker(
            plan=plan,
            browser_session=self.fulfillment_mode in {"browser_automation", "stealth_browser"},
            proxy_configured=bool(
                _normalize_optional_proxy_url(plan.get("request_proxy_url"))
                or _normalize_optional_proxy_url(plan.get("browser_proxy_url"))
                or normalize_identity_pool_entries(
                    plan.get("request_identity_pool"),
                    field_name="worker_plan.request_identity_pool",
                )
                or normalize_identity_pool_entries(
                    plan.get("browser_identity_pool"),
                    field_name="worker_plan.browser_identity_pool",
                )
            ),
        )
        self.deadline = time.monotonic() + (self.max_ms / 1000.0)
        self.sim_telemetry_secret = sim_telemetry_secret
        self.request_sequence = 0
        self.generated_requests = 0
        self.failed_requests = 0
        self.bytes_observed = 0
        self.last_response_status: int | None = None
        self.last_transport_error: str | None = None
        self.response_status_count: Counter[str] = Counter()
        self.scope_rejections: Counter[str] = Counter()
        self.surface_receipts: dict[str, dict[str, Any]] = {}

    def should_stop(self) -> bool:
        return (
            self.realism_tracker.activity_limit_reached()
            or self.bytes_observed >= self.max_bytes
            or time.monotonic() >= self.deadline
        )

    def remaining_ms(self) -> int:
        return max(0, int((self.deadline - time.monotonic()) * 1000))

    def next_headers(self, extra_headers: dict[str, str] | None = None) -> dict[str, str]:
        self.request_sequence += 1
        return _signed_headers(
            self.sim_telemetry_secret,
            run_id=self.run_id,
            profile=self.sim_profile,
            lane=self.lane,
            fulfillment_mode=self.fulfillment_mode,
            seq=self.request_sequence,
            extra_headers=extra_headers,
        )

    def record_rejection(self, reason: str | None) -> None:
        reason_key = str(reason or "malformed_url").strip() or "malformed_url"
        self.scope_rejections[reason_key] += 1

    def allowed_request(
        self,
        current_url: str,
        raw_target: str,
        *,
        is_redirect: bool,
        record_rejection: bool = True,
    ) -> tuple[bool, str | None]:
        if is_redirect:
            decision = shared_host_scope.evaluate_redirect_target(
                current_url, raw_target, self.descriptor
            )
        else:
            decision = shared_host_scope.evaluate_url_candidate(raw_target, self.descriptor)
        if not decision.allowed or not decision.normalized_url:
            if record_rejection:
                self.record_rejection(decision.rejection_reason)
            return False, None
        return True, decision.normalized_url

    def record_response(
        self,
        response: Any,
        surface_ids: list[str] | None = None,
        *,
        request_method: str | None = None,
        request_target: str | None = None,
    ) -> None:
        self.generated_requests += 1
        self.last_response_status = int(response.status)
        body_bytes = bytes(response.body)
        self.bytes_observed += len(body_bytes)
        status_key = f"status_{int(response.status)}"
        self.response_status_count[status_key] += 1
        if surface_ids:
            _record_surface_receipt(
                self.surface_receipts,
                surface_ids=surface_ids,
                coverage_status=_coverage_status_for_http_status(int(response.status)),
                request_method=request_method or str(getattr(response.request, "method", "GET")),
                request_target=request_target or str(getattr(response, "url", "")),
                response_status=int(response.status),
            )

    def record_failure(
        self,
        error: Exception,
        *,
        surface_ids: list[str] | None = None,
        request_method: str = "",
        request_target: str = "",
    ) -> None:
        self.failed_requests += 1
        self.last_transport_error = f"{type(error).__name__}: {error}"
        if surface_ids:
            _record_surface_receipt(
                self.surface_receipts,
                surface_ids=surface_ids,
                coverage_status="transport_error",
                request_method=request_method,
                request_target=request_target,
                response_status=None,
            )

    def result_payload(self) -> dict[str, Any]:
        failure_class = "transport" if self.last_transport_error else None
        return {
            "schema_version": "adversary-sim-scrapling-worker-result.v1",
            "run_id": self.run_id,
            "tick_id": self.tick_id,
            "lane": self.lane,
            "fulfillment_mode": self.fulfillment_mode,
            "category_targets": _normalize_category_targets(self.plan.get("category_targets")),
            "worker_id": socket.gethostname(),
            "tick_started_at": int(self.plan.get("tick_started_at") or int(time.time())),
            "tick_completed_at": int(time.time()),
            "generated_requests": self.generated_requests,
            "failed_requests": self.failed_requests,
            "last_response_status": self.last_response_status,
            "failure_class": failure_class,
            "error": self.last_transport_error,
            "crawl_stats": {
                "requests_count": self.generated_requests,
                "offsite_requests_count": 0,
                "blocked_requests_count": 0,
                "response_status_count": dict(self.response_status_count),
                "response_bytes": self.bytes_observed,
            },
            "realism_receipt": self.realism_tracker.render_receipt(
                bytes_observed=self.bytes_observed,
                deadline_reached=time.monotonic() >= self.deadline,
                activity_sequence_exhausted=not self.should_stop(),
                transport_failure=bool(self.last_transport_error),
            ),
            "scope_rejections": dict(sorted(self.scope_rejections.items())),
            "surface_receipts": _render_surface_receipts(self.surface_receipts),
        }


def _execute_request_sequence(
    session: Any,
    *,
    tracker: _DirectPersonaTracker,
    base_url: str,
    requests: list[dict[str, Any]],
) -> None:
    for request_spec in requests:
        if tracker.should_stop():
            break
        _perform_request(
            session,
            tracker=tracker,
            base_url=base_url,
            request_spec=request_spec,
        )


def _perform_request(
    session: Any,
    *,
    tracker: _DirectPersonaTracker,
    base_url: str,
    request_spec: dict[str, Any],
) -> Any | None:
    method_name = str(request_spec.get("method") or "").strip().lower()
    raw_target = str(request_spec.get("target") or "").strip()
    surface_ids = [str(value) for value in list(request_spec.get("surface_ids") or []) if str(value).strip()]
    if not method_name or not raw_target:
        return None
    allowed, normalized_url = tracker.allowed_request(
        base_url,
        raw_target,
        is_redirect=False,
    )
    if not allowed or not normalized_url:
        return None
    try:
        response = getattr(session, method_name)(
            normalized_url,
            headers=tracker.next_headers(dict(request_spec.get("headers") or {})),
            cookies=request_spec.get("cookies"),
            data=request_spec.get("data"),
            json=request_spec.get("json"),
            follow_redirects=False,
        )
        tracker.record_response(
            response,
            surface_ids,
            request_method=method_name,
            request_target=normalized_url,
        )
        location = str(response.headers.get("location") or "").strip()
        if (
            request_spec.get("follow_redirect")
            and 300 <= int(response.status) < 400
            and location
            and not tracker.should_stop()
        ):
            allowed, redirect_url = tracker.allowed_request(
                response.url,
                location,
                is_redirect=True,
            )
            if allowed and redirect_url:
                redirect_response = session.get(
                    redirect_url,
                    headers=tracker.next_headers({"accept": "application/json"}),
                    cookies=request_spec.get("cookies"),
                    follow_redirects=False,
                )
                tracker.record_response(
                    redirect_response,
                    surface_ids,
                    request_method="get",
                    request_target=redirect_url,
                )
                return redirect_response
        return response
    except Exception as exc:
        tracker.record_failure(
            exc,
            surface_ids=surface_ids,
            request_method=method_name,
            request_target=normalized_url or raw_target,
        )
        return None


def _browser_root_discovery_page_action(state: dict[str, Any]):
    def action(page) -> None:
        state["links"] = page.evaluate(
            """() => Array.from(document.querySelectorAll('a[href]'))
                .map((anchor) => anchor.getAttribute('href') || '')
            """
        )

    return action


def _browser_discovered_target(
    root_state: dict[str, Any],
    *,
    base_url: str,
    predicate,
) -> str | None:
    candidates = _normalize_browser_discovery_targets(
        base_url,
        list(root_state.get("links") or []),
    )
    return _first_matching_target(candidates, predicate)


def _browser_captured_xhr_paths(response: Any, *, base_url: str) -> list[str]:
    base_parts = urlsplit(str(base_url or ""))
    base_origin = (base_parts.scheme.lower(), base_parts.netloc.lower())
    paths: list[str] = []
    for captured in list(getattr(response, "captured_xhr", []) or []):
        raw_url = str(getattr(captured, "url", "") or "").strip()
        if not raw_url:
            continue
        resolved = urlsplit(urljoin(base_url, raw_url))
        candidate_origin = (resolved.scheme.lower(), resolved.netloc.lower())
        if candidate_origin != base_origin:
            continue
        request_path = _request_path_value(raw_url)
        if request_path and request_path not in paths:
            paths.append(request_path)
    return paths


def _execute_bulk_scraper_persona(
    fetcher_session_cls: Any,
    *,
    plan: dict[str, Any],
    descriptor: shared_host_scope.SharedHostScopeDescriptor,
    seed_inventory: dict[str, Any],
    sim_telemetry_secret: str,
) -> dict[str, Any]:
    tracker = _DirectPersonaTracker(
        plan=plan,
        descriptor=descriptor,
        sim_telemetry_secret=sim_telemetry_secret,
    )
    surface_targets = set(_normalize_surface_targets(plan.get("surface_targets")))
    request_proxy_url = _normalize_optional_proxy_url(plan.get("request_proxy_url"))
    request_identity_pool = normalize_identity_pool_entries(
        plan.get("request_identity_pool"),
        field_name="worker_plan.request_identity_pool",
    )
    start_urls = _normalized_start_urls(seed_inventory)
    if not start_urls:
        raise WorkerConfigError("seed inventory must contain at least one accepted start or hint URL")
    base_url = start_urls[0]
    visited: set[str] = set()
    public_candidates: list[tuple[str, int]] = []
    challenge_page: str | None = None

    def note_discovery(response: Any, *, current_depth: int) -> None:
        nonlocal challenge_page, public_candidates
        current_url = str(getattr(response, "url", "") or "").strip()
        tracker.realism_tracker.observe_exploration_visit(
            target=current_url,
            depth=current_depth,
            content_type=_response_content_type(response),
        )
        discovered_targets: list[str] = []
        for candidate in _response_anchor_targets(response):
            allowed, normalized_url = tracker.allowed_request(
                current_url,
                candidate,
                is_redirect=False,
                record_rejection=False,
            )
            if not allowed or not normalized_url:
                continue
            tracker.realism_tracker.observe_discovered_target(normalized_url)
            discovered_targets.append(normalized_url)
        if challenge_page is None:
            challenge_page = _first_matching_target(
                discovered_targets,
                lambda candidate: _path_contains(candidate, "not-a-bot"),
            )
        next_depth = current_depth + 1
        for candidate in discovered_targets:
            if _looks_bulk_scraper_public_target(candidate):
                public_candidates.append((candidate, next_depth))
        unique_candidates: list[tuple[str, int]] = []
        seen_targets: set[str] = set()
        for candidate, depth in public_candidates:
            if candidate in visited or candidate in seen_targets:
                continue
            seen_targets.add(candidate)
            unique_candidates.append((candidate, depth))
        public_candidates = sorted(
            unique_candidates,
            key=lambda item: _bulk_scraper_priority(item[0]),
        )

    with _PacedRequestNativeSession(
        fetcher_session_cls,
        tracker=tracker,
        timeout_seconds=max(1.0, min(30.0, tracker.max_ms / 1000.0)),
        accept_header="text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        proxy_url=request_proxy_url,
        request_transport=resolve_request_transport_observation(
            tracker.realism_tracker.profile
        ),
        identity_pool=request_identity_pool,
    ) as session:
        tracker.realism_tracker.observe_discovered_target(base_url)
        root_surface_ids = ["public_path_traversal", *_public_discovery_surface_ids(surface_targets)]
        root_response = _perform_request(
            session,
            tracker=tracker,
            base_url=base_url,
            request_spec=_request_spec(
                "get",
                base_url,
                surface_ids=root_surface_ids,
            ),
        )
        if root_response is not None:
            visited.add(str(getattr(root_response, "url", base_url)))
            note_discovery(root_response, current_depth=0)

        while public_candidates and not tracker.should_stop():
            candidate, candidate_depth = public_candidates.pop(0)
            if candidate in visited:
                continue
            response = _perform_request(
                session,
                tracker=tracker,
                base_url=base_url,
                request_spec=_request_spec(
                    "get",
                    candidate,
                    surface_ids=["public_path_traversal"],
                ),
            )
            if response is None:
                continue
            visited.add(str(getattr(response, "url", candidate)))
            note_discovery(response, current_depth=candidate_depth)

        if not tracker.should_stop() and challenge_page:
            challenge_response = _perform_request(
                session,
                tracker=tracker,
                base_url=base_url,
                request_spec=_request_spec(
                    "get",
                    challenge_page,
                ),
            )
            form_targets = _response_form_targets(challenge_response) if challenge_response is not None else []
            not_a_bot_target = _first_matching_target(
                form_targets,
                lambda candidate: _path_contains(candidate, "not-a-bot"),
            )
            puzzle_target = _first_matching_target(
                form_targets,
                lambda candidate: _path_contains(candidate, "puzzle"),
            )
            if "not_a_bot_submit" in surface_targets and not tracker.should_stop() and not_a_bot_target:
                _perform_request(
                    session,
                    tracker=tracker,
                    base_url=base_url,
                    request_spec=_request_spec(
                        "post",
                        not_a_bot_target,
                        surface_ids=["not_a_bot_submit"],
                        headers={
                            "accept": "application/json",
                            "content-type": "application/x-www-form-urlencoded",
                        },
                        data=_invalid_not_a_bot_body(),
                    ),
                )
            if (
                "puzzle_submit_or_escalation" in surface_targets
                and not tracker.should_stop()
                and puzzle_target
            ):
                _perform_request(
                    session,
                    tracker=tracker,
                    base_url=base_url,
                    request_spec=_request_spec(
                        "post",
                        puzzle_target,
                        surface_ids=["puzzle_submit_or_escalation"],
                        headers={
                            "accept": "application/json",
                            "content-type": "application/x-www-form-urlencoded",
                        },
                        data="answer=bad&seed=invalid&return_to=%2F",
                    ),
                )
    return tracker.result_payload()


def _execute_http_agent_persona(
    fetcher_session_cls: Any,
    *,
    plan: dict[str, Any],
    descriptor: shared_host_scope.SharedHostScopeDescriptor,
    seed_inventory: dict[str, Any],
    sim_telemetry_secret: str,
) -> dict[str, Any]:
    tracker = _DirectPersonaTracker(
        plan=plan,
        descriptor=descriptor,
        sim_telemetry_secret=sim_telemetry_secret,
    )
    surface_targets = set(_normalize_surface_targets(plan.get("surface_targets")))
    request_proxy_url = _normalize_optional_proxy_url(plan.get("request_proxy_url"))
    request_identity_pool = normalize_identity_pool_entries(
        plan.get("request_identity_pool"),
        field_name="worker_plan.request_identity_pool",
    )
    start_urls = _normalized_start_urls(seed_inventory)
    if not start_urls:
        raise WorkerConfigError("seed inventory must contain at least one accepted start or hint URL")
    base_url = start_urls[0]
    with _PacedRequestNativeSession(
        fetcher_session_cls,
        tracker=tracker,
        timeout_seconds=max(1.0, min(30.0, tracker.max_ms / 1000.0)),
        accept_header="application/json",
        proxy_url=request_proxy_url,
        request_transport=resolve_request_transport_observation(
            tracker.realism_tracker.profile
        ),
        identity_pool=request_identity_pool,
    ) as session:
        root_response = _perform_request(
            session,
            tracker=tracker,
            base_url=base_url,
            request_spec=_request_spec(
                "get",
                base_url,
                surface_ids=_public_discovery_surface_ids(surface_targets),
            ),
        )
        root_links = _response_anchor_targets(root_response) if root_response is not None else []

        challenge_target = _first_matching_target(
            root_links,
            lambda candidate: _path_contains(candidate, "not-a-bot"),
        )
        pow_target = _first_matching_target(
            root_links,
            lambda candidate: _request_path_value(candidate).lower() == "/pow",
        )
        redirect_target = _first_matching_target(
            root_links,
            lambda candidate: _path_contains(candidate, "redirect"),
        )

        if challenge_target and not tracker.should_stop():
            challenge_response = _perform_request(
                session,
                tracker=tracker,
                base_url=base_url,
                request_spec=_request_spec(
                    "get",
                    challenge_target,
                ),
            )
            challenge_forms = _response_form_targets(challenge_response) if challenge_response is not None else []
            not_a_bot_target = _first_matching_target(
                challenge_forms,
                lambda candidate: _path_contains(candidate, "not-a-bot"),
            )
            puzzle_target = _first_matching_target(
                challenge_forms,
                lambda candidate: _path_contains(candidate, "puzzle"),
            )
            if "not_a_bot_submit" in surface_targets and not tracker.should_stop() and not_a_bot_target:
                _perform_request(
                    session,
                    tracker=tracker,
                    base_url=base_url,
                    request_spec=_request_spec(
                        "post",
                        not_a_bot_target,
                        surface_ids=["not_a_bot_submit"],
                        headers={
                            "accept": "application/json",
                            "content-type": "application/x-www-form-urlencoded",
                        },
                        data=_invalid_not_a_bot_body(),
                    ),
                )
            if (
                "puzzle_submit_or_escalation" in surface_targets
                and not tracker.should_stop()
                and puzzle_target
            ):
                _perform_request(
                    session,
                    tracker=tracker,
                    base_url=base_url,
                    request_spec=_request_spec(
                        "post",
                        puzzle_target,
                        surface_ids=["puzzle_submit_or_escalation"],
                        headers={
                            "accept": "application/json",
                            "content-type": "application/x-www-form-urlencoded",
                        },
                        data="answer=bad&seed=invalid&return_to=%2F",
                    ),
                )

        if pow_target and not tracker.should_stop():
            pow_response = _perform_request(
                session,
                tracker=tracker,
                base_url=base_url,
                request_spec=_request_spec(
                    "get",
                    pow_target,
                ),
            )
            pow_forms = _response_form_targets(pow_response) if pow_response is not None else []
            pow_verify_target = _first_matching_target(
                pow_forms,
                lambda candidate: _path_contains(candidate, "/pow/verify"),
            )
            tarpit_target = _first_matching_target(
                pow_forms,
                lambda candidate: _path_contains(candidate, "/tarpit/progress"),
            )
            if "pow_verify_abuse" in surface_targets and not tracker.should_stop() and pow_verify_target:
                _perform_request(
                    session,
                    tracker=tracker,
                    base_url=base_url,
                    request_spec=_request_spec(
                        "post",
                        pow_verify_target,
                        surface_ids=["pow_verify_abuse"],
                        headers={
                            "accept": "application/json",
                            "content-type": "application/json",
                        },
                        json_body={"seed": "invalid-seed", "nonce": "invalid-nonce"},
                    ),
                )
            if "tarpit_progress_abuse" in surface_targets and not tracker.should_stop() and tarpit_target:
                _perform_request(
                    session,
                    tracker=tracker,
                    base_url=base_url,
                    request_spec=_request_spec(
                        "post",
                        tarpit_target,
                        surface_ids=["tarpit_progress_abuse"],
                        headers={
                            "accept": "application/json",
                            "content-type": "application/json",
                        },
                        json_body={
                            "token": "invalid",
                            "operation_id": "invalid",
                            "proof_nonce": "invalid",
                        },
                    ),
                )

        if redirect_target and not tracker.should_stop():
            _perform_request(
                session,
                tracker=tracker,
                base_url=base_url,
                request_spec=_request_spec(
                    "get",
                    redirect_target,
                    headers={"accept": "application/json"},
                    follow_redirect=True,
                ),
            )
    return tracker.result_payload()


def _record_browser_surface_result(
    tracker: _DirectPersonaTracker,
    *,
    surface_id: str,
    coverage_status: str,
    request_target: str,
    response_status: int | None,
) -> None:
    _record_surface_receipt(
        tracker.surface_receipts,
        surface_ids=[surface_id],
        coverage_status=coverage_status,
        request_method="GET",
        request_target=request_target,
        response_status=response_status,
    )


def _pow_surface_page_action(state: dict[str, Any]):
    def action(page) -> None:
        state["pow_details"] = page.evaluate(
            """async () => {
                const hasCheck = typeof window._checkCDPAutomation === "function";
                let detected = null;
                if (hasCheck) {
                    try {
                        const result = await window._checkCDPAutomation();
                        if (typeof result === "boolean") {
                            detected = result;
                        } else if (result && typeof result.detected === "boolean") {
                            detected = result.detected;
                        }
                    } catch (_error) {
                        detected = null;
                    }
                }
                return {
                    has_check: hasCheck,
                    detected: detected,
                    cookie: document.cookie || "",
                    body_text: (document.body && document.body.innerText) || "",
                    location: window.location.pathname + window.location.search,
                };
            }"""
        )

    return action


def _maze_navigation_page_action(state: dict[str, Any]):
    def action(page) -> None:
        state["before_url"] = page.url
        locator = page.locator("[data-link-kind='maze']").first
        state["link_count"] = locator.count()
        if int(state["link_count"] or 0) < 1:
            return
        locator.click()
        page.wait_for_load_state("networkidle")
        try:
            bootstrap = page.locator("#maze-bootstrap").text_content()
        except Exception as exc:  # pragma: no cover - real page variance is receipted below.
            state["bootstrap_error"] = str(exc)
            bootstrap = ""
        state["bootstrap"] = bootstrap or ""
        state["after_url"] = page.url

    return action


def _execute_browser_persona(
    browser_session_cls: Any,
    *,
    plan: dict[str, Any],
    descriptor: shared_host_scope.SharedHostScopeDescriptor,
    seed_inventory: dict[str, Any],
    sim_telemetry_secret: str,
) -> dict[str, Any]:
    tracker = _DirectPersonaTracker(
        plan=plan,
        descriptor=descriptor,
        sim_telemetry_secret=sim_telemetry_secret,
    )
    surface_targets = set(_normalize_surface_targets(plan.get("surface_targets")))
    browser_identity_pool = normalize_identity_pool_entries(
        plan.get("browser_identity_pool"),
        field_name="worker_plan.browser_identity_pool",
    )
    browser_proxy_url = _normalize_optional_proxy_url(plan.get("browser_proxy_url")) or _normalize_optional_proxy_url(
        plan.get("request_proxy_url")
    )
    if browser_identity_pool:
        browser_proxy_url = (
            str(browser_identity_pool[0].get("proxy_url") or "").strip()
            or browser_proxy_url
        )
    browser_country_code = (
        str(browser_identity_pool[0].get("country_code") or "").strip().upper()
        if browser_identity_pool
        else None
    )
    browser_transport = resolve_browser_transport_observation(
        tracker.realism_tracker.profile,
        country_code=browser_country_code,
    )
    tracker.realism_tracker.observe_transport(
        transport_profile=str(browser_transport.get("transport_profile") or ""),
        user_agent_family=str(browser_transport.get("user_agent_family") or ""),
        accept_language=str(browser_transport.get("accept_language") or ""),
        browser_locale=str(browser_transport.get("browser_locale") or ""),
    )
    start_urls = _normalized_start_urls(seed_inventory)
    if not start_urls:
        raise WorkerConfigError("seed inventory must contain at least one accepted start or hint URL")
    base_url = start_urls[0]
    timeout_ms = max(1000, tracker.max_ms)

    with browser_session_cls(
        **_browser_session_kwargs(
            fulfillment_mode=tracker.fulfillment_mode,
            timeout_ms=timeout_ms,
            locale=str(browser_transport.get("browser_locale") or "en-US"),
            useragent=str(browser_transport.get("user_agent") or ""),
            proxy_url=browser_proxy_url,
        ),
    ) as session:
        root_state: dict[str, Any] = {}
        root_target = base_url
        browser_session_handle = (
            f"browser-session-{str(browser_identity_pool[0].get('label') or '').strip()}"
            if browser_identity_pool
            else "browser-session-1"
        )
        try:
            tracker.realism_tracker.prepare_browser_action(
                browser_session_handle,
                remaining_ms=tracker.remaining_ms(),
                country_code=browser_country_code,
            )
            root_response = session.fetch(
                root_target,
                extra_headers=tracker.next_headers(
                    {"accept-language": str(browser_transport.get("accept_language") or "")}
                ),
                page_action=_browser_root_discovery_page_action(root_state),
            )
            tracker.record_response(
                root_response,
                _public_discovery_surface_ids(surface_targets),
                request_method="get",
                request_target=root_target,
            )
            tracker.realism_tracker.observe_browser_secondary_traffic(
                capture_mode="xhr_capture",
                background_paths=_browser_captured_xhr_paths(
                    root_response,
                    base_url=base_url,
                ),
            )
        except Exception as exc:
            tracker.record_failure(
                exc,
                surface_ids=_public_discovery_surface_ids(surface_targets),
                request_method="get",
                request_target=root_target,
            )
            return tracker.result_payload()

        if (
            {"js_verification_execution", "browser_automation_detection"} & surface_targets
            and not tracker.should_stop()
        ):
            pow_target = _browser_discovered_target(
                root_state,
                base_url=base_url,
                predicate=lambda candidate: _request_path_value(candidate).lower() == "/pow",
            )
            if pow_target is None:
                return tracker.result_payload()
            pow_state: dict[str, Any] = {}
            try:
                tracker.realism_tracker.prepare_browser_action(
                    browser_session_handle,
                    remaining_ms=tracker.remaining_ms(),
                    country_code=browser_country_code,
                )
                response = session.fetch(
                    pow_target,
                    extra_headers=tracker.next_headers(
                        {"accept-language": str(browser_transport.get("accept_language") or "")}
                    ),
                    page_action=_pow_surface_page_action(pow_state),
                )
                tracker.record_response(
                    response,
                    request_method="get",
                    request_target=pow_target,
                )
                tracker.realism_tracker.observe_browser_secondary_traffic(
                    capture_mode="xhr_capture",
                    background_paths=_browser_captured_xhr_paths(
                        response,
                        base_url=base_url,
                    ),
                )
                response_status = int(response.status)
                pow_details = (
                    pow_state.get("pow_details")
                    if isinstance(pow_state.get("pow_details"), dict)
                    else {}
                )
                pow_cookie = str(pow_details.get("cookie") or "")
                pow_body_text = str(pow_details.get("body_text") or "")
                js_executed = bool(pow_details.get("has_check")) or "js_verified=" in pow_cookie or "Verifying" in pow_body_text
                if "js_verification_execution" in surface_targets:
                    _record_browser_surface_result(
                        tracker,
                        surface_id="js_verification_execution",
                        coverage_status="pass_observed" if js_executed else "fail_observed",
                        request_target=pow_target,
                        response_status=response_status,
                    )
                if "browser_automation_detection" in surface_targets:
                    detected = pow_details.get("detected")
                    detection_status = (
                        "fail_observed"
                        if detected is True
                        else "pass_observed"
                        if detected is False
                        else "fail_observed"
                        if js_executed
                        else "transport_error"
                    )
                    _record_browser_surface_result(
                        tracker,
                        surface_id="browser_automation_detection",
                        coverage_status=detection_status,
                        request_target=pow_target,
                        response_status=response_status,
                    )
            except Exception as exc:
                tracker.record_failure(
                    exc,
                    surface_ids=[
                        surface_id
                        for surface_id in (
                            "js_verification_execution",
                            "browser_automation_detection",
                        )
                        if surface_id in surface_targets
                    ],
                    request_method="get",
                    request_target=pow_target,
                )
                return tracker.result_payload()

        if "maze_navigation" in surface_targets and not tracker.should_stop():
            maze_target = _browser_discovered_target(
                root_state,
                base_url=base_url,
                predicate=lambda candidate: _path_contains(candidate, "/maze/"),
            )
            if maze_target is None:
                return tracker.result_payload()
            maze_state: dict[str, Any] = {}
            try:
                tracker.realism_tracker.prepare_browser_action(
                    browser_session_handle,
                    remaining_ms=tracker.remaining_ms(),
                    country_code=browser_country_code,
                )
                response = session.fetch(
                    maze_target,
                    extra_headers=tracker.next_headers(
                        {"accept-language": str(browser_transport.get("accept_language") or "")}
                    ),
                    page_action=_maze_navigation_page_action(maze_state),
                )
                tracker.record_response(
                    response,
                    request_method="get",
                    request_target=maze_target,
                )
                tracker.realism_tracker.observe_browser_secondary_traffic(
                    capture_mode="xhr_capture",
                    background_paths=_browser_captured_xhr_paths(
                        response,
                        base_url=base_url,
                    ),
                )
                after_url = str(maze_state.get("after_url") or "")
                before_url = str(maze_state.get("before_url") or "")
                maze_passed = (
                    int(maze_state.get("link_count") or 0) > 0
                    and bool(after_url)
                    and after_url != before_url
                )
                _record_browser_surface_result(
                    tracker,
                    surface_id="maze_navigation",
                    coverage_status="pass_observed" if maze_passed else "fail_observed",
                    request_target=maze_target,
                    response_status=int(response.status),
                )
            except Exception as exc:
                tracker.record_failure(
                    exc,
                    surface_ids=["maze_navigation"],
                    request_method="get",
                    request_target=maze_target,
                )

    return tracker.result_payload()


def execute_worker_plan(
    beat_payload: dict[str, Any],
    *,
    scope_descriptor_path: Path,
    seed_inventory_path: Path,
    crawldir: Path,
    sim_telemetry_secret: str,
) -> dict[str, Any]:
    try:
        plan = beat_payload.get("worker_plan")
        if not isinstance(plan, dict):
            raise WorkerConfigError("worker_plan object is required")
        if str(plan.get("schema_version") or "").strip() != "adversary-sim-scrapling-worker-plan.v1":
            raise WorkerConfigError("worker_plan schema_version must be adversary-sim-scrapling-worker-plan.v1")
        if str(plan.get("lane") or "").strip() != "scrapling_traffic":
            raise WorkerConfigError("worker_plan lane must be scrapling_traffic")
        fulfillment_mode = str(plan.get("fulfillment_mode") or "").strip()
        if fulfillment_mode not in SCRAPLING_FULFILLMENT_MODES:
            raise WorkerConfigError(
                "worker_plan fulfillment_mode must be one of crawler, bulk_scraper, browser_automation, stealth_browser, http_agent"
            )
        category_targets = _normalize_category_targets(plan.get("category_targets"))
        expected_targets = _expected_category_targets_for_mode(fulfillment_mode)
        if category_targets != expected_targets:
            raise WorkerConfigError(
                "worker_plan category_targets must match the bounded fulfillment_mode mapping"
            )
        surface_targets = _normalize_surface_targets(plan.get("surface_targets"))
        expected_surface_targets = _expected_surface_targets_for_mode(fulfillment_mode)
        if surface_targets != expected_surface_targets:
            raise WorkerConfigError(
                "worker_plan surface_targets must match the bounded fulfillment_mode mapping"
            )
        realism_profile = normalize_lane_realism_profile(
            plan.get("realism_profile"),
            field_name="worker_plan.realism_profile",
        )
        expected_realism_profile = resolve_lane_realism_profile(
            "scrapling_traffic",
            fulfillment_mode,
        )
        if realism_profile != expected_realism_profile:
            raise WorkerConfigError(
                "worker_plan realism_profile must match the canonical lane realism contract"
            )
        if not sim_telemetry_secret.strip():
            raise WorkerConfigError("SHUMA_SIM_TELEMETRY_SECRET is required for Scrapling worker tagging")

        descriptor_payload = _load_json(scope_descriptor_path)
        descriptor = shared_host_scope.descriptor_from_payload(descriptor_payload)
        seed_inventory = _load_json(seed_inventory_path)
        if not _normalized_start_urls(seed_inventory):
            raise WorkerConfigError("seed inventory must contain at least one accepted start or hint URL")

        dynamic_session_cls, fetcher_session_cls, stealthy_session_cls, request_cls, spider_cls = _import_scrapling()
        if fulfillment_mode == "crawler":
            spider_class = _build_spider_class(fetcher_session_cls, request_cls, spider_cls)
            crawldir.mkdir(parents=True, exist_ok=True)
            spider = spider_class(
                plan=plan,
                descriptor=descriptor,
                seed_inventory=seed_inventory,
                crawldir=crawldir,
                sim_telemetry_secret=sim_telemetry_secret,
            )
            crawl_result = spider.start()
            stats = crawl_result.stats
            failure_class = None
            error = None
            if spider.last_transport_error:
                failure_class = "transport"
                error = spider.last_transport_error
            return {
                "schema_version": "adversary-sim-scrapling-worker-result.v1",
                "run_id": str(plan.get("run_id") or ""),
                "tick_id": str(plan.get("tick_id") or ""),
                "lane": "scrapling_traffic",
                "fulfillment_mode": fulfillment_mode,
                "category_targets": category_targets,
                "worker_id": socket.gethostname(),
                "tick_started_at": int(plan.get("tick_started_at") or int(time.time())),
                "tick_completed_at": int(time.time()),
                "generated_requests": int(stats.requests_count),
                "failed_requests": int(stats.failed_requests_count),
                "last_response_status": spider.last_response_status,
                "failure_class": failure_class,
                "error": error,
                "crawl_stats": {
                    "requests_count": int(stats.requests_count),
                    "offsite_requests_count": int(stats.offsite_requests_count),
                    "blocked_requests_count": int(stats.blocked_requests_count),
                    "response_status_count": dict(stats.response_status_count),
                    "response_bytes": int(stats.response_bytes),
                },
                "realism_receipt": spider.realism_tracker.render_receipt(
                    bytes_observed=spider.bytes_observed,
                    deadline_reached=time.monotonic() >= spider.deadline,
                    activity_sequence_exhausted=bool(
                        spider.activity_sequence_exhausted
                    ),
                    transport_failure=bool(spider.last_transport_error),
                ),
                "scope_rejections": dict(sorted(spider.scope_rejections.items())),
                "surface_receipts": _render_surface_receipts(spider.surface_receipts),
            }
        if fulfillment_mode == "bulk_scraper":
            return _execute_bulk_scraper_persona(
                fetcher_session_cls,
                plan=plan,
                descriptor=descriptor,
                seed_inventory=seed_inventory,
                sim_telemetry_secret=sim_telemetry_secret,
            )
        if fulfillment_mode == "browser_automation":
            return _execute_browser_persona(
                dynamic_session_cls,
                plan=plan,
                descriptor=descriptor,
                seed_inventory=seed_inventory,
                sim_telemetry_secret=sim_telemetry_secret,
            )
        if fulfillment_mode == "stealth_browser":
            return _execute_browser_persona(
                stealthy_session_cls,
                plan=plan,
                descriptor=descriptor,
                seed_inventory=seed_inventory,
                sim_telemetry_secret=sim_telemetry_secret,
            )
        return _execute_http_agent_persona(
            fetcher_session_cls,
            plan=plan,
            descriptor=descriptor,
            seed_inventory=seed_inventory,
            sim_telemetry_secret=sim_telemetry_secret,
        )
    except Exception as exc:
        return _build_failure_result(
            beat_payload,
            failure_class="transport",
            error=str(exc),
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Execute one bounded Scrapling worker plan.")
    parser.add_argument("--beat-response-file", required=True, help="Beat response JSON file")
    parser.add_argument("--result-output-file", help="Write result JSON to this file")
    parser.add_argument("--scope-descriptor", help="Shared-host scope descriptor JSON path")
    parser.add_argument("--seed-inventory", help="Shared-host seed inventory JSON path")
    parser.add_argument("--crawldir", help="Persistent Scrapling crawldir path")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    beat_payload = _load_json(Path(args.beat_response_file))
    scope_descriptor = _env_or_arg(
        args.scope_descriptor,
        "ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH",
    )
    seed_inventory = _env_or_arg(
        args.seed_inventory,
        "ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH",
    )
    crawldir = _env_or_arg(
        args.crawldir,
        "ADVERSARY_SIM_SCRAPLING_CRAWLDIR",
    )
    if not scope_descriptor or not seed_inventory or not crawldir:
        result = _build_failure_result(
            beat_payload,
            failure_class="transport",
            error=(
                "scope descriptor, seed inventory, and crawldir must be provided via "
                "arguments or ADVERSARY_SIM_SCRAPLING_* environment variables"
            ),
        )
    else:
        result = execute_worker_plan(
            beat_payload,
            scope_descriptor_path=Path(scope_descriptor),
            seed_inventory_path=Path(seed_inventory),
            crawldir=Path(crawldir),
            sim_telemetry_secret=os.environ.get("SHUMA_SIM_TELEMETRY_SECRET", ""),
        )

    rendered = json.dumps(result, separators=(",", ":"), sort_keys=True)
    if args.result_output_file:
        output_path = Path(args.result_output_file)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(rendered, encoding="utf-8")
    else:
        print(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
