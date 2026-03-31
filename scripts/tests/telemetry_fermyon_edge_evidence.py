#!/usr/bin/env python3
"""Capture live telemetry-read evidence for the current Fermyon Akamai-edge deployment."""

from __future__ import annotations

import argparse
import json
import ssl
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.local_env import read_env_file
from scripts.tests.telemetry_evidence_common import (
    evaluate_budget_report as evaluate_budget_report_common,
    summarize_recent_event_rows,
    utc_now_iso,
)

DEFAULT_ENV_FILE = REPO_ROOT / ".env.local"
DEFAULT_RECEIPT_PATH = REPO_ROOT / ".shuma" / "fermyon-akamai-edge-deploy.json"
DEFAULT_REPORT_PATH = REPO_ROOT / ".spin" / "telemetry_fermyon_edge_evidence.json"
DEFAULT_HOURS = 24
DEFAULT_BOOTSTRAP_LIMIT = 10
DEFAULT_DELTA_LIMIT = 40
DEFAULT_FORENSIC_LIMIT = 40
BOOTSTRAP_BUDGET_MS = 2000.0
DELTA_BUDGET_MS = 750.0


class EvidenceFailure(RuntimeError):
    pass


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Capture live telemetry-read evidence for the current Fermyon Akamai-edge deployment."
    )
    parser.add_argument("--env-file", default=str(DEFAULT_ENV_FILE))
    parser.add_argument("--receipt-path", default=str(DEFAULT_RECEIPT_PATH))
    parser.add_argument("--hours", type=int, default=DEFAULT_HOURS)
    parser.add_argument("--report-path", default=str(DEFAULT_REPORT_PATH))
    return parser.parse_args(argv)


def evaluate_budget_report(
    *,
    bootstrap_measurement: dict[str, Any],
    delta_measurement: dict[str, Any],
) -> dict[str, float | bool]:
    return evaluate_budget_report_common(
        bootstrap_measurement=bootstrap_measurement,
        delta_measurement=delta_measurement,
        bootstrap_budget_ms=BOOTSTRAP_BUDGET_MS,
        delta_budget_ms=DELTA_BUDGET_MS,
    )


def build_evidence_report(
    *,
    receipt: dict[str, Any],
    bootstrap_measurement: dict[str, Any],
    delta_measurement: dict[str, Any],
    forensic_measurement: dict[str, Any],
) -> dict[str, Any]:
    budgets = evaluate_budget_report(
        bootstrap_measurement=bootstrap_measurement,
        delta_measurement=delta_measurement,
    )
    shaping_reason = (
        bootstrap_measurement.get("payload", {})
        .get("details", {})
        .get("events", {})
        .get("recent_events_window", {})
        .get("response_shaping_reason")
    )
    forensic_events = (
        forensic_measurement.get("payload", {})
        .get("details", {})
        .get("events", {})
        .get("recent_events", [])
    )
    fermyon = receipt.get("fermyon", {})
    return {
        "captured_at_utc": utc_now_iso(),
        "edge": {
            "app_id": fermyon.get("app_id"),
            "app_name": fermyon.get("app_name"),
            "account_name": fermyon.get("account_name"),
            "base_url": fermyon.get("primary_url"),
            "git_head": receipt.get("git_head"),
        },
        "budgets": budgets,
        "recent_event_sample": summarize_recent_event_rows(
            forensic_events if isinstance(forensic_events, list) else []
        ),
        "payloads": {
            "monitoring_bootstrap": {
                "status": bootstrap_measurement.get("status"),
                "latency_ms": bootstrap_measurement.get("latency_ms"),
                "response_bytes": bootstrap_measurement.get("response_bytes"),
                "content_encoding": bootstrap_measurement.get("content_encoding"),
                "response_shaping_reason": shaping_reason,
            },
            "monitoring_delta": {
                "status": delta_measurement.get("status"),
                "latency_ms": delta_measurement.get("latency_ms"),
                "response_bytes": delta_measurement.get("response_bytes"),
                "content_encoding": delta_measurement.get("content_encoding"),
            },
        },
    }


