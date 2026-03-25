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


def _import_scrapling() -> tuple[Any, Any, Any, Any, Any]:
    from scrapling.fetchers import DynamicSession, FetcherSession, StealthySession
    from scrapling.spiders import Request, Spider

    return FetcherSession, DynamicSession, StealthySession, Request, Spider


class WorkerConfigError(ValueError):
    """Raised when required worker inputs are missing or invalid."""


SCRAPLING_FULFILLMENT_MODES = {"crawler", "bulk_scraper", "http_agent"}


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


def _normalize_runtime_paths(raw_paths: Any) -> dict[str, str]:
    required_keys = (
        "public_search",
        "not_a_bot_checkbox",
        "challenge_submit",
        "pow_verify",
        "tarpit_progress",
    )
    if not isinstance(raw_paths, dict):
        raise WorkerConfigError("worker_plan runtime_paths must be an object")
    normalized: dict[str, str] = {}
    for key in required_keys:
        value = str(raw_paths.get(key) or "").strip()
        if not value:
            raise WorkerConfigError(f"worker_plan runtime_paths.{key} must be a non-empty string")
        normalized[key] = value
    return normalized


def _absolute_target(base_url: str, raw_target: str) -> str:
    if str(raw_target).startswith("http://") or str(raw_target).startswith("https://"):
        return str(raw_target)
    return urljoin(base_url, str(raw_target))


def _route_with_query(path: str, query: str) -> str:
    separator = "&" if "?" in path else "?"
    return f"{path}{separator}{query}"


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
    transport: str = "request_native",
    browser_action: str | None = None,
) -> dict[str, Any]:
    spec: dict[str, Any] = {
        "method": method,
        "target": target,
        "headers": dict(headers or {}),
        "follow_redirect": follow_redirect,
        "transport": str(transport or "request_native"),
    }
    if surface_ids:
        spec["surface_ids"] = list(surface_ids)
    if cookies:
        spec["cookies"] = dict(cookies)
    if data is not None:
        spec["data"] = data
    if json_body is not None:
        spec["json"] = json_body
    if browser_action:
        spec["browser_action"] = str(browser_action)
    return spec


def _request_path_value(raw_target: str) -> str:
    parsed = urlsplit(str(raw_target))
    path = parsed.path or "/"
    if parsed.query:
        return f"{path}?{parsed.query}"
    return path


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
) -> dict[str, Any]:
    return {
        "impersonate": "chrome",
        "stealthy_headers": True,
        "follow_redirects": False,
        "timeout": timeout_seconds,
        "retries": 1,
        "headers": {"accept": accept_header},
    }


def _dynamic_session_kwargs(
    *,
    timeout_ms: int,
    accept_header: str,
) -> dict[str, Any]:
    return {
        "timeout": int(timeout_ms),
        "wait": 250,
        "network_idle": True,
        "disable_resources": True,
        "extra_headers": {"accept": accept_header},
    }


def _stealth_session_kwargs(
    *,
    timeout_ms: int,
    accept_header: str,
) -> dict[str, Any]:
    return {
        "timeout": int(timeout_ms),
        "wait": 250,
        "network_idle": True,
        "disable_resources": True,
        "solve_cloudflare": True,
        "block_webrtc": True,
        "hide_canvas": True,
        "allow_webgl": True,
        "extra_headers": {"accept": accept_header},
    }


def _browser_session_strategy_for_mode(fulfillment_mode: str) -> str:
    return {
        "crawler": "request_native",
        "bulk_scraper": "dynamic",
        "http_agent": "stealth",
    }.get(str(fulfillment_mode or "").strip(), "request_native")


def _browser_session_class_for_strategy(
    strategy: str,
    *,
    dynamic_session_cls: Any,
    stealthy_session_cls: Any,
) -> Any | None:
    if strategy == "dynamic":
        return dynamic_session_cls
    if strategy == "stealth":
        return stealthy_session_cls
    return None


