#!/usr/bin/env python3
"""Deterministic SIM2 realtime benchmark harness with threshold diagnostics."""

from __future__ import annotations

import argparse
import json
import math
import sys
import time
from pathlib import Path
from typing import Any, Dict, List

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tests.adversarial_artifact_paths import (
    SIM2_REALTIME_BENCH_REPORT_PATH,
    SIM2_REALTIME_BENCH_SUMMARY_PATH,
)

DEFAULT_OUTPUT_PATH = SIM2_REALTIME_BENCH_REPORT_PATH
DEFAULT_SUMMARY_PATH = SIM2_REALTIME_BENCH_SUMMARY_PATH

SSE_P95_MAX_MS = 300
SSE_P99_MAX_MS = 500
SSE_OVERFLOW_OR_DROP_MAX = 0
SSE_REQUEST_BUDGET_MAX_REQ_PER_SEC_CLIENT = 1.0
BASELINE_EVENTS_PER_SEC = 1000
BASELINE_OPERATOR_CLIENTS = 5


def percentile(values: List[int], pct: float) -> int:
    if not values:
        return 0
    ordered = sorted(values)
    index = min(len(ordered) - 1, max(0, int(math.ceil(len(ordered) * pct) - 1)))
    return int(ordered[index])


def summarize_latencies(latencies_ms: List[int]) -> Dict[str, float]:
    if not latencies_ms:
        return {"p50_ms": 0, "p95_ms": 0, "p99_ms": 0, "mean_ms": 0.0}
    total = float(sum(latencies_ms))
    count = float(len(latencies_ms))
    return {
        "p50_ms": float(percentile(latencies_ms, 0.50)),
        "p95_ms": float(percentile(latencies_ms, 0.95)),
        "p99_ms": float(percentile(latencies_ms, 0.99)),
        "mean_ms": round(total / count, 2),
    }


def generate_constant_events(duration_ms: int, events_per_sec: int) -> List[int]:
    events: List[int] = []
    accumulator = 0.0
    per_ms = float(events_per_sec) / 1000.0
    for timestamp_ms in range(max(0, int(duration_ms))):
        accumulator += per_ms
        emit_count = int(math.floor(accumulator))
        accumulator -= float(emit_count)
        for _ in range(emit_count):
            events.append(timestamp_ms)
    return events


def simulate_cursor_polling(
    event_ts_ms: List[int],
    *,
    duration_ms: int,
    clients: int,
    poll_interval_ms: int,
    delta_limit: int,
) -> Dict[str, Any]:
    if poll_interval_ms <= 0:
        raise ValueError("poll_interval_ms must be >= 1")
    duration_ms = max(1, int(duration_ms))
    clients = max(1, int(clients))
    delta_limit = max(1, int(delta_limit))
    cursors = [0 for _ in range(clients)]
    event_upper_idx = 0
    latencies: List[int] = []
    delivered = 0
    overflow = 0
    calls = 0
    poll_ticks = 0

    current_ms = 0
    while current_ms <= duration_ms:
        poll_ticks += 1
        while event_upper_idx < len(event_ts_ms) and event_ts_ms[event_upper_idx] <= current_ms:
            event_upper_idx += 1
        for client_idx in range(clients):
            calls += 1
            available = max(0, event_upper_idx - cursors[client_idx])
            take = min(available, delta_limit)
            if take == 0:
                continue
            start_idx = cursors[client_idx]
            end_idx = start_idx + take
            for idx in range(start_idx, end_idx):
                latencies.append(max(0, current_ms - event_ts_ms[idx]))
            delivered += take
            cursors[client_idx] = end_idx
            if available > delta_limit:
                overflow += available - delta_limit
        current_ms += poll_interval_ms

    duration_seconds = max(1.0, float(duration_ms) / 1000.0)
    avg_req_per_sec_client = float(calls) / (duration_seconds * float(clients))
    return {
        "delivery_mode": "cursor_polling",
        "poll_interval_ms": poll_interval_ms,
        "delta_limit": delta_limit,
        "clients": clients,
        "poll_ticks": poll_ticks,
        "delivered_events": delivered,
        "overflow_or_drop_count": overflow,
        "requests_or_connections": calls,
        "avg_requests_per_sec_client": round(avg_req_per_sec_client, 3),
        "latency": summarize_latencies(latencies),
    }