class TelemetryFermyonEdgeEvidence:
    def __init__(self, *, env_file: Path, receipt_path: Path, report_path: Path, hours: int = DEFAULT_HOURS) -> None:
        self.env_file = env_file
        self.receipt_path = receipt_path
        self.report_path = report_path
        self.hours = max(1, int(hours))
        self.local_env = read_env_file(env_file)
        self.api_key = self.local_env.get("SHUMA_API_KEY", "").strip()
        if not self.api_key:
            raise EvidenceFailure("SHUMA_API_KEY must be present in the selected env file.")
        if not self.receipt_path.exists():
            raise EvidenceFailure(f"Fermyon deploy receipt not found: {self.receipt_path}")
        self.receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.base_url = str(self.receipt.get("fermyon", {}).get("primary_url", "")).strip().rstrip("/")
        if not self.base_url:
            raise EvidenceFailure("Fermyon deploy receipt must include fermyon.primary_url.")
        self.ssl_context = None if not self.base_url.endswith(".sslip.io") else ssl._create_unverified_context()

    def _request(self, path: str) -> dict[str, Any]:
        request = urllib.request.Request(
            self.base_url + path,
            method="GET",
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
            },
        )
        started = time.perf_counter()
        try:
            with urllib.request.urlopen(request, timeout=20, context=self.ssl_context) as response:
                raw = response.read()
                status = int(response.status)
                content_type = response.headers.get("Content-Type", "")
                content_encoding = (response.headers.get("Content-Encoding", "") or "none").lower()
        except urllib.error.HTTPError as exc:
            raw = exc.read()
            status = int(exc.code)
            content_type = exc.headers.get("Content-Type", "")
            content_encoding = (exc.headers.get("Content-Encoding", "") or "none").lower()
        elapsed_ms = round((time.perf_counter() - started) * 1000.0, 2)
        if status != 200:
            raise EvidenceFailure(f"GET {path} returned {status}")
        payload: Any
        if "application/json" in content_type.lower():
            payload = json.loads(raw.decode("utf-8"))
        else:
            payload = raw.decode("utf-8", errors="replace")
        return {
            "status": status,
            "latency_ms": elapsed_ms,
            "response_bytes": len(raw),
            "content_encoding": content_encoding,
            "payload": payload,
        }

    def measure_json_endpoint(self, path: str) -> dict[str, Any]:
        return self._request(path)

    def run(self) -> dict[str, Any]:
        bootstrap_measurement = self.measure_json_endpoint(
            f"/shuma/admin/monitoring?hours={self.hours}&limit={DEFAULT_BOOTSTRAP_LIMIT}&bootstrap=1"
        )
        delta_measurement = self.measure_json_endpoint(
            f"/shuma/admin/monitoring/delta?hours={self.hours}&limit={DEFAULT_DELTA_LIMIT}"
        )
        forensic_measurement = self.measure_json_endpoint(
            f"/shuma/admin/monitoring?hours={self.hours}&limit={DEFAULT_FORENSIC_LIMIT}&bootstrap=1&forensic=1"
        )
        report = build_evidence_report(
            receipt=self.receipt,
            bootstrap_measurement=bootstrap_measurement,
            delta_measurement=delta_measurement,
            forensic_measurement=forensic_measurement,
        )
        if not report["budgets"]["bootstrap_within_budget"] or not report["budgets"]["delta_within_budget"]:
            raise EvidenceFailure(
                "Fermyon edge telemetry hot-read budgets exceeded: "
                f"bootstrap={report['payloads']['monitoring_bootstrap']['latency_ms']}ms "
                f"(budget={report['budgets']['bootstrap_budget_ms']}ms), "
                f"delta={report['payloads']['monitoring_delta']['latency_ms']}ms "
                f"(budget={report['budgets']['delta_budget_ms']}ms)."
            )
        self.report_path.parent.mkdir(parents=True, exist_ok=True)
        self.report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return report


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    collector = TelemetryFermyonEdgeEvidence(
        env_file=Path(args.env_file).expanduser(),
        receipt_path=Path(args.receipt_path).expanduser(),
        report_path=Path(args.report_path).expanduser(),
        hours=args.hours,
    )
    report = collector.run()
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
