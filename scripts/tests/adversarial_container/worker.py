#!/usr/bin/env python3
"""Container-side black-box adversary worker."""

from __future__ import annotations

import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Callable, Dict, List

from scripts.tests.frontier_action_contract import (
    FrontierActionContractError,
    FrontierActionValidationError,
    load_frontier_action_contract,
    resolve_frontier_actions,
)
from scripts.tests.frontier_capability_envelope import (
    parse_action_capability_envelopes,
    validate_action_capability_envelopes,
)


FORBIDDEN_ENV_PREFIXES = ("SHUMA_",)
FORBIDDEN_ENV_KEYS = {
    "SHUMA_API_KEY",
    "SHUMA_ADMIN_READONLY_API_KEY",
    "SHUMA_JS_SECRET",
    "SHUMA_CHALLENGE_SECRET",
    "SHUMA_HEALTH_SECRET",
    "SHUMA_FORWARDED_IP_SECRET",
    "SHUMA_SIM_TELEMETRY_SECRET",
}
DEFAULT_CONTRACT_DIR = Path(os.environ.get("BLACKBOX_CONTRACT_DIR", "scripts/tests/adversarial"))
LANE_CONTRACT_PATH = DEFAULT_CONTRACT_DIR / "lane_contract.v1.json"
SIM_TAG_CONTRACT_PATH = DEFAULT_CONTRACT_DIR / "sim_tag_contract.v1.json"
FRONTIER_ACTION_CONTRACT_PATH = DEFAULT_CONTRACT_DIR / "frontier_action_contract.v1.json"
FRONTIER_ACTIONS_ENV = "BLACKBOX_ACTIONS"
ALLOWED_TOOLS_ENV = "BLACKBOX_ALLOWED_TOOLS"
CAPABILITY_ENVELOPES_ENV = "BLACKBOX_ACTION_ENVELOPES"
CAPABILITY_VERIFY_KEY_ENV = "BLACKBOX_CAPABILITY_VERIFY_KEY"


def bool_env(name: str, default: bool = False) -> bool:
    raw = os.environ.get(name)
    if raw is None:
        return default
    return str(raw).strip().lower() in {"1", "true", "yes", "on"}


def parse_positive_int(name: str, default: int) -> int:
    raw = os.environ.get(name, str(default))
    try:
        parsed = int(str(raw).strip())
    except Exception:
        return default
    return max(1, parsed)


def has_forbidden_env(observed_keys: List[str]) -> bool:
    for key in observed_keys:
        if any(key.startswith(prefix) for prefix in FORBIDDEN_ENV_PREFIXES):
            return True
        if key in FORBIDDEN_ENV_KEYS:
            return True
    return False


def load_lane_contract(path: Path = LANE_CONTRACT_PATH) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"lane contract not found: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError("lane contract must be a JSON object")
    return payload


def load_sim_tag_contract(path: Path = SIM_TAG_CONTRACT_PATH) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"sim tag contract not found: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError("sim tag contract must be a JSON object")
    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != "sim-tag-contract.v1":
        raise RuntimeError(
            f"sim tag contract schema_version must be sim-tag-contract.v1 (got {schema_version})"
        )
    return payload


SIM_TAG_CONTRACT = load_sim_tag_contract()
SIM_TAG_HEADERS = {
    str(key): str(value).strip().lower()
    for key, value in dict(SIM_TAG_CONTRACT.get("headers") or {}).items()
    if str(key).strip() and str(value).strip()
}
SIM_TAG_HEADER_RUN_ID = SIM_TAG_HEADERS.get("sim_run_id", "x-shuma-sim-run-id")
SIM_TAG_HEADER_PROFILE = SIM_TAG_HEADERS.get("sim_profile", "x-shuma-sim-profile")
SIM_TAG_HEADER_LANE = SIM_TAG_HEADERS.get("sim_lane", "x-shuma-sim-lane")
SIM_TAG_HEADER_TIMESTAMP = SIM_TAG_HEADERS.get("sim_timestamp", "x-shuma-sim-ts")
SIM_TAG_HEADER_NONCE = SIM_TAG_HEADERS.get("sim_nonce", "x-shuma-sim-nonce")
SIM_TAG_HEADER_SIGNATURE = SIM_TAG_HEADERS.get("sim_signature", "x-shuma-sim-signature")
SIM_TAG_CANONICAL_PREFIX = str(
    dict(SIM_TAG_CONTRACT.get("canonical") or {}).get("prefix") or "sim-tag.v1"
).strip()
SIM_TAG_CANONICAL_SEPARATOR = str(
    dict(SIM_TAG_CONTRACT.get("canonical") or {}).get("separator") or "\n"
)
SIM_TAG_ENVELOPE_ENV = "BLACKBOX_SIM_TAG_ENVELOPES"