def simulate_sse(
    event_ts_ms: List[int],
    *,
    duration_ms: int,
    clients: int,
    consume_interval_ms: int,
    queue_capacity: int,
) -> Dict[str, Any]:
    if consume_interval_ms <= 0:
        raise ValueError("consume_interval_ms must be >= 1")
    duration_ms = max(1, int(duration_ms))
    clients = max(1, int(clients))
    queue_capacity = max(1, int(queue_capacity))
    queues: List[List[int]] = [[] for _ in range(clients)]
    latencies: List[int] = []
    delivered = 0
    dropped = 0
    peak_queue_depth = 0
    event_idx = 0

    for current_ms in range(0, duration_ms + 1):
        while event_idx < len(event_ts_ms) and event_ts_ms[event_idx] <= current_ms:
            for queue in queues:
                if len(queue) >= queue_capacity:
                    dropped += 1
                else:
                    queue.append(event_idx)
                    if len(queue) > peak_queue_depth:
                        peak_queue_depth = len(queue)
            event_idx += 1

        if current_ms % consume_interval_ms != 0:
            continue
        for queue in queues:
            while queue:
                idx = queue.pop(0)
                latencies.append(max(0, current_ms - event_ts_ms[idx]))
                delivered += 1

    duration_seconds = max(1.0, float(duration_ms) / 1000.0)
    requests_or_connections = clients
    avg_req_per_sec_client = float(requests_or_connections) / (duration_seconds * float(clients))
    return {
        "delivery_mode": "sse",
        "consume_interval_ms": consume_interval_ms,
        "queue_capacity": queue_capacity,
        "clients": clients,
        "delivered_events": delivered,
        "overflow_or_drop_count": dropped,
        "requests_or_connections": requests_or_connections,
        "avg_requests_per_sec_client": round(avg_req_per_sec_client, 3),
        "peak_queue_depth": peak_queue_depth,
        "latency": summarize_latencies(latencies),
    }


def evaluate_thresholds(results: Dict[str, Dict[str, Any]]) -> List[str]:
    failures: List[str] = []
    sse = dict(results.get("sse") or {})
    latency = dict(sse.get("latency") or {})
    p95 = float(latency.get("p95_ms") or 0.0)
    p99 = float(latency.get("p99_ms") or 0.0)
    overflow_or_drop = int(sse.get("overflow_or_drop_count") or 0)
    avg_req_per_sec_client = float(sse.get("avg_requests_per_sec_client") or 0.0)
    if p95 > SSE_P95_MAX_MS:
        failures.append(
            f"latency_p95_exceeded:required<={SSE_P95_MAX_MS} observed={p95:.2f}"
        )
    if p99 > SSE_P99_MAX_MS:
        failures.append(
            f"latency_p99_exceeded:required<={SSE_P99_MAX_MS} observed={p99:.2f}"
        )
    if overflow_or_drop > SSE_OVERFLOW_OR_DROP_MAX:
        failures.append(
            "overflow_or_drop_exceeded:"
            f"required<={SSE_OVERFLOW_OR_DROP_MAX} observed={overflow_or_drop}"
        )
    if avg_req_per_sec_client > SSE_REQUEST_BUDGET_MAX_REQ_PER_SEC_CLIENT:
        failures.append(
            "request_budget_exceeded:"
            f"required<={SSE_REQUEST_BUDGET_MAX_REQ_PER_SEC_CLIENT} observed={avg_req_per_sec_client:.3f}"
        )
    return failures