def _browser_session_kwargs_for_strategy(
    strategy: str,
    *,
    timeout_ms: int,
    accept_header: str,
) -> dict[str, Any]:
    if strategy == "dynamic":
        kwargs = _dynamic_session_kwargs(timeout_ms=timeout_ms, accept_header=accept_header)
        kwargs["wait"] = 0
        kwargs["network_idle"] = False
        return kwargs
    if strategy == "stealth":
        kwargs = _stealth_session_kwargs(timeout_ms=timeout_ms, accept_header=accept_header)
        # These first browser-backed owned surfaces are Shuma DOM challenges, not Cloudflare pages.
        # Keep the broader stealth foundation intact, but do not spend the request budget on irrelevant solving attempts here.
        kwargs["wait"] = 0
        kwargs["network_idle"] = False
        kwargs["solve_cloudflare"] = False
        return kwargs
    raise WorkerConfigError(f"unsupported browser session strategy: {strategy}")


def _playwright_attr_value(value: Any, attr_name: str) -> Any:
    attr = getattr(value, attr_name, None)
    if callable(attr):
        return attr()
    return attr


def _build_wrong_challenge_output(current_output: str) -> str:
    baseline = str(current_output or "") or "0" * 16
    first = "1" if baseline[:1] == "0" else "0"
    return f"{first}{baseline[1:]}"


def _browser_cookie_params(target_url: str, raw_cookies: Any) -> list[dict[str, Any]] | None:
    if not raw_cookies:
        return None
    if isinstance(raw_cookies, list):
        return raw_cookies
    if not isinstance(raw_cookies, dict):
        return None
    return [
        {
            "name": str(name),
            "value": str(value),
            "url": target_url,
        }
        for name, value in raw_cookies.items()
        if str(name).strip()
    ] or None


def _perform_human_like_checkbox_activation(page: Any, checkbox: Any) -> None:
    bounds = checkbox.bounding_box()
    if not bounds:
        raise RuntimeError("browser_not_a_bot_checkbox_bounds_missing")
    target_x = float(bounds["x"]) + (float(bounds["width"]) / 2.0)
    target_y = float(bounds["y"]) + (float(bounds["height"]) / 2.0)
    entry_x = max(8.0, target_x - 48.0)
    entry_y = max(8.0, target_y + 26.0)
    approach_x = target_x - 10.0
    approach_y = target_y + 6.0

    page.mouse.move(entry_x, entry_y)
    page.wait_for_timeout(30)
    page.mouse.move(approach_x, approach_y, steps=8)
    page.wait_for_timeout(20)
    page.mouse.move(target_x, target_y, steps=8)
    page.wait_for_timeout(20)
    page.mouse.down()
    page.wait_for_timeout(20)
    page.mouse.up()


def _build_not_a_bot_page_action(evidence: dict[str, Any]):
    def page_action(page: Any) -> None:
        checkbox = page.locator("#not-a-bot-checkbox")
        if not checkbox.is_visible():
            evidence["error"] = "browser_not_a_bot_checkbox_missing"
            return
        with page.expect_request(
            lambda request: (
                str(_playwright_attr_value(request, "method") or "").upper() == "POST"
                and "/challenge/not-a-bot-checkbox"
                in str(_playwright_attr_value(request, "url") or "")
            ),
            timeout=15_000,
        ) as submit_request_info:
            with page.expect_response(
                lambda response: (
                    str(_playwright_attr_value(_playwright_attr_value(response, "request"), "method") or "").upper()
                    == "POST"
                    and "/challenge/not-a-bot-checkbox"
                    in str(_playwright_attr_value(response, "url") or "")
                ),
                timeout=15_000,
            ) as submit_response_info:
                _perform_human_like_checkbox_activation(page, checkbox)
        submit_request = submit_request_info.value
        submit_response = submit_response_info.value
        submit_body = str(_playwright_attr_value(submit_request, "post_data") or "")
        submit_status = int(_playwright_attr_value(submit_response, "status") or 0)
        evidence["request_method"] = "POST"
        evidence["request_target"] = str(_playwright_attr_value(submit_request, "url") or "")
        evidence["response_status"] = submit_status
        evidence["additional_response_statuses"] = [submit_status]
        evidence["coverage_status"] = (
            "pass_observed"
            if "checked=1" in submit_body and "telemetry=" in submit_body and 200 <= submit_status < 400
            else "fail_observed"
        )

    return page_action