def lane_required_sim_headers(lane_contract: Dict[str, Any]) -> List[str]:
    attacker = lane_contract.get("attacker")
    if not isinstance(attacker, dict):
        return []
    headers = attacker.get("required_sim_headers")
    if not isinstance(headers, list):
        return []
    return [str(item).strip().lower() for item in headers if str(item).strip()]


def lane_forbidden_headers(lane_contract: Dict[str, Any]) -> List[str]:
    attacker = lane_contract.get("attacker")
    if not isinstance(attacker, dict):
        return []
    headers = attacker.get("forbidden_headers")
    if not isinstance(headers, list):
        return []
    return [str(item).strip().lower() for item in headers if str(item).strip()]


def parse_sim_tag_envelopes(raw_value: str) -> List[Dict[str, str]]:
    text = str(raw_value or "").strip()
    if not text:
        return []
    try:
        payload = json.loads(text)
    except Exception:
        return []
    if not isinstance(payload, list):
        return []

    envelopes: List[Dict[str, str]] = []
    seen_nonces = set()
    for item in payload:
        if not isinstance(item, dict):
            return []
        timestamp = str(item.get("ts") or "").strip()
        nonce = str(item.get("nonce") or "").strip()
        signature = str(item.get("signature") or "").strip()
        if not timestamp or not nonce or not signature:
            return []
        if nonce in seen_nonces:
            return []
        seen_nonces.add(nonce)
        envelopes.append({"ts": timestamp, "nonce": nonce, "signature": signature})
    return envelopes


def workspace_mount_absent() -> bool:
    try:
        mounts = open("/proc/mounts", "r", encoding="utf-8", errors="replace").read().lower()
    except Exception:
        return False
    markers = ("shuma-gorath", "/users/jamestindall/projects", "/workspace")
    return not any(marker in mounts for marker in markers)


def enforce_allowlist(url: str, allowed_origins: List[str]) -> bool:
    parsed = urllib.parse.urlparse(url)
    origin = f"{parsed.scheme}://{parsed.netloc}"
    return origin in allowed_origins


def make_request(
    url: str, sim_headers: Dict[str, str], timeout_seconds: float = 10.0
) -> Dict[str, Any]:
    request = urllib.request.Request(url, method="GET")
    request.add_header("User-Agent", "ShumaContainerBlackBox/1.0")
    for key, value in sim_headers.items():
        request.add_header(key, value)
    start = time.monotonic()
    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            body = response.read().decode("utf-8", errors="replace")
            latency_ms = int((time.monotonic() - start) * 1000)
            return {
                "url": url,
                "status": response.status,
                "latency_ms": latency_ms,
                "body_sample": body[:160],
            }
    except urllib.error.HTTPError as exc:
        latency_ms = int((time.monotonic() - start) * 1000)
        return {
            "url": url,
            "status": int(exc.code),
            "latency_ms": latency_ms,
            "error": f"http_error_{exc.code}",
        }
    except Exception as exc:
        latency_ms = int((time.monotonic() - start) * 1000)
        return {
            "url": url,
            "status": 0,
            "latency_ms": latency_ms,
            "error": str(exc),
        }


