#!/usr/bin/env python3
"""Capture shared-host telemetry storage/query evidence for the active ssh_systemd remote."""

from __future__ import annotations

import argparse
import gzip
import json
import shlex
import ssl
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from urllib.parse import urlparse
import urllib.error
import urllib.request

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.local_env import read_env_file
from scripts.deploy.remote_target import (
    DEFAULT_ENV_FILE,
    DEFAULT_REMOTE_RECEIPTS_DIR,
    select_remote,
    ssh_command_for_operation,
)
from scripts.tests.telemetry_evidence_common import (
    compression_ratio_percent,
    evaluate_budget_report as evaluate_budget_report_common,
    utc_now_iso,
)

DEFAULT_REPORT_PATH = REPO_ROOT / ".spin" / "telemetry_shared_host_evidence.json"
DEFAULT_HOURS = 24
DEFAULT_BOOTSTRAP_LIMIT = 10
DEFAULT_DELTA_LIMIT = 40
BOOTSTRAP_BUDGET_MS = 750.0
DELTA_BUDGET_MS = 250.0
REMOTE_SQLITE_KV_PATH = ".spin/sqlite_key_value.db"
TARPIT_ACTIVE_BUCKET_PREFIX = "tarpit:budget:active:bucket:"
TARPIT_ACTIVE_BUCKET_CATALOG_PREFIX = "tarpit:budget:active:bucket:catalog:"
RETENTION_BUCKET_INDEX_PREFIX = "telemetry:retention:v1:bucket:"
RETENTION_CATALOG_PREFIX = "telemetry:retention:v1:catalog:"
HOT_READ_BOOTSTRAP_KEY = "telemetry:hot_read:v1:bootstrap:default"
HOT_READ_RECENT_EVENTS_TAIL_KEY = "telemetry:hot_read:v1:recent_events_tail:default"


class EvidenceFailure(RuntimeError):
    pass

def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Capture shared-host telemetry storage/query evidence for the selected "
            "ssh_systemd remote."
        )
    )
    parser.add_argument("--env-file", default=str(DEFAULT_ENV_FILE))
    parser.add_argument("--receipts-dir", default=str(DEFAULT_REMOTE_RECEIPTS_DIR))
    parser.add_argument("--name", help="Override the active remote target")
    parser.add_argument("--hours", type=int, default=DEFAULT_HOURS)
    parser.add_argument("--report-path", default=str(DEFAULT_REPORT_PATH))
    return parser.parse_args(argv)


def _append_hour(counter: dict[int, int], hour: int) -> None:
    counter[hour] = counter.get(hour, 0) + 1


def _sorted_hour_counts(counter: dict[int, int], label: str = "hour") -> list[dict[str, int]]:
    return [{label: hour, "key_count": counter[hour]} for hour in sorted(counter)]


def _monitoring_hour(key: str) -> int | None:
    try:
        return int(key.rsplit(":", 1)[1])
    except (IndexError, ValueError):
        return None


def _eventlog_hour(key: str) -> int | None:
    parts = key.split(":")
    if len(parts) < 4:
        return None
    try:
        return int(parts[2])
    except ValueError:
        return None


def _rollup_day_start_hour(key: str) -> int | None:
    parts = key.split(":")
    if len(parts) < 4:
        return None
    try:
        return int(parts[3])
    except ValueError:
        return None