def _build_puzzle_fail_page_action(evidence: dict[str, Any]):
    def page_action(page: Any) -> None:
        heading = page.locator("h2")
        output_grid = page.locator("#challenge-output-grid")
        output_field = page.locator("#challenge-output")
        if heading.count() < 1:
            evidence["error"] = "browser_puzzle_heading_missing"
            return
        if output_grid.count() < 1:
            evidence["error"] = "browser_puzzle_output_grid_missing"
            return
        if output_field.count() < 1:
            evidence["error"] = "browser_puzzle_output_field_missing"
            return
        wrong_output = _build_wrong_challenge_output(output_field.input_value())
        output_field.evaluate("(node, value) => { node.value = value; }", wrong_output)
        with page.expect_request(
            lambda request: (
                str(_playwright_attr_value(request, "method") or "").upper() == "POST"
                and "/challenge/puzzle" in str(_playwright_attr_value(request, "url") or "")
            ),
            timeout=15_000,
        ) as submit_request_info:
            with page.expect_response(
                lambda response: (
                    str(_playwright_attr_value(_playwright_attr_value(response, "request"), "method") or "").upper()
                    == "POST"
                    and "/challenge/puzzle" in str(_playwright_attr_value(response, "url") or "")
                ),
                timeout=15_000,
            ) as submit_response_info:
                page.click("button[type='submit']")
        submit_request = submit_request_info.value
        submit_response = submit_response_info.value
        submit_status = int(_playwright_attr_value(submit_response, "status") or 0)
        page_body = str(page.content() or "")
        evidence["request_method"] = "POST"
        evidence["request_target"] = str(_playwright_attr_value(submit_request, "url") or "")
        evidence["response_status"] = submit_status
        evidence["coverage_status"] = (
            "fail_observed"
            if (
                submit_status >= 400
                or 'data-link-kind="maze"' in page_body
                or "maze-nav-grid" in page_body
                or "Incorrect" in page_body
                or "Request new challenge" in page_body
            )
            else "pass_observed"
        )

    return page_action


