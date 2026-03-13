import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "tests" / "telemetry_fermyon_edge_evidence.py"
SPEC = importlib.util.spec_from_file_location("telemetry_fermyon_edge_evidence", SCRIPT)
TELEMETRY_FERMYON_EDGE_EVIDENCE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(TELEMETRY_FERMYON_EDGE_EVIDENCE)


class TelemetryFermyonEdgeEvidenceTests(unittest.TestCase):
    def test_budget_evaluation_uses_edge_targets(self) -> None:
        budget_report = TELEMETRY_FERMYON_EDGE_EVIDENCE.evaluate_budget_report(
            bootstrap_measurement={"latency_ms": 1500.0},
            delta_measurement={"latency_ms": 600.0},
        )

        self.assertEqual(
            budget_report,
            {
                "bootstrap_budget_ms": 2000.0,
                "bootstrap_within_budget": True,
                "delta_budget_ms": 750.0,
                "delta_within_budget": True,
            },
        )

    def test_build_report_extracts_budget_and_receipt_context(self) -> None:
        receipt = {
            "schema": "shuma.fermyon.akamai_edge_deploy.v1",
            "fermyon": {
                "account_name": "atomless",
                "app_id": "app-123",
                "app_name": "shuma-edge-prod",
                "primary_url": "https://edge.example.com",
                "status": {"urls": ["https://edge.example.com"]},
            },
            "git_head": "abc123",
        }
        bootstrap_measurement = {
            "status": 200,
            "latency_ms": 1200.0,
            "response_bytes": 8192,
            "content_encoding": "none",
            "payload": {"details": {"events": {"recent_events_window": {"response_shaping_reason": "edge_profile_bounded_details"}}}},
        }
        delta_measurement = {
            "status": 200,
            "latency_ms": 510.0,
            "response_bytes": 1400,
            "content_encoding": "none",
            "payload": {"events": []},
        }

        report = TELEMETRY_FERMYON_EDGE_EVIDENCE.build_evidence_report(
            receipt=receipt,
            bootstrap_measurement=bootstrap_measurement,
            delta_measurement=delta_measurement,
        )

        self.assertEqual(report["edge"]["app_name"], "shuma-edge-prod")
        self.assertEqual(report["edge"]["base_url"], "https://edge.example.com")
        self.assertEqual(report["budgets"]["bootstrap_budget_ms"], 2000.0)
        self.assertTrue(report["budgets"]["bootstrap_within_budget"])
        self.assertEqual(report["budgets"]["delta_budget_ms"], 750.0)
        self.assertTrue(report["budgets"]["delta_within_budget"])
        self.assertEqual(
            report["payloads"]["monitoring_bootstrap"]["response_shaping_reason"],
            "edge_profile_bounded_details",
        )

    def test_run_reads_receipt_and_writes_report(self) -> None:
        with tempfile.TemporaryDirectory(prefix="telemetry-fermyon-edge-evidence-") as temp_dir:
            temp_path = Path(temp_dir)
            env_file = temp_path / ".env.local"
            receipt_path = temp_path / ".shuma" / "fermyon-akamai-edge-deploy.json"
            receipt_path.parent.mkdir(parents=True)
            report_path = temp_path / "report.json"
            env_file.write_text("SHUMA_API_KEY=test-admin-key\n", encoding="utf-8")
            receipt = {
                "schema": "shuma.fermyon.akamai_edge_deploy.v1",
                "fermyon": {
                    "account_name": "atomless",
                    "app_id": "app-123",
                    "app_name": "shuma-edge-prod",
                    "primary_url": "https://edge.example.com",
                    "status": {"urls": ["https://edge.example.com"]},
                },
                "git_head": "abc123",
            }
            receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")

            collector = TELEMETRY_FERMYON_EDGE_EVIDENCE.TelemetryFermyonEdgeEvidence(
                env_file=env_file,
                receipt_path=receipt_path,
                report_path=report_path,
            )

            with patch.object(
                collector,
                "measure_json_endpoint",
                side_effect=[
                    {
                        "status": 200,
                        "latency_ms": 1000.0,
                        "response_bytes": 2048,
                        "content_encoding": "none",
                        "payload": {
                            "details": {
                                "events": {
                                    "recent_events_window": {
                                        "response_shaping_reason": "edge_profile_bounded_details"
                                    }
                                }
                            }
                        },
                    },
                    {
                        "status": 200,
                        "latency_ms": 350.0,
                        "response_bytes": 512,
                        "content_encoding": "none",
                        "payload": {"events": []},
                    },
                ],
            ) as measure_json_endpoint:
                report = collector.run()

            self.assertEqual(report["edge"]["base_url"], "https://edge.example.com")
            persisted = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(persisted["edge"]["app_id"], "app-123")
            self.assertEqual(
                measure_json_endpoint.call_args_list[0].args[0],
                "/admin/monitoring?hours=24&limit=10&bootstrap=1",
            )
            self.assertEqual(
                measure_json_endpoint.call_args_list[1].args[0],
                "/admin/monitoring/delta?hours=24&limit=40",
            )