def parse_allowed_tools(raw_value: str) -> List[str]:
    text = str(raw_value or "").strip()
    if not text:
        return []
    try:
        payload = json.loads(text)
    except Exception:
        return []
    if not isinstance(payload, list):
        return []
    normalized = []
    for item in payload:
        value = str(item or "").strip()
        if not value or value in normalized:
            continue
        normalized.append(value)
    return normalized


def resolve_worker_actions(
    raw_actions: str,
    *,
    contract: Dict[str, Any],
    base_url: str,
    allowed_origins: List[str],
    request_budget: int,
    allowed_tools: List[str],
) -> List[Dict[str, Any]]:
    action_type_overrides = allowed_tools or None
    return resolve_frontier_actions(
        raw_actions,
        contract=contract,
        base_url=base_url,
        allowed_origins=allowed_origins,
        request_budget=request_budget,
        allowed_tools_override=allowed_tools or None,
        allowed_action_types_override=action_type_overrides,
    )


def default_playwright_factory():
    from playwright.sync_api import sync_playwright

    return sync_playwright()


def _response_status(response: Any, fallback: int = 0) -> int:
    if response is None:
        return fallback
    status_attr = getattr(response, "status", None)
    if callable(status_attr):
        try:
            return int(status_attr())
        except Exception:
            return fallback
    try:
        return int(status_attr or fallback)
    except Exception:
        return fallback


def _page_title(page: Any) -> str:
    try:
        return str(page.title() or "").strip()
    except Exception:
        return ""


def _page_content_sample(page: Any, max_chars: int = 160) -> str:
    try:
        return str(page.content() or "")[:max_chars]
    except Exception:
        return ""


def _page_url(page: Any) -> str:
    value = getattr(page, "url", "")
    if callable(value):
        try:
            value = value()
        except Exception:
            value = ""
    return str(value or "").strip()


def _navigate_page(page: Any, url: str) -> Any:
    return page.goto(url, wait_until="domcontentloaded", timeout=15000)


def _browser_selector_candidates(action: Dict[str, Any]) -> List[str]:
    label = str(action.get("label") or "").strip()
    if not label:
        return ["a[href]", "button", '[role="button"]']
    escaped = label.replace("\\", "\\\\").replace('"', '\\"')
    return [
        f'a:has-text("{escaped}")',
        f'button:has-text("{escaped}")',
        f'[role="button"]:has-text("{escaped}")',
        f'text="{escaped}"',
    ]


def _browser_click(page: Any, action: Dict[str, Any]) -> None:
    selectors = _browser_selector_candidates(action)
    for selector in selectors:
        locator = page.locator(selector)
        try:
            count = int(locator.count() or 0)
        except Exception:
            count = 0
        if count < 1:
            continue
        try:
            if hasattr(locator, "is_visible") and not locator.is_visible():
                continue
        except Exception:
            continue
        locator.click(timeout=5000)
        try:
            page.wait_for_load_state("domcontentloaded", timeout=5000)
        except Exception:
            pass
        try:
            page.wait_for_timeout(250)
        except Exception:
            pass
        return
    raise RuntimeError(
        "browser_click_target_not_found:"
        + (str(action.get("label") or "").strip() or str(action.get("path") or "").strip() or "unknown")
    )