def _browser_page_action(browser_action: str, evidence: dict[str, Any]):
    if browser_action == "not_a_bot_pass":
        return _build_not_a_bot_page_action(evidence)
    if browser_action == "puzzle_fail":
        return _build_puzzle_fail_page_action(evidence)
    raise WorkerConfigError(f"unsupported browser action: {browser_action}")


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
            self.runtime_paths = _normalize_runtime_paths(plan.get("runtime_paths"))
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
            if "challenge_routing" in self.surface_targets:
                challenge_probe = _absolute_target(
                    self.start_urls[0] if self.start_urls else "",
                    _route_with_query(
                        self.runtime_paths["public_search"],
                        "q=scrapling-crawler-probe",
                    ),
                )
                if challenge_probe and challenge_probe not in self.start_urls:
                    self.start_urls.insert(0, challenge_probe)
            super().__init__(crawldir=str(crawldir), interval=0.0)

        def configure_sessions(self, manager) -> None:
            timeout_seconds = max(1.0, min(30.0, self.max_ms / 1000.0))
            manager.add(
                "default",
                fetcher_session_cls(**_request_native_session_kwargs(
                    timeout_seconds=timeout_seconds,
                    accept_header="*/*",
                )),
            )

        def _should_stop(self) -> bool:
            return (
                self.requests_observed >= self.max_requests
                or self.bytes_observed >= self.max_bytes
                or time.monotonic() >= self.deadline
            )

        def _next_headers(self) -> dict[str, str]:
            self.request_sequence += 1
            return _signed_headers(
                self.sim_telemetry_secret,
                run_id=self.run_id,
                profile=self.sim_profile,
                lane=self.lane,
                fulfillment_mode=self.fulfillment_mode,
                seq=self.request_sequence,
            )

        def _record_rejection(self, reason: str | None) -> None:
            reason_key = str(reason or "malformed_url").strip() or "malformed_url"
            self.scope_rejections[reason_key] += 1

        def _surface_ids_for_response(self, response) -> list[str]:
            path = urlsplit(str(getattr(response, "url", ""))).path or "/"
            public_search_path = urlsplit(self.runtime_paths["public_search"]).path or "/"
            if path == public_search_path:
                return [
                    surface_id
                    for surface_id in (
                        "challenge_routing",
                        "rate_pressure",
                        "geo_ip_policy",
                    )
                    if surface_id in self.surface_targets
                ]
            if "public_path_traversal" in self.surface_targets:
                return ["public_path_traversal"]
            return []

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
                yield request_cls(
                    url,
                    sid="default",
                    meta={"depth": 0},
                    headers=self._next_headers(),
                )

        async def parse(self, response) -> AsyncGenerator[Any, None]:
            self.requests_observed += 1
            self.bytes_observed += len(response.body)
            self.last_response_status = int(response.status)
            surface_ids = self._surface_ids_for_response(response)
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

            depth = int((response.meta or {}).get("depth") or 0)
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
                        yield response.follow(
                            normalized_url,
                            meta={"depth": next_depth},
                            headers=self._next_headers(),
                        )
                return

            sitemap_targets = [value.strip() for value in response.css("loc::text").getall()]
            if sitemap_targets:
                for raw_target in sitemap_targets:
                    if not raw_target or next_depth > self.max_depth:
                        continue
                    allowed, normalized_url = self._allowed_request(
                        response.url,
                        raw_target,
                        is_redirect=False,
                    )
                    if allowed and normalized_url:
                        yield response.follow(
                            normalized_url,
                            meta={"depth": next_depth},
                            headers=self._next_headers(),
                        )
                        if self._should_stop():
                            self.pause()
                            return
                return

            for href in response.css("a::attr(href)").getall():
                candidate = urljoin(response.url, href)
                if next_depth > self.max_depth:
                    continue
                allowed, normalized_url = self._allowed_request(
                    response.url,
                    candidate,
                    is_redirect=False,
                )
                if allowed and normalized_url:
                    yield response.follow(
                        normalized_url,
                        meta={"depth": next_depth},
                        headers=self._next_headers(),
                    )
                    if self._should_stop():
                        self.pause()
                        return

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
            self.generated_requests >= self.max_requests
            or self.bytes_observed >= self.max_bytes
            or time.monotonic() >= self.deadline
        )

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
    ) -> tuple[bool, str | None]:
        if is_redirect:
            decision = shared_host_scope.evaluate_redirect_target(
                current_url, raw_target, self.descriptor
            )
        else:
            decision = shared_host_scope.evaluate_url_candidate(raw_target, self.descriptor)
        if not decision.allowed or not decision.normalized_url:
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

    def record_additional_statuses(self, statuses: list[int] | tuple[int, ...]) -> None:
        for raw_status in statuses:
            status = int(raw_status)
            self.generated_requests += 1
            self.last_response_status = status
            self.response_status_count[f"status_{status}"] += 1

    def result_payload(self) -> dict[str, Any]:
        failure_class = "transport" if self.last_transport_error else None
        return {
            "schema_version": "adversary-sim-scrapling-worker-result.v1",
            "run_id": self.run_id,
            "tick_id": self.tick_id,
            "lane": self.lane,
            "fulfillment_mode": self.fulfillment_mode,
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
            "scope_rejections": dict(sorted(self.scope_rejections.items())),
            "surface_receipts": _render_surface_receipts(self.surface_receipts),
        }


def _execute_request_sequence(
    session: Any,
    *,
    tracker: _DirectPersonaTracker,
    base_url: str,
    requests: list[dict[str, Any]],
    browser_session: Any | None = None,
) -> None:
    for request_spec in requests:
        if tracker.should_stop():
            break
        method_name = str(request_spec.get("method") or "").strip().lower()
        raw_target = str(request_spec.get("target") or "").strip()
        surface_ids = [str(value) for value in list(request_spec.get("surface_ids") or []) if str(value).strip()]
        transport = str(request_spec.get("transport") or "request_native").strip().lower()
        if not method_name or not raw_target:
            continue
        allowed, normalized_url = tracker.allowed_request(
            base_url,
            raw_target,
            is_redirect=False,
        )
        if not allowed or not normalized_url:
            continue
        if transport != "request_native":
            if browser_session is None:
                tracker.record_failure(
                    WorkerConfigError(f"browser transport {transport} requires an active browser session"),
                    surface_ids=surface_ids,
                    request_method=method_name,
                    request_target=normalized_url or raw_target,
                )
                break
            browser_action = str(request_spec.get("browser_action") or "").strip()
            evidence: dict[str, Any] = {}
            try:
                response = browser_session.fetch(
                    normalized_url,
                    extra_headers=tracker.next_headers(dict(request_spec.get("headers") or {})),
                    cookies=_browser_cookie_params(normalized_url, request_spec.get("cookies")),
                    page_action=_browser_page_action(browser_action, evidence),
                )
                tracker.record_response(response)
                tracker.record_additional_statuses(list(evidence.get("additional_response_statuses") or []))
                coverage_status = str(evidence.get("coverage_status") or "").strip()
                if coverage_status:
                    _record_surface_receipt(
                        tracker.surface_receipts,
                        surface_ids=surface_ids,
                        coverage_status=coverage_status,
                        request_method=str(evidence.get("request_method") or method_name),
                        request_target=str(evidence.get("request_target") or normalized_url),
                        response_status=int(evidence["response_status"]) if evidence.get("response_status") is not None else None,
                    )
                elif surface_ids:
                    raise RuntimeError(str(evidence.get("error") or "browser_surface_outcome_missing"))
            except Exception as exc:
                tracker.record_failure(
                    exc,
                    surface_ids=surface_ids,
                    request_method=method_name,
                    request_target=normalized_url or raw_target,
                )
                break
            continue
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
        except Exception as exc:
            tracker.record_failure(
                exc,
                surface_ids=surface_ids,
                request_method=method_name,
                request_target=normalized_url or raw_target,
            )
            break


