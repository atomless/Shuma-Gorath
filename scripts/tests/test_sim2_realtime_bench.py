#!/usr/bin/env python3

import unittest

import scripts.tests.sim2_realtime_bench as bench


class Sim2RealtimeBenchUnitTests(unittest.TestCase):
    def test_run_benchmark_is_deterministic_for_fixed_timestamp(self):
        first = bench.run_benchmark(now_unix=1_700_000_000)
        second = bench.run_benchmark(now_unix=1_700_000_000)
        self.assertEqual(first, second)
        self.assertEqual(first["schema_version"], "sim2-realtime-bench.v1")
        self.assertEqual(first["generated_at_unix"], 1_700_000_000)
        self.assertIn("sse", first["results"])

    def test_evaluate_thresholds_emits_named_failure_diagnostics(self):
        failures = bench.evaluate_thresholds(
            {
                "sse": {
                    "latency": {"p95_ms": 500.0, "p99_ms": 900.0},
                    "overflow_or_drop_count": 3,
                    "avg_requests_per_sec_client": 2.0,
                }
            }
        )
        joined = " ".join(failures)
        self.assertIn("latency_p95_exceeded", joined)
        self.assertIn("latency_p99_exceeded", joined)
        self.assertIn("overflow_or_drop_exceeded", joined)
        self.assertIn("request_budget_exceeded", joined)

    def test_render_summary_includes_latency_overflow_and_budget_sections(self):
        payload = bench.run_benchmark(now_unix=1_700_000_000)
        summary = bench.render_summary(payload)
        self.assertIn("SIM2 Realtime Benchmark Summary", summary)
        self.assertIn("latency percentiles", summary)
        self.assertIn("overflow/drop counts", summary)
        self.assertIn("request budget metrics", summary)


if __name__ == "__main__":
    unittest.main()
