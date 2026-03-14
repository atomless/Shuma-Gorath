import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "tests" / "telemetry_shared_host_evidence.py"
SPEC = importlib.util.spec_from_file_location("telemetry_shared_host_evidence", SCRIPT)
TELEMETRY_SHARED_HOST_EVIDENCE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(TELEMETRY_SHARED_HOST_EVIDENCE)


class TelemetrySharedHostEvidenceTests(unittest.TestCase):
    def test_budget_evaluation_uses_canonical_bootstrap_and_delta_targets(self) -> None:
        budget_report = TELEMETRY_SHARED_HOST_EVIDENCE.evaluate_budget_report(
            bootstrap_measurement={"latency_ms": 640.0},
            delta_measurement={"latency_ms": 200.0},
        )

        self.assertEqual(
            budget_report,
            {
                "bootstrap_budget_ms": 750.0,
                "bootstrap_within_budget": True,
                "delta_budget_ms": 250.0,
                "delta_within_budget": True,
            },
        )

    def test_summarize_remote_keys_counts_domains_and_adjacent_surfaces(self) -> None:
        summary = TELEMETRY_SHARED_HOST_EVIDENCE.summarize_remote_keys(
            [
                "monitoring:v1:challenge:total:100",
                "monitoring:v1:pow:total:100",
                "monitoring_rollup:v1:day:72",
                "eventlog:v2:99:1-a",
                "eventlog:v2:99:2-b",
                "eventlog:v2:100:1-c",
                "maze_hits:bucket-a",
                "maze_hits:catalog:v1",
                "tarpit:budget:active:bucket:site-a:bucket-a:10",
                "tarpit:budget:active:bucket:catalog:site-a",
                "telemetry:retention:v1:bucket:monitoring:100",
                "telemetry:retention:v1:bucket:eventlog:99",
                "telemetry:retention:v1:bucket:monitoring_rollup:72",
                "telemetry:retention:v1:catalog:monitoring",
                "telemetry:retention:v1:catalog:eventlog",
                "config:default",
            ]
        )

        self.assertEqual(summary["default_store_total_keys"], 16)
        self.assertEqual(summary["domains"]["monitoring"]["total_keys"], 2)
        self.assertEqual(summary["domains"]["monitoring_rollup"]["total_keys"], 1)
        self.assertEqual(summary["domains"]["eventlog"]["total_keys"], 3)
        self.assertEqual(
            summary["domains"]["eventlog"]["keys_per_hour"],
            [
                {"hour": 99, "key_count": 2},
                {"hour": 100, "key_count": 1},
            ],
        )
        self.assertEqual(summary["telemetry_adjacent"]["maze_hits"]["total_keys"], 2)
        self.assertTrue(summary["telemetry_adjacent"]["maze_hits"]["catalog_present"])
        self.assertEqual(
            summary["telemetry_adjacent"]["tarpit_active_bucket_state"]["total_keys"], 1
        )
        self.assertEqual(
            summary["telemetry_adjacent"]["tarpit_active_bucket_catalog"]["total_keys"], 1
        )
        self.assertEqual(
            summary["telemetry_adjacent"]["retention_bucket_indexes"],
            {"eventlog": 1, "monitoring": 1, "monitoring_rollup": 1},
        )
        self.assertEqual(
            summary["telemetry_adjacent"]["retention_catalogs"],
            {"eventlog": 1, "monitoring": 1, "monitoring_rollup": 0},
        )

    def test_build_report_extracts_retention_and_query_budget_fields(self) -> None:
        remote = {
            "identity": {"name": "blog-prod"},
            "ssh": {"host": "203.0.113.10"},
            "runtime": {
                "app_dir": "/opt/shuma-gorath",
                "public_base_url": "https://shuma.example.com",
            },
        }
        keyspace_summary = {
            "default_store_total_keys": 400,
            "domains": {
                "monitoring": {"keys_per_hour": [{"hour": 100, "key_count": 2}]},
                "monitoring_rollup": {"keys_per_hour": [{"day_start_hour": 72, "key_count": 1}]},
                "eventlog": {
                    "keys_per_hour": [
                        {"hour": 99, "key_count": 2},
                        {"hour": 100, "key_count": 1},
                    ]
                },
            },
            "telemetry_adjacent": {},
        }
        snapshot_measurement = {
            "status": 200,
            "latency_ms": 42.5,
            "response_bytes": 12000,
            "content_encoding": "none",
            "payload": {
                "details": {
                    "retention_health": {
                        "state": "healthy",
                        "purge_lag_hours": 0.0,
                        "pending_expired_buckets": 0,
                    },
                    "cost_governance": {
                        "query_budget_status": "within_budget",
                        "query_budget": {
                            "cost_units": 128,
                            "bucket_density": 12.0,
                            "density_penalty_units": 0,
                        },
                        "read_surface": {
                            "monitoring_keys": 20,
                            "eventlog_keys": 5,
                            "residual_scan_keys": 0,
                        },
                    },
                }
            },
        }
        snapshot_gzip_measurement = {
            "status": 200,
            "latency_ms": 40.0,
            "response_bytes": 4000,
            "content_encoding": "gzip",
            "payload": {},
        }
        delta_measurement = {
            "status": 200,
            "latency_ms": 18.0,
            "response_bytes": 1800,
            "content_encoding": "none",
            "payload": {"rows": []},
        }
        stream_measurement = {
            "status": 200,
            "latency_ms": 22.0,
            "response_bytes": 900,
            "content_encoding": "none",
            "payload": {"event_count": 1},
        }

        report = TELEMETRY_SHARED_HOST_EVIDENCE.build_evidence_report(
            remote=remote,
            keyspace_summary=keyspace_summary,
            storage_samples={
                "eventlog_rows": {
                    "sample_count": 2,
                    "min_bytes": 128,
                    "max_bytes": 256,
                    "avg_bytes": 192.0,
                    "rows": [
                        {"key": "eventlog:v2:1:1-a", "bytes": 128, "reason": "botness_gate_challenge"},
                        {"key": "eventlog:v2:1:2-b", "bytes": 256, "reason": "ip_range_policy_forbidden"},
                    ],
                },
                "hot_read_documents": {
                    "bootstrap_document_bytes": 4096,
                    "recent_events_tail_document_bytes": 2048,
                },
                "retained_value_bytes": {
                    "domains": {
                        "monitoring": 640,
                        "monitoring_rollup": 320,
                        "eventlog": 768,
                    },
                    "retention_bucket_indexes": {
                        "monitoring": 96,
                        "monitoring_rollup": 48,
                        "eventlog": 128,
                    },
                    "retention_catalogs": {
                        "monitoring": 32,
                        "monitoring_rollup": 16,
                        "eventlog": 24,
                    },
                },
            },
            bootstrap_measurement=snapshot_measurement,
            bootstrap_gzip_measurement=snapshot_gzip_measurement,
            delta_measurement=delta_measurement,
            stream_measurement=stream_measurement,
        )

        self.assertEqual(report["remote"]["name"], "blog-prod")
        self.assertEqual(report["retention_health"]["state"], "healthy")
        self.assertEqual(report["query_cost"]["query_budget_status"], "within_budget")
        self.assertEqual(report["query_cost"]["density_penalty_units"], 0)
        self.assertEqual(report["storage"]["eventlog_rows"]["sample_count"], 2)
        self.assertEqual(
            report["storage"]["hot_read_documents"]["recent_events_tail_document_bytes"], 2048
        )
        self.assertEqual(
            report["storage_pressure"]["domains"]["eventlog"],
            {
                "total_value_bytes": 768,
                "active_windows": 2,
                "bytes_per_active_window": 384.0,
            },
        )
        self.assertEqual(report["storage_pressure"]["hot_read_documents_total_value_bytes"], 6144)
        self.assertEqual(report["storage_pressure"]["telemetry_total_value_bytes"], 8216)
        self.assertEqual(report["payloads"]["monitoring_bootstrap"]["response_bytes"], 12000)
        self.assertEqual(report["payloads"]["monitoring_bootstrap_gzip"]["response_bytes"], 4000)
        self.assertEqual(
            report["payloads"]["monitoring_bootstrap_gzip"]["compression_ratio_percent"], 66.67
        )
        self.assertEqual(report["budgets"]["bootstrap_budget_ms"], 750.0)
        self.assertTrue(report["budgets"]["bootstrap_within_budget"])
        self.assertEqual(report["budgets"]["delta_budget_ms"], 250.0)
        self.assertTrue(report["budgets"]["delta_within_budget"])

    def test_run_writes_report_with_remote_measurements(self) -> None:
        with tempfile.TemporaryDirectory(prefix="telemetry-shared-host-evidence-") as temp_dir:
            temp_path = Path(temp_dir)
            env_file = temp_path / ".env.local"
            receipts_dir = temp_path / ".shuma" / "remotes"
            receipts_dir.mkdir(parents=True)
            report_path = temp_path / "report.json"
            env_file.write_text("SHUMA_API_KEY=test-admin-key\n", encoding="utf-8")
            remote_receipt = {
                "schema": "shuma.remote_target.v1",
                "identity": {
                    "name": "blog-prod",
                    "backend_kind": "ssh_systemd",
                    "provider_kind": "linode",
                },
                "ssh": {
                    "host": "203.0.113.10",
                    "port": 22,
                    "user": "shuma",
                    "private_key_path": "/tmp/key",
                },
                "runtime": {
                    "app_dir": "/opt/shuma-gorath",
                    "service_name": "shuma-gorath",
                    "public_base_url": "https://shuma.example.com",
                },
                "deploy": {
                    "spin_manifest_path": "/opt/shuma-gorath/spin.gateway.toml",
                    "surface_catalog_path": str(temp_path / "surface.json"),
                    "smoke_path": "/health",
                },
                "metadata": {
                    "last_deployed_commit": "abc123",
                    "last_deployed_at_utc": "2026-03-11T10:00:00Z",
                },
                "provider": {},
            }
            (receipts_dir / "blog-prod.json").write_text(
                json.dumps(remote_receipt, indent=2) + "\n", encoding="utf-8"
            )

            collector = TELEMETRY_SHARED_HOST_EVIDENCE.TelemetrySharedHostEvidence(
                env_file=env_file,
                receipts_dir=receipts_dir,
                remote_name="blog-prod",
                report_path=report_path,
            )

            with patch.object(
                collector,
                "collect_remote_keyspace_summary",
                return_value={
                    "default_store_total_keys": 3,
                    "domains": {
                        "monitoring": {"keys_per_hour": []},
                        "monitoring_rollup": {"keys_per_hour": []},
                        "eventlog": {"keys_per_hour": [{"hour": 1, "key_count": 1}]},
                    },
                    "telemetry_adjacent": {},
                },
            ), patch.object(
                collector,
                "collect_remote_storage_samples",
                return_value={
                    "eventlog_rows": {
                        "sample_count": 1,
                        "min_bytes": 144,
                        "max_bytes": 144,
                        "avg_bytes": 144.0,
                        "rows": [
                            {
                                "key": "eventlog:v2:1:1-a",
                                "bytes": 144,
                                "reason": "botness_gate_challenge",
                                "outcome_code": "served",
                            }
                        ],
                    },
                    "hot_read_documents": {
                        "bootstrap_document_bytes": 5120,
                        "recent_events_tail_document_bytes": 2048,
                    },
                    "retained_value_bytes": {
                        "domains": {
                            "monitoring": 0,
                            "monitoring_rollup": 0,
                            "eventlog": 144,
                        },
                        "retention_bucket_indexes": {
                            "monitoring": 0,
                            "monitoring_rollup": 0,
                            "eventlog": 64,
                        },
                        "retention_catalogs": {
                            "monitoring": 0,
                            "monitoring_rollup": 0,
                            "eventlog": 16,
                        },
                    },
                },
            ), patch.object(
                collector,
                "measure_json_endpoint",
                side_effect=[
                    {
                        "status": 200,
                        "latency_ms": 10.0,
                        "response_bytes": 100,
                        "content_encoding": "none",
                        "payload": {
                            "details": {
                                "retention_health": {"state": "healthy", "purge_lag_hours": 0.0},
                                "cost_governance": {
                                    "query_budget_status": "within_budget",
                                    "query_budget": {
                                        "cost_units": 10,
                                        "bucket_density": 1.0,
                                        "density_penalty_units": 0,
                                    },
                                    "read_surface": {"residual_scan_keys": 0},
                                },
                            }
                        },
                    },
                    {
                        "status": 200,
                        "latency_ms": 8.0,
                        "response_bytes": 50,
                        "content_encoding": "gzip",
                        "payload": {},
                    },
                    {
                        "status": 200,
                        "latency_ms": 9.0,
                        "response_bytes": 40,
                        "content_encoding": "none",
                        "payload": {},
                    },
                ],
            ) as measure_json_endpoint, patch.object(
                collector,
                "measure_stream_endpoint",
                return_value={
                    "status": 200,
                    "latency_ms": 11.0,
                    "response_bytes": 32,
                    "content_encoding": "none",
                    "payload": {"event_count": 1},
                },
            ):
                report = collector.run()

            self.assertEqual(report["remote"]["name"], "blog-prod")
            self.assertTrue(report_path.exists())
            persisted = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(persisted["remote"]["base_url"], "https://shuma.example.com")
            self.assertEqual(
                persisted["storage"]["hot_read_documents"]["recent_events_tail_document_bytes"], 2048
            )
            self.assertEqual(
                persisted["storage_pressure"]["domains"]["eventlog"]["bytes_per_active_window"], 144.0
            )
            self.assertEqual(
                measure_json_endpoint.call_args_list[0].args[0],
                "/admin/monitoring?hours=24&limit=10&bootstrap=1",
            )
            self.assertEqual(
                measure_json_endpoint.call_args_list[1].args[0],
                "/admin/monitoring?hours=24&limit=10&bootstrap=1",
            )
            self.assertEqual(
                measure_json_endpoint.call_args_list[2].args[0],
                "/admin/monitoring/delta?hours=24&limit=40",
            )

    def test_main_cli_uses_current_argument_shape(self) -> None:
        with tempfile.TemporaryDirectory(prefix="telemetry-shared-host-cli-") as temp_dir:
            temp_path = Path(temp_dir)
            env_file = temp_path / ".env.local"
            receipts_dir = temp_path / ".shuma" / "remotes"
            report_path = temp_path / "report.json"
            receipts_dir.mkdir(parents=True)
            env_file.write_text(
                "SHUMA_API_KEY=test-admin-key\nSHUMA_ACTIVE_REMOTE=blog-prod\n",
                encoding="utf-8",
            )
            (receipts_dir / "blog-prod.json").write_text(
                json.dumps(
                    {
                        "schema": "shuma.remote_target.v1",
                        "identity": {
                            "name": "blog-prod",
                            "backend_kind": "ssh_systemd",
                            "provider_kind": "linode",
                        },
                        "ssh": {
                            "host": "203.0.113.10",
                            "port": 22,
                            "user": "shuma",
                            "private_key_path": "/tmp/key",
                        },
                        "runtime": {
                            "app_dir": "/opt/shuma-gorath",
                            "service_name": "shuma-gorath",
                            "public_base_url": "https://shuma.example.com",
                        },
                        "deploy": {
                            "spin_manifest_path": "/opt/shuma-gorath/spin.gateway.toml",
                            "surface_catalog_path": str(temp_path / "surface.json"),
                            "smoke_path": "/health",
                            "upstream_origin": "http://127.0.0.1:8080",
                        },
                        "metadata": {
                            "last_deployed_commit": "",
                            "last_deployed_at_utc": "",
                        },
                        "provider": {},
                    },
                    indent=2,
                )
                + "\n",
                encoding="utf-8",
            )

            with patch.object(
                TELEMETRY_SHARED_HOST_EVIDENCE.TelemetrySharedHostEvidence,
                "run",
                return_value={"budgets": {"bootstrap_within_budget": True, "delta_within_budget": True}},
            ):
                rc = TELEMETRY_SHARED_HOST_EVIDENCE.main(
                    [
                        "--env-file",
                        str(env_file),
                        "--receipts-dir",
                        str(receipts_dir),
                        "--report-path",
                        str(report_path),
                    ]
                )

        self.assertEqual(rc, 0)


if __name__ == "__main__":
    unittest.main()