def _bulk_scraper_request_urls(start_urls: list[str]) -> list[str]:
    if not start_urls:
        return []
    base_url = start_urls[0]
    ordered = [
        urljoin(base_url, "/catalog?page=1"),
        urljoin(base_url, "/catalog?page=2"),
        urljoin(base_url, "/detail/1"),
        urljoin(base_url, "/detail/2"),
    ]
    deduped: list[str] = []
    for target in ordered:
        if target not in deduped:
            deduped.append(target)
    return deduped


def _bulk_scraper_owned_surface_requests(
    base_url: str,
    *,
    surface_targets: set[str],
    runtime_paths: dict[str, str],
) -> list[dict[str, Any]]:
    requests: list[dict[str, Any]] = []
    if {"challenge_routing", "rate_pressure", "geo_ip_policy"} & surface_targets:
        requests.append(
            _request_spec(
                "get",
                _absolute_target(
                    base_url,
                    _route_with_query(
                        runtime_paths["public_search"],
                        "q=scrapling-bulk-scraper",
                    ),
                ),
                surface_ids=["challenge_routing", "rate_pressure", "geo_ip_policy"],
                headers={"accept": "application/json"},
            )
        )
    if "not_a_bot_submit" in surface_targets:
        requests.append(
            _request_spec(
                "get",
                _absolute_target(base_url, runtime_paths["not_a_bot_checkbox"]),
                surface_ids=["not_a_bot_submit"],
                headers={"accept": "text/html,application/xhtml+xml"},
                transport="dynamic",
                browser_action="not_a_bot_pass",
            )
        )
    if "puzzle_submit_or_escalation" in surface_targets:
        requests.append(
            _request_spec(
                "get",
                _absolute_target(base_url, runtime_paths["challenge_submit"]),
                surface_ids=["puzzle_submit_or_escalation"],
                headers={"accept": "text/html,application/xhtml+xml"},
                transport="dynamic",
                browser_action="puzzle_fail",
            )
        )
    return requests


