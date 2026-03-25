#!/usr/bin/env python3
"""Host-side LLM runtime worker for bounded bot_red_team execution."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import urllib.parse
import subprocess
import sys
import tempfile
import time
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tests.adversarial_runner import llm_fulfillment


LLM_RUNTIME_RESULT_SCHEMA_VERSION = "adversary-sim-llm-runtime-result.v1"
DEFAULT_PUBLIC_HINT_PATHS = ["/robots.txt"]


class WorkerConfigError(ValueError):
    """Raised when required worker inputs are missing or invalid."""


def _load_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise WorkerConfigError(f"JSON payload at {path} must be an object")
    return payload


def extract_llm_fulfillment_plan(beat_response_payload: dict[str, Any]) -> dict[str, Any]:
    plan = beat_response_payload.get("llm_fulfillment_plan")
    if not isinstance(plan, dict):
        raise RuntimeError("beat response must include nested llm_fulfillment_plan object")
    return dict(plan)


def _normalized_host_root_entrypoint(base_url: str) -> str:
    parsed = urllib.parse.urlparse(str(base_url or "").strip())
    if not parsed.scheme or not parsed.netloc:
        raise RuntimeError("host_root_entrypoint must be an absolute URL")
    return urllib.parse.urlunparse(
        (
            parsed.scheme.lower(),
            parsed.netloc,
            "/",
            "",
            "",
            "",
        )
    )


def _action_receipts(
    generation_result: dict[str, Any],
    report_payload: dict[str, Any] | None,
) -> list[dict[str, Any]]:
    actions = [
        dict(item)
        for item in list(generation_result.get("actions") or [])
        if isinstance(item, dict)
    ]
    traffic = []
    if isinstance(report_payload, dict):
        worker_payload = dict(report_payload.get("worker_payload") or {})
        traffic = [
            dict(item)
            for item in list(worker_payload.get("traffic") or [])
            if isinstance(item, dict)
        ]
    traffic_by_index = {
        int(item.get("action_index") or index + 1): item
        for index, item in enumerate(traffic)
    }

    receipts: list[dict[str, Any]] = []
    for index, action in enumerate(actions):
        action_index = int(action.get("action_index") or index + 1)
        traffic_row = traffic_by_index.get(action_index, {})
        receipt = {
            "action_index": action_index,
            "action_type": str(action.get("action_type") or "").strip(),
            "path": str(action.get("path") or "").strip() or "/",
            "label": str(action.get("label") or "").strip() or None,
            "status": traffic_row.get("status"),
            "error": str(traffic_row.get("error") or "").strip() or None,
        }
        receipts.append(receipt)
    return receipts


def _report_terminal_failure(report_payload: dict[str, Any] | None) -> str | None:
    if not isinstance(report_payload, dict):
        return None
    terminal = report_payload.get("terminal_failure")
    if isinstance(terminal, dict):
        value = str(terminal.get("terminal_failure") or "").strip()
        return value if value and value.lower() != "none" else None
    value = str(terminal or "").strip()
    return value if value and value.lower() != "none" else None


def _report_error(report_payload: dict[str, Any] | None) -> str | None:
    if not isinstance(report_payload, dict):
        return None
    worker_failure_detail = str(report_payload.get("worker_failure_detail") or "").strip()
    if worker_failure_detail:
        return worker_failure_detail
    worker_payload = dict(report_payload.get("worker_payload") or {})
    errors = [
        str(item).strip()
        for item in list(worker_payload.get("errors") or [])
        if str(item).strip()
    ]
    if errors:
        return errors[0]
    terminal = report_payload.get("terminal_failure")
    if isinstance(terminal, dict):
        reason = str(terminal.get("reason") or "").strip()
        if reason:
            return reason
    return None


def _report_failure_class(
    report_payload: dict[str, Any] | None,
    *,
    action_receipts: list[dict[str, Any]],
) -> str | None:
    if not isinstance(report_payload, dict):
        return None
    terminal_failure = _report_terminal_failure(report_payload)
    if terminal_failure in {"deadline_exceeded", "heartbeat_loss"}:
        return "timeout"
    if terminal_failure in {"forced_kill_path", "cancelled"}:
        return "cancelled"

    worker_payload = dict(report_payload.get("worker_payload") or {})
    if any(str(item).strip() for item in list(worker_payload.get("errors") or [])):
        if any(receipt.get("status") == 0 for receipt in action_receipts):
            return "transport"
        return "transport"

    if any(receipt.get("status") not in (None, 200, 302, 303, 403, 404, 429) for receipt in action_receipts):
        return "http"
    if any(receipt.get("status") is not None and int(receipt.get("status") or 0) == 0 for receipt in action_receipts):
        return "transport"
    return None


def build_llm_runtime_result(
    *,
    fulfillment_plan: dict[str, Any],
    generation_result: dict[str, Any],
    report_payload: dict[str, Any] | None,
    tick_completed_at: int,
    worker_id: str,
    error: str | None = None,
    failure_class: str | None = None,
    terminal_failure: str | None = None,
) -> dict[str, Any]:
    action_receipts = _action_receipts(generation_result, report_payload)
    generated_action_count = len(
        [item for item in list(generation_result.get("actions") or []) if isinstance(item, dict)]
    )
    worker_payload = dict(report_payload.get("worker_payload") or {}) if isinstance(report_payload, dict) else {}
    traffic = [
        dict(item)
        for item in list(worker_payload.get("traffic") or [])
        if isinstance(item, dict)
    ]
    executed_action_count = int(worker_payload.get("requests_sent") or len(traffic))
    failed_action_count = sum(
        1
        for receipt in action_receipts
        if receipt.get("error")
        or int(receipt.get("status") or 0) == 0
    )
    last_response_status = None
    if traffic:
        last_response_status = traffic[-1].get("status")

    derived_error = error or _report_error(report_payload)
    derived_terminal_failure = terminal_failure or _report_terminal_failure(report_payload)
    derived_failure_class = failure_class or _report_failure_class(
        report_payload,
        action_receipts=action_receipts,
    )

    passed = bool(report_payload.get("passed")) if isinstance(report_payload, dict) else False
    if derived_error or derived_terminal_failure or derived_failure_class:
        passed = False

    return {
        "schema_version": LLM_RUNTIME_RESULT_SCHEMA_VERSION,
        "run_id": str(fulfillment_plan.get("run_id") or "").strip(),
        "tick_id": str(fulfillment_plan.get("tick_id") or "").strip(),
        "lane": str(fulfillment_plan.get("lane") or "").strip() or "bot_red_team",
        "fulfillment_mode": str(fulfillment_plan.get("fulfillment_mode") or "").strip(),
        "worker_id": str(worker_id).strip(),
        "tick_started_at": int(fulfillment_plan.get("tick_started_at") or 0),
        "tick_completed_at": int(tick_completed_at),
        "backend_kind": str(fulfillment_plan.get("backend_kind") or "").strip(),
        "backend_state": str(fulfillment_plan.get("backend_state") or "").strip(),
        "generation_source": str(generation_result.get("generation_source") or "runtime_failure").strip(),
        "provider": str(generation_result.get("provider") or "").strip(),
        "model_id": str(generation_result.get("model_id") or "").strip(),
        "fallback_reason": str(generation_result.get("fallback_reason") or "").strip() or None,
        "category_targets": [
            str(item).strip()
            for item in list(fulfillment_plan.get("category_targets") or [])
            if str(item).strip()
        ],
        "generated_action_count": generated_action_count,
        "executed_action_count": executed_action_count,
        "failed_action_count": failed_action_count,
        "last_response_status": last_response_status,
        "passed": passed,
        "failure_class": derived_failure_class,
        "error": derived_error,
        "terminal_failure": derived_terminal_failure,
        "action_receipts": action_receipts,
    }


def run_request_mode_blackbox(
    *,
    base_url: str,
    fulfillment_plan: dict[str, Any],
    generation_result: dict[str, Any],
    runner: Any = subprocess.run,
    report_path: Path | None = None,
) -> dict[str, Any]:
    capability_envelope = dict(fulfillment_plan.get("capability_envelope") or {})
    request_budget = max(
        1,
        int(capability_envelope.get("max_actions") or len(list(generation_result.get("actions") or [])) or 1),
    )
    time_budget_seconds = max(
        10,
        int(capability_envelope.get("max_time_budget_seconds") or 120),
    )
    if report_path is None:
        report_fd, report_file = tempfile.mkstemp(
            prefix="shuma-llm-runtime-report-",
            suffix=".json",
        )
        os.close(report_fd)
        report_output_path = Path(report_file)
    else:
        report_output_path = report_path
    command = [
        sys.executable,
        str(REPO_ROOT / "scripts" / "tests" / "adversarial_container_runner.py"),
        "--mode",
        "blackbox",
        "--base-url",
        str(base_url).strip(),
        "--frontier-actions",
        json.dumps(list(generation_result.get("actions") or []), separators=(",", ":")),
        "--request-budget",
        str(request_budget),
        "--time-budget-seconds",
        str(time_budget_seconds),
        "--report",
        str(report_output_path),
    ]
    completed = runner(
        command,
        capture_output=True,
        text=True,
        check=False,
        cwd=str(REPO_ROOT),
    )
    if not report_output_path.exists():
        raise RuntimeError(
            "container_runner_report_missing:"
            f"exit_code={completed.returncode}:stderr={str(completed.stderr or '').strip()}"
        )
    try:
        payload = json.loads(report_output_path.read_text(encoding="utf-8"))
    finally:
        report_output_path.unlink(missing_ok=True)
    if not isinstance(payload, dict):
        raise RuntimeError("container_runner_report_invalid")
    payload["_runner_exit_code"] = int(completed.returncode)
    payload["_runner_stdout"] = str(completed.stdout or "")
    payload["_runner_stderr"] = str(completed.stderr or "")
    return payload


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run bounded LLM runtime actions for the bot_red_team lane"
    )
    parser.add_argument("--beat-response-file", required=True)
    parser.add_argument("--result-output-file", required=True)
    parser.add_argument(
        "--base-url",
        default=os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000"),
        help="Public host root entrypoint for black-box attacker execution.",
    )
    parser.add_argument(
        "--public-hint-path",
        action="append",
        dest="public_hint_paths",
        help="Optional additional public host-derived hint path.",
    )
    args = parser.parse_args()

    beat_response_payload = _load_json(Path(args.beat_response_file))
    fulfillment_plan = extract_llm_fulfillment_plan(beat_response_payload)
    base_url = _normalized_host_root_entrypoint(str(args.base_url or "").strip())
    public_hint_paths = list(args.public_hint_paths or DEFAULT_PUBLIC_HINT_PATHS)
    tick_completed_at = int(time.time())
    worker_id = f"llm-runtime-worker-{os.getpid()}"

    generation_result = llm_fulfillment.generate_llm_frontier_actions(
        fulfillment_plan=fulfillment_plan,
        host_root_entrypoint=base_url,
        public_hint_paths=public_hint_paths,
    )

    if str(fulfillment_plan.get("fulfillment_mode") or "").strip() == "browser_mode":
        result = build_llm_runtime_result(
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
            report_payload=None,
            tick_completed_at=tick_completed_at,
            worker_id=worker_id,
            error="browser_mode_dispatch_not_yet_supported_by_blackbox_worker",
            failure_class="transport",
            terminal_failure="browser_mode_not_supported",
        )
    else:
        report_payload = run_request_mode_blackbox(
            base_url=base_url,
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
        )
        result = build_llm_runtime_result(
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
            report_payload=report_payload,
            tick_completed_at=tick_completed_at,
            worker_id=worker_id,
        )

    Path(args.result_output_file).write_text(
        json.dumps(result, separators=(",", ":")),
        encoding="utf-8",
    )
    return 0 if bool(result.get("passed")) else 1


if __name__ == "__main__":
    raise SystemExit(main())