def summarize_remote_keys(keys: list[str]) -> dict[str, Any]:
    monitoring_hours: dict[int, int] = {}
    rollup_hours: dict[int, int] = {}
    eventlog_hours: dict[int, int] = {}
    maze_hits_total = 0
    maze_hits_catalog_present = False
    tarpit_active_bucket_total = 0
    tarpit_active_bucket_catalog_total = 0
    retention_bucket_indexes = {
        "monitoring": 0,
        "eventlog": 0,
        "monitoring_rollup": 0,
    }
    retention_catalogs = {
        "monitoring": 0,
        "eventlog": 0,
        "monitoring_rollup": 0,
    }

    for key in keys:
        if key.startswith("monitoring:v1:"):
            hour = _monitoring_hour(key)
            if hour is not None:
                _append_hour(monitoring_hours, hour)
            continue
        if key.startswith("monitoring_rollup:v1:day:"):
            hour = _rollup_day_start_hour(key)
            if hour is not None:
                _append_hour(rollup_hours, hour)
            continue
        if key.startswith("eventlog:v2:"):
            hour = _eventlog_hour(key)
            if hour is not None:
                _append_hour(eventlog_hours, hour)
            continue
        if key.startswith("maze_hits:"):
            maze_hits_total += 1
            if key == "maze_hits:catalog:v1":
                maze_hits_catalog_present = True
            continue
        if key.startswith(TARPIT_ACTIVE_BUCKET_CATALOG_PREFIX):
            tarpit_active_bucket_catalog_total += 1
            continue
        if key.startswith(TARPIT_ACTIVE_BUCKET_PREFIX):
            tarpit_active_bucket_total += 1
            continue
        if key.startswith(RETENTION_BUCKET_INDEX_PREFIX):
            suffix = key[len(RETENTION_BUCKET_INDEX_PREFIX) :]
            domain = suffix.rsplit(":", 1)[0]
            if domain in retention_bucket_indexes:
                retention_bucket_indexes[domain] += 1
            continue
        if key.startswith(RETENTION_CATALOG_PREFIX):
            domain = key[len(RETENTION_CATALOG_PREFIX) :]
            if domain in retention_catalogs:
                retention_catalogs[domain] += 1

    return {
        "default_store_total_keys": len(keys),
        "domains": {
            "monitoring": {
                "total_keys": sum(monitoring_hours.values()),
                "keys_per_hour": _sorted_hour_counts(monitoring_hours),
            },
            "monitoring_rollup": {
                "total_keys": sum(rollup_hours.values()),
                "keys_per_hour": _sorted_hour_counts(rollup_hours, label="day_start_hour"),
            },
            "eventlog": {
                "total_keys": sum(eventlog_hours.values()),
                "keys_per_hour": _sorted_hour_counts(eventlog_hours),
            },
        },
        "telemetry_adjacent": {
            "maze_hits": {
                "total_keys": maze_hits_total,
                "catalog_present": maze_hits_catalog_present,
            },
            "tarpit_active_bucket_state": {
                "total_keys": tarpit_active_bucket_total,
            },
            "tarpit_active_bucket_catalog": {
                "total_keys": tarpit_active_bucket_catalog_total,
            },
            "retention_bucket_indexes": retention_bucket_indexes,
            "retention_catalogs": retention_catalogs,
        },
    }


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
    remote: dict[str, Any],
    keyspace_summary: dict[str, Any],
    storage_samples: dict[str, Any],
    bootstrap_measurement: dict[str, Any],
    bootstrap_gzip_measurement: dict[str, Any],
    delta_measurement: dict[str, Any],
    stream_measurement: dict[str, Any],
) -> dict[str, Any]:
    bootstrap_payload = bootstrap_measurement.get("payload", {})
    retention_health = (
        bootstrap_payload.get("details", {}).get("retention_health", {}) if isinstance(bootstrap_payload, dict) else {}
    )
    cost_governance = (
        bootstrap_payload.get("details", {}).get("cost_governance", {}) if isinstance(bootstrap_payload, dict) else {}
    )
    query_budget = cost_governance.get("query_budget", {}) if isinstance(cost_governance, dict) else {}
    read_surface = cost_governance.get("read_surface", {}) if isinstance(cost_governance, dict) else {}
    gzip_ratio = compression_ratio_percent(
        bootstrap_measurement.get("response_bytes", 0),
        bootstrap_gzip_measurement.get("response_bytes", 0),
    )
    budgets = evaluate_budget_report(
        bootstrap_measurement=bootstrap_measurement,
        delta_measurement=delta_measurement,
    )
    retained_value_bytes = storage_samples.get("retained_value_bytes", {})
    retained_domains = retained_value_bytes.get("domains", {}) if isinstance(retained_value_bytes, dict) else {}
    keyspace_domains = keyspace_summary.get("domains", {}) if isinstance(keyspace_summary, dict) else {}

    retained_value_pressure = {"domains": {}}
    for domain in ("monitoring", "monitoring_rollup", "eventlog"):
        domain_summary = keyspace_domains.get(domain, {}) if isinstance(keyspace_domains, dict) else {}
        active_windows = len(domain_summary.get("keys_per_hour", []) or domain_summary.get("keys_per_day_start_hour", []))
        total_value_bytes = retained_domains.get(domain)
        bytes_per_active_window = None
        if isinstance(total_value_bytes, int) and active_windows > 0:
            bytes_per_active_window = round(total_value_bytes / active_windows, 2)
        retained_value_pressure["domains"][domain] = {
            "total_value_bytes": total_value_bytes,
            "active_windows": active_windows,
            "bytes_per_active_window": bytes_per_active_window,
        }

    hot_read_documents = storage_samples.get("hot_read_documents", {})
    hot_read_total_value_bytes = sum(
        int(value)
        for value in hot_read_documents.values()
        if isinstance(value, int)
    )
    retained_value_pressure["retention_bucket_indexes"] = retained_value_bytes.get("retention_bucket_indexes", {})
    retained_value_pressure["retention_catalogs"] = retained_value_bytes.get("retention_catalogs", {})
    retained_value_pressure["hot_read_documents_total_value_bytes"] = hot_read_total_value_bytes
    retained_value_pressure["telemetry_total_value_bytes"] = (
        sum(int(value) for value in retained_domains.values() if isinstance(value, int))
        + sum(
            int(value)
            for value in retained_value_bytes.get("retention_bucket_indexes", {}).values()
            if isinstance(value, int)
        )
        + sum(
            int(value)
            for value in retained_value_bytes.get("retention_catalogs", {}).values()
            if isinstance(value, int)
        )
        + hot_read_total_value_bytes
    )

    return {
        "captured_at_utc": utc_now_iso(),
        "remote": {
            "name": remote["identity"]["name"],
            "host": remote["ssh"]["host"],
            "base_url": remote["runtime"]["public_base_url"],
            "app_dir": remote["runtime"]["app_dir"],
        },
        "keyspace": keyspace_summary,
        "storage": storage_samples,
        "storage_pressure": retained_value_pressure,
        "retention_health": retention_health,
        "budgets": budgets,
        "query_cost": {
            "query_budget_status": cost_governance.get("query_budget_status"),
            "cost_units": query_budget.get("cost_units"),
            "cost_class": query_budget.get("cost_class"),
            "bucket_density": query_budget.get("bucket_density"),
            "density_penalty_units": query_budget.get("density_penalty_units"),
            "read_surface": read_surface,
        },
        "payloads": {
            "monitoring_bootstrap": {
                "status": bootstrap_measurement.get("status"),
                "latency_ms": bootstrap_measurement.get("latency_ms"),
                "response_bytes": bootstrap_measurement.get("response_bytes"),
                "content_encoding": bootstrap_measurement.get("content_encoding"),
            },
            "monitoring_bootstrap_gzip": {
                "status": bootstrap_gzip_measurement.get("status"),
                "latency_ms": bootstrap_gzip_measurement.get("latency_ms"),
                "response_bytes": bootstrap_gzip_measurement.get("response_bytes"),
                "content_encoding": bootstrap_gzip_measurement.get("content_encoding"),
                "compression_ratio_percent": gzip_ratio,
            },
            "monitoring_delta": {
                "status": delta_measurement.get("status"),
                "latency_ms": delta_measurement.get("latency_ms"),
                "response_bytes": delta_measurement.get("response_bytes"),
                "content_encoding": delta_measurement.get("content_encoding"),
            },
            "monitoring_stream": {
                "status": stream_measurement.get("status"),
                "latency_ms": stream_measurement.get("latency_ms"),
                "response_bytes": stream_measurement.get("response_bytes"),
                "content_encoding": stream_measurement.get("content_encoding"),
                "event_count": stream_measurement.get("payload", {}).get("event_count"),
            },
        },
    }