def _http_agent_request_sequence(
    base_url: str,
    *,
    tracker: _DirectPersonaTracker,
    surface_targets: set[str],
    runtime_paths: dict[str, str],
) -> list[dict[str, Any]]:
    cookies = {"shuma_agent_mode": "http_agent"}
    requests = [
        _request_spec(
            "get",
            urljoin(base_url, "/agent/ping?mode=http_agent"),
            headers={"accept": "application/json"},
            cookies=cookies,
        ),
    ]
    if {"challenge_routing", "rate_pressure", "geo_ip_policy"} & surface_targets:
        requests.append(
            _request_spec(
                "get",
                _absolute_target(
                    base_url,
                    _route_with_query(
                        runtime_paths["public_search"],
                        "q=scrapling-http-agent",
                    ),
                ),
                surface_ids=["challenge_routing", "rate_pressure", "geo_ip_policy"],
                headers={"accept": "application/json"},
                cookies=cookies,
            )
        )
    if "not_a_bot_submit" in surface_targets:
        requests.append(
            _request_spec(
                "get",
                _absolute_target(base_url, runtime_paths["not_a_bot_checkbox"]),
                surface_ids=["not_a_bot_submit"],
                cookies=cookies,
                headers={"accept": "text/html,application/xhtml+xml"},
                transport="stealth",
                browser_action="not_a_bot_pass",
            )
        )
    if "puzzle_submit_or_escalation" in surface_targets:
        requests.append(
            _request_spec(
                "get",
                _absolute_target(base_url, runtime_paths["challenge_submit"]),
                surface_ids=["puzzle_submit_or_escalation"],
                cookies=cookies,
                headers={"accept": "text/html,application/xhtml+xml"},
                transport="stealth",
                browser_action="puzzle_fail",
            )
        )
    if "pow_verify_abuse" in surface_targets:
        requests.append(
            _request_spec(
                "post",
                _absolute_target(base_url, runtime_paths["pow_verify"]),
                surface_ids=["pow_verify_abuse"],
                headers={
                    "accept": "application/json",
                    "content-type": "application/json",
                },
                cookies=cookies,
                json_body={"seed": "invalid-seed", "nonce": "invalid-nonce"},
            )
        )
    if "tarpit_progress_abuse" in surface_targets:
        requests.append(
            _request_spec(
                "post",
                _absolute_target(base_url, runtime_paths["tarpit_progress"]),
                surface_ids=["tarpit_progress_abuse"],
                headers={
                    "accept": "application/json",
                    "content-type": "application/json",
                },
                cookies=cookies,
                json_body={
                    "token": "invalid",
                    "operation_id": "invalid",
                    "proof_nonce": "invalid",
                },
            )
        )
    requests.extend(
        [
            _request_spec(
                "post",
                urljoin(base_url, "/agent/submit"),
                headers={
                    "accept": "application/json",
                    "content-type": "application/json",
                },
                cookies=cookies,
                json_body={
                    "mode": "http_agent",
                    "run_id": tracker.run_id,
                    "tick_id": tracker.tick_id,
                },
            ),
            _request_spec(
                "put",
                urljoin(base_url, "/agent/update"),
                headers={
                    "accept": "application/json",
                    "content-type": "application/json",
                },
                cookies=cookies,
                json_body={
                    "mode": "http_agent",
                    "request_sequence": 3,
                },
            ),
            _request_spec(
                "get",
                urljoin(base_url, "/agent/redirect"),
                headers={"accept": "application/json"},
                cookies=cookies,
                follow_redirect=True,
            ),
        ]
    )
    return requests