def execute_browser_actions(
    *,
    base_url: str,
    sim_headers: Dict[str, str],
    resolved_actions: List[Dict[str, Any]],
    time_budget_seconds: int,
    playwright_factory: Callable[[], Any] | None = None,
    run_id: str = "",
) -> Dict[str, Any]:
    factory = playwright_factory or default_playwright_factory
    started = time.monotonic()
    traffic: List[Dict[str, Any]] = []
    errors: List[str] = []
    requests_sent = 0

    with factory() as playwright:
        browser = playwright.chromium.launch(headless=True)
        try:
            context = browser.new_context(
                extra_http_headers=sim_headers,
                ignore_https_errors=True,
            )
            try:
                page = context.new_page()
                for action in resolved_actions:
                    if (time.monotonic() - started) >= max(1, int(time_budget_seconds)):
                        errors.append("time_budget_exhausted")
                        break

                    if run_id:
                        emit_heartbeat(run_id, "before_action", action_index=requests_sent + 1)

                    action_type = str(action.get("action_type") or "").strip()
                    action_url = str(action.get("url") or "").strip()
                    receipt: Dict[str, Any] = {
                        "action_index": int(action.get("action_index") or requests_sent + 1),
                        "action_type": action_type,
                        "path": str(action.get("path") or "").strip(),
                        "url": action_url,
                        "status": 0,
                        "latency_ms": 0,
                    }
                    if str(action.get("label") or "").strip():
                        receipt["label"] = str(action.get("label")).strip()

                    action_started = time.monotonic()
                    try:
                        if action_type == "browser_navigate":
                            response = _navigate_page(page, action_url)
                            receipt["status"] = _response_status(response, fallback=200)
                            receipt["page_title"] = _page_title(page)
                            receipt["body_sample"] = _page_content_sample(page)
                            receipt["observed_url"] = _page_url(page) or action_url
                        elif action_type == "browser_snapshot":
                            response = _navigate_page(page, action_url)
                            receipt["status"] = _response_status(response, fallback=200)
                            receipt["page_title"] = _page_title(page)
                            receipt["body_sample"] = _page_content_sample(page)
                            receipt["observed_url"] = _page_url(page) or action_url
                        elif action_type == "browser_click":
                            if _page_url(page) != action_url:
                                response = _navigate_page(page, action_url)
                            else:
                                response = None
                            _browser_click(page, action)
                            receipt["status"] = _response_status(response, fallback=200) or 200
                            receipt["page_title"] = _page_title(page)
                            receipt["body_sample"] = _page_content_sample(page)
                            receipt["observed_url"] = _page_url(page) or action_url
                        else:
                            raise RuntimeError(f"unsupported_browser_action:{action_type}")
                    except Exception as exc:
                        receipt["status"] = 0
                        receipt["error"] = str(exc)
                        errors.append(str(exc))

                    receipt["latency_ms"] = int((time.monotonic() - action_started) * 1000)
                    traffic.append(receipt)
                    requests_sent += 1

                    if run_id:
                        emit_heartbeat(run_id, "after_action", action_index=requests_sent)
            finally:
                context.close()
        finally:
            browser.close()

    return {
        "requests_sent": requests_sent,
        "traffic": traffic,
        "errors": errors,
        "driver_runtime": "playwright_chromium",
    }


def append_policy_audit_event(
    events: List[Dict[str, Any]],
    *,
    stage: str,
    decision: str,
    code: str,
    detail: str = "",
    action: Dict[str, Any] | None = None,
) -> None:
    entry: Dict[str, Any] = {
        "stage": str(stage).strip(),
        "decision": str(decision).strip(),
        "code": str(code).strip(),
        "detail": str(detail).strip(),
        "ts_unix": int(time.time()),
    }
    if isinstance(action, dict):
        entry["action_index"] = int(action.get("action_index") or 0)
        entry["action_type"] = str(action.get("action_type") or "")
        entry["path"] = str(action.get("path") or "")
    events.append(entry)


def emit_heartbeat(run_id: str, stage: str, action_index: int = 0) -> None:
    print(
        "[frontier-heartbeat] "
        f"run_id={str(run_id).strip()} stage={str(stage).strip()} "
        f"action_index={int(action_index)} ts={int(time.time())}",
        file=sys.stderr,
        flush=True,
    )