class TelemetrySharedHostEvidence:
    def __init__(
        self,
        *,
        env_file: Path,
        receipts_dir: Path,
        remote_name: str | None,
        report_path: Path,
        hours: int = DEFAULT_HOURS,
    ) -> None:
        self.env_file = env_file
        self.receipts_dir = receipts_dir
        self.report_path = report_path
        self.hours = max(1, int(hours))
        self.receipt = select_remote(remote_name, env_file, receipts_dir)
        self.local_env = read_env_file(env_file)
        self.api_key = self.local_env.get("SHUMA_API_KEY", "").strip()
        if not self.api_key:
            raise EvidenceFailure("SHUMA_API_KEY must be present in the selected env file.")
        self.base_url = self.receipt["runtime"]["public_base_url"].rstrip("/")
        self.ssl_context = self._build_ssl_context()

    def _build_ssl_context(self):
        hostname = urlparse(self.base_url).hostname or ""
        if hostname.endswith(".sslip.io"):
            return ssl._create_unverified_context()
        return None

    def _request(
        self,
        path: str,
        *,
        accept_gzip: bool = False,
        expected_statuses: tuple[int, ...] = (200,),
    ) -> dict[str, Any]:
        url = self.base_url + path
        headers = {
            "Authorization": f"Bearer {self.api_key}",
        }
        if accept_gzip:
            headers["Accept-Encoding"] = "gzip"
        request = urllib.request.Request(url, method="GET", headers=headers)
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
        if status not in expected_statuses:
            raise EvidenceFailure(f"GET {path} returned {status}")
        body = raw
        if content_encoding == "gzip":
            body = gzip.decompress(raw)
        payload: Any
        if "application/json" in content_type.lower():
            payload = json.loads(body.decode("utf-8"))
        else:
            payload = body.decode("utf-8", errors="replace")
        return {
            "status": status,
            "latency_ms": elapsed_ms,
            "response_bytes": len(raw),
            "content_encoding": content_encoding,
            "payload": payload,
        }

    def measure_json_endpoint(self, path: str, *, accept_gzip: bool = False) -> dict[str, Any]:
        return self._request(path, accept_gzip=accept_gzip)

    def measure_stream_endpoint(self, path: str) -> dict[str, Any]:
        measurement = self._request(path)
        payload = measurement["payload"]
        if not isinstance(payload, str):
            raise EvidenceFailure(f"Expected text/event-stream payload for {path}")
        event_count = sum(1 for line in payload.splitlines() if line.startswith("event:"))
        measurement["payload"] = {
            "event_count": event_count,
        }
        return measurement

    def collect_remote_keyspace_summary(self) -> dict[str, Any]:
        runtime = self.receipt["runtime"]
        sqlite_path = f"{runtime['app_dir']}/{REMOTE_SQLITE_KV_PATH}"
        remote_script = """python3 - <<'PY'
import json
import os
import sqlite3

db_path = os.environ["SHUMA_REMOTE_KV_PATH"]
conn = sqlite3.connect(db_path)
cur = conn.cursor()
keys = [
    row[0]
    for row in cur.execute(
        "SELECT key FROM spin_key_value WHERE store = 'default' ORDER BY key"
    ).fetchall()
]
print(json.dumps({"kv_path": db_path, "keys": keys}))
PY"""
        remote_command = (
            f"SHUMA_REMOTE_KV_PATH={shlex.quote(sqlite_path)} "
            f"bash -c {shlex.quote(remote_script)}"
        )
        result = subprocess.run(
            ssh_command_for_operation(self.receipt, remote_command),
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            raise EvidenceFailure(
                f"Failed to query remote telemetry keyspace: {(result.stderr or result.stdout).strip()}"
            )
        payload = json.loads((result.stdout or "").strip())
        summary = summarize_remote_keys(payload.get("keys", []))
        summary["kv_path"] = payload.get("kv_path", sqlite_path)
        return summary

    def collect_remote_storage_samples(self) -> dict[str, Any]:
        runtime = self.receipt["runtime"]
        sqlite_path = f"{runtime['app_dir']}/{REMOTE_SQLITE_KV_PATH}"
        remote_script = """python3 - <<'PY'
import json
import os
import sqlite3

BOOTSTRAP_KEY = os.environ["SHUMA_HOT_READ_BOOTSTRAP_KEY"]
RECENT_TAIL_KEY = os.environ["SHUMA_HOT_READ_RECENT_EVENTS_TAIL_KEY"]
db_path = os.environ["SHUMA_REMOTE_KV_PATH"]
conn = sqlite3.connect(db_path)
cur = conn.cursor()

def row_length(key):
    row = cur.execute(
        "SELECT length(value) FROM spin_key_value WHERE store = 'default' AND key = ?",
        (key,),
    ).fetchone()
    return int(row[0]) if row and row[0] is not None else None

def prefix_total_bytes(prefix):
    row = cur.execute(
        "SELECT COALESCE(SUM(length(value)), 0) FROM spin_key_value WHERE store = 'default' AND key LIKE ?",
        (f"{prefix}%",),
    ).fetchone()
    return int(row[0] or 0)

event_rows = []
for key, value in cur.execute(
    "SELECT key, value FROM spin_key_value WHERE store = 'default' AND key LIKE 'eventlog:v2:%' ORDER BY key DESC LIMIT 10"
).fetchall():
    row = {"key": key, "bytes": len(value)}
    try:
        payload = json.loads(value.decode("utf-8"))
    except Exception:
        payload = {}
    taxonomy = payload.get("taxonomy") if isinstance(payload, dict) else {}
    row["event"] = payload.get("event") if isinstance(payload, dict) else None
    row["reason"] = payload.get("reason") if isinstance(payload, dict) else None
    row["outcome_code"] = payload.get("outcome_code") if isinstance(payload, dict) else None
    row["botness_score"] = payload.get("botness_score") if isinstance(payload, dict) else None
    row["taxonomy_level"] = taxonomy.get("level") if isinstance(taxonomy, dict) else None
    event_rows.append(row)

row_bytes = [row["bytes"] for row in event_rows]
summary = {
    "eventlog_rows": {
        "sample_count": len(event_rows),
        "min_bytes": min(row_bytes) if row_bytes else None,
        "max_bytes": max(row_bytes) if row_bytes else None,
        "avg_bytes": round(sum(row_bytes) / len(row_bytes), 2) if row_bytes else None,
        "rows": event_rows,
    },
    "hot_read_documents": {
        "bootstrap_document_bytes": row_length(BOOTSTRAP_KEY),
        "recent_events_tail_document_bytes": row_length(RECENT_TAIL_KEY),
    },
    "retained_value_bytes": {
        "domains": {
            "monitoring": prefix_total_bytes("monitoring:v1:"),
            "monitoring_rollup": prefix_total_bytes("monitoring_rollup:v1:"),
            "eventlog": prefix_total_bytes("eventlog:v2:"),
        },
        "retention_bucket_indexes": {
            "monitoring": prefix_total_bytes("telemetry:retention:v1:bucket:monitoring:"),
            "monitoring_rollup": prefix_total_bytes("telemetry:retention:v1:bucket:monitoring_rollup:"),
            "eventlog": prefix_total_bytes("telemetry:retention:v1:bucket:eventlog:"),
        },
        "retention_catalogs": {
            "monitoring": row_length("telemetry:retention:v1:catalog:monitoring") or 0,
            "monitoring_rollup": row_length("telemetry:retention:v1:catalog:monitoring_rollup") or 0,
            "eventlog": row_length("telemetry:retention:v1:catalog:eventlog") or 0,
        },
    },
}
print(json.dumps(summary))
PY"""
        remote_command = (
            f"SHUMA_REMOTE_KV_PATH={shlex.quote(sqlite_path)} "
            f"SHUMA_HOT_READ_BOOTSTRAP_KEY={shlex.quote(HOT_READ_BOOTSTRAP_KEY)} "
            f"SHUMA_HOT_READ_RECENT_EVENTS_TAIL_KEY={shlex.quote(HOT_READ_RECENT_EVENTS_TAIL_KEY)} "
            f"bash -c {shlex.quote(remote_script)}"
        )
        result = subprocess.run(
            ssh_command_for_operation(self.receipt, remote_command),
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            raise EvidenceFailure(
                f"Failed to query remote telemetry storage samples: {(result.stderr or result.stdout).strip()}"
            )
        return json.loads((result.stdout or "").strip())

    def run(self) -> dict[str, Any]:
        keyspace_summary = self.collect_remote_keyspace_summary()
        storage_samples = self.collect_remote_storage_samples()
        bootstrap_path = (
            f"/admin/monitoring?hours={self.hours}&limit={DEFAULT_BOOTSTRAP_LIMIT}&bootstrap=1"
        )
        delta_path = f"/admin/monitoring/delta?hours={self.hours}&limit={DEFAULT_DELTA_LIMIT}"
        stream_path = f"/admin/monitoring/stream?hours={self.hours}&limit={DEFAULT_DELTA_LIMIT}"
        bootstrap_measurement = self.measure_json_endpoint(bootstrap_path)
        bootstrap_gzip_measurement = self.measure_json_endpoint(bootstrap_path, accept_gzip=True)
        delta_measurement = self.measure_json_endpoint(delta_path)
        stream_measurement = self.measure_stream_endpoint(stream_path)
        report = build_evidence_report(
            remote=self.receipt,
            keyspace_summary=keyspace_summary,
            storage_samples=storage_samples,
            bootstrap_measurement=bootstrap_measurement,
            bootstrap_gzip_measurement=bootstrap_gzip_measurement,
            delta_measurement=delta_measurement,
            stream_measurement=stream_measurement,
        )
        if not report["budgets"]["bootstrap_within_budget"] or not report["budgets"]["delta_within_budget"]:
            raise EvidenceFailure(
                "Shared-host telemetry hot-read budgets exceeded: "
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
    collector = TelemetrySharedHostEvidence(
        env_file=Path(args.env_file).expanduser(),
        receipts_dir=Path(args.receipts_dir).expanduser(),
        remote_name=args.name,
        report_path=Path(args.report_path).expanduser(),
        hours=args.hours,
    )
    report = collector.run()
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
