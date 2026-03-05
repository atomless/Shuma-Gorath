import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gateway_tls_wasm_harness as harness


class GatewayTlsWasmHarnessUnitTests(unittest.TestCase):
    def test_build_manifest_rewrites_allowed_outbound_hosts(self) -> None:
        manifest = """
spin_manifest_version = 2
[component.bot-defence]
source = \"dist/wasm/shuma_gorath.wasm\"
allowed_outbound_hosts = []
"""
        rewritten = harness.build_manifest_with_allowed_outbound_hosts(
            manifest,
            ["https://expired.badssl.com:443", "https://wrong.host.badssl.com:443"],
        )
        self.assertIn(
            'allowed_outbound_hosts = ["https://expired.badssl.com:443", "https://wrong.host.badssl.com:443"]',
            rewritten,
        )

    def test_build_manifest_requires_allowed_outbound_hosts_field(self) -> None:
        with self.assertRaises(ValueError):
            harness.build_manifest_with_allowed_outbound_hosts(
                "spin_manifest_version = 2\n[component.bot-defence]\n",
                ["https://expired.badssl.com:443"],
            )

    def test_parse_prometheus_counter_extracts_class_value(self) -> None:
        metrics = """
# TYPE bot_defence_forward_failure_total counter
bot_defence_forward_failure_total{class=\"timeout\"} 1
bot_defence_forward_failure_total{class=\"transport\"} 3
"""
        self.assertEqual(harness.parse_prometheus_counter(metrics, "transport"), 3.0)
        self.assertEqual(harness.parse_prometheus_counter(metrics, "timeout"), 1.0)
        self.assertEqual(harness.parse_prometheus_counter(metrics, "policy_denied"), 0.0)

    def test_parse_forward_failure_classes_extracts_gateway_log_labels(self) -> None:
        logs = """
[gateway-forward] failed status=502 class=transport reason=tls_error
[gateway-forward] failed status=504 class=timeout reason=upstream_timeout
"""
        self.assertEqual(
            harness.parse_forward_failure_classes(logs),
            ["transport", "timeout"],
        )

    def test_evaluate_gateway_failure_requires_fallback_body(self) -> None:
        ok, detail = harness.evaluate_gateway_failure(502, b"Gateway forwarding unavailable")
        self.assertTrue(ok)
        self.assertIn("status=502", detail)

        ok2, _ = harness.evaluate_gateway_failure(502, b"some other body")
        self.assertFalse(ok2)


if __name__ == "__main__":
    unittest.main()