def main() -> int:
    mode = str(os.environ.get("BLACKBOX_MODE", "blackbox")).strip().lower()
    base_url = str(os.environ.get("BLACKBOX_BASE_URL", "")).strip().rstrip("/")
    run_id = str(os.environ.get("BLACKBOX_RUN_ID", "")).strip() or f"container-{int(time.time())}"
    allowed_origins_raw = str(os.environ.get("BLACKBOX_ALLOWED_ORIGINS", "")).strip()
    allowed_origins = [origin.strip() for origin in allowed_origins_raw.split(",") if origin.strip()]
    request_budget = parse_positive_int("BLACKBOX_REQUEST_BUDGET", 24)
    time_budget_seconds = parse_positive_int("BLACKBOX_TIME_BUDGET_SECONDS", 120)
    allowed_tools = parse_allowed_tools(os.environ.get(ALLOWED_TOOLS_ENV, ""))
    start = time.monotonic()
    lane_contract_error = ""
    lane_contract: Dict[str, Any] = {}
    try:
        lane_contract = load_lane_contract()
    except Exception as exc:
        lane_contract_error = str(exc)
    frontier_action_contract_error = ""
    frontier_action_contract: Dict[str, Any] = {}
    try:
        frontier_action_contract = load_frontier_action_contract(FRONTIER_ACTION_CONTRACT_PATH)
    except FrontierActionContractError as exc:
        frontier_action_contract_error = str(exc)

    observed_env_keys = sorted(list(os.environ.keys()))
    forbidden_env_present = has_forbidden_env(observed_env_keys)
    non_root = os.getuid() != 0
    no_workspace_mount = workspace_mount_absent()
    admin_credentials_absent = not forbidden_env_present
    tooling_limited = True  # Worker is intentionally limited to bounded contract-approved tooling.
    egress_allowlist_enforced = True
    ephemeral_run_identity = bool(run_id)

    payload: Dict[str, Any] = {
        "schema_version": "adversarial-container-worker.v1",
        "mode": mode,
        "run_id": run_id,
        "lane_contract_schema_version": str(lane_contract.get("schema_version") or ""),
        "lane_contract_error": lane_contract_error,
        "frontier_action_contract_schema_version": str(
            frontier_action_contract.get("schema_version") or ""
        ),
        "frontier_action_contract_error": frontier_action_contract_error,
        "runtime_hardening_non_root": non_root,
        "workspace_mount_absent": no_workspace_mount,
        "admin_credentials_absent": admin_credentials_absent,
        "tooling_limited": tooling_limited,
        "egress_allowlist_enforced": egress_allowlist_enforced,
        "ephemeral_run_identity": ephemeral_run_identity,
        "action_validation_passed": False,
        "resolved_action_count": 0,
        "resolved_actions": [],
        "policy_audit": [],
        "policy_violation_count": 0,
        "policy_violation_blocking": False,
        "capability_validation_passed": False,
        "capability_envelope_count": 0,
        "capability_validation_errors": [],
        "command_channel_direction": "host_to_worker_one_way",
        "evidence_channel_append_only": True,
        "control_plane_mutation_allowed": False,
        "observed_env_keys": [key for key in observed_env_keys if key.startswith("BLACKBOX_")],
        "allowed_tools": allowed_tools,
        "request_budget": request_budget,
        "time_budget_seconds": time_budget_seconds,
        "traffic": [],
        "passed": False,
    }

    contract_pass = (
        non_root
        and no_workspace_mount
        and admin_credentials_absent
        and tooling_limited
        and egress_allowlist_enforced
        and ephemeral_run_identity
        and not lane_contract_error
        and not frontier_action_contract_error
    )
    emit_heartbeat(run_id, "worker_started", action_index=0)

    if mode == "isolation":
        payload["passed"] = contract_pass
        print(json.dumps(payload, separators=(",", ":")))
        return 0 if payload["passed"] else 1

    if mode != "blackbox":
        payload["error"] = f"unsupported_mode:{mode}"
        print(json.dumps(payload, separators=(",", ":")))
        return 1

    if not base_url:
        payload["error"] = "missing_base_url"
        print(json.dumps(payload, separators=(",", ":")))
        return 1
    if not allowed_origins:
        payload["error"] = "missing_allowed_origins"
        print(json.dumps(payload, separators=(",", ":")))
        return 1

    statuses: List[int] = []
    errors: List[str] = []
    policy_audit = payload["policy_audit"]
    sim_tag_envelopes = parse_sim_tag_envelopes(os.environ.get(SIM_TAG_ENVELOPE_ENV, ""))
    if not sim_tag_envelopes:
        errors.append("missing_or_invalid_sim_tag_envelopes")
        append_policy_audit_event(
            policy_audit,
            stage="validation",
            decision="deny",
            code="missing_or_invalid_sim_tag_envelopes",
        )
    sim_headers = {
        SIM_TAG_HEADER_RUN_ID: run_id,
        SIM_TAG_HEADER_PROFILE: mode,
        SIM_TAG_HEADER_LANE: "container_blackbox",
    }
    sim_header_names = {
        SIM_TAG_HEADER_RUN_ID,
        SIM_TAG_HEADER_PROFILE,
        SIM_TAG_HEADER_LANE,
        SIM_TAG_HEADER_TIMESTAMP,
        SIM_TAG_HEADER_NONCE,
        SIM_TAG_HEADER_SIGNATURE,
    }
    required_sim_headers = set(lane_required_sim_headers(lane_contract))
    missing_sim_headers = sorted([header for header in required_sim_headers if header not in sim_header_names])
    if missing_sim_headers:
        errors.append("missing_sim_headers:" + ",".join(missing_sim_headers))
        append_policy_audit_event(
            policy_audit,
            stage="validation",
            decision="deny",
            code="missing_sim_headers",
            detail=",".join(missing_sim_headers),
        )
    forbidden_sim_headers = set(lane_forbidden_headers(lane_contract))
    forbidden_present = sorted([header for header in sim_header_names if header in forbidden_sim_headers])
    if forbidden_present:
        errors.append("forbidden_sim_headers:" + ",".join(forbidden_present))
        append_policy_audit_event(
            policy_audit,
            stage="validation",
            decision="deny",
            code="forbidden_sim_headers",
            detail=",".join(forbidden_present),
        )
    resolved_actions: List[Dict[str, Any]] = []
    capability_validation_errors: List[str] = []
    if not errors:
        try:
            resolved_actions = resolve_worker_actions(
                os.environ.get(FRONTIER_ACTIONS_ENV, ""),
                contract=frontier_action_contract,
                base_url=base_url,
                allowed_origins=allowed_origins,
                request_budget=request_budget,
                allowed_tools=allowed_tools,
            )
            payload["action_validation_passed"] = True
            payload["resolved_action_count"] = len(resolved_actions)
            payload["resolved_actions"] = [
                {
                    "action_index": action.get("action_index"),
                    "action_type": action.get("action_type"),
                    "path": action.get("path"),
                    "label": action.get("label"),
                }
                for action in resolved_actions
            ]
        except FrontierActionValidationError as exc:
            errors.append(f"action_validation_failed:{exc}")
            append_policy_audit_event(
                policy_audit,
                stage="validation",
                decision="deny",
                code="action_validation_failed",
                detail=str(exc),
            )
    if not errors:
        capability_envelopes = parse_action_capability_envelopes(
            os.environ.get(CAPABILITY_ENVELOPES_ENV, "")
        )
        capability_verify_key = str(os.environ.get(CAPABILITY_VERIFY_KEY_ENV, "")).strip()
        payload["capability_envelope_count"] = len(capability_envelopes)
        if not capability_verify_key:
            capability_validation_errors = ["missing_capability_verify_key"]
        else:
            capability_validation_errors = validate_action_capability_envelopes(
                capability_envelopes,
                verify_key=capability_verify_key,
                run_id=run_id,
                actions=resolved_actions,
            )
        payload["capability_validation_errors"] = capability_validation_errors
        if capability_validation_errors:
            errors.append("capability_validation_failed")
            append_policy_audit_event(
                policy_audit,
                stage="validation",
                decision="deny",
                code="capability_validation_failed",
                detail=";".join(capability_validation_errors),
            )
        else:
            payload["capability_validation_passed"] = True

    requests_sent = 0
    has_browser_actions = any(
        str(action.get("action_type") or "").strip().startswith("browser_")
        for action in resolved_actions
    )
    has_request_actions = any(
        not str(action.get("action_type") or "").strip().startswith("browser_")
        for action in resolved_actions
    )
    if has_browser_actions and has_request_actions:
        errors.append("mixed_action_types_unsupported")
        append_policy_audit_event(
            policy_audit,
            stage="validation",
            decision="deny",
            code="mixed_action_types_unsupported",
        )

    if not errors and has_browser_actions:
        browser_headers = dict(sim_headers)
        if sim_tag_envelopes:
            envelope = sim_tag_envelopes[0]
            browser_headers[SIM_TAG_HEADER_TIMESTAMP] = envelope["ts"]
            browser_headers[SIM_TAG_HEADER_NONCE] = envelope["nonce"]
            browser_headers[SIM_TAG_HEADER_SIGNATURE] = envelope["signature"]
        browser_report = execute_browser_actions(
            base_url=base_url,
            sim_headers=browser_headers,
            resolved_actions=resolved_actions,
            time_budget_seconds=time_budget_seconds,
            run_id=run_id,
        )
        payload["traffic"].extend(list(browser_report.get("traffic") or []))
        requests_sent = int(browser_report.get("requests_sent") or 0)
        statuses.extend(
            int(dict(item).get("status") or 0)
            for item in list(browser_report.get("traffic") or [])
            if isinstance(item, dict)
        )
        for error in list(browser_report.get("errors") or []):
            errors.append(str(error))
            append_policy_audit_event(
                policy_audit,
                stage="execution",
                decision="deny",
                code="browser_action_failed",
                detail=str(error),
            )
    else:
        for action in resolved_actions:
            if errors:
                break
            if requests_sent >= request_budget:
                break
            if requests_sent >= len(sim_tag_envelopes):
                errors.append("sim_tag_envelopes_exhausted")
                append_policy_audit_event(
                    policy_audit,
                    stage="execution",
                    decision="deny",
                    code="sim_tag_envelopes_exhausted",
                    action=action,
                )
                break
            if (time.monotonic() - start) >= time_budget_seconds:
                errors.append("time_budget_exhausted")
                append_policy_audit_event(
                    policy_audit,
                    stage="execution",
                    decision="deny",
                    code="time_budget_exhausted",
                    action=action,
                )
                break
            envelope = sim_tag_envelopes[requests_sent]
            request_headers = dict(sim_headers)
            request_headers[SIM_TAG_HEADER_TIMESTAMP] = envelope["ts"]
            request_headers[SIM_TAG_HEADER_NONCE] = envelope["nonce"]
            request_headers[SIM_TAG_HEADER_SIGNATURE] = envelope["signature"]
            url = str(action.get("url") or "")
            if not enforce_allowlist(url, allowed_origins):
                errors.append(f"egress_disallowed:{url}")
                append_policy_audit_event(
                    policy_audit,
                    stage="execution",
                    decision="deny",
                    code="egress_disallowed",
                    detail=url,
                    action=action,
                )
                break
            emit_heartbeat(run_id, "before_action", action_index=requests_sent + 1)
            result = make_request(url, request_headers)
            result["action_index"] = action.get("action_index")
            result["action_type"] = action.get("action_type")
            result["path"] = action.get("path")
            if str(action.get("label") or "").strip():
                result["label"] = str(action.get("label")).strip()
            payload["traffic"].append(result)
            requests_sent += 1
            statuses.append(int(result.get("status", 0)))
            emit_heartbeat(run_id, "after_action", action_index=requests_sent)
            if result.get("status", 0) == 0:
                errors.append(str(result.get("error") or "request_failed"))
                append_policy_audit_event(
                    policy_audit,
                    stage="execution",
                    decision="deny",
                    code="request_failed",
                    detail=str(result.get("error") or "request_failed"),
                    action=action,
                )

    payload["requests_sent"] = requests_sent
    payload["errors"] = errors
    payload["policy_violation_count"] = len(policy_audit)
    payload["policy_violation_blocking"] = bool(policy_audit)
    payload["allowed_statuses"] = [200, 302, 303, 403, 404, 429]

    status_ok = all(status in payload["allowed_statuses"] for status in statuses if status != 0)
    payload["passed"] = contract_pass and requests_sent > 0 and status_ok and not errors
    emit_heartbeat(run_id, "worker_completed", action_index=requests_sent)
    print(json.dumps(payload, separators=(",", ":")))
    return 0 if payload["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
