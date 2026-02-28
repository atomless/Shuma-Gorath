#!/usr/bin/env python3
"""Host-side orchestrator for containerized black-box adversary runs."""

from __future__ import annotations

import argparse
import hashlib
import hmac
import json
import os
import selectors
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Dict, List, Tuple

from scripts.tests.frontier_action_contract import (
    FrontierActionContractError,
    FrontierActionValidationError,
    load_frontier_action_contract,
    resolve_frontier_actions,
)
from scripts.tests.frontier_capability_envelope import (
    build_action_capability_envelopes,
)


DEFAULT_IMAGE_TAG = "shuma-adversary-blackbox:local"
DEFAULT_WORKER_PATH = "scripts/tests/adversarial_container/worker.py"
DEFAULT_DOCKERFILE_PATH = "scripts/tests/adversarial_container/Dockerfile"
DEFAULT_BLACKBOX_REPORT = "scripts/tests/adversarial/container_blackbox_report.json"
DEFAULT_ISOLATION_REPORT = "scripts/tests/adversarial/container_isolation_report.json"
DEFAULT_ATTACK_PLAN_PATH = "scripts/tests/adversarial/attack_plan.json"
SIM_TAG_CONTRACT_PATH = "scripts/tests/adversarial/sim_tag_contract.v1.json"
DEFAULT_FRONTIER_ACTION_CONTRACT_PATH = "scripts/tests/adversarial/frontier_action_contract.v1.json"
DEFAULT_CONTAINER_RUNTIME_PROFILE_PATH = "scripts/tests/adversarial/container_runtime_profile.v1.json"
FRONTIER_ACTIONS_ENV = "SHUMA_BLACKBOX_ACTIONS"
CAPABILITY_ENVELOPES_ENV = "BLACKBOX_ACTION_ENVELOPES"
CAPABILITY_VERIFY_KEY_ENV = "BLACKBOX_CAPABILITY_VERIFY_KEY"
DEFAULT_CLEANUP_TTL_HOURS = 72
DEFAULT_CLEANUP_MAX_DELETE = 32
DEFAULT_COMMAND_QUEUE_CAPACITY = 24
DEFAULT_WORKER_HEARTBEAT_TIMEOUT_SECONDS = 20
DEFAULT_WORKER_HARD_DEADLINE_BUFFER_SECONDS = 15
DEFAULT_KILL_SWITCH_FILE = "scripts/tests/adversarial/frontier_kill_switch.flag"
KILL_SWITCH_STOP_TIMEOUT_SECONDS = 10
FRONTIER_FORBIDDEN_FIELD_SUBSTRINGS = (
    "secret",
    "api_key",
    "apikey",
    "authorization",
    "cookie",
    "session",
    "token",
    "password",
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


def load_sim_tag_contract(path: str = SIM_TAG_CONTRACT_PATH) -> Dict[str, Any]:
    payload = json.loads(Path(path).read_text(encoding="utf-8"))
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
SIM_TAG_HEADER_TIMESTAMP = SIM_TAG_HEADERS.get("sim_timestamp", "x-shuma-sim-ts")
SIM_TAG_HEADER_NONCE = SIM_TAG_HEADERS.get("sim_nonce", "x-shuma-sim-nonce")
SIM_TAG_HEADER_SIGNATURE = SIM_TAG_HEADERS.get("sim_signature", "x-shuma-sim-signature")
SIM_TAG_CANONICAL_PREFIX = str(
    dict(SIM_TAG_CONTRACT.get("canonical") or {}).get("prefix") or "sim-tag.v1"
).strip()
SIM_TAG_CANONICAL_SEPARATOR = str(
    dict(SIM_TAG_CONTRACT.get("canonical") or {}).get("separator") or "\n"
)


def normalize_container_base_url(base_url: str) -> str:
    parsed = urllib.parse.urlparse(base_url)
    if parsed.hostname in {"127.0.0.1", "localhost"}:
        replacement = "host.docker.internal"
        netloc = replacement
        if parsed.port:
            netloc = f"{replacement}:{parsed.port}"
        return urllib.parse.urlunparse(
            (parsed.scheme, netloc, parsed.path, parsed.params, parsed.query, parsed.fragment)
        )
    return base_url


def target_origin(url: str) -> str:
    parsed = urllib.parse.urlparse(url)
    if not parsed.scheme or not parsed.netloc:
        raise RuntimeError(f"target base URL is invalid: {url}")
    return f"{parsed.scheme}://{parsed.netloc}"


def docker_available() -> bool:
    try:
        result = subprocess.run(
            ["docker", "version", "--format", "{{.Server.Version}}"],
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        return False
    return result.returncode == 0


def run_cmd(command: List[str], *, env: Dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, capture_output=True, text=True, check=False, env=env)


def build_sim_tag_canonical_message(
    run_id: str, profile: str, lane: str, timestamp: str, nonce: str
) -> str:
    parts = [
        SIM_TAG_CANONICAL_PREFIX,
        str(run_id).strip(),
        str(profile).strip(),
        str(lane).strip(),
        str(timestamp).strip(),
        str(nonce).strip(),
    ]
    return SIM_TAG_CANONICAL_SEPARATOR.join(parts)


def sign_sim_tag(
    secret: str, run_id: str, profile: str, lane: str, timestamp: str, nonce: str
) -> str:
    message = build_sim_tag_canonical_message(run_id, profile, lane, timestamp, nonce)
    return hmac.new(
        str(secret).encode("utf-8"),
        message.encode("utf-8"),
        hashlib.sha256,
    ).hexdigest()


def build_sim_tag_envelopes(
    *,
    secret: str,
    run_id: str,
    profile: str,
    lane: str,
    count: int,
) -> List[Dict[str, str]]:
    count = max(0, int(count))
    if not secret or count == 0:
        return []
    timestamp = str(int(time.time()))
    envelopes: List[Dict[str, str]] = []
    for index in range(count):
        nonce_raw = f"{run_id}:{profile}:{lane}:{timestamp}:{index + 1}"
        nonce = hashlib.sha256(nonce_raw.encode("utf-8")).hexdigest()[:24]
        signature = sign_sim_tag(
            secret=secret,
            run_id=run_id,
            profile=profile,
            lane=lane,
            timestamp=timestamp,
            nonce=nonce,
        )
        envelopes.append({"ts": timestamp, "nonce": nonce, "signature": signature})
    return envelopes


def ensure_image_built(image_tag: str, dockerfile_path: str) -> None:
    result = run_cmd(
        ["docker", "build", "-f", dockerfile_path, "-t", image_tag, "."]
    )
    if result.returncode != 0:
        raise RuntimeError(
            "failed to build adversary container image:\n"
            f"stdout:\n{result.stdout}\n\nstderr:\n{result.stderr}"
        )


def forwarded_headers(forwarded_secret: str, health_secret: str) -> Dict[str, str]:
    headers: Dict[str, str] = {"X-Forwarded-For": "127.0.0.1"}
    if forwarded_secret:
        headers["X-Shuma-Forwarded-Secret"] = forwarded_secret
    if health_secret:
        headers["X-Shuma-Health-Secret"] = health_secret
    return headers


def wait_ready(base_url: str, forwarded_secret: str, health_secret: str, timeout_seconds: int = 30) -> None:
    deadline = time.monotonic() + timeout_seconds
    health_url = base_url.rstrip("/") + "/health"
    headers = forwarded_headers(forwarded_secret, health_secret)
    while time.monotonic() < deadline:
        request = urllib.request.Request(health_url, method="GET")
        for key, value in headers.items():
            request.add_header(key, value)
        try:
            with urllib.request.urlopen(request, timeout=5.0) as response:
                body = response.read().decode("utf-8", errors="replace")
                if response.status == 200 and "OK" in body:
                    return
        except Exception:
            time.sleep(1)
            continue
        time.sleep(1)
    raise RuntimeError(f"Spin server was not ready at {health_url} within {timeout_seconds}s")


def orchestrator_reset_hook(base_url: str, api_key: str, forwarded_secret: str) -> Dict[str, Any]:
    if not api_key:
        return {
            "hook": "orchestrator_reset",
            "performed": False,
            "reason": "missing_api_key",
        }
    config_url = base_url.rstrip("/") + "/admin/config"
    payload = {
        "test_mode": False,
        "adversary_sim_enabled": False,
    }
    body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
    request = urllib.request.Request(config_url, method="POST", data=body)
    request.add_header("Authorization", f"Bearer {api_key}")
    request.add_header("Content-Type", "application/json")
    if forwarded_secret:
        request.add_header("X-Shuma-Forwarded-Secret", forwarded_secret)
    try:
        with urllib.request.urlopen(request, timeout=10.0) as response:
            return {
                "hook": "orchestrator_reset",
                "performed": response.status == 200,
                "status": response.status,
            }
    except urllib.error.HTTPError as exc:
        return {
            "hook": "orchestrator_reset",
            "performed": False,
            "status": exc.code,
            "error": f"http_error_{exc.code}",
        }
    except Exception as exc:
        return {
            "hook": "orchestrator_reset",
            "performed": False,
            "error": str(exc),
        }


def parse_worker_json(stdout_text: str) -> Dict[str, Any]:
    for line in reversed(stdout_text.splitlines()):
        candidate = line.strip()
        if not candidate:
            continue
        if not candidate.startswith("{"):
            continue
        try:
            parsed = json.loads(candidate)
        except Exception:
            continue
        if isinstance(parsed, dict):
            return parsed
    raise RuntimeError("container worker did not emit JSON payload")


def load_attack_plan(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"attack plan not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"attack plan JSON invalid: {path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError("attack plan must be a JSON object")
    if str(payload.get("schema_version") or "").strip() != "attack-plan.v1":
        raise RuntimeError("attack plan schema_version must be attack-plan.v1")
    candidates = payload.get("candidates")
    if not isinstance(candidates, list) or not candidates:
        raise RuntimeError("attack plan candidates must be a non-empty array")
    return payload


def load_container_runtime_profile(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"container runtime profile not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"container runtime profile JSON invalid: {path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError("container runtime profile must be a JSON object")
    if str(payload.get("schema_version") or "").strip() != "container-runtime-profile.v1":
        raise RuntimeError("container runtime profile schema_version must be container-runtime-profile.v1")
    for key in ("required_flags", "forbidden_flag_prefixes", "forbidden_mount_substrings"):
        values = payload.get(key)
        if not isinstance(values, list):
            raise RuntimeError(f"container runtime profile {key} must be an array")
    required_user_mode = str(payload.get("required_user_mode") or "").strip()
    if required_user_mode != "non_root_image_user":
        raise RuntimeError(
            "container runtime profile required_user_mode must be non_root_image_user"
        )
    return payload


def evaluate_container_command_against_profile(
    command: List[str],
    runtime_profile: Dict[str, Any],
) -> List[str]:
    violations: List[str] = []
    tokens = [str(token) for token in command]
    required_flags = [
        str(item).strip()
        for item in list(runtime_profile.get("required_flags") or [])
        if str(item).strip()
    ]
    for required in required_flags:
        if required not in tokens:
            violations.append(f"missing_required_flag:{required}")

    forbidden_prefixes = [
        str(item).strip()
        for item in list(runtime_profile.get("forbidden_flag_prefixes") or [])
        if str(item).strip()
    ]
    for token in tokens:
        for prefix in forbidden_prefixes:
            if token.startswith(prefix):
                violations.append(f"forbidden_flag:{token}")

    forbidden_mount_substrings = [
        str(item).strip()
        for item in list(runtime_profile.get("forbidden_mount_substrings") or [])
        if str(item).strip()
    ]
    for token in tokens:
        lowered = token.lower()
        for fragment in forbidden_mount_substrings:
            if fragment.lower() in lowered:
                violations.append(f"forbidden_mount_fragment:{token}")

    for index, token in enumerate(tokens):
        if token in {"-u", "--user"}:
            next_value = tokens[index + 1] if index + 1 < len(tokens) else ""
            if str(next_value).strip() in {"0", "root", "0:0", "root:root"}:
                violations.append("forbidden_user_override:root")
        if token.startswith("--user="):
            value = token.split("=", 1)[1].strip()
            if value in {"0", "root", "0:0", "root:root"}:
                violations.append("forbidden_user_override:root")

    deduped: List[str] = []
    for item in violations:
        if item not in deduped:
            deduped.append(item)
    return deduped


def extract_frontier_actions_from_attack_plan(
    attack_plan: Dict[str, Any],
    request_budget: int,
    forbidden_secret_values: List[str],
) -> Tuple[List[Dict[str, Any]], List[Dict[str, Any]], List[Dict[str, Any]]]:
    request_budget = max(1, int(request_budget))
    candidates = attack_plan.get("candidates")
    if not isinstance(candidates, list):
        raise RuntimeError("attack plan candidates must be a list")

    actions: List[Dict[str, Any]] = []
    lineage: List[Dict[str, Any]] = []
    rejected: List[Dict[str, Any]] = []
    for index, candidate in enumerate(candidates):
        if len(actions) >= request_budget:
            break
        entry = dict(candidate or {})
        payload = dict(entry.get("payload") or {})
        scenario_id = str(entry.get("scenario_id") or "").strip()
        request_id = str(payload.get("request_id") or "").strip()
        rejection_reasons = validate_attack_plan_candidate_payload(
            payload,
            forbidden_secret_values=forbidden_secret_values,
        )
        if rejection_reasons:
            rejected.append(
                {
                    "candidate_index": index + 1,
                    "scenario_id": scenario_id,
                    "request_id": request_id,
                    "reasons": rejection_reasons,
                }
            )
            continue
        target = dict(payload.get("target") or {})
        path_hint = str(target.get("path_hint") or "").strip() or "/"
        path_hint = path_hint.split("?", 1)[0].split("#", 1)[0] or "/"

        next_action_index = len(actions) + 1
        action = {
            "action_type": "http_get",
            "path": path_hint,
            "label": scenario_id or f"candidate-{index + 1}",
        }
        actions.append(action)
        lineage.append(
            {
                "candidate_index": index + 1,
                "scenario_id": scenario_id,
                "request_id": request_id,
                "proposed_action": {
                    "action_index": next_action_index,
                    "action_type": action["action_type"],
                    "path": action["path"],
                    "label": action["label"],
                },
            }
        )

    if not actions:
        raise RuntimeError(
            "attack plan did not yield any executable candidate actions after sanitization"
        )
    return actions, lineage, rejected


def collect_candidate_paths(value: Any, path: str = "") -> List[str]:
    findings: List[str] = []
    if isinstance(value, dict):
        for key, nested in value.items():
            key_text = str(key).strip()
            path_name = f"{path}.{key_text}" if path else key_text
            lowered = key_text.lower()
            if any(token in lowered for token in FRONTIER_FORBIDDEN_FIELD_SUBSTRINGS):
                findings.append(f"forbidden_key:{path_name}")
            findings.extend(collect_candidate_paths(nested, path_name))
        return findings
    if isinstance(value, list):
        for index, nested in enumerate(value):
            path_name = f"{path}[{index}]"
            findings.extend(collect_candidate_paths(nested, path_name))
        return findings
    return findings


def payload_contains_secret_literal(value: Any, secret_values: List[str]) -> bool:
    if isinstance(value, dict):
        return any(payload_contains_secret_literal(item, secret_values) for item in value.values())
    if isinstance(value, list):
        return any(payload_contains_secret_literal(item, secret_values) for item in value)
    if isinstance(value, str):
        text = value.strip()
        return any(secret and secret in text for secret in secret_values)
    return False


def validate_attack_plan_candidate_payload(
    payload: Dict[str, Any],
    forbidden_secret_values: List[str],
) -> List[str]:
    reasons: List[str] = []
    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != "frontier_payload_schema.v1":
        reasons.append(
            f"payload_schema_mismatch:expected=frontier_payload_schema.v1 got={schema_version}"
        )
    reasons.extend(collect_candidate_paths(payload))
    if payload_contains_secret_literal(payload, forbidden_secret_values):
        reasons.append("literal_secret_value_detected")
    target = dict(payload.get("target") or {})
    path_hint = str(target.get("path_hint") or "").strip()
    if not path_hint.startswith("/"):
        reasons.append("target_path_hint_must_start_with_slash")
    if path_hint.startswith("/admin/"):
        reasons.append("target_path_hint_forbidden_prefix")
    return reasons


def admin_read_json(
    base_url: str,
    api_key: str,
    forwarded_secret: str,
    path: str,
) -> Dict[str, Any]:
    if not api_key:
        raise RuntimeError("missing_api_key")
    url = base_url.rstrip("/") + path
    request = urllib.request.Request(url, method="GET")
    request.add_header("Authorization", f"Bearer {api_key}")
    if forwarded_secret:
        request.add_header("X-Shuma-Forwarded-Secret", forwarded_secret)
    with urllib.request.urlopen(request, timeout=10.0) as response:
        payload = json.loads(response.read().decode("utf-8", errors="replace"))
    if not isinstance(payload, dict):
        raise RuntimeError(f"admin read {path} did not return JSON object")
    return payload


def collect_run_events_from_payload(payload: Dict[str, Any], run_id: str) -> List[Dict[str, Any]]:
    recent_events = payload.get("recent_events")
    if not isinstance(recent_events, list):
        return []
    matching: List[Dict[str, Any]] = []
    for event in recent_events:
        record = dict(event or {})
        if str(record.get("sim_run_id") or "").strip() != run_id:
            continue
        matching.append(record)
    return matching


def build_frontier_lineage_summary(
    frontier_action_lineage: List[Dict[str, Any]],
    worker_payload: Dict[str, Any],
    runtime_events: List[Dict[str, Any]],
    monitoring_events: List[Dict[str, Any]],
) -> Dict[str, Any]:
    traffic = list(worker_payload.get("traffic") or [])
    executed_by_index: Dict[int, Dict[str, Any]] = {}
    for entry in traffic:
        record = dict(entry or {})
        action_index = int(record.get("action_index") or 0)
        if action_index <= 0:
            continue
        executed_by_index[action_index] = record

    rows: List[Dict[str, Any]] = []
    missing_indices: List[int] = []
    for entry in frontier_action_lineage:
        lineage = dict(entry or {})
        proposed = dict(lineage.get("proposed_action") or {})
        action_index = int(proposed.get("action_index") or 0)
        executed = dict(executed_by_index.get(action_index) or {})
        if not executed:
            missing_indices.append(action_index)
        rows.append(
            {
                "candidate_index": int(lineage.get("candidate_index") or 0),
                "scenario_id": str(lineage.get("scenario_id") or ""),
                "request_id": str(lineage.get("request_id") or ""),
                "proposed_action": proposed,
                "executed": bool(executed),
                "executed_status": int(executed.get("status") or 0),
                "executed_error": str(executed.get("error") or ""),
            }
        )

    runtime_reasons = sorted(
        {
            str(dict(event).get("reason") or "").strip()
            for event in runtime_events
            if str(dict(event).get("reason") or "").strip()
        }
    )
    monitoring_reasons = sorted(
        {
            str(dict(event).get("reason") or "").strip()
            for event in monitoring_events
            if str(dict(event).get("reason") or "").strip()
        }
    )
    model_count = len(frontier_action_lineage)
    executed_count = len([row for row in rows if row.get("executed")])
    lineage_complete = (
        model_count > 0
        and executed_count == model_count
        and len(runtime_events) > 0
        and len(monitoring_events) > 0
    )
    return {
        "model_suggestion_count": model_count,
        "executed_action_count": executed_count,
        "runtime_event_count": len(runtime_events),
        "monitoring_event_count": len(monitoring_events),
        "missing_action_indices": sorted([index for index in missing_indices if index > 0]),
        "runtime_event_reasons": runtime_reasons,
        "monitoring_event_reasons": monitoring_reasons,
        "lineage_complete": lineage_complete,
        "rows": rows,
    }


def build_frontier_runtime_state(
    mode: str,
    frontier_actions_source: str,
    frontier_action_source_error: str,
    frontier_lineage: Dict[str, Any],
) -> Dict[str, Any]:
    if mode != "blackbox":
        return {
            "status": "ok",
            "degraded": False,
            "reasons": [],
            "detail": "not_applicable",
        }

    reasons: List[str] = []
    if frontier_actions_source == "contract_default_fallback":
        reasons.append("attack_plan_unavailable_or_invalid")
    if frontier_action_source_error:
        reasons.append(frontier_action_source_error)
    lineage_detail = str(frontier_lineage.get("detail") or "").strip()
    if lineage_detail and lineage_detail != "ok":
        reasons.append(lineage_detail)

    deduped = []
    for reason in reasons:
        text = str(reason).strip()
        if not text or text in deduped:
            continue
        deduped.append(text)
    return {
        "status": "degraded" if deduped else "ok",
        "degraded": bool(deduped),
        "reasons": deduped,
        "detail": "frontier_runtime_degraded" if deduped else "ok",
    }


def cleanup_frontier_artifacts(
    artifacts_dir: Path,
    *,
    ttl_hours: int,
    max_delete: int,
) -> Dict[str, Any]:
    ttl_hours = max(1, int(ttl_hours))
    max_delete = max(1, int(max_delete))
    cutoff_unix = time.time() - float(ttl_hours * 3600)
    deleted: List[str] = []
    failed: List[Dict[str, str]] = []
    scanned = 0

    candidates = sorted(artifacts_dir.glob("container_*_report.json"), key=lambda path: path.name)
    for candidate in candidates:
        scanned += 1
        if len(deleted) >= max_delete:
            break
        try:
            stat = candidate.stat()
        except Exception as exc:
            failed.append({"path": str(candidate), "error": str(exc)})
            continue
        if stat.st_mtime >= cutoff_unix:
            continue
        try:
            candidate.unlink()
            deleted.append(str(candidate))
        except Exception as exc:
            failed.append({"path": str(candidate), "error": str(exc)})

    return {
        "ttl_hours": ttl_hours,
        "cutoff_unix": int(cutoff_unix),
        "max_delete": max_delete,
        "scanned": scanned,
        "deleted_count": len(deleted),
        "deleted": deleted,
        "failed": failed,
        "failed_count": len(failed),
    }


def prepare_command_channel(
    actions: List[Dict[str, Any]],
    *,
    queue_capacity: int,
) -> Dict[str, Any]:
    queue_capacity = max(1, int(queue_capacity))
    queued_actions = list(actions[:queue_capacity])
    overflow_count = max(0, len(actions) - len(queued_actions))
    return {
        "direction": "host_to_worker_one_way",
        "queue_capacity": queue_capacity,
        "queued_action_count": len(queued_actions),
        "overflow_count": overflow_count,
        "backpressure_applied": overflow_count > 0,
        "queued_actions": queued_actions,
        "evidence_channel_append_only_expected": True,
        "control_plane_mutation_allowed": False,
    }


def container_command(
    image_tag: str,
    mode: str,
    base_url: str,
    allowed_origin: str,
    run_id: str,
    request_budget: int,
    time_budget_seconds: int,
    sim_tag_envelopes_json: str,
    frontier_actions_json: str,
    capability_envelopes_json: str,
    capability_verify_key: str,
) -> List[str]:
    command = [
        "docker",
        "run",
        "--rm",
        "--read-only",
        "--cap-drop=ALL",
        "--security-opt=no-new-privileges",
        "--pids-limit=128",
        "--memory=256m",
        "--cpus=1.0",
        "--tmpfs=/tmp:rw,nosuid,nodev,size=64m",
        "--add-host=host.docker.internal:host-gateway",
        "--network=bridge",
        "-e",
        f"BLACKBOX_MODE={mode}",
        "-e",
        f"BLACKBOX_BASE_URL={base_url}",
        "-e",
        f"BLACKBOX_ALLOWED_ORIGINS={allowed_origin}",
        "-e",
        f"BLACKBOX_RUN_ID={run_id}",
        "-e",
        f"BLACKBOX_REQUEST_BUDGET={request_budget}",
        "-e",
        f"BLACKBOX_TIME_BUDGET_SECONDS={time_budget_seconds}",
        "-e",
        f"BLACKBOX_SIM_TAG_ENVELOPES={sim_tag_envelopes_json}",
        "-e",
        f"BLACKBOX_ACTIONS={frontier_actions_json}",
        "-e",
        f"{CAPABILITY_ENVELOPES_ENV}={capability_envelopes_json}",
        "-e",
        f"{CAPABILITY_VERIFY_KEY_ENV}={capability_verify_key}",
        image_tag,
    ]
    return command


def run_container_worker(
    image_tag: str,
    mode: str,
    base_url: str,
    allowed_origin: str,
    run_id: str,
    request_budget: int,
    time_budget_seconds: int,
    sim_tag_envelopes_json: str,
    frontier_actions_json: str,
    capability_envelopes_json: str,
    capability_verify_key: str,
    runtime_profile: Dict[str, Any],
    kill_switch_file: str,
    heartbeat_timeout_seconds: int,
    hard_deadline_buffer_seconds: int,
) -> Tuple[Dict[str, Any], subprocess.CompletedProcess[str], List[str], List[str], Dict[str, Any]]:
    command = container_command(
        image_tag=image_tag,
        mode=mode,
        base_url=base_url,
        allowed_origin=allowed_origin,
        run_id=run_id,
        request_budget=request_budget,
        time_budget_seconds=time_budget_seconds,
        sim_tag_envelopes_json=sim_tag_envelopes_json,
        frontier_actions_json=frontier_actions_json,
        capability_envelopes_json=capability_envelopes_json,
        capability_verify_key=capability_verify_key,
    )
    violations = evaluate_container_command_against_profile(command, runtime_profile)
    if violations:
        raise RuntimeError("runtime_profile_violation:" + ";".join(violations))
    process = subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )
    if process.stdout is None or process.stderr is None:
        process.kill()
        raise RuntimeError("worker_stream_initialization_failed")

    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ, data="stdout")
    selector.register(process.stderr, selectors.EVENT_READ, data="stderr")

    started_monotonic = time.monotonic()
    last_heartbeat_monotonic = started_monotonic
    hard_deadline_seconds = max(10, int(time_budget_seconds) + max(0, int(hard_deadline_buffer_seconds)))
    heartbeat_timeout_seconds = max(5, int(heartbeat_timeout_seconds))
    kill_switch_path = Path(kill_switch_file) if str(kill_switch_file or "").strip() else None

    stdout_lines: List[str] = []
    stderr_lines: List[str] = []
    termination_reason = ""
    stop_latency_ms = 0
    forced_kill = False

    while True:
        events = selector.select(timeout=0.2)
        for key, _ in events:
            stream = key.fileobj
            line = stream.readline()
            if not line:
                continue
            text = line.rstrip("\n")
            if key.data == "stdout":
                stdout_lines.append(text)
            else:
                stderr_lines.append(text)
            if "[frontier-heartbeat]" in text:
                last_heartbeat_monotonic = time.monotonic()

        if process.poll() is not None:
            break

        now = time.monotonic()
        if kill_switch_path and kill_switch_path.exists():
            termination_reason = "kill_switch_triggered"
        elif (now - started_monotonic) > hard_deadline_seconds:
            termination_reason = "hard_deadline_exceeded"
        elif (now - last_heartbeat_monotonic) > heartbeat_timeout_seconds:
            termination_reason = "heartbeat_timeout"

        if not termination_reason:
            continue

        stop_started = time.monotonic()
        process.terminate()
        try:
            process.wait(timeout=KILL_SWITCH_STOP_TIMEOUT_SECONDS)
        except subprocess.TimeoutExpired:
            forced_kill = True
            process.kill()
            process.wait(timeout=5)
        stop_latency_ms = int(max(0.0, (time.monotonic() - stop_started) * 1000.0))
        break

    try:
        remaining_stdout, remaining_stderr = process.communicate(timeout=2)
    except subprocess.TimeoutExpired:
        process.kill()
        remaining_stdout, remaining_stderr = process.communicate()
    if remaining_stdout:
        stdout_lines.extend([line for line in remaining_stdout.splitlines() if line])
    if remaining_stderr:
        stderr_lines.extend([line for line in remaining_stderr.splitlines() if line])

    control = {
        "kill_switch_file": str(kill_switch_path) if kill_switch_path else "",
        "heartbeat_timeout_seconds": heartbeat_timeout_seconds,
        "hard_deadline_seconds": hard_deadline_seconds,
        "termination_reason": termination_reason,
        "stop_latency_ms": stop_latency_ms,
        "forced_kill": forced_kill,
    }
    result = subprocess.CompletedProcess(
        args=command,
        returncode=int(process.returncode or 0),
        stdout="\n".join(stdout_lines),
        stderr="\n".join(stderr_lines),
    )
    if termination_reason:
        raise RuntimeError(
            f"{termination_reason}:stop_latency_ms={stop_latency_ms}:hard_deadline_seconds={hard_deadline_seconds}:forced_kill={str(forced_kill).lower()}"
        )
    parsed = parse_worker_json(result.stdout)
    return parsed, result, command, violations, control


def parse_worker_failure_taxonomy(detail: str) -> Dict[str, Any]:
    text = str(detail or "").strip()
    if not text:
        return {
            "reason": "none",
            "terminal_failure": "none",
            "forced_kill": False,
            "stop_latency_ms": 0,
            "hard_deadline_seconds": 0,
            "raw": "",
        }

    parts = [part.strip() for part in text.split(":") if part.strip()]
    reason = parts[0] if parts else "worker_execution_failure"
    stop_latency_ms = 0
    hard_deadline_seconds = 0
    forced_kill = False
    for part in parts[1:]:
        if "=" not in part:
            continue
        key, value = part.split("=", 1)
        key = key.strip()
        value = value.strip()
        if key == "stop_latency_ms":
            try:
                stop_latency_ms = max(0, int(value))
            except Exception:
                stop_latency_ms = 0
        elif key == "hard_deadline_seconds":
            try:
                hard_deadline_seconds = max(0, int(value))
            except Exception:
                hard_deadline_seconds = 0
        elif key == "forced_kill":
            forced_kill = value.lower() in {"1", "true", "yes", "on"}

    taxonomy_map = {
        "hard_deadline_exceeded": "deadline_exceeded",
        "heartbeat_timeout": "heartbeat_loss",
        "kill_switch_triggered": "forced_kill_path",
    }
    terminal_failure = taxonomy_map.get(reason, "worker_execution_failure")
    if forced_kill:
        terminal_failure = "forced_kill_path"
    return {
        "reason": reason,
        "terminal_failure": terminal_failure,
        "forced_kill": forced_kill,
        "stop_latency_ms": stop_latency_ms,
        "hard_deadline_seconds": hard_deadline_seconds,
        "raw": text,
    }


def report_path_for_mode(mode: str, custom_report: str) -> Path:
    if custom_report:
        return Path(custom_report)
    if mode == "isolation":
        return Path(DEFAULT_ISOLATION_REPORT)
    return Path(DEFAULT_BLACKBOX_REPORT)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run containerized black-box adversary worker and isolation checks"
    )
    parser.add_argument("--mode", choices=["isolation", "blackbox"], required=True)
    parser.add_argument(
        "--base-url",
        default=os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000"),
        help="Target base URL (host runtime).",
    )
    parser.add_argument("--image-tag", default=DEFAULT_IMAGE_TAG, help="Container image tag")
    parser.add_argument("--dockerfile", default=DEFAULT_DOCKERFILE_PATH, help="Dockerfile path")
    parser.add_argument("--worker-path", default=DEFAULT_WORKER_PATH, help="Worker script path")
    parser.add_argument("--report", default="", help="Optional report output path")
    parser.add_argument(
        "--request-budget",
        default=os.environ.get("SHUMA_BLACKBOX_REQUEST_BUDGET", "24"),
        help="Maximum worker requests",
    )
    parser.add_argument(
        "--time-budget-seconds",
        default=os.environ.get("SHUMA_BLACKBOX_TIME_BUDGET_SECONDS", "120"),
        help="Maximum worker runtime",
    )
    parser.add_argument(
        "--attack-plan",
        default=DEFAULT_ATTACK_PLAN_PATH,
        help="Attack plan artifact path used for model-suggestion -> executable action conversion",
    )
    parser.add_argument(
        "--frontier-action-contract",
        default=DEFAULT_FRONTIER_ACTION_CONTRACT_PATH,
        help="Frontier action contract path",
    )
    parser.add_argument(
        "--runtime-profile",
        default=DEFAULT_CONTAINER_RUNTIME_PROFILE_PATH,
        help="Container runtime hardening profile path",
    )
    parser.add_argument(
        "--frontier-actions",
        default=os.environ.get(FRONTIER_ACTIONS_ENV, ""),
        help="Optional frontier action JSON list (defaults to contract default_actions)",
    )
    parser.add_argument(
        "--cleanup-ttl-hours",
        default=os.environ.get("SHUMA_FRONTIER_ARTIFACT_TTL_HOURS", str(DEFAULT_CLEANUP_TTL_HOURS)),
        help="Retention TTL for frontier container report artifacts",
    )
    parser.add_argument(
        "--cleanup-max-delete",
        default=os.environ.get("SHUMA_FRONTIER_ARTIFACT_CLEANUP_MAX_DELETE", str(DEFAULT_CLEANUP_MAX_DELETE)),
        help="Maximum number of stale frontier report artifacts to delete per run",
    )
    parser.add_argument(
        "--command-queue-capacity",
        default=os.environ.get("SHUMA_FRONTIER_COMMAND_QUEUE_CAPACITY", str(DEFAULT_COMMAND_QUEUE_CAPACITY)),
        help="Maximum queued frontier actions sent host->worker per run",
    )
    parser.add_argument(
        "--kill-switch-file",
        default=os.environ.get("SHUMA_FRONTIER_KILL_SWITCH_FILE", DEFAULT_KILL_SWITCH_FILE),
        help="Path to kill-switch sentinel file checked during worker execution",
    )
    parser.add_argument(
        "--heartbeat-timeout-seconds",
        default=os.environ.get(
            "SHUMA_FRONTIER_HEARTBEAT_TIMEOUT_SECONDS",
            str(DEFAULT_WORKER_HEARTBEAT_TIMEOUT_SECONDS),
        ),
        help="Maximum worker heartbeat silence before fail-closed termination",
    )
    parser.add_argument(
        "--hard-deadline-buffer-seconds",
        default=os.environ.get(
            "SHUMA_FRONTIER_HARD_DEADLINE_BUFFER_SECONDS",
            str(DEFAULT_WORKER_HARD_DEADLINE_BUFFER_SECONDS),
        ),
        help="Extra seconds added to configured time budget before forced termination",
    )
    args = parser.parse_args()

    request_budget = int(str(args.request_budget).strip())
    time_budget_seconds = int(str(args.time_budget_seconds).strip())
    cleanup_ttl_hours = int(str(args.cleanup_ttl_hours).strip())
    cleanup_max_delete = int(str(args.cleanup_max_delete).strip())
    command_queue_capacity = int(str(args.command_queue_capacity).strip())
    heartbeat_timeout_seconds = int(str(args.heartbeat_timeout_seconds).strip())
    hard_deadline_buffer_seconds = int(str(args.hard_deadline_buffer_seconds).strip())
    if request_budget < 1:
        print("request budget must be >= 1", file=sys.stderr)
        return 2
    if time_budget_seconds < 10:
        print("time budget must be >= 10 seconds", file=sys.stderr)
        return 2

    if not docker_available():
        print("docker is required for container adversary targets", file=sys.stderr)
        return 2
    if not Path(args.dockerfile).exists():
        print(f"Dockerfile not found: {args.dockerfile}", file=sys.stderr)
        return 2
    if not Path(args.worker_path).exists():
        print(f"worker path not found: {args.worker_path}", file=sys.stderr)
        return 2

    host_base_url = args.base_url.rstrip("/")
    container_base_url = normalize_container_base_url(host_base_url)
    allowed_origin = target_origin(container_base_url)
    run_id = f"container-{int(time.time())}"
    try:
        frontier_action_contract = load_frontier_action_contract(
            Path(args.frontier_action_contract)
        )
    except FrontierActionContractError as exc:
        print(f"[adversarial-container] {exc}", file=sys.stderr)
        return 2
    try:
        runtime_profile = load_container_runtime_profile(Path(args.runtime_profile))
    except Exception as exc:
        print(f"[adversarial-container] {exc}", file=sys.stderr)
        return 2

    contract_budgets = dict(frontier_action_contract.get("budgets") or {})
    contract_max_actions = int(contract_budgets.get("max_actions_per_run") or request_budget)
    contract_max_runtime = int(contract_budgets.get("max_time_budget_seconds") or time_budget_seconds)
    request_budget = max(1, min(request_budget, contract_max_actions))
    time_budget_seconds = max(10, min(time_budget_seconds, contract_max_runtime))

    frontier_actions_source = "explicit_input"
    frontier_action_source_error = ""
    frontier_action_lineage: List[Dict[str, Any]] = []
    frontier_candidate_rejections: List[Dict[str, Any]] = []
    forbidden_secret_values = [
        str(os.environ.get(env_key) or "").strip()
        for env_key in sorted(FORBIDDEN_ENV_KEYS)
        if str(os.environ.get(env_key) or "").strip()
    ]
    forbidden_secret_values.extend(
        [
            str(os.environ.get("SHUMA_FRONTIER_OPENAI_API_KEY") or "").strip(),
            str(os.environ.get("SHUMA_FRONTIER_ANTHROPIC_API_KEY") or "").strip(),
            str(os.environ.get("SHUMA_FRONTIER_GOOGLE_API_KEY") or "").strip(),
            str(os.environ.get("SHUMA_FRONTIER_XAI_API_KEY") or "").strip(),
        ]
    )
    forbidden_secret_values = sorted(
        {value for value in forbidden_secret_values if value}
    )
    frontier_actions_raw = str(args.frontier_actions or "").strip()
    if not frontier_actions_raw and args.mode == "blackbox":
        frontier_actions_source = "attack_plan_candidates"
        try:
            attack_plan_payload = load_attack_plan(Path(args.attack_plan))
            (
                attack_plan_actions,
                attack_plan_lineage,
                attack_plan_rejections,
            ) = extract_frontier_actions_from_attack_plan(
                attack_plan_payload,
                request_budget=request_budget,
                forbidden_secret_values=forbidden_secret_values,
            )
            frontier_actions_raw = json.dumps(attack_plan_actions, separators=(",", ":"))
            frontier_action_lineage = attack_plan_lineage
            frontier_candidate_rejections = attack_plan_rejections
        except Exception as exc:
            frontier_actions_source = "contract_default_fallback"
            frontier_action_source_error = str(exc)
            frontier_actions_raw = ""
    elif not frontier_actions_raw:
        frontier_actions_source = "contract_default_fallback"

    try:
        frontier_actions = resolve_frontier_actions(
            frontier_actions_raw,
            contract=frontier_action_contract,
            base_url=container_base_url,
            allowed_origins=[allowed_origin],
            request_budget=request_budget,
        )
    except FrontierActionValidationError as exc:
        print(f"[adversarial-container] frontier action validation failed: {exc}", file=sys.stderr)
        return 1
    command_channel = prepare_command_channel(
        frontier_actions,
        queue_capacity=command_queue_capacity,
    )
    if command_channel["overflow_count"] > 0:
        print(
            "[adversarial-container] command channel overflow: "
            f"queued={command_channel['queued_action_count']} overflow={command_channel['overflow_count']}",
            file=sys.stderr,
        )
        return 1
    frontier_actions = list(command_channel["queued_actions"])

    try:
        ensure_image_built(args.image_tag, args.dockerfile)
    except Exception as exc:
        print(f"[adversarial-container] {exc}", file=sys.stderr)
        return 1

    forwarded_secret = os.environ.get("SHUMA_FORWARDED_IP_SECRET", "").strip()
    health_secret = os.environ.get("SHUMA_HEALTH_SECRET", "").strip()
    api_key = os.environ.get("SHUMA_API_KEY", "").strip()
    sim_tag_secret = os.environ.get("SHUMA_SIM_TELEMETRY_SECRET", "").strip()
    sim_tag_envelopes: List[Dict[str, str]] = []
    orchestrator_hook = {"hook": "orchestrator_reset", "performed": False, "reason": "not_applicable"}
    frontier_lineage: Dict[str, Any] = {
        "lineage_complete": False,
        "summary": {},
        "detail": "lineage_not_collected",
    }
    capability_verify_key = ""
    capability_envelopes: List[Dict[str, Any]] = []

    if args.mode == "blackbox":
        if not sim_tag_secret:
            print(
                "[adversarial-container] missing SHUMA_SIM_TELEMETRY_SECRET for blackbox sim-tag envelopes",
                file=sys.stderr,
            )
            return 1
        try:
            wait_ready(host_base_url, forwarded_secret, health_secret, timeout_seconds=30)
        except Exception as exc:
            print(f"[adversarial-container] {exc}", file=sys.stderr)
            return 1
        orchestrator_hook = orchestrator_reset_hook(host_base_url, api_key, forwarded_secret)
        sim_tag_envelopes = build_sim_tag_envelopes(
            secret=sim_tag_secret,
            run_id=run_id,
            profile=args.mode,
            lane="container_blackbox",
            count=request_budget,
        )

    sim_tag_envelopes_json = json.dumps(sim_tag_envelopes, separators=(",", ":"))
    frontier_actions_json = json.dumps(frontier_actions, separators=(",", ":"))
    if args.mode == "blackbox":
        capability_verify_key, capability_envelopes = build_action_capability_envelopes(
            sim_tag_secret,
            run_id,
            frontier_actions,
            ttl_seconds=min(300, max(30, time_budget_seconds)),
            key_id="sim-tag-derived-v1",
        )
    capability_envelopes_json = json.dumps(capability_envelopes, separators=(",", ":"))

    worker_failure_detail = ""
    try:
        (
            worker_payload,
            worker_result,
            command,
            runtime_profile_violations,
            execution_control,
        ) = run_container_worker(
            image_tag=args.image_tag,
            mode=args.mode,
            base_url=container_base_url,
            allowed_origin=allowed_origin,
            run_id=run_id,
            request_budget=request_budget,
            time_budget_seconds=time_budget_seconds,
            sim_tag_envelopes_json=sim_tag_envelopes_json,
            frontier_actions_json=frontier_actions_json,
            capability_envelopes_json=capability_envelopes_json,
            capability_verify_key=capability_verify_key,
            runtime_profile=runtime_profile,
            kill_switch_file=str(args.kill_switch_file),
            heartbeat_timeout_seconds=heartbeat_timeout_seconds,
            hard_deadline_buffer_seconds=hard_deadline_buffer_seconds,
        )
    except Exception as exc:
        worker_failure_detail = str(exc)
        worker_payload = {
            "passed": False,
            "errors": [worker_failure_detail],
            "policy_audit": [],
            "policy_violation_count": 0,
            "policy_violation_blocking": True,
            "capability_validation_passed": False,
        }
        worker_result = subprocess.CompletedProcess(
            args=[],
            returncode=1,
            stdout="",
            stderr=worker_failure_detail,
        )
        command = []
        runtime_profile_violations = []
        execution_control = {
            "kill_switch_file": str(args.kill_switch_file),
            "heartbeat_timeout_seconds": heartbeat_timeout_seconds,
            "hard_deadline_seconds": max(10, time_budget_seconds + hard_deadline_buffer_seconds),
            "termination_reason": worker_failure_detail,
            "stop_latency_ms": 0,
            "forced_kill": False,
        }
        print(f"[adversarial-container] worker execution failed: {worker_failure_detail}", file=sys.stderr)

    terminal_failure = parse_worker_failure_taxonomy(
        worker_failure_detail
        if worker_failure_detail
        else str(execution_control.get("termination_reason") or "")
    )

    if args.mode == "blackbox":
        try:
            events_payload = admin_read_json(
                host_base_url,
                api_key,
                forwarded_secret,
                "/admin/events?hours=24&limit=1000",
            )
            runtime_events = collect_run_events_from_payload(events_payload, run_id)
            monitoring_payload = admin_read_json(
                host_base_url,
                api_key,
                forwarded_secret,
                "/admin/monitoring?hours=24&limit=25",
            )
            monitoring_details = dict(monitoring_payload.get("details") or {})
            monitoring_events = collect_run_events_from_payload(
                dict(monitoring_details.get("events") or {}),
                run_id,
            )
            frontier_lineage_summary = build_frontier_lineage_summary(
                frontier_action_lineage,
                worker_payload,
                runtime_events,
                monitoring_events,
            )
            frontier_lineage = {
                "lineage_complete": bool(frontier_lineage_summary.get("lineage_complete")),
                "summary": frontier_lineage_summary,
                "detail": "ok",
            }
        except Exception as exc:
            frontier_lineage = {
                "lineage_complete": False,
                "summary": {},
                "detail": f"lineage_collection_error:{exc}",
            }

    frontier_runtime_state = build_frontier_runtime_state(
        mode=args.mode,
        frontier_actions_source=frontier_actions_source,
        frontier_action_source_error=frontier_action_source_error,
        frontier_lineage=frontier_lineage,
    )

    has_forbidden_env = not bool(worker_payload.get("admin_credentials_absent"))
    isolation_contract = {
        "container_process_boundary": True,
        "workspace_mount_absent": bool(worker_payload.get("workspace_mount_absent")),
        "shuma_env_absent": bool(worker_payload.get("admin_credentials_absent")),
        "admin_credentials_absent": bool(worker_payload.get("admin_credentials_absent")),
        "egress_allowlist_enforced": bool(worker_payload.get("egress_allowlist_enforced")),
        "tooling_limited_http_browser_class": bool(worker_payload.get("tooling_limited")),
        "ephemeral_run_identity": bool(worker_payload.get("ephemeral_run_identity")),
        "orchestrator_reset_hook_outside_container": bool(orchestrator_hook.get("performed"))
        if args.mode == "blackbox"
        else True,
        "runtime_hardening_non_root": bool(worker_payload.get("runtime_hardening_non_root")),
        "runtime_hardening_no_new_privileges": True,
        "runtime_hardening_cap_drop_all": True,
        "runtime_hardening_read_only_rootfs": True,
        "runtime_profile_passed": not bool(runtime_profile_violations),
        "command_channel_one_way": str(command_channel.get("direction") or "") == "host_to_worker_one_way",
        "command_channel_backpressure_passed": not bool(command_channel.get("overflow_count")),
        "action_grammar_reject_by_default": bool(worker_payload.get("action_validation_passed"))
        if args.mode == "blackbox"
        else True,
        "policy_violation_execution_rate_zero": int(
            worker_payload.get("policy_violation_count") or 0
        )
        == 0
        if args.mode == "blackbox"
        else True,
    }
    contract_pass = all(isolation_contract.values())

    passed = (
        bool(worker_payload.get("passed"))
        and worker_result.returncode == 0
        and contract_pass
        and not bool(frontier_runtime_state.get("degraded"))
    )
    report = {
        "schema_version": "adversarial-container-report.v1",
        "mode": args.mode,
        "passed": passed,
        "run_id": run_id,
        "host_base_url": host_base_url,
        "container_base_url": container_base_url,
        "allowed_origin": allowed_origin,
        "request_budget": request_budget,
        "time_budget_seconds": time_budget_seconds,
        "frontier_action_contract": {
            "path": str(args.frontier_action_contract),
            "schema_version": str(frontier_action_contract.get("schema_version") or ""),
            "allowed_tools": list(frontier_action_contract.get("allowed_tools") or []),
            "max_actions_per_run": contract_max_actions,
            "max_time_budget_seconds": contract_max_runtime,
        },
        "runtime_profile": {
            "path": str(args.runtime_profile),
            "schema_version": str(runtime_profile.get("schema_version") or ""),
            "required_user_mode": str(runtime_profile.get("required_user_mode") or ""),
            "violations": runtime_profile_violations,
            "passed": not bool(runtime_profile_violations),
        },
        "capability_envelopes": {
            "count": len(capability_envelopes),
            "key_id": "sim-tag-derived-v1" if capability_envelopes else "",
            "verify_key_present": bool(capability_verify_key),
        },
        "command_channel": {
            "direction": str(command_channel.get("direction") or ""),
            "queue_capacity": int(command_channel.get("queue_capacity") or 0),
            "queued_action_count": int(command_channel.get("queued_action_count") or 0),
            "overflow_count": int(command_channel.get("overflow_count") or 0),
            "backpressure_applied": bool(command_channel.get("backpressure_applied")),
            "evidence_channel_append_only_expected": bool(
                command_channel.get("evidence_channel_append_only_expected")
            ),
            "control_plane_mutation_allowed": bool(
                command_channel.get("control_plane_mutation_allowed")
            ),
        },
        "attack_plan_path": str(args.attack_plan),
        "frontier_actions": frontier_actions,
        "frontier_action_source": frontier_actions_source,
        "frontier_action_source_error": frontier_action_source_error,
        "frontier_action_lineage": frontier_action_lineage,
        "frontier_candidate_rejections": frontier_candidate_rejections,
        "frontier_lineage": frontier_lineage,
        "frontier_runtime_state": frontier_runtime_state,
        "isolation_contract": isolation_contract,
        "orchestrator_hook": orchestrator_hook,
        "worker_result": {
            "exit_code": worker_result.returncode,
            "stdout_tail": worker_result.stdout.splitlines()[-20:],
            "stderr_tail": worker_result.stderr.splitlines()[-20:],
        },
        "worker_payload": worker_payload,
        "policy_audit": {
            "violation_count": int(worker_payload.get("policy_violation_count") or 0),
            "blocking": bool(worker_payload.get("policy_violation_blocking")),
            "events": list(worker_payload.get("policy_audit") or []),
        },
        "runtime_flags": {
            "docker_command": command,
        },
        "execution_control": execution_control,
        "terminal_failure": terminal_failure,
        "worker_failure_detail": worker_failure_detail,
        "generated_at_unix": int(time.time()),
    }

    report_path = report_path_for_mode(args.mode, args.report)
    cleanup_policy = cleanup_frontier_artifacts(
        report_path.parent,
        ttl_hours=cleanup_ttl_hours,
        max_delete=cleanup_max_delete,
    )
    report["cleanup_policy"] = cleanup_policy
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"[adversarial-container] report={report_path}")
    print(
        "[adversarial-container] mode={} passed={} contract_pass={} exit_code={}".format(
            args.mode, passed, contract_pass, worker_result.returncode
        )
    )
    if not passed:
        print("[adversarial-container] isolation_contract=" + json.dumps(isolation_contract, sort_keys=True))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
