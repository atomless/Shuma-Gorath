#!/usr/bin/env python3
"""Real Scrapling worker for the adversary-sim Scrapling lane."""

from __future__ import annotations

import argparse
from collections import Counter
from collections.abc import AsyncGenerator
from html.parser import HTMLParser
import ipaddress
import json
import os
from pathlib import Path
import socket
import sys
import time
from typing import Any, Callable
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
        self.transport_realism_class = ""
        self.transport_emission_basis = ""
        self.transport_degraded_reason = ""
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

    def remaining_activity_budget(self) -> int:
        return max(0, self.effective_activity_budget - self.activity_count)

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
        transport_realism_class: str,
        transport_emission_basis: str,
        transport_degraded_reason: str,
        user_agent_family: str,
        accept_language: str,
        browser_locale: str | None = None,
    ) -> None:
        normalized_transport = str(transport_profile or "").strip()
        if normalized_transport:
            self.transport_profile = normalized_transport
        normalized_class = str(transport_realism_class or "").strip()
        if normalized_class:
            self.transport_realism_class = normalized_class
        normalized_basis = str(transport_emission_basis or "").strip()
        if normalized_basis:
            self.transport_emission_basis = normalized_basis
        normalized_degraded_reason = str(transport_degraded_reason or "").strip()
        self.transport_degraded_reason = normalized_degraded_reason
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
            "transport_realism_class": self.transport_realism_class,
            "transport_emission_basis": self.transport_emission_basis,
            "transport_degraded_reason": self.transport_degraded_reason,
            "observed_user_agent_families": list(self.observed_user_agent_families),
            "observed_accept_languages": list(self.observed_accept_languages),
            "identity_realism_status": identity_summary["identity_realism_status"],
            "identity_provenance_mode": identity_summary["identity_provenance_mode"],
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
            "reentry_scope": str(self.recurrence_context.get("reentry_scope") or ""),
            "dormancy_truth_mode": str(
                self.recurrence_context.get("dormancy_truth_mode") or ""
            ),
            "session_index": int(self.recurrence_context.get("session_index") or 0),
            "reentry_count": int(self.recurrence_context.get("reentry_count") or 0),
            "max_reentries_per_run": int(
                self.recurrence_context.get("max_reentries_per_run") or 0
            ),
            "planned_dormant_gap_seconds": int(
                self.recurrence_context.get("planned_dormant_gap_seconds") or 0
            ),
            "representative_dormant_gap_seconds": int(
                self.recurrence_context.get("representative_dormant_gap_seconds") or 0
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
                    "action_types_attempted": ["browser_navigate"],
                    "capability_state": "native_persona",
                    "targeting_strategy": "surface_probe",
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
        self._explicit_client_ip = (
            str(self.tracker.plan.get("local_request_client_ip") or "").strip() or None
        )
        self.tracker.local_trusted_forwarding_headers = (
            self._initial_local_trusted_forwarding_headers()
        )

    def __enter__(self) -> "_PacedRequestNativeSession":
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        if self._session_cm is not None:
            self._session_cm.__exit__(exc_type, exc, tb)
        self._session_cm = None
        self._session = None

    def _initial_local_trusted_forwarding_headers(self) -> dict[str, str]:
        proxy_url = self.proxy_url
        country_code: str | None = None
        if self.identity_pool:
            first_entry = dict(self.identity_pool[0])
            proxy_url = str(first_entry.get("proxy_url") or "").strip() or None
            country_code = str(first_entry.get("country_code") or "").strip().upper() or None
        forwarded_country_code = country_code or _local_fallback_country_code(
            self.tracker.fulfillment_mode
        )
        return _resolved_local_trusted_forwarding_headers(
            proxy_url,
            forwarded_country_code,
            self._explicit_client_ip,
        )

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
        forwarded_country_code = current_country_code or _local_fallback_country_code(
            self.tracker.fulfillment_mode
        )
        self._current_country_code = forwarded_country_code
        self.tracker.local_trusted_forwarding_headers = _resolved_local_trusted_forwarding_headers(
            current_proxy_url,
            forwarded_country_code,
            self._explicit_client_ip,
        )
        self.tracker.realism_tracker.observe_transport(
            transport_profile=str(request_transport.get("transport_profile") or ""),
            transport_realism_class=str(
                request_transport.get("transport_realism_class") or ""
            ),
            transport_emission_basis=str(
                request_transport.get("transport_emission_basis") or ""
            ),
            transport_degraded_reason=str(
                request_transport.get("transport_degraded_reason") or ""
            ),
            user_agent_family=str(request_transport.get("user_agent_family") or ""),
            accept_language=str(request_transport.get("accept_language") or ""),
        )
        if self._session_index > 1:
            self.tracker.realism_tracker.note_rotation()

    def fetch(self, target: str, **kwargs) -> Any:
        allow_deadline_overrun = bool(kwargs.pop("allow_deadline_overrun", False))
        if not self.tracker.can_start_request(
            allow_deadline_overrun=allow_deadline_overrun
        ):
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
        allow_deadline_overrun = bool(kwargs.pop("allow_deadline_overrun", False))
        if not self.tracker.can_start_request(
            allow_deadline_overrun=allow_deadline_overrun
        ):
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


def _env_flag_enabled(name: str) -> bool:
    return str(os.environ.get(name, "")).strip().lower() in {"1", "true", "yes", "on"}


def _parse_ip_literal(raw_value: str | None) -> str | None:
    value = str(raw_value or "").strip()
    if not value:
        return None
    try:
        return str(ipaddress.ip_address(value))
    except ValueError:
        return None


def _host_is_loopback(raw_host: str | None) -> bool:
    host = str(raw_host or "").strip()
    if not host:
        return False
    if host.lower() == "localhost":
        return True
    try:
        return ipaddress.ip_address(host).is_loopback
    except ValueError:
        return False


def _normalize_country_code(value: str | None) -> str:
    candidate = str(value or "").strip().upper()
    if len(candidate) != 2 or not candidate.isalpha():
        return ""
    return candidate


def _local_fallback_country_code(fulfillment_mode: str) -> str | None:
    normalized_mode = str(fulfillment_mode or "").strip()
    fallback_by_mode = {
        "crawler": "RU",
        "bulk_scraper": "BR",
        "http_agent": "DE",
        "browser_automation": "DE",
        "stealth_browser": "DE",
    }
    return fallback_by_mode.get(normalized_mode)


def _local_contributor_client_ip(
    proxy_url: str | None,
    explicit_client_ip: str | None = None,
) -> str | None:
    parsed_explicit_ip = _parse_ip_literal(explicit_client_ip)
    if parsed_explicit_ip is not None:
        return parsed_explicit_ip
    parsed = urlsplit(str(proxy_url or "").strip())
    if not parsed.scheme or not _host_is_loopback(parsed.hostname):
        return None
    return _parse_ip_literal(parsed.username)


def _local_contributor_proxy_metadata_headers(
    proxy_url: str | None,
    country_code: str | None = None,
    explicit_client_ip: str | None = None,
) -> dict[str, str]:
    if not _env_flag_enabled("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE"):
        return {}
    if _local_contributor_client_ip(proxy_url, explicit_client_ip) is None:
        return {}
    headers: dict[str, str] = {}
    normalized_country = _normalize_country_code(country_code)
    if normalized_country:
        headers["X-Geo-Country"] = normalized_country
    return headers


def _local_contributor_forwarding_headers(
    proxy_url: str | None,
    country_code: str | None = None,
    explicit_client_ip: str | None = None,
) -> dict[str, str]:
    if not (
        _env_flag_enabled("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE")
        and _env_flag_enabled("SHUMA_LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING")
    ):
        return {}
    forwarded_secret = str(os.environ.get("SHUMA_FORWARDED_IP_SECRET") or "").strip()
    if not forwarded_secret:
        return {}
    client_ip = _local_contributor_client_ip(proxy_url, explicit_client_ip)
    if client_ip is None:
        return {}
    headers = _local_contributor_proxy_metadata_headers(
        proxy_url,
        country_code,
        explicit_client_ip,
    )
    headers.update({
        "X-Forwarded-For": client_ip,
        "X-Forwarded-Proto": "https",
        "X-Shuma-Forwarded-Secret": forwarded_secret,
    })
    return headers


def _resolved_local_trusted_forwarding_headers(
    proxy_url: str | None,
    country_code: str | None = None,
    explicit_client_ip: str | None = None,
) -> dict[str, str]:
    headers = _local_contributor_proxy_metadata_headers(
        proxy_url,
        country_code,
        explicit_client_ip,
    )
    headers.update(
        _local_contributor_forwarding_headers(
            proxy_url,
            country_code,
            explicit_client_ip,
        )
    )
    return headers


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
            "tarpit_progress_abuse",
            "maze_navigation",
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


def _request_native_not_a_bot_body(seed: str, *, behavior: str = "escalate") -> str:
    if behavior == "fail":
        telemetry_payload = {
            "has_pointer": False,
            "pointer_move_count": 0,
            "pointer_path_length": 0.0,
            "pointer_direction_changes": 0,
            "down_up_ms": 70,
            "focus_changes": 4,
            "visibility_changes": 0,
            "interaction_elapsed_ms": 700,
            "keyboard_used": False,
            "touch_used": False,
            "activation_method": "unknown",
            "activation_trusted": False,
            "activation_count": 1,
            "control_focused": False,
        }
    else:
        telemetry_payload = {
            "has_pointer": False,
            "pointer_move_count": 0,
            "pointer_path_length": 0.0,
            "pointer_direction_changes": 0,
            "down_up_ms": 90,
            "focus_changes": 2,
            "visibility_changes": 0,
            "interaction_elapsed_ms": 900,
            "keyboard_used": False,
            "touch_used": False,
            "activation_method": "unknown",
            "activation_trusted": False,
            "activation_count": 1,
            "control_focused": False,
        }
    telemetry = json.dumps(
        telemetry_payload,
        separators=(",", ":"),
    )
    return f"seed={seed}&checked=1&telemetry={telemetry}"


def _request_native_puzzle_body(fields: dict[str, str]) -> str:
    seed = str(fields.get("seed") or "invalid").strip() or "invalid"
    output = fields.get("output")
    if output is not None:
        return f"seed={seed}&output={_build_wrong_challenge_output(str(output))}"
    if "answer" in fields:
        return f"seed={seed}&answer=bad&return_to=%2F"
    return "answer=bad&seed=invalid&return_to=%2F"


def _request_native_abusive_puzzle_body(fields: dict[str, str]) -> str:
    seed = str(fields.get("seed") or "invalid").strip() or "invalid"
    output = fields.get("output")
    if output is not None:
        return f"seed={seed}&output=bad"
    if "answer" in fields:
        return f"seed={seed}&answer=&return_to=%2F"
    return "seed=invalid&output=bad"


def _request_native_followup_pause_ms(
    tracker: "_DirectPersonaTracker",
    *,
    flow_label: str,
    ordinal: int,
    minimum_ms: int,
    maximum_ms: int,
) -> int:
    if minimum_ms <= 0 or maximum_ms < minimum_ms:
        return 0
    remaining_ms = tracker.remaining_ms()
    if remaining_ms <= minimum_ms:
        return 0
    upper_bound = min(maximum_ms, max(minimum_ms, remaining_ms - 200))
    if upper_bound < minimum_ms:
        return 0
    span = upper_bound - minimum_ms
    bucket = _stable_bucket(
        tracker.run_id,
        tracker.tick_id,
        tracker.fulfillment_mode,
        flow_label,
        ordinal,
    )
    return minimum_ms + (bucket % (span + 1))


def _request_native_followup_pause_window_ms(flow_label: str) -> tuple[int, int]:
    normalized_flow = str(flow_label or "").strip().lower()
    if normalized_flow == "challenge_puzzle_submit":
        return (1_000, 1_900)
    if normalized_flow == "challenge_puzzle_abuse":
        return (1_300, 2_400)
    if normalized_flow == "pow_verify_abuse":
        return (1_000, 2_200)
    return (0, 0)


def _pause_request_native_followup(
    tracker: "_DirectPersonaTracker",
    *,
    flow_label: str,
    ordinal: int,
    minimum_ms: int,
    maximum_ms: int,
) -> None:
    pause_ms = _request_native_followup_pause_ms(
        tracker,
        flow_label=flow_label,
        ordinal=ordinal,
        minimum_ms=minimum_ms,
        maximum_ms=maximum_ms,
    )
    _sleep_ms(pause_ms)


def _request_spec(
    method: str,
    target: str,
    *,
    surface_ids: list[str] | None = None,
    response_surface_targets: list[str] | None = None,
    headers: dict[str, str] | None = None,
    cookies: dict[str, str] | None = None,
    data: str | bytes | None = None,
    json_body: dict[str, Any] | list[Any] | None = None,
    follow_redirect: bool = False,
    allow_deadline_overrun: bool = False,
) -> dict[str, Any]:
    spec: dict[str, Any] = {
        "method": method,
        "target": target,
        "headers": dict(headers or {}),
        "follow_redirect": follow_redirect,
    }
    if surface_ids:
        spec["surface_ids"] = list(surface_ids)
    if response_surface_targets:
        spec["response_surface_targets"] = list(response_surface_targets)
    if cookies:
        spec["cookies"] = dict(cookies)
    if data is not None:
        spec["data"] = data
    if json_body is not None:
        spec["json"] = json_body
    if allow_deadline_overrun:
        spec["allow_deadline_overrun"] = True
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


class _FormFieldParser(HTMLParser):
    def __init__(self, *, base_url: str) -> None:
        super().__init__(convert_charrefs=True)
        self.base_url = base_url
        self.forms: list[dict[str, Any]] = []
        self._current_form: dict[str, Any] | None = None

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        normalized = {str(key or "").strip().lower(): str(value or "") for key, value in attrs}
        if tag.lower() == "form":
            self._current_form = {
                "action": urljoin(self.base_url, normalized.get("action", "")),
                "method": normalized.get("method", "get").strip().lower() or "get",
                "fields": {},
            }
            return
        if tag.lower() != "input" or self._current_form is None:
            return
        field_name = normalized.get("name", "").strip()
        if not field_name:
            return
        self._current_form["fields"][field_name] = normalized.get("value", "")

    def handle_endtag(self, tag: str) -> None:
        if tag.lower() != "form" or self._current_form is None:
            return
        self.forms.append(self._current_form)
        self._current_form = None


def _response_forms(response: Any) -> list[dict[str, Any]]:
    current_url = str(getattr(response, "url", "") or "").strip() or "/"
    parser = _FormFieldParser(base_url=current_url)
    parser.feed(_response_body_text(response))
    parser.close()
    return parser.forms


def _response_form_fields(response: Any | None, predicate) -> dict[str, str]:
    if response is None:
        return {}
    for form in _response_forms(response):
        action = str(form.get("action") or "").strip()
        if action and predicate(action):
            return {
                str(key or "").strip(): str(value or "")
                for key, value in dict(form.get("fields") or {}).items()
                if str(key or "").strip()
            }
    return {}


def _response_header_value(response: Any, header_name: str) -> str:
    headers = getattr(response, "headers", None)
    if headers is None:
        return ""
    try:
        value = headers.get(header_name)
    except Exception:  # pragma: no cover - third-party header containers vary.
        value = None
    return str(value or "").strip()


def _response_body_text(response: Any) -> str:
    body = getattr(response, "body", b"")
    if isinstance(body, bytes):
        return body.decode("utf-8", errors="replace")
    return str(body or "")


def _response_defence_candidate_paths(response: Any) -> list[str]:
    current_url = str(getattr(response, "url", "") or "").strip()
    candidates: list[str] = []
    if current_url:
        candidates.append(current_url)
    location = _response_header_value(response, "location")
    if location:
        candidates.append(urljoin(current_url or "/", location))
    candidates.extend(_response_form_targets(response))
    return [
        _request_path_value(candidate).lower()
        for candidate in _ordered_unique(candidates)
        if _request_path_value(candidate)
    ]


def _response_contains_markup_marker(response: Any, *markers: str) -> bool:
    body_text = _response_body_text(response).lower()
    return any(str(marker or "").lower() in body_text for marker in markers)


def _response_indicates_not_a_bot_surface(response: Any) -> bool:
    candidate_paths = _response_defence_candidate_paths(response)
    if any(path.startswith("/challenge/not-a-bot-checkbox") for path in candidate_paths):
        return True
    return _response_contains_markup_marker(
        response,
        'id="not-a-bot-form"',
        "id='not-a-bot-form'",
        'id="not-a-bot-checkbox"',
        "id='not-a-bot-checkbox'",
        'action="/challenge/not-a-bot-checkbox"',
        "action='/challenge/not-a-bot-checkbox'",
    )


def _response_indicates_puzzle_surface(response: Any) -> bool:
    candidate_paths = _response_defence_candidate_paths(response)
    if any(path.startswith("/challenge/puzzle") for path in candidate_paths):
        return True
    return _response_contains_markup_marker(
        response,
        'id="challenge-puzzle-form"',
        "id='challenge-puzzle-form'",
        'action="/challenge/puzzle"',
        "action='/challenge/puzzle'",
    )


def _response_indicates_pow_interstitial(response: Any) -> bool:
    has_pow_submit = _response_contains_markup_marker(
        response,
        "fetch('/pow/verify'",
        'fetch("/pow/verify"',
        'action="/pow/verify"',
        "action='/pow/verify'",
    )
    has_runtime_markers = _response_contains_markup_marker(
        response,
        "_checkCDPAutomation",
        "const POW_SEED =",
        "function showVerifying()",
        "solvePow(",
        "crypto.subtle",
        "document.body.innerText = 'Verifying...'",
        'document.body.innerText = "Verifying..."',
    )
    return has_pow_submit and has_runtime_markers


def _response_indicates_pow_surface(response: Any) -> bool:
    candidate_paths = _response_defence_candidate_paths(response)
    if any(path == "/pow" or path.startswith("/pow/verify") for path in candidate_paths):
        return True
    return _response_indicates_pow_interstitial(response) or _response_contains_markup_marker(
        response,
        'id="pow-bootstrap"',
        "id='pow-bootstrap'",
        'action="/pow/verify"',
        "action='/pow/verify'",
        "document.cookie='js_verified=",
        'document.cookie="js_verified=',
        "fetch('/pow/verify'",
        'fetch("/pow/verify"',
    )


def _response_inferred_browser_detection(response: Any) -> bool | None:
    body_text = _response_body_text(response).lower()
    if any(
        marker in body_text
        for marker in (
            'data-detected="1"',
            "data-detected='1'",
            "detected: true",
            "detected=true",
            "detected === true",
        )
    ):
        return True
    if any(
        marker in body_text
        for marker in (
            'data-detected="0"',
            "data-detected='0'",
            "detected: false",
            "detected=false",
            "detected === false",
        )
    ):
        return False
    return None


def _preserved_browser_surface_state(
    response: Any,
    challenge_state: dict[str, Any] | None,
) -> dict[str, Any]:
    preserved: dict[str, Any] = {}
    if _response_indicates_pow_surface(response):
        preserved["has_js_verification_script"] = True
    existing_cookie = (
        str(challenge_state.get("cookie") or "").strip()
        if isinstance(challenge_state, dict)
        else ""
    )
    if existing_cookie:
        preserved["cookie"] = existing_cookie
    elif _response_contains_markup_marker(
        response,
        "document.cookie='js_verified=",
        'document.cookie="js_verified=',
    ):
        preserved["cookie"] = "js_verified=1"
    detected = (
        challenge_state.get("detected")
        if isinstance(challenge_state, dict) and challenge_state.get("detected") in {True, False}
        else None
    )
    if detected is None:
        detected = _response_inferred_browser_detection(response)
    if detected in {True, False}:
        preserved["detected"] = detected
    return preserved


def _response_indicates_maze_surface(response: Any) -> bool:
    candidate_paths = _response_defence_candidate_paths(response)
    if any(path.startswith("/maze/") for path in candidate_paths):
        return True
    return _response_contains_markup_marker(
        response,
        'id="maze-bootstrap"',
        "id='maze-bootstrap'",
        'data-link-kind="maze"',
        "data-link-kind='maze'",
    )


def _response_indicates_challenge_routing(response: Any) -> bool:
    return (
        _response_indicates_not_a_bot_surface(response)
        or _response_indicates_puzzle_surface(response)
        or _response_indicates_pow_surface(response)
        or _response_indicates_maze_surface(response)
        or _response_contains_markup_marker(
            response,
            'action="/challenge/puzzle"',
            "action='/challenge/puzzle'",
            "verifying...",
        )
    )


def _response_indicates_request_native_followup_surface(response: Any) -> bool:
    return (
        _response_indicates_not_a_bot_surface(response)
        or _response_indicates_puzzle_surface(response)
        or _response_indicates_maze_surface(response)
    )


def _is_maze_navigation_target(raw_target: str) -> bool:
    request_path = _request_path_value(str(raw_target or "")).lower()
    return (
        request_path == "/_/"
        or request_path.startswith("/_/")
        or request_path.startswith("/maze/")
    )


def _response_observed_followup_targets(response: Any) -> list[str]:
    current_url = str(getattr(response, "url", "") or "").strip() or "/"
    targets = list(_response_form_targets(response))
    targets.extend(_response_anchor_targets(response))
    inferred_targets: list[str] = []
    if _response_indicates_not_a_bot_surface(response):
        inferred_targets.append(urljoin(current_url, "/challenge/not-a-bot-checkbox"))
    if _response_indicates_puzzle_surface(response):
        inferred_targets.append(urljoin(current_url, "/challenge/puzzle"))
    if _response_indicates_pow_interstitial(response):
        inferred_targets.append(urljoin(current_url, "/pow/verify"))
    if _response_contains_markup_marker(
        response,
        'action="/tarpit/progress"',
        "action='/tarpit/progress'",
        "fetch('/tarpit/progress'",
        'fetch("/tarpit/progress"',
        "window.__shumaTarpit",
        "Progress endpoint:",
        "progress endpoint:",
    ):
        inferred_targets.append(urljoin(current_url, "/tarpit/progress"))
    targets.extend(inferred_targets)
    return _ordered_unique(targets)


def _response_indicates_rate_pressure(response: Any) -> bool:
    status = int(getattr(response, "status", 0) or 0)
    if status == 429:
        return True
    if _response_header_value(response, "retry-after"):
        return True
    return _response_contains_markup_marker(
        response,
        "rate limit exceeded",
        "too many requests have been received from your ip address",
    )


def _response_indicates_geo_ip_policy(response: Any) -> bool:
    candidate_paths = _response_defence_candidate_paths(response)
    if any("geo-policy" in path for path in candidate_paths):
        return True
    if _response_contains_markup_marker(
        response,
        "blocked by regional access policy",
        "regional access policy",
    ):
        return True
    current_path = _request_path_value(str(getattr(response, "url", "") or "")).lower()
    public_request_path = (
        current_path == "/"
        or (
            bool(current_path)
            and not current_path.startswith("/challenge/")
            and not current_path.startswith("/pow")
            and not current_path.startswith("/tarpit/")
            and not current_path.startswith("/maze/")
            and not current_path.startswith("/_/")
        )
    )
    anchor_paths = [
        _request_path_value(candidate).lower()
        for candidate in _response_anchor_targets(response)
        if _request_path_value(candidate)
    ]
    if (
        public_request_path
        and _response_indicates_maze_surface(response)
        and (
            any(path.startswith("/_/") for path in anchor_paths)
            or _response_contains_markup_marker(
                response,
                '"path_prefix":"/_/',
                "\"path_prefix\":\"/_/",
                "'path_prefix':'/_/",
            )
        )
    ):
        return True
    return False


def _response_policy_surface_ids(response: Any, surface_targets: set[str]) -> list[str]:
    surface_ids: list[str] = []
    if "challenge_routing" in surface_targets and _response_indicates_challenge_routing(response):
        surface_ids.append("challenge_routing")
    if "rate_pressure" in surface_targets and _response_indicates_rate_pressure(response):
        surface_ids.append("rate_pressure")
    if "geo_ip_policy" in surface_targets and _response_indicates_geo_ip_policy(response):
        surface_ids.append("geo_ip_policy")
    return surface_ids


def _path_contains(raw_target: str, *needles: str) -> bool:
    candidate = _request_path_value(raw_target).lower()
    return any(str(needle or "").lower() in candidate for needle in needles)


def _build_wrong_challenge_output(current_output: str) -> str:
    normalized = str(current_output or "").strip()
    if normalized and all(ch in {"0", "1", "2"} for ch in normalized):
        baseline = normalized
    else:
        baseline = "0" * 16
    first = "1" if baseline[0] == "0" else "0"
    return f"{first}{baseline[1:]}"


def _request_native_browser_navigation_headers(accept_language: str) -> dict[str, str]:
    return {
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "accept-language": accept_language,
        "upgrade-insecure-requests": "1",
    }


def _looks_bulk_scraper_public_target(raw_target: str) -> bool:
    candidate = _request_path_value(raw_target).lower()
    return (
        candidate == "/"
        or candidate == "/feed-root"
        or candidate == "/timeline"
        or candidate == "/page"
        or candidate.startswith("/page/")
        or candidate in {"/about/", "/research/", "/plans/", "/work/"}
        or candidate.startswith("/research/")
        or candidate.startswith("/plans/")
        or candidate.startswith("/work/")
        or candidate.startswith("/timeline/")
        or "catalog" in candidate
        or "/detail/" in candidate
    )


def _bulk_scraper_priority(raw_target: str) -> tuple[int, str]:
    candidate = _request_path_value(raw_target).lower()
    if candidate == "/feed-root":
        return (0, candidate)
    if "catalog" in candidate:
        return (1, candidate)
    if candidate == "/page":
        return (2, candidate)
    if candidate.startswith("/page/"):
        return (3, candidate)
    if candidate in {"/research/", "/plans/", "/work/", "/about/"}:
        return (4, candidate)
    if candidate.startswith("/research/") or candidate.startswith("/plans/") or candidate.startswith("/work/"):
        return (5, candidate)
    if "/detail/" in candidate:
        return (6, candidate)
    if candidate in {"/", "/timeline"} or candidate.startswith("/timeline/"):
        return (7, candidate)
    return (8, candidate)


def _crawler_link_priority(raw_target: str) -> tuple[int, str]:
    candidate = _request_path_value(raw_target).lower()
    if candidate == "/feed-root":
        return (0, candidate)
    if candidate == "/page":
        return (1, candidate)
    if candidate.startswith("/page/"):
        return (2, candidate)
    if candidate in {"/research/", "/plans/", "/work/", "/about/"}:
        return (3, candidate)
    if candidate.startswith("/research/") or candidate.startswith("/plans/") or candidate.startswith("/work/"):
        return (4, candidate)
    if candidate == "/timeline" or candidate.startswith("/timeline/"):
        return (5, candidate)
    if "redirect" in candidate:
        return (6, candidate)
    if "catalog" in candidate:
        return (7, candidate)
    if "/detail/" in candidate:
        return (8, candidate)
    if "challenge" in candidate:
        return (9, candidate)
    if candidate == "/pow":
        return (10, candidate)
    if "/maze/" in candidate:
        return (11, candidate)
    return (12, candidate)


def _first_matching_target(candidates: list[str], predicate) -> str | None:
    for candidate in candidates:
        if predicate(candidate):
            return candidate
    return None


def _request_native_followup_target(response: Any | None, predicate) -> str | None:
    if response is None:
        return None
    return _first_matching_target(_response_observed_followup_targets(response), predicate)


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


def _set_cookie_contains(response: Any, fragment: str) -> bool:
    return fragment.lower() in _response_header_value(response, "set-cookie").lower()


def _surface_coverage_status_for_response(surface_id: str, response: Any) -> str:
    surface_key = str(surface_id or "").strip()
    status = int(getattr(response, "status", 0) or 0)
    if surface_key == "challenge_routing":
        return "pass_observed" if _response_indicates_challenge_routing(response) else _coverage_status_for_http_status(status)
    if surface_key == "not_a_bot_submit":
        if status == 303 and _set_cookie_contains(response, "not_a_bot"):
            return "pass_observed"
        return "fail_observed"
    if surface_key == "puzzle_submit_or_escalation":
        if status == 200 and "thank you! challenge complete." in _response_body_text(response).lower():
            return "pass_observed"
        return "fail_observed"
    if surface_key == "pow_verify_abuse":
        body_text = _response_body_text(response).lower()
        if _set_cookie_contains(response, "js_verified=") or '"verified": true' in body_text:
            return "pass_observed"
        return "fail_observed"
    if surface_key == "tarpit_progress_abuse":
        body_text = _response_body_text(response).lower()
        if '"accepted": true' in body_text or '"verified": true' in body_text:
            return "pass_observed"
        return "fail_observed"
    if surface_key == "rate_pressure":
        if _response_indicates_rate_pressure(response):
            return "fail_observed"
    if surface_key == "geo_ip_policy":
        if _response_indicates_geo_ip_policy(response):
            return "fail_observed"
    return _coverage_status_for_http_status(status)


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


def _surface_receipt_observed(
    receipts: dict[str, dict[str, Any]],
    surface_id: str,
    *,
    coverage_status: str | None = None,
) -> bool:
    prefix = f"{str(surface_id or '').strip()}:"
    if not prefix or prefix == ":":
        return False
    for receipt_key, receipt in receipts.items():
        if not str(receipt_key or "").startswith(prefix):
            continue
        if coverage_status is None:
            return True
        if str(receipt.get("coverage_status") or "").strip() == coverage_status:
            return True
    return False


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
        "surface_receipts": [],
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
    nonce = sim_tag_helpers.build_sim_tag_nonce(
        run_id,
        sim_profile,
        lane,
        seq,
        timestamp,
    )
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
        handle_httpstatus_all = True

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
            self.local_country_code = (
                str(self.request_identity_pool[0].get("country_code") or "").strip().upper()
                if self.request_identity_pool
                else (_local_fallback_country_code(self.fulfillment_mode) or "")
            )
            self.request_transport = resolve_request_transport_observation(
                self.realism_tracker.profile,
                country_code=self.local_country_code or None,
            )
            self.realism_tracker.observe_transport(
                transport_profile=str(self.request_transport.get("transport_profile") or ""),
                transport_realism_class=str(
                    self.request_transport.get("transport_realism_class") or ""
                ),
                transport_emission_basis=str(
                    self.request_transport.get("transport_emission_basis") or ""
                ),
                transport_degraded_reason=str(
                    self.request_transport.get("transport_degraded_reason") or ""
                ),
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
            self.activity_sequence_exhausted = False
            request_proxy_url = _normalize_optional_proxy_url(self.plan.get("request_proxy_url"))
            if self.request_identity_pool:
                request_proxy_url = (
                    str(self.request_identity_pool[0].get("proxy_url") or "").strip()
                    or request_proxy_url
                )
            self.local_trusted_forwarding_headers = _resolved_local_trusted_forwarding_headers(
                request_proxy_url,
                self.local_country_code or None,
                str(self.plan.get("local_request_client_ip") or "").strip() or None,
            )
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
            headers = _signed_headers(
                self.sim_telemetry_secret,
                run_id=self.run_id,
                profile=self.sim_profile,
                lane=self.lane,
                fulfillment_mode=self.fulfillment_mode,
                seq=self.request_sequence,
                extra_headers=extra_headers,
            )
            headers.update(self.local_trusted_forwarding_headers)
            return headers

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
                    meta={"depth": 0, "handle_httpstatus_all": True},
                    headers=self._next_headers(
                        {"accept-language": str(self.request_transport.get("accept_language") or "")}
                    ),
                )

        def _response_depth(self, response: Any) -> int:
            meta = getattr(response, "meta", None)
            if not isinstance(meta, dict):
                request = getattr(response, "request", None)
                meta = getattr(request, "meta", None)
            if not isinstance(meta, dict):
                return 0
            return int(meta.get("depth") or 0)

        def _record_response_surfaces(self, response: Any) -> None:
            surface_ids: list[str] = []
            if "public_path_traversal" in self.surface_targets:
                surface_ids.append("public_path_traversal")
            for surface_id in _response_policy_surface_ids(response, self.surface_targets):
                if surface_id not in surface_ids:
                    surface_ids.append(surface_id)
            if not surface_ids:
                return
            request_method = str(getattr(response.request, "method", "GET"))
            request_target = str(getattr(response, "url", ""))
            for surface_id in surface_ids:
                _record_surface_receipt(
                    self.surface_receipts,
                    surface_ids=[surface_id],
                    coverage_status=_surface_coverage_status_for_response(
                        surface_id,
                        response,
                    ),
                    request_method=request_method,
                    request_target=request_target,
                    response_status=int(response.status),
                )

        def _observe_response(self, response: Any) -> int:
            depth = self._response_depth(response)
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
                country_code=self.local_country_code or None,
            )
            self.requests_observed += 1
            self.bytes_observed += len(getattr(response, "body", b""))
            self.last_response_status = int(response.status)
            self._record_response_surfaces(response)
            return depth

        async def is_blocked(self, response) -> bool:
            blocked = await super().is_blocked(response)
            if blocked:
                self._observe_response(response)
            return blocked

        async def parse(self, response) -> AsyncGenerator[Any, None]:
            depth = self._observe_response(response)
            next_depth = depth + 1

            if 300 <= int(response.status) < 400:
                location = str(response.headers.get("location") or "").strip()
                if location:
                    allowed, normalized_url = self._allowed_request(
                        response.url,
                        location,
                        is_redirect=True,
                    )
                    if (
                        allowed
                        and normalized_url
                        and next_depth <= self.max_depth
                        and not self._should_stop()
                    ):
                        self.realism_tracker.observe_discovered_target(normalized_url)
                        yield response.follow(
                            normalized_url,
                            meta={"depth": next_depth, "handle_httpstatus_all": True},
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

            if self._should_stop():
                self.pause()
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
                            meta={"depth": next_depth, "handle_httpstatus_all": True},
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
                        meta={"depth": next_depth, "handle_httpstatus_all": True},
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
        self._time_budget_started_at: float | None = None
        self.deadline = 0.0
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
        self.local_trusted_forwarding_headers: dict[str, str] = {}

    def _ensure_time_budget_started(self) -> None:
        if self._time_budget_started_at is not None:
            return
        self._time_budget_started_at = time.monotonic()
        self.deadline = self._time_budget_started_at + (self.max_ms / 1000.0)

    def _deadline_reached(self) -> bool:
        return self._time_budget_started_at is not None and time.monotonic() >= self.deadline

    def should_stop(self) -> bool:
        return (
            self.realism_tracker.activity_limit_reached()
            or self.bytes_observed >= self.max_bytes
            or self._deadline_reached()
        )

    def hard_stop_reached(self) -> bool:
        return self.realism_tracker.activity_limit_reached() or self.bytes_observed >= self.max_bytes

    def can_start_request(self, *, allow_deadline_overrun: bool = False) -> bool:
        if self.hard_stop_reached():
            return False
        if not allow_deadline_overrun and self._deadline_reached():
            return False
        return True

    def remaining_ms(self) -> int:
        if self._time_budget_started_at is None:
            return self.max_ms
        return max(0, int((self.deadline - time.monotonic()) * 1000))

    def next_headers(self, extra_headers: dict[str, str] | None = None) -> dict[str, str]:
        self._ensure_time_budget_started()
        self.request_sequence += 1
        headers = _signed_headers(
            self.sim_telemetry_secret,
            run_id=self.run_id,
            profile=self.sim_profile,
            lane=self.lane,
            fulfillment_mode=self.fulfillment_mode,
            seq=self.request_sequence,
            extra_headers=extra_headers,
        )
        headers.update(self.local_trusted_forwarding_headers)
        return headers

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
            sample_request_method = request_method or str(getattr(response.request, "method", "GET"))
            sample_request_target = request_target or str(getattr(response, "url", ""))
            for surface_id in surface_ids:
                _record_surface_receipt(
                    self.surface_receipts,
                    surface_ids=[surface_id],
                    coverage_status=_surface_coverage_status_for_response(surface_id, response),
                    request_method=sample_request_method,
                    request_target=sample_request_target,
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
                deadline_reached=self._deadline_reached(),
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
    response_surface_targets = {
        str(value).strip()
        for value in list(request_spec.get("response_surface_targets") or [])
        if str(value).strip()
    }
    allow_deadline_overrun = bool(request_spec.get("allow_deadline_overrun"))
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
            allow_deadline_overrun=allow_deadline_overrun,
        )
        merged_surface_ids = list(surface_ids)
        for surface_id in _response_policy_surface_ids(response, response_surface_targets):
            if surface_id not in merged_surface_ids:
                merged_surface_ids.append(surface_id)
        tracker.record_response(
            response,
            merged_surface_ids or None,
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
                    allow_deadline_overrun=allow_deadline_overrun,
                )
                redirect_surface_ids = list(merged_surface_ids)
                for surface_id in _response_policy_surface_ids(
                    redirect_response,
                    response_surface_targets,
                ):
                    if surface_id not in redirect_surface_ids:
                        redirect_surface_ids.append(surface_id)
                tracker.record_response(
                    redirect_response,
                    redirect_surface_ids or None,
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


def _hostile_request_burst(
    session: Any,
    *,
    tracker: _DirectPersonaTracker,
    base_url: str,
    target: str,
    response_surface_targets: list[str],
    max_attempts: int,
    reserve_activities: int = 0,
    pivot_on_first_confrontation: bool = False,
    request_headers: dict[str, str] | None = None,
    pivot_predicate: Callable[[Any], bool] | None = None,
) -> Any | None:
    burst_attempts = max(0, int(max_attempts))
    reserved = max(0, int(reserve_activities))
    first_confrontation_response: Any | None = None
    last_response: Any | None = None
    confrontation_predicate = pivot_predicate or (
        lambda response: _response_indicates_challenge_routing(response)
        or _response_indicates_pow_surface(response)
    )
    for _ in range(burst_attempts):
        if tracker.should_stop() or tracker.realism_tracker.remaining_activity_budget() <= reserved:
            break
        response = _perform_request(
            session,
            tracker=tracker,
            base_url=base_url,
            request_spec=_request_spec(
                "get",
                target,
                headers=request_headers,
                response_surface_targets=response_surface_targets,
            ),
        )
        if response is not None:
            last_response = response
            if first_confrontation_response is None and confrontation_predicate(response):
                first_confrontation_response = response
                if pivot_on_first_confrontation:
                    break
        if _surface_receipt_observed(
            tracker.surface_receipts,
            "rate_pressure",
            coverage_status="fail_observed",
        ):
            break
    return first_confrontation_response or last_response


def _frontloaded_rate_pressure_reserve(
    *,
    challenge_target: str | None,
    pow_target: str | None,
    redirect_target: str | None,
) -> int:
    reserve = 0
    if challenge_target:
        reserve += 5
    if pow_target:
        reserve += 2
    if redirect_target:
        reserve += 1
    return reserve


def _request_native_rate_abuse_attempt_budget(tracker: _DirectPersonaTracker) -> int:
    planned_burst_size = max(1, int(tracker.realism_tracker.planned_burst_size or 1))
    effective_activity_budget = max(1, int(tracker.realism_tracker.effective_activity_budget or 1))
    return max(24, min(48, max(planned_burst_size, effective_activity_budget)))


def _request_native_maze_followup_target(*responses: Any | None) -> str | None:
    for response in responses:
        if response is None:
            continue
        maze_target = _request_native_followup_target(
            response,
            _is_maze_navigation_target,
        )
        if maze_target:
            return maze_target
        current_url = str(getattr(response, "url", "") or "").strip()
        if current_url and _is_maze_navigation_target(current_url):
            return current_url
    return None


def _execute_request_native_challenge_followups(
    session: Any,
    *,
    tracker: _DirectPersonaTracker,
    base_url: str,
    surface_targets: list[str],
    seed_response: Any,
    pow_seed_response: Any | None = None,
    preserve_followon_targets: bool = False,
    initial_not_a_bot_behavior: str = "escalate",
    rate_attempt_budget_override: int | None = None,
    suppress_timing_pauses: bool = False,
) -> Any:
    followup_response = seed_response
    preserved_tarpit_seed: Any | None = None
    normalized_surface_targets = _normalize_surface_targets(surface_targets)
    surface_target_set = set(normalized_surface_targets)
    abuse_attempt_ceiling = max(
        1,
        min(3, int(tracker.realism_tracker.profile.get("retry_ceiling") or 1)),
    )

    def allow_surface_completion() -> bool:
        return tracker.can_start_request(allow_deadline_overrun=True)

    not_a_bot_target = _request_native_followup_target(
        followup_response,
        lambda candidate: _path_contains(candidate, "not-a-bot"),
    )
    not_a_bot_fields = _response_form_fields(
        followup_response,
        lambda candidate: _path_contains(candidate, "not-a-bot"),
    )
    if (
        "not_a_bot_submit" in surface_target_set
        and not_a_bot_target
        and allow_surface_completion()
    ):
        candidate_response = _perform_request(
            session,
            tracker=tracker,
            base_url=base_url,
            request_spec=_request_spec(
                "post",
                not_a_bot_target,
                surface_ids=["not_a_bot_submit"],
                response_surface_targets=normalized_surface_targets,
                allow_deadline_overrun=True,
                headers={
                    "accept": "application/json",
                    "content-type": "application/x-www-form-urlencoded",
                },
                data=_request_native_not_a_bot_body(
                    str(not_a_bot_fields.get("seed") or "invalid-seed"),
                    behavior=initial_not_a_bot_behavior,
                ),
            ),
        )
        if candidate_response is not None:
            followup_response = candidate_response

    if "maze_navigation" in surface_target_set and allow_surface_completion():
        maze_target = _request_native_maze_followup_target(
            followup_response,
            seed_response,
        )
        if maze_target:
            candidate_response = _perform_request(
                session,
                tracker=tracker,
                base_url=base_url,
                request_spec=_request_spec(
                    "get",
                    maze_target,
                    surface_ids=["maze_navigation"],
                    response_surface_targets=normalized_surface_targets,
                    allow_deadline_overrun=True,
                ),
            )
            if candidate_response is not None:
                followup_response = candidate_response

    puzzle_target = _request_native_followup_target(
        followup_response,
        lambda candidate: _path_contains(candidate, "puzzle"),
    ) or _request_native_followup_target(
        seed_response,
        lambda candidate: _path_contains(candidate, "puzzle"),
    )
    puzzle_fields = _response_form_fields(
        followup_response,
        lambda candidate: _path_contains(candidate, "puzzle"),
    ) or _response_form_fields(
        seed_response,
        lambda candidate: _path_contains(candidate, "puzzle"),
    )
    if (
        "puzzle_submit_or_escalation" in surface_target_set
        and puzzle_target
        and allow_surface_completion()
    ):
        challenge_submit_min_ms, challenge_submit_max_ms = _request_native_followup_pause_window_ms(
            "challenge_puzzle_submit"
        )
        if not suppress_timing_pauses:
            _pause_request_native_followup(
                tracker,
                flow_label="challenge_puzzle_submit",
                ordinal=0,
                minimum_ms=challenge_submit_min_ms,
                maximum_ms=challenge_submit_max_ms,
            )
        candidate_response = _perform_request(
            session,
            tracker=tracker,
            base_url=base_url,
            request_spec=_request_spec(
                "post",
                puzzle_target,
                surface_ids=["puzzle_submit_or_escalation"],
                response_surface_targets=normalized_surface_targets,
                allow_deadline_overrun=True,
                headers={
                    "accept": "application/json",
                    "content-type": "application/x-www-form-urlencoded",
                },
                data=_request_native_puzzle_body(puzzle_fields),
            ),
        )
        if candidate_response is not None:
            followup_response = candidate_response
    if (
        "tarpit_progress_abuse" in surface_target_set
        and puzzle_target
        and allow_surface_completion()
    ):
        challenge_abuse_min_ms, challenge_abuse_max_ms = _request_native_followup_pause_window_ms(
            "challenge_puzzle_abuse"
        )
        if not suppress_timing_pauses:
            _pause_request_native_followup(
                tracker,
                flow_label="challenge_puzzle_abuse",
                ordinal=1,
                minimum_ms=challenge_abuse_min_ms,
                maximum_ms=challenge_abuse_max_ms,
            )
        abusive_response = _perform_request(
            session,
            tracker=tracker,
            base_url=base_url,
            request_spec=_request_spec(
                "post",
                puzzle_target,
                response_surface_targets=normalized_surface_targets,
                allow_deadline_overrun=True,
                headers={
                    "accept": "application/json",
                    "content-type": "application/x-www-form-urlencoded",
                },
                data=_request_native_abusive_puzzle_body(puzzle_fields),
            ),
        )
        if abusive_response is not None:
            preserved_tarpit_seed = abusive_response
            followup_response = abusive_response
    pow_verify_target = _request_native_followup_target(
        pow_seed_response,
        lambda candidate: _request_path_value(candidate).lower() == "/pow/verify",
    ) or _request_native_followup_target(
        followup_response,
        lambda candidate: _path_contains(candidate, "/pow/verify"),
    ) or _request_native_followup_target(
        seed_response,
        lambda candidate: _request_path_value(candidate).lower() == "/pow/verify",
    )
    tarpit_target = _request_native_followup_target(
        preserved_tarpit_seed,
        lambda candidate: _path_contains(candidate, "/tarpit/progress"),
    ) or _request_native_followup_target(
        followup_response,
        lambda candidate: _path_contains(candidate, "/tarpit/progress"),
    ) or _request_native_followup_target(
        seed_response,
        lambda candidate: _request_path_value(candidate).lower() == "/tarpit/progress",
    )
    if (
        "tarpit_progress_abuse" in surface_target_set
        and allow_surface_completion()
        and not preserve_followon_targets
        and tarpit_target is None
        and preserved_tarpit_seed is not None
    ):
        tarpit_probe_response = _perform_request(
            session,
            tracker=tracker,
            base_url=base_url,
            request_spec=_request_spec(
                "get",
                str(getattr(seed_response, "url", "") or base_url),
                response_surface_targets=normalized_surface_targets,
                allow_deadline_overrun=True,
            ),
        )
        if tarpit_probe_response is not None:
            tarpit_target = _request_native_followup_target(
                tarpit_probe_response,
                lambda candidate: _path_contains(candidate, "/tarpit/progress"),
            )
    if (
        "pow_verify_abuse" in surface_target_set
        and pow_verify_target
        and allow_surface_completion()
    ):
        pow_abuse_min_ms, pow_abuse_max_ms = _request_native_followup_pause_window_ms(
            "pow_verify_abuse"
        )
        pow_abuse_attempts = abuse_attempt_ceiling if not tracker.should_stop() else 1
        for attempt_index in range(pow_abuse_attempts):
            if not allow_surface_completion():
                break
            if not suppress_timing_pauses:
                _pause_request_native_followup(
                    tracker,
                    flow_label="pow_verify_abuse",
                    ordinal=attempt_index,
                    minimum_ms=pow_abuse_min_ms,
                    maximum_ms=pow_abuse_max_ms,
                )
            candidate_response = _perform_request(
                session,
                tracker=tracker,
                base_url=base_url,
                request_spec=_request_spec(
                    "post",
                    pow_verify_target,
                    surface_ids=["pow_verify_abuse"],
                    response_surface_targets=normalized_surface_targets,
                    allow_deadline_overrun=True,
                    headers={
                        "accept": "application/json",
                        "content-type": "application/json",
                    },
                    json_body={"seed": "invalid-seed", "nonce": "invalid-nonce"},
                ),
            )
            if candidate_response is not None:
                followup_response = candidate_response
            if _surface_receipt_observed(
                tracker.surface_receipts,
                "pow_verify_abuse",
                coverage_status="fail_observed",
            ):
                break
    if (
        "tarpit_progress_abuse" in surface_target_set
        and tarpit_target
        and allow_surface_completion()
    ):
        tarpit_abuse_attempts = abuse_attempt_ceiling if not tracker.should_stop() else 1
        for _ in range(tarpit_abuse_attempts):
            if not allow_surface_completion():
                break
            candidate_response = _perform_request(
                session,
                tracker=tracker,
                base_url=base_url,
                request_spec=_request_spec(
                    "post",
                    tarpit_target,
                    surface_ids=["tarpit_progress_abuse"],
                    response_surface_targets=normalized_surface_targets,
                    allow_deadline_overrun=True,
                    headers={
                        "accept": "application/json",
                        "content-type": "application/json",
                    },
                    json_body={
                        "token": "invalid",
                        "nonce": "invalid",
                    },
                ),
            )
            if candidate_response is not None:
                followup_response = candidate_response
            if _surface_receipt_observed(
                tracker.surface_receipts,
                "tarpit_progress_abuse",
                coverage_status="fail_observed",
            ):
                break
    if (
        "rate_pressure" in surface_target_set
        and not preserve_followon_targets
        and not_a_bot_target
        and allow_surface_completion()
        and not _surface_receipt_observed(
            tracker.surface_receipts,
            "rate_pressure",
            coverage_status="fail_observed",
        )
    ):
        repeated_not_a_bot_body = _request_native_not_a_bot_body(
            str(not_a_bot_fields.get("seed") or "invalid-seed"),
            behavior="fail",
        )
        if tracker.should_stop():
            rate_attempts = 2
        elif rate_attempt_budget_override is not None:
            rate_attempts = max(1, int(rate_attempt_budget_override))
        else:
            rate_attempts = _request_native_rate_abuse_attempt_budget(tracker)
        for _ in range(rate_attempts):
            if not allow_surface_completion():
                break
            candidate_response = _perform_request(
                session,
                tracker=tracker,
                base_url=base_url,
                request_spec=_request_spec(
                    "post",
                    not_a_bot_target,
                    response_surface_targets=normalized_surface_targets,
                    allow_deadline_overrun=True,
                    headers={
                        "accept": "application/json",
                        "content-type": "application/x-www-form-urlencoded",
                    },
                    data=repeated_not_a_bot_body,
                ),
            )
            if candidate_response is not None:
                followup_response = candidate_response
            if _surface_receipt_observed(
                tracker.surface_receipts,
                "rate_pressure",
                coverage_status="fail_observed",
            ):
                break
    return followup_response


def _evaluate_browser_challenge_state(page) -> dict[str, Any]:
    return page.evaluate(
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
            const documentHtml = document.documentElement ? (document.documentElement.innerHTML || "") : "";
            return {
                has_check: hasCheck,
                detected: detected,
                cookie: document.cookie || "",
                body_text: (document.body && document.body.innerText) || "",
                location: window.location.pathname + window.location.search,
                form_actions: Array.from(document.querySelectorAll('form[action]'))
                    .map((form) => form.getAttribute('action') || ''),
                has_maze_link: !!document.querySelector("[data-link-kind='maze']"),
                has_maze_bootstrap: !!document.querySelector("#maze-bootstrap"),
                has_pow_bootstrap: !!document.querySelector("#pow-bootstrap"),
                has_maze_script:
                    documentHtml.includes('id="maze-bootstrap"')
                    || documentHtml.includes("id='maze-bootstrap'")
                    || documentHtml.includes('data-link-kind="maze"')
                    || documentHtml.includes("data-link-kind='maze'"),
                has_js_verification_script:
                    documentHtml.includes("/pow/verify")
                    || documentHtml.includes("document.cookie = 'js_verified=")
                    || documentHtml.includes('document.cookie = "js_verified=')
                    || documentHtml.includes("document.cookie='js_verified=")
                    || documentHtml.includes('document.cookie="js_verified=')
                    || documentHtml.includes("showVerifying")
                    || documentHtml.includes("solvePow(")
                    || documentHtml.includes("/fingerprint-report")
                    || documentHtml.includes("/cdp-report"),
            };
        }"""
    )


def _browser_root_discovery_page_action(state: dict[str, Any]):
    def action(page) -> None:
        state["links"] = page.evaluate(
            """() => Array.from(document.querySelectorAll('a[href]'))
                .map((anchor) => anchor.getAttribute('href') || '')
            """
        )
        state["challenge_state"] = _evaluate_browser_challenge_state(page)

    return action


def _browser_root_served_challenge_page_action(
    state: dict[str, Any],
    *,
    next_submit_headers: Callable[[], dict[str, str]] | None = None,
    low_score_not_a_bot: bool = False,
):
    def action(page) -> None:
        telemetry_payload = (
            {
                "has_pointer": False,
                "pointer_move_count": 0,
                "pointer_path_length": 0,
                "pointer_direction_changes": 0,
                "down_up_ms": 0,
                "focus_changes": 4,
                "visibility_changes": 0,
                "interaction_elapsed_ms": 300,
                "keyboard_used": False,
                "touch_used": False,
                "activation_method": "unknown",
                "activation_trusted": False,
                "activation_count": 1,
                "control_focused": False,
            }
            if low_score_not_a_bot
            else {
                "has_pointer": False,
                "pointer_move_count": 0,
                "pointer_path_length": 0,
                "pointer_direction_changes": 0,
                "down_up_ms": 120,
                "focus_changes": 2,
                "visibility_changes": 0,
                "interaction_elapsed_ms": 1100,
                "keyboard_used": False,
                "touch_used": False,
                "activation_method": "unknown",
                "activation_trusted": False,
                "activation_count": 1,
                "control_focused": True,
            }
        )
        telemetry_json = json.dumps(telemetry_payload)
        state["pre_submit_challenge_state"] = _evaluate_browser_challenge_state(page)

        def submit_form(action_fragment: str, field_values: dict[str, str]) -> bool:
            locator = page.locator(f"form[action*='{action_fragment}']").first
            if int(locator.count() or 0) < 1:
                return False
            try:
                prepared = page.evaluate(
                    """({ actionFragment, values }) => {
                        const form = Array.from(document.querySelectorAll('form[action]'))
                            .find((candidate) => (candidate.getAttribute('action') || '').includes(actionFragment));
                        if (!form) {
                            return false;
                        }
                        const wrongChallengeOutput = (currentValue) => {
                            const normalized = String(currentValue ?? '');
                            const baseline = normalized || '0'.repeat(16);
                            const first = baseline[0] === '0' ? '1' : '0';
                            return `${first}${baseline.slice(1)}`;
                        };
                        const ensureField = (name) => {
                            let field = form.querySelector(`[name="${name}"]`);
                            if (!field) {
                                field = document.createElement('input');
                                field.setAttribute('type', 'hidden');
                                field.setAttribute('name', name);
                                form.appendChild(field);
                            }
                            return field;
                        };
                        if ((form.getAttribute('action') || '').includes('/challenge/puzzle')) {
                            const outputField =
                                form.querySelector('[name="output"]')
                                || document.getElementById('challenge-output');
                            if (outputField) {
                                outputField.value = wrongChallengeOutput(outputField.value);
                            }
                        }
                        for (const [name, value] of Object.entries(values || {})) {
                            const normalizedName = String(name ?? '');
                            if (!normalizedName) {
                                continue;
                            }
                            const field = ensureField(normalizedName);
                            field.value = String(value ?? '');
                            if ((field.getAttribute('type') || '').toLowerCase() === 'checkbox') {
                                field.checked = field.value === '1';
                            }
                        }
                        const checkbox = document.getElementById('not-a-bot-checkbox');
                        if (checkbox && Object.prototype.hasOwnProperty.call(values || {}, 'checked')) {
                            checkbox.checked = String((values || {}).checked ?? '') === '1';
                        }
                        return true;
                    }""",
                    {
                        "actionFragment": action_fragment,
                        "values": field_values,
                    },
                )
                if prepared is not True:
                    return False
                if next_submit_headers is not None:
                    page.context.set_extra_http_headers(next_submit_headers())
                try:
                    locator.evaluate(
                        """(form) => {
                            if (typeof form.requestSubmit === 'function') {
                                form.requestSubmit();
                            } else {
                                form.submit();
                            }
                        }"""
                    )
                except Exception:
                    submit_button = locator.locator("button[type='submit'], input[type='submit']").first
                    if int(submit_button.count() or 0) > 0:
                        submit_button.click()
                    else:
                        raise
                try:
                    page.wait_for_load_state("networkidle", timeout=5_000)
                except Exception:
                    page.wait_for_load_state("domcontentloaded")
            except Exception:
                try:
                    page.wait_for_load_state("networkidle")
                except Exception:
                    page.wait_for_load_state("domcontentloaded")
            return True

        submitted_not_a_bot = submit_form(
            "/challenge/not-a-bot-checkbox",
            {
                "checked": "1",
                "telemetry": telemetry_json,
            },
        )
        if not submitted_not_a_bot:
            submit_form(
                "/challenge/puzzle",
                {},
            )
        elif int(page.locator("form[action*='/challenge/puzzle']").count() or 0) > 0:
            submit_form(
                "/challenge/puzzle",
                {},
            )

        state["post_submit_challenge_state"] = _evaluate_browser_challenge_state(page)

        state["links"] = page.evaluate(
            """() => Array.from(document.querySelectorAll('a[href]'))
                .map((anchor) => anchor.getAttribute('href') || '')
            """
        )
        maze_locator = page.locator("[data-link-kind='maze']").first
        state["maze_link_count"] = int(maze_locator.count() or 0)
        state["maze_click_attempted"] = False
        if int(state["maze_link_count"] or 0) > 0:
            state["maze_before_url"] = page.url
            try:
                state["maze_click_attempted"] = True
                maze_locator.click()
                page.wait_for_load_state("networkidle")
            except Exception:
                try:
                    page.wait_for_load_state("domcontentloaded")
                except Exception:
                    pass
            state["maze_after_url"] = page.url
            try:
                state["maze_bootstrap_after_click"] = (
                    page.locator("#maze-bootstrap").text_content() or ""
                )
            except Exception:
                state["maze_bootstrap_after_click"] = ""
        state["challenge_state"] = _merge_browser_challenge_states(
            _evaluate_browser_challenge_state(page),
            state.get("post_submit_challenge_state"),
            state.get("pre_submit_challenge_state"),
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


def _browser_note_discovery(
    tracker: _DirectPersonaTracker,
    *,
    response: Any,
    page_state: dict[str, Any],
    base_url: str,
    current_depth: int,
    visited_targets: set[str],
    public_candidates: list[tuple[str, int]],
    discovered_targets_accumulator: list[str],
) -> None:
    current_url = str(getattr(response, "url", "") or "").strip()
    tracker.realism_tracker.observe_exploration_visit(
        target=current_url,
        depth=current_depth,
        content_type=_response_content_type(response),
    )
    discovered_targets: list[str] = []
    for candidate in _normalize_browser_discovery_targets(
        base_url,
        list(page_state.get("links") or []),
    ):
        allowed, normalized_url = tracker.allowed_request(
            current_url,
            candidate,
            is_redirect=False,
            record_rejection=False,
        )
        if not allowed or not normalized_url:
            continue
        tracker.realism_tracker.observe_discovered_target(normalized_url)
        if normalized_url not in discovered_targets_accumulator:
            discovered_targets_accumulator.append(normalized_url)
        discovered_targets.append(normalized_url)
    next_depth = current_depth + 1
    for candidate in discovered_targets:
        if _looks_bulk_scraper_public_target(candidate):
            public_candidates.append((candidate, next_depth))
    unique_candidates: list[tuple[str, int]] = []
    seen_targets: set[str] = set()
    for candidate, depth in public_candidates:
        if candidate in visited_targets or candidate in seen_targets:
            continue
        seen_targets.add(candidate)
        unique_candidates.append((candidate, depth))
    public_candidates[:] = sorted(
        unique_candidates,
        key=lambda item: _bulk_scraper_priority(item[0]),
    )


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


def _browser_state_indicates_pow_surface(
    challenge_state: dict[str, Any],
    *,
    background_paths: list[str],
) -> bool:
    cookie = str(challenge_state.get("cookie") or "")
    body_text = str(challenge_state.get("body_text") or "")
    form_actions = [
        _request_path_value(value).lower()
        for value in list(challenge_state.get("form_actions") or [])
        if _request_path_value(value)
    ]
    return (
        bool(challenge_state.get("has_check"))
        or bool(challenge_state.get("has_pow_bootstrap"))
        or bool(challenge_state.get("has_js_verification_script"))
        or "js_verified=" in cookie
        or "Verifying" in body_text
        or any(path.startswith("/pow/verify") for path in form_actions)
        or any(
            path in {"/pow/verify", "/fingerprint-report", "/cdp-report"}
            for path in background_paths
        )
    )


def _browser_state_indicates_maze_surface(challenge_state: dict[str, Any]) -> bool:
    current_path = _request_path_value(str(challenge_state.get("location") or "")).lower()
    return (
        bool(challenge_state.get("has_maze_link"))
        or bool(challenge_state.get("has_maze_bootstrap"))
        or bool(challenge_state.get("has_maze_script"))
        or _is_maze_navigation_target(current_path)
    )


def _browser_state_has_followup_challenge_form(challenge_state: dict[str, Any]) -> bool:
    for value in list(challenge_state.get("form_actions") or []):
        action_path = _request_path_value(str(value or "")).lower()
        if action_path.startswith("/challenge/not-a-bot-checkbox") or action_path.startswith(
            "/challenge/puzzle"
        ):
            return True
    return False


def _merge_browser_challenge_states(*challenge_states: dict[str, Any] | None) -> dict[str, Any]:
    merged: dict[str, Any] = {
        "has_check": False,
        "detected": None,
        "cookie": "",
        "body_text": "",
        "location": "",
        "form_actions": [],
        "has_maze_link": False,
        "has_maze_bootstrap": False,
        "has_pow_bootstrap": False,
        "has_maze_script": False,
        "has_js_verification_script": False,
    }
    merged_form_actions: list[str] = []
    detection_state: bool | None = None
    for challenge_state in challenge_states:
        if not isinstance(challenge_state, dict):
            continue
        merged["has_check"] = bool(merged["has_check"]) or bool(challenge_state.get("has_check"))
        detected = challenge_state.get("detected")
        if detected is True:
            detection_state = True
        elif detected is False and detection_state is None:
            detection_state = False
        for key in ("cookie", "body_text", "location"):
            value = str(challenge_state.get(key) or "")
            if value and not merged.get(key):
                merged[key] = value
        merged["has_maze_link"] = bool(merged["has_maze_link"]) or bool(
            challenge_state.get("has_maze_link")
        )
        merged["has_maze_bootstrap"] = bool(merged["has_maze_bootstrap"]) or bool(
            challenge_state.get("has_maze_bootstrap")
        )
        merged["has_pow_bootstrap"] = bool(merged["has_pow_bootstrap"]) or bool(
            challenge_state.get("has_pow_bootstrap")
        )
        merged["has_maze_script"] = bool(merged["has_maze_script"]) or bool(
            challenge_state.get("has_maze_script")
        )
        merged["has_js_verification_script"] = bool(
            merged["has_js_verification_script"]
        ) or bool(challenge_state.get("has_js_verification_script"))
        for value in list(challenge_state.get("form_actions") or []):
            action = str(value or "").strip()
            if action and action not in merged_form_actions:
                merged_form_actions.append(action)
    merged["detected"] = detection_state
    merged["form_actions"] = merged_form_actions
    return merged


def _browser_state_current_target(
    challenge_state: dict[str, Any],
    *,
    base_url: str,
    fallback_target: str,
) -> str:
    location = str(challenge_state.get("location") or "").strip()
    if location:
        return urljoin(base_url, location)
    return fallback_target


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
            current_url_path = _request_path_value(current_url).lower()
            if _response_indicates_challenge_routing(response) or current_url_path.startswith("/challenge/"):
                challenge_page = current_url
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

    def execute_challenge_followups(
        session: Any,
        *,
        seed_response: Any,
        preserve_followon_targets: bool,
    ) -> Any:
        return _execute_request_native_challenge_followups(
            session,
            tracker=tracker,
            base_url=base_url,
            surface_targets=list(surface_targets),
            seed_response=seed_response,
            preserve_followon_targets=preserve_followon_targets,
            initial_not_a_bot_behavior="fail",
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
        root_response = _perform_request(
            session,
            tracker=tracker,
            base_url=base_url,
            request_spec=_request_spec(
                "get",
                base_url,
                surface_ids=["public_path_traversal"],
                response_surface_targets=list(surface_targets),
            ),
        )
        if root_response is not None:
            visited.add(str(getattr(root_response, "url", base_url)))
            note_discovery(root_response, current_depth=0)

        if (
            root_response is not None
            and challenge_page is not None
            and not tracker.should_stop()
        ):
            challenge_seed_response = root_response
            root_response_url = str(getattr(root_response, "url", "") or "").strip()
            if challenge_page != root_response_url:
                challenge_seed_response = _perform_request(
                    session,
                    tracker=tracker,
                    base_url=base_url,
                    request_spec=_request_spec(
                        "get",
                        challenge_page,
                        response_surface_targets=list(surface_targets),
                    ),
                )
            if challenge_seed_response is not None:
                execute_challenge_followups(
                    session,
                    seed_response=challenge_seed_response,
                    preserve_followon_targets=True,
                )

        if (
            "rate_pressure" in surface_targets
            and challenge_page is None
            and root_response is not None
            and not tracker.should_stop()
            and not _response_indicates_challenge_routing(root_response)
            and not _surface_receipt_observed(
                tracker.surface_receipts,
                "rate_pressure",
                coverage_status="fail_observed",
            )
        ):
            pressure_response = _hostile_request_burst(
                session,
                tracker=tracker,
                base_url=base_url,
                target=base_url,
                response_surface_targets=list(surface_targets),
                max_attempts=max(18, min(54, tracker.realism_tracker.effective_activity_budget)),
                reserve_activities=6,
                pivot_on_first_confrontation=True,
            )
            if pressure_response is not None and _response_indicates_challenge_routing(
                pressure_response
            ):
                challenge_page = str(
                    getattr(pressure_response, "url", challenge_page or base_url)
                    or challenge_page
                    or base_url
                )
                execute_challenge_followups(
                    session,
                    seed_response=pressure_response,
                    preserve_followon_targets=True,
                )

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
                    response_surface_targets=list(surface_targets),
                ),
            )
            if response is None:
                continue
            visited.add(str(getattr(response, "url", candidate)))
            note_discovery(response, current_depth=candidate_depth)
            if challenge_page is not None:
                break

        while not tracker.should_stop() and challenge_page:
            challenge_response = _perform_request(
                session,
                tracker=tracker,
                base_url=base_url,
                request_spec=_request_spec(
                    "get",
                    challenge_page,
                    response_surface_targets=list(surface_targets),
                ),
            )
            if challenge_response is None:
                break
            execute_challenge_followups(
                session,
                seed_response=challenge_response,
                preserve_followon_targets=False,
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
    request_transport = resolve_request_transport_observation(
        tracker.realism_tracker.profile
    )
    request_accept_language = str(request_transport.get("accept_language") or "")

    def execute_challenge_followups(
        session: Any,
        *,
        seed_response: Any,
        pow_seed_response: Any | None = None,
        preserve_followon_targets: bool = False,
        rate_attempt_budget_override: int | None = None,
        suppress_timing_pauses: bool = False,
    ) -> Any:
        return _execute_request_native_challenge_followups(
            session,
            tracker=tracker,
            base_url=base_url,
            surface_targets=list(surface_targets),
            seed_response=seed_response,
            pow_seed_response=pow_seed_response,
            preserve_followon_targets=preserve_followon_targets,
            initial_not_a_bot_behavior="escalate",
            rate_attempt_budget_override=rate_attempt_budget_override,
            suppress_timing_pauses=suppress_timing_pauses,
        )

    def root_rate_pressure_unmet() -> bool:
        return (
            "rate_pressure" in surface_targets
            and not tracker.should_stop()
            and not _surface_receipt_observed(
                tracker.surface_receipts,
                "rate_pressure",
                coverage_status="fail_observed",
            )
        )

    with _PacedRequestNativeSession(
        fetcher_session_cls,
        tracker=tracker,
        timeout_seconds=max(1.0, min(30.0, tracker.max_ms / 1000.0)),
        accept_header="application/json",
        proxy_url=request_proxy_url,
        request_transport=request_transport,
        identity_pool=request_identity_pool,
    ) as session:
        root_response = _perform_request(
            session,
            tracker=tracker,
            base_url=base_url,
            request_spec=_request_spec(
                "get",
                base_url,
                headers=_request_native_browser_navigation_headers(request_accept_language),
                response_surface_targets=list(surface_targets),
            ),
        )
        root_links = _response_anchor_targets(root_response) if root_response is not None else []

        root_serves_challenge = (
            root_response is not None and _response_indicates_challenge_routing(root_response)
        )
        root_serves_pow = root_response is not None and _response_indicates_pow_surface(root_response)
        challenge_target = (
            None
            if root_serves_challenge
            else _first_matching_target(
                root_links,
                lambda candidate: _path_contains(candidate, "not-a-bot", "challenge/puzzle"),
            )
        )
        pow_target = (
            None
            if root_serves_pow
            else _first_matching_target(
                root_links,
                lambda candidate: _request_path_value(candidate).lower() == "/pow",
            )
        )
        redirect_target = _first_matching_target(
            root_links,
            lambda candidate: _path_contains(candidate, "redirect"),
        )
        root_followon_target_count = sum(
            1
            for available in (
                challenge_target if not root_serves_challenge else "",
                pow_target if not root_serves_pow else "",
                redirect_target,
            )
            if available
        )
        root_has_direct_escalation_targets = bool(
            challenge_target or pow_target or redirect_target
        )
        frontloaded_rate_reserve = _frontloaded_rate_pressure_reserve(
            challenge_target=challenge_target,
            pow_target=pow_target,
            redirect_target=redirect_target,
        )

        hostile_burst_attempts = max(
            24,
            min(72, tracker.realism_tracker.effective_activity_budget),
        )

        if (
            "rate_pressure" in surface_targets
            and not tracker.should_stop()
            and not root_serves_challenge
            and not root_serves_pow
            and not root_has_direct_escalation_targets
            and not _surface_receipt_observed(
                tracker.surface_receipts,
                "rate_pressure",
                coverage_status="fail_observed",
            )
        ):
            pressure_response = _hostile_request_burst(
                session,
                tracker=tracker,
                base_url=base_url,
                target=base_url,
                response_surface_targets=list(surface_targets),
                max_attempts=hostile_burst_attempts,
                reserve_activities=frontloaded_rate_reserve,
                pivot_on_first_confrontation=True,
                request_headers=_request_native_browser_navigation_headers(
                    request_accept_language
                ),
            )
            if pressure_response is not None and (
                _response_indicates_challenge_routing(pressure_response)
                or _response_indicates_pow_surface(pressure_response)
            ):
                execute_challenge_followups(
                    session,
                    seed_response=pressure_response,
                    pow_seed_response=pressure_response
                    if _response_indicates_pow_surface(pressure_response)
                    else None,
                    preserve_followon_targets=True,
                )
                if (
                    not tracker.should_stop()
                    and not _surface_receipt_observed(
                        tracker.surface_receipts,
                        "rate_pressure",
                        coverage_status="fail_observed",
                    )
                ):
                    _hostile_request_burst(
                        session,
                        tracker=tracker,
                        base_url=base_url,
                        target=base_url,
                        response_surface_targets=list(surface_targets),
                        max_attempts=hostile_burst_attempts,
                        request_headers=_request_native_browser_navigation_headers(
                            request_accept_language
                        ),
                    )

        if root_response is not None and (root_serves_challenge or root_serves_pow):
            execute_challenge_followups(
                session,
                seed_response=root_response,
                pow_seed_response=root_response if root_serves_pow else None,
                preserve_followon_targets=root_followon_target_count > 0
                or root_rate_pressure_unmet(),
                suppress_timing_pauses=root_rate_pressure_unmet(),
            )
            if root_rate_pressure_unmet():
                pressure_response = _hostile_request_burst(
                    session,
                    tracker=tracker,
                    base_url=base_url,
                    target=base_url,
                    response_surface_targets=list(surface_targets),
                    max_attempts=hostile_burst_attempts,
                    pivot_on_first_confrontation=True,
                    request_headers=_request_native_browser_navigation_headers(
                        request_accept_language
                    ),
                    pivot_predicate=_response_indicates_request_native_followup_surface,
                )
                if pressure_response is not None and _response_indicates_request_native_followup_surface(
                    pressure_response
                ):
                    later_root_rate_abuse_budget = max(
                        2,
                        min(
                            24,
                            max(
                                2,
                                tracker.realism_tracker.remaining_activity_budget() - 9,
                            ),
                        ),
                    )
                    execute_challenge_followups(
                        session,
                        seed_response=pressure_response,
                        preserve_followon_targets=root_rate_pressure_unmet(),
                        rate_attempt_budget_override=None
                        if root_rate_pressure_unmet()
                        else later_root_rate_abuse_budget,
                        suppress_timing_pauses=root_rate_pressure_unmet(),
                    )
                if root_rate_pressure_unmet():
                    _hostile_request_burst(
                        session,
                        tracker=tracker,
                        base_url=base_url,
                        target=base_url,
                        response_surface_targets=list(surface_targets),
                        max_attempts=hostile_burst_attempts,
                        request_headers=_request_native_browser_navigation_headers(
                            request_accept_language
                        ),
                    )

        cycle_targets = _ordered_unique(
            [
                challenge_target or "",
                pow_target or "",
                redirect_target or "",
            ]
        )
        while not tracker.should_stop() and cycle_targets:
            for cycle_target in cycle_targets:
                if tracker.should_stop():
                    break
                if cycle_target == redirect_target:
                    _perform_request(
                        session,
                        tracker=tracker,
                        base_url=base_url,
                        request_spec=_request_spec(
                            "get",
                            cycle_target,
                            headers={"accept": "application/json"},
                            follow_redirect=True,
                            response_surface_targets=list(surface_targets),
                        ),
                    )
                    continue
                cycle_response = _perform_request(
                    session,
                    tracker=tracker,
                    base_url=base_url,
                    request_spec=_request_spec(
                        "get",
                        cycle_target,
                        response_surface_targets=list(surface_targets),
                    ),
                )
                if cycle_response is None:
                    continue
                execute_challenge_followups(
                    session,
                    seed_response=cycle_response,
                    preserve_followon_targets=any(
                        target and target != cycle_target for target in cycle_targets
                    ),
                )
        if (
            "rate_pressure" in surface_targets
            and not tracker.should_stop()
            and not _surface_receipt_observed(
                tracker.surface_receipts,
                "rate_pressure",
                coverage_status="fail_observed",
            )
        ):
            _hostile_request_burst(
                session,
                tracker=tracker,
                base_url=base_url,
                target=base_url,
                response_surface_targets=list(surface_targets),
                max_attempts=hostile_burst_attempts,
                request_headers=_request_native_browser_navigation_headers(
                    request_accept_language
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
        state["pow_details"] = _evaluate_browser_challenge_state(page)

    return action


def _maze_navigation_page_action(state: dict[str, Any]):
    def action(page) -> None:
        state["before_url"] = page.url
        locator = page.locator("[data-link-kind='maze']").first
        state["link_count"] = locator.count()
        try:
            bootstrap = page.locator("#maze-bootstrap").text_content()
        except Exception as exc:  # pragma: no cover - real page variance is receipted below.
            state["bootstrap_error"] = str(exc)
            bootstrap = ""
        if int(state["link_count"] or 0) < 1:
            state["bootstrap"] = bootstrap or ""
            state["after_url"] = page.url
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
    fallback_browser_country_code = (
        _local_fallback_country_code(tracker.fulfillment_mode)
        if _env_flag_enabled("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE")
        else None
    )
    browser_country_code = (
        str(browser_identity_pool[0].get("country_code") or "").strip().upper()
        if browser_identity_pool
        else (fallback_browser_country_code or None)
    )
    local_browser_forwarding_headers = _resolved_local_trusted_forwarding_headers(
        browser_proxy_url,
        browser_country_code,
        str(plan.get("local_browser_client_ip") or "").strip() or None,
    )
    if local_browser_forwarding_headers:
        browser_proxy_url = None
    browser_transport = resolve_browser_transport_observation(
        tracker.realism_tracker.profile,
        country_code=browser_country_code,
    )
    tracker.realism_tracker.observe_transport(
        transport_profile=str(browser_transport.get("transport_profile") or ""),
        transport_realism_class=str(browser_transport.get("transport_realism_class") or ""),
        transport_emission_basis=str(browser_transport.get("transport_emission_basis") or ""),
        transport_degraded_reason=str(
            browser_transport.get("transport_degraded_reason") or ""
        ),
        user_agent_family=str(browser_transport.get("user_agent_family") or ""),
        accept_language=str(browser_transport.get("accept_language") or ""),
        browser_locale=str(browser_transport.get("browser_locale") or ""),
    )
    tracker.local_trusted_forwarding_headers = dict(local_browser_forwarding_headers)
    start_urls = _normalized_start_urls(seed_inventory)
    if not start_urls:
        raise WorkerConfigError("seed inventory must contain at least one accepted start or hint URL")
    base_url = start_urls[0]
    timeout_ms = max(1000, tracker.max_ms)
    prefer_low_score_not_a_bot = tracker.fulfillment_mode == "browser_automation"

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
        visited_public_targets: set[str] = set()
        public_candidates: list[tuple[str, int]] = []
        browser_discovered_targets: list[str] = []
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
                _response_policy_surface_ids(root_response, surface_targets),
                request_method="get",
                request_target=root_target,
            )
            root_background_paths = _browser_captured_xhr_paths(
                root_response,
                base_url=base_url,
            )
            tracker.realism_tracker.observe_browser_secondary_traffic(
                capture_mode="xhr_capture",
                background_paths=root_background_paths,
            )
            visited_public_targets.add(
                str(getattr(root_response, "url", root_target) or root_target)
            )
            _browser_note_discovery(
                tracker,
                response=root_response,
                page_state=root_state,
                base_url=base_url,
                current_depth=0,
                visited_targets=visited_public_targets,
                public_candidates=public_candidates,
                discovered_targets_accumulator=browser_discovered_targets,
            )
        except Exception as exc:
            tracker.record_failure(
                exc,
                surface_ids=[],
                request_method="get",
                request_target=root_target,
            )
            return tracker.result_payload()

        current_response = root_response
        current_state = dict(root_state)
        current_background_paths = list(root_background_paths)
        current_challenge_state = (
            current_state.get("challenge_state")
            if isinstance(current_state.get("challenge_state"), dict)
            else {}
        )
        current_target = _browser_state_current_target(
            current_challenge_state,
            base_url=base_url,
            fallback_target=str(getattr(root_response, "url", root_target) or root_target),
        )
        root_has_direct_hostile_target = bool(
            _first_matching_target(
                browser_discovered_targets,
                lambda candidate: (
                    _request_path_value(candidate).lower() == "/pow"
                    or _path_contains(candidate, "/maze/", "not-a-bot", "challenge/puzzle")
                ),
            )
        )
        max_public_navigations = max(
            1,
            min(6, tracker.realism_tracker.effective_activity_budget - 1),
        )
        public_navigation_count = 0
        while (
            public_candidates
            and public_navigation_count < max_public_navigations
            and not tracker.should_stop()
            and not root_has_direct_hostile_target
            and not (
                _response_indicates_challenge_routing(current_response)
                or _browser_state_indicates_pow_surface(
                    current_challenge_state,
                    background_paths=current_background_paths,
                )
                or _browser_state_indicates_maze_surface(current_challenge_state)
            )
        ):
            candidate, candidate_depth = public_candidates.pop(0)
            if candidate in visited_public_targets:
                continue
            candidate_state: dict[str, Any] = {}
            try:
                tracker.realism_tracker.prepare_browser_action(
                    browser_session_handle,
                    remaining_ms=tracker.remaining_ms(),
                    country_code=browser_country_code,
                )
                candidate_response = session.fetch(
                    candidate,
                    extra_headers=tracker.next_headers(
                        {"accept-language": str(browser_transport.get("accept_language") or "")}
                    ),
                    page_action=_browser_root_discovery_page_action(candidate_state),
                )
                tracker.record_response(
                    candidate_response,
                    _response_policy_surface_ids(candidate_response, surface_targets),
                    request_method="get",
                    request_target=candidate,
                )
                candidate_background_paths = _browser_captured_xhr_paths(
                    candidate_response,
                    base_url=base_url,
                )
                tracker.realism_tracker.observe_browser_secondary_traffic(
                    capture_mode="xhr_capture",
                    background_paths=candidate_background_paths,
                )
                visited_public_targets.add(
                    str(getattr(candidate_response, "url", candidate) or candidate)
                )
                _browser_note_discovery(
                    tracker,
                    response=candidate_response,
                    page_state=candidate_state,
                    base_url=base_url,
                    current_depth=candidate_depth,
                    visited_targets=visited_public_targets,
                    public_candidates=public_candidates,
                    discovered_targets_accumulator=browser_discovered_targets,
                )
                current_response = candidate_response
                current_state = candidate_state
                current_background_paths = list(candidate_background_paths)
                current_challenge_state = (
                    current_state.get("challenge_state")
                    if isinstance(current_state.get("challenge_state"), dict)
                    else {}
                )
                current_target = _browser_state_current_target(
                    current_challenge_state,
                    base_url=base_url,
                    fallback_target=str(getattr(candidate_response, "url", candidate) or candidate),
                )
                public_navigation_count += 1
            except Exception as exc:
                tracker.record_failure(
                    exc,
                    surface_ids=[],
                    request_method="get",
                    request_target=candidate,
                )
                break

        if (
            not tracker.should_stop()
            and (
                _response_indicates_challenge_routing(root_response)
                or _browser_state_has_followup_challenge_form(current_challenge_state)
            )
            and not _browser_state_indicates_maze_surface(current_challenge_state)
            and _browser_state_has_followup_challenge_form(current_challenge_state)
        ):
            challenge_state: dict[str, Any] = {}
            preserved_pre_challenge_state = _preserved_browser_surface_state(
                current_response,
                current_challenge_state,
            )
            try:
                tracker.realism_tracker.prepare_browser_action(
                    browser_session_handle,
                    remaining_ms=tracker.remaining_ms(),
                    country_code=browser_country_code,
                )
                current_response = session.fetch(
                    current_target,
                    extra_headers=tracker.next_headers(
                        {"accept-language": str(browser_transport.get("accept_language") or "")}
                    ),
                    page_action=_browser_root_served_challenge_page_action(
                        challenge_state,
                        next_submit_headers=lambda: tracker.next_headers(
                            {"accept-language": str(browser_transport.get("accept_language") or "")}
                        ),
                        low_score_not_a_bot=prefer_low_score_not_a_bot,
                    ),
                )
                tracker.record_response(
                    current_response,
                    _response_policy_surface_ids(current_response, surface_targets),
                    request_method="get",
                    request_target=current_target,
                )
                current_background_paths = _browser_captured_xhr_paths(
                    current_response,
                    base_url=base_url,
                )
                tracker.realism_tracker.observe_browser_secondary_traffic(
                    capture_mode="xhr_capture",
                    background_paths=current_background_paths,
                )
                current_state = challenge_state
                current_challenge_state = _merge_browser_challenge_states(
                    current_state.get("challenge_state")
                    if isinstance(current_state.get("challenge_state"), dict)
                    else {},
                    current_state.get("pre_submit_challenge_state")
                    if isinstance(current_state.get("pre_submit_challenge_state"), dict)
                    else {},
                    preserved_pre_challenge_state,
                )
                current_target = _browser_state_current_target(
                    current_challenge_state,
                    base_url=base_url,
                    fallback_target=str(
                        getattr(current_response, "url", current_target) or current_target
                    ),
                )
                _browser_note_discovery(
                    tracker,
                    response=current_response,
                    page_state=current_state,
                    base_url=base_url,
                    current_depth=1,
                    visited_targets=visited_public_targets,
                    public_candidates=public_candidates,
                    discovered_targets_accumulator=browser_discovered_targets,
                )
            except Exception as exc:
                tracker.record_failure(
                    exc,
                    surface_ids=["challenge_routing"] if "challenge_routing" in surface_targets else [],
                    request_method="get",
                    request_target=current_target,
                )

        js_verified_root_revisit_needed = False
        response_serves_pow_surface = _response_indicates_pow_surface(current_response)
        challenge_state_serves_pow_surface = _browser_state_indicates_pow_surface(
            current_challenge_state,
            background_paths=current_background_paths,
        )
        if (
            {"js_verification_execution", "browser_automation_detection"} & surface_targets
            and (
                not tracker.should_stop()
                or response_serves_pow_surface
                or challenge_state_serves_pow_surface
            )
        ):
            pow_target = (
                current_target
                if (response_serves_pow_surface or challenge_state_serves_pow_surface)
                else _browser_discovered_target(
                    {"links": browser_discovered_targets},
                    base_url=base_url,
                    predicate=lambda candidate: _request_path_value(candidate).lower() == "/pow",
                )
            )
            if pow_target is not None:
                pow_state: dict[str, Any] = {}
                try:
                    response = current_response
                    pow_details = current_challenge_state
                    background_paths = list(current_background_paths)
                    if current_target != pow_target:
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
                            _response_policy_surface_ids(response, surface_targets),
                            request_method="get",
                            request_target=pow_target,
                        )
                        background_paths = _browser_captured_xhr_paths(
                            response,
                            base_url=base_url,
                        )
                        tracker.realism_tracker.observe_browser_secondary_traffic(
                            capture_mode="xhr_capture",
                            background_paths=background_paths,
                        )
                        pow_details = (
                            pow_state.get("pow_details")
                            if isinstance(pow_state.get("pow_details"), dict)
                            else {}
                        )
                    response_status = int(response.status)
                    js_executed = _browser_state_indicates_pow_surface(
                        pow_details,
                        background_paths=background_paths,
                    )
                    js_verified_root_revisit_needed = bool(
                        js_executed
                        and not _browser_state_has_followup_challenge_form(current_challenge_state)
                        and not _browser_state_indicates_maze_surface(current_challenge_state)
                    )
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

        if js_verified_root_revisit_needed:
            revisit_state: dict[str, Any] = {}
            revisit_target = root_target
            try:
                tracker.realism_tracker.prepare_browser_action(
                    browser_session_handle,
                    remaining_ms=tracker.remaining_ms(),
                    country_code=browser_country_code,
                )
                revisit_response = session.fetch(
                    revisit_target,
                    extra_headers=tracker.next_headers(
                        {"accept-language": str(browser_transport.get("accept_language") or "")}
                    ),
                    page_action=_browser_root_discovery_page_action(revisit_state),
                )
                tracker.record_response(
                    revisit_response,
                    _response_policy_surface_ids(revisit_response, surface_targets),
                    request_method="get",
                    request_target=revisit_target,
                )
                current_background_paths = _browser_captured_xhr_paths(
                    revisit_response,
                    base_url=base_url,
                )
                tracker.realism_tracker.observe_browser_secondary_traffic(
                    capture_mode="xhr_capture",
                    background_paths=current_background_paths,
                )
                current_response = revisit_response
                current_state = revisit_state
                current_challenge_state = _merge_browser_challenge_states(
                    current_state.get("challenge_state")
                    if isinstance(current_state.get("challenge_state"), dict)
                    else {},
                    current_state.get("pre_submit_challenge_state")
                    if isinstance(current_state.get("pre_submit_challenge_state"), dict)
                    else {},
                )
                current_target = _browser_state_current_target(
                    current_challenge_state,
                    base_url=base_url,
                    fallback_target=str(
                        getattr(revisit_response, "url", revisit_target) or revisit_target
                    ),
                )
                _browser_note_discovery(
                    tracker,
                    response=current_response,
                    page_state=current_state,
                    base_url=base_url,
                    current_depth=0,
                    visited_targets=visited_public_targets,
                    public_candidates=public_candidates,
                    discovered_targets_accumulator=browser_discovered_targets,
                )
                if (
                    _browser_state_has_followup_challenge_form(current_challenge_state)
                    and not _browser_state_indicates_maze_surface(current_challenge_state)
                ):
                    challenge_state: dict[str, Any] = {}
                    tracker.realism_tracker.prepare_browser_action(
                        browser_session_handle,
                        remaining_ms=tracker.remaining_ms(),
                        country_code=browser_country_code,
                    )
                    current_response = session.fetch(
                        current_target,
                        extra_headers=tracker.next_headers(
                            {"accept-language": str(browser_transport.get("accept_language") or "")}
                        ),
                        page_action=_browser_root_served_challenge_page_action(
                            challenge_state,
                            next_submit_headers=lambda: tracker.next_headers(
                                {"accept-language": str(browser_transport.get("accept_language") or "")}
                            ),
                            low_score_not_a_bot=prefer_low_score_not_a_bot,
                        ),
                    )
                    tracker.record_response(
                        current_response,
                        _response_policy_surface_ids(current_response, surface_targets),
                        request_method="get",
                        request_target=current_target,
                    )
                    current_background_paths = _browser_captured_xhr_paths(
                        current_response,
                        base_url=base_url,
                    )
                    tracker.realism_tracker.observe_browser_secondary_traffic(
                        capture_mode="xhr_capture",
                        background_paths=current_background_paths,
                    )
                    current_state = challenge_state
                    current_challenge_state = _merge_browser_challenge_states(
                        current_state.get("challenge_state")
                        if isinstance(current_state.get("challenge_state"), dict)
                        else {},
                        current_state.get("pre_submit_challenge_state")
                        if isinstance(current_state.get("pre_submit_challenge_state"), dict)
                        else {},
                    )
                    current_target = _browser_state_current_target(
                        current_challenge_state,
                        base_url=base_url,
                        fallback_target=str(
                            getattr(current_response, "url", current_target) or current_target
                        ),
                    )
                    _browser_note_discovery(
                        tracker,
                        response=current_response,
                        page_state=current_state,
                        base_url=base_url,
                        current_depth=1,
                        visited_targets=visited_public_targets,
                        public_candidates=public_candidates,
                        discovered_targets_accumulator=browser_discovered_targets,
                    )
            except Exception as exc:
                tracker.record_failure(
                    exc,
                    surface_ids=[
                        surface_id
                        for surface_id in (
                            "js_verification_execution",
                            "challenge_routing",
                            "maze_navigation",
                        )
                        if surface_id in surface_targets
                    ],
                    request_method="get",
                    request_target=revisit_target,
                )

        if "maze_navigation" in surface_targets:
            inline_maze_passed = (
                int(current_state.get("maze_link_count") or 0) > 0
                and (
                    bool(current_state.get("maze_click_attempted"))
                    or (
                        bool(current_state.get("maze_after_url"))
                        and str(current_state.get("maze_after_url"))
                        != str(current_state.get("maze_before_url") or "")
                    )
                    or bool(str(current_state.get("maze_bootstrap_after_click") or ""))
                )
            )
            maze_surface_observed = (
                _response_indicates_maze_surface(current_response)
                or _browser_state_indicates_maze_surface(current_challenge_state)
            )
            if inline_maze_passed:
                _record_browser_surface_result(
                    tracker,
                    surface_id="maze_navigation",
                    coverage_status="pass_observed",
                    request_target=_request_path_value(
                        str(current_state.get("maze_after_url") or current_target) or current_target
                    ),
                            response_status=int(getattr(current_response, "status", 200) or 200),
                )
                return tracker.result_payload()
            if tracker.should_stop() and not maze_surface_observed:
                return tracker.result_payload()
            current_page_maze_target = _browser_discovered_target(
                current_state if isinstance(current_state, dict) else {"links": []},
                base_url=base_url,
                predicate=_is_maze_navigation_target,
            )
            discovered_maze_target = current_page_maze_target or _browser_discovered_target(
                {"links": browser_discovered_targets},
                base_url=base_url,
                predicate=_is_maze_navigation_target,
            )
            maze_target = (
                discovered_maze_target
                or (
                    current_target
                    if maze_surface_observed
                    else None
                )
            )
            if maze_target is not None:
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
                        _response_policy_surface_ids(response, surface_targets),
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
                    bootstrap_text = str(maze_state.get("bootstrap") or "")
                    maze_request_path = _request_path_value(maze_target).lower()
                    current_request_path = _request_path_value(current_target).lower()
                    entered_maze_surface = (
                        bool(bootstrap_text)
                        and _is_maze_navigation_target(maze_request_path)
                        and maze_request_path != current_request_path
                    )
                    maze_passed = (
                        (
                            int(maze_state.get("link_count") or 0) > 0
                            and bool(after_url)
                            and after_url != before_url
                        )
                        or entered_maze_surface
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