def _execute_bulk_scraper_persona(
    fetcher_session_cls: Any,
    dynamic_session_cls: Any,
    stealthy_session_cls: Any,
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
    runtime_paths = _normalize_runtime_paths(plan.get("runtime_paths"))
    start_urls = _normalized_start_urls(seed_inventory)
    request_targets = _bulk_scraper_request_urls(start_urls)
    visited: set[str] = set()
    browser_strategy = _browser_session_strategy_for_mode(str(plan.get("fulfillment_mode") or ""))
    browser_session_cls = _browser_session_class_for_strategy(
        browser_strategy,
        dynamic_session_cls=dynamic_session_cls,
        stealthy_session_cls=stealthy_session_cls,
    )
    with fetcher_session_cls(
        **_request_native_session_kwargs(
            timeout_seconds=max(1.0, min(30.0, tracker.max_ms / 1000.0)),
            accept_header="text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        ),
    ) as session:
        for raw_target in request_targets:
            if tracker.should_stop():
                break
            allowed, normalized_url = tracker.allowed_request(
                start_urls[0] if start_urls else raw_target,
                raw_target,
                is_redirect=False,
            )
            if not allowed or not normalized_url or normalized_url in visited:
                continue
            try:
                response = session.get(
                    normalized_url,
                    headers=tracker.next_headers(
                        {"accept-language": "en-GB,en;q=0.8"}
                    ),
                )
                visited.add(normalized_url)
                tracker.record_response(response, ["public_path_traversal"])
                if tracker.should_stop():
                    break
                for href in response.css("a::attr(href)").getall():
                    if tracker.should_stop():
                        break
                    candidate = urljoin(response.url, href)
                    allowed, discovered_url = tracker.allowed_request(
                        response.url,
                        candidate,
                        is_redirect=False,
                    )
                    if not allowed or not discovered_url or discovered_url in visited:
                        continue
                    discovered_response = session.get(
                        discovered_url,
                        headers=tracker.next_headers(
                            {"accept-language": "en-GB,en;q=0.8"}
                        ),
                    )
                    visited.add(discovered_url)
                    tracker.record_response(discovered_response, ["public_path_traversal"])
            except Exception as exc:
                tracker.record_failure(
                    exc,
                    surface_ids=["public_path_traversal"],
                    request_method="get",
                    request_target=normalized_url or raw_target,
                )
                break
        if not tracker.should_stop() and start_urls:
            if browser_session_cls is not None:
                with browser_session_cls(
                    **_browser_session_kwargs_for_strategy(
                        browser_strategy,
                        timeout_ms=tracker.max_ms,
                        accept_header="text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                    )
                ) as browser_session:
                    _execute_request_sequence(
                        session,
                        tracker=tracker,
                        base_url=start_urls[0],
                        requests=_bulk_scraper_owned_surface_requests(
                            start_urls[0],
                            surface_targets=surface_targets,
                            runtime_paths=runtime_paths,
                        ),
                        browser_session=browser_session,
                    )
            else:
                _execute_request_sequence(
                    session,
                    tracker=tracker,
                    base_url=start_urls[0],
                    requests=_bulk_scraper_owned_surface_requests(
                        start_urls[0],
                        surface_targets=surface_targets,
                        runtime_paths=runtime_paths,
                    ),
                )
    return tracker.result_payload()


def _execute_http_agent_persona(
    fetcher_session_cls: Any,
    dynamic_session_cls: Any,
    stealthy_session_cls: Any,
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
    runtime_paths = _normalize_runtime_paths(plan.get("runtime_paths"))
    start_urls = _normalized_start_urls(seed_inventory)
    if not start_urls:
        raise WorkerConfigError("seed inventory must contain at least one accepted start or hint URL")
    base_url = start_urls[0]
    requests = _http_agent_request_sequence(
        base_url,
        tracker=tracker,
        surface_targets=surface_targets,
        runtime_paths=runtime_paths,
    )
    browser_strategy = _browser_session_strategy_for_mode(str(plan.get("fulfillment_mode") or ""))
    browser_session_cls = _browser_session_class_for_strategy(
        browser_strategy,
        dynamic_session_cls=dynamic_session_cls,
        stealthy_session_cls=stealthy_session_cls,
    )
    with fetcher_session_cls(
        **_request_native_session_kwargs(
            timeout_seconds=max(1.0, min(30.0, tracker.max_ms / 1000.0)),
            accept_header="application/json",
        ),
    ) as session:
        if browser_session_cls is not None:
            with browser_session_cls(
                **_browser_session_kwargs_for_strategy(
                    browser_strategy,
                    timeout_ms=tracker.max_ms,
                    accept_header="text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                )
            ) as browser_session:
                _execute_request_sequence(
                    session,
                    tracker=tracker,
                    base_url=base_url,
                    requests=requests,
                    browser_session=browser_session,
                )
        else:
            _execute_request_sequence(
                session,
                tracker=tracker,
                base_url=base_url,
                requests=requests,
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
                "worker_plan fulfillment_mode must be one of crawler, bulk_scraper, http_agent"
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
        _normalize_runtime_paths(plan.get("runtime_paths"))
        if not sim_telemetry_secret.strip():
            raise WorkerConfigError("SHUMA_SIM_TELEMETRY_SECRET is required for Scrapling worker tagging")

        descriptor_payload = _load_json(scope_descriptor_path)
        descriptor = shared_host_scope.descriptor_from_payload(descriptor_payload)
        seed_inventory = _load_json(seed_inventory_path)
        if not _normalized_start_urls(seed_inventory):
            raise WorkerConfigError("seed inventory must contain at least one accepted start or hint URL")

        fetcher_session_cls, dynamic_session_cls, stealthy_session_cls, request_cls, spider_cls = _import_scrapling()
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
                "scope_rejections": dict(sorted(spider.scope_rejections.items())),
                "surface_receipts": _render_surface_receipts(spider.surface_receipts),
            }
        if fulfillment_mode == "bulk_scraper":
            return _execute_bulk_scraper_persona(
                fetcher_session_cls,
                dynamic_session_cls,
                stealthy_session_cls,
                plan=plan,
                descriptor=descriptor,
                seed_inventory=seed_inventory,
                sim_telemetry_secret=sim_telemetry_secret,
            )
        return _execute_http_agent_persona(
            fetcher_session_cls,
            dynamic_session_cls,
            stealthy_session_cls,
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