def run_benchmark(now_unix: int) -> Dict[str, Any]:
    duration_ms = 120_000
    events = generate_constant_events(duration_ms, events_per_sec=BASELINE_EVENTS_PER_SEC)
    results = {
        "cursor_polling_default": simulate_cursor_polling(
            events,
            duration_ms=duration_ms,
            clients=BASELINE_OPERATOR_CLIENTS,
            poll_interval_ms=1000,
            delta_limit=600,
        ),
        "cursor_polling_fast": simulate_cursor_polling(
            events,
            duration_ms=duration_ms,
            clients=BASELINE_OPERATOR_CLIENTS,
            poll_interval_ms=250,
            delta_limit=400,
        ),
        "sse": simulate_sse(
            events,
            duration_ms=duration_ms,
            clients=BASELINE_OPERATOR_CLIENTS,
            consume_interval_ms=200,
            queue_capacity=1024,
        ),
    }
    failures = evaluate_thresholds(results)
    payload: Dict[str, Any] = {
        "schema_version": "sim2-realtime-bench.v1",
        "generated_at_unix": int(now_unix),
        "workload": {
            "profile": "baseline",
            "duration_ms": duration_ms,
            "events_per_sec": BASELINE_EVENTS_PER_SEC,
            "operator_clients": BASELINE_OPERATOR_CLIENTS,
        },
        "verification_scope": {
            "harness_type": "synthetic_benchmark",
            "runtime_profile_claims": {
                "runtime_dev": "synthetic_contract_check_only",
                "runtime_prod": "not_verified_by_this_harness",
            },
            "claims_runtime_prod_verification": False,
        },
        "thresholds": {
            "sse_latency_p95_max_ms": SSE_P95_MAX_MS,
            "sse_latency_p99_max_ms": SSE_P99_MAX_MS,
            "sse_overflow_or_drop_max": SSE_OVERFLOW_OR_DROP_MAX,
            "sse_request_budget_max_req_per_sec_client": SSE_REQUEST_BUDGET_MAX_REQ_PER_SEC_CLIENT,
        },
        "results": results,
        "status": {
            "passed": len(failures) == 0,
            "failures": failures,
        },
        "diagnostics": {
            "latency_percentiles": {
                "cursor_polling_default": results["cursor_polling_default"]["latency"],
                "cursor_polling_fast": results["cursor_polling_fast"]["latency"],
                "sse": results["sse"]["latency"],
            },
            "overflow_or_drop_counts": {
                "cursor_polling_default": results["cursor_polling_default"][
                    "overflow_or_drop_count"
                ],
                "cursor_polling_fast": results["cursor_polling_fast"][
                    "overflow_or_drop_count"
                ],
                "sse": results["sse"]["overflow_or_drop_count"],
            },
            "request_budget_metrics": {
                "cursor_polling_default_avg_req_per_sec_client": results[
                    "cursor_polling_default"
                ]["avg_requests_per_sec_client"],
                "cursor_polling_fast_avg_req_per_sec_client": results[
                    "cursor_polling_fast"
                ]["avg_requests_per_sec_client"],
                "sse_avg_req_per_sec_client": results["sse"][
                    "avg_requests_per_sec_client"
                ],
            },
        },
    }
    return payload


def render_summary(payload: Dict[str, Any]) -> str:
    status = dict(payload.get("status") or {})
    diagnostics = dict(payload.get("diagnostics") or {})
    latencies = dict(diagnostics.get("latency_percentiles") or {})
    overflow_counts = dict(diagnostics.get("overflow_or_drop_counts") or {})
    request_budgets = dict(diagnostics.get("request_budget_metrics") or {})
    lines: List[str] = []
    lines.append("# SIM2 Realtime Benchmark Summary")
    lines.append("")
    lines.append(
        "- status: {}".format("PASS" if bool(status.get("passed")) else "FAIL")
    )
    scope = dict(payload.get("verification_scope") or {})
    runtime_claims = dict(scope.get("runtime_profile_claims") or {})
    lines.append(
        "- verification scope: harness_type={} runtime_dev={} runtime_prod={} claims_runtime_prod_verification={}".format(
            scope.get("harness_type", "unknown"),
            runtime_claims.get("runtime_dev", "unknown"),
            runtime_claims.get("runtime_prod", "unknown"),
            bool(scope.get("claims_runtime_prod_verification")),
        )
    )
    failures = list(status.get("failures") or [])
    if failures:
        lines.append("- failures:")
        for failure in failures:
            lines.append(f"  - {failure}")
    lines.append("- latency percentiles:")
    for mode in ("cursor_polling_default", "cursor_polling_fast", "sse"):
        row = dict(latencies.get(mode) or {})
        lines.append(
            "  - {}: p50={}ms p95={}ms p99={}ms".format(
                mode,
                row.get("p50_ms", 0),
                row.get("p95_ms", 0),
                row.get("p99_ms", 0),
            )
        )
    lines.append("- overflow/drop counts:")
    for mode in ("cursor_polling_default", "cursor_polling_fast", "sse"):
        lines.append(f"  - {mode}: {overflow_counts.get(mode, 0)}")
    lines.append("- request budget metrics (avg req/sec/client):")
    for key in sorted(request_budgets.keys()):
        lines.append(f"  - {key}: {request_budgets[key]}")
    return "\n".join(lines) + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run deterministic SIM2 realtime benchmark and emit diagnostics artifacts."
    )
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT_PATH))
    parser.add_argument("--summary", default=str(DEFAULT_SUMMARY_PATH))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    output_path = Path(args.output)
    summary_path = Path(args.summary)
    payload = run_benchmark(now_unix=int(time.time()))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_text(render_summary(payload), encoding="utf-8")
    print(f"[sim2-realtime-bench] report={output_path}")
    print(f"[sim2-realtime-bench] summary={summary_path}")
    if not bool(dict(payload.get("status") or {}).get("passed")):
        print("[sim2-realtime-bench] FAIL")
        for failure in list(dict(payload.get("status") or {}).get("failures") or []):
            print(f"- {failure}")
        return 1
    print("[sim2-realtime-bench] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
