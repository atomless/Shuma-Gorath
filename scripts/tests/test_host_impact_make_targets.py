import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


class HostImpactMakeTargetTests(unittest.TestCase):
    def test_host_impact_telemetry_target_uses_current_seam_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-host-impact-telemetry:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "runtime::request_outcome::tests::forwarded_outcome_preserves_forwarded_upstream_latency",
            body,
        )
        self.assertIn(
            "observability::monitoring::tests::record_request_outcome_records_origin_scope_outcome_and_lane_counters",
            body,
        )
        self.assertIn(
            "observability::monitoring::tests::record_request_outcome_records_non_human_category_counters_for_verified_crosswalks",
            body,
        )
        self.assertIn(
            "observability::monitoring::tests::record_request_outcome_does_not_increment_latency_for_non_forwarded_outcomes",
            body,
        )
        self.assertIn(
            "observability::hot_read_projection::tests::counter_flush_refresh_preserves_request_outcome_summary_rows_in_summary_and_bootstrap",
            body,
        )
        self.assertIn("scripts/tests/test_host_impact_make_targets.py", body)

    def test_host_impact_benchmark_target_uses_current_seam_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-host-impact-benchmark:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "observability::operator_snapshot::tests::snapshot_payload_projects_suspicious_forwarded_latency_budget_row",
            body,
        )
        self.assertIn(
            "observability::benchmark_results::tests::benchmark_results_materialize_host_impact_metrics_in_suspicious_origin_cost_family",
            body,
        )
        self.assertIn(
            "observability::benchmark_comparison::tests::prior_window_comparison_marks_host_impact_metrics_as_lower_is_better",
            body,
        )
        self.assertIn(
            "observability::benchmark_suite::tests::benchmark_suite_v1_exposes_small_machine_first_family_registry",
            body,
        )
        self.assertIn("scripts/tests/test_host_impact_make_targets.py", body)

    def test_oversight_host_impact_target_uses_current_seam_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-oversight-host-impact:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "admin::oversight_reconcile::tests::primary_pressure_treats_latency_share_budget_miss_as_suspicious_origin_cost",
            body,
        )
        self.assertIn("scripts/tests/test_host_impact_make_targets.py", body)


if __name__ == "__main__":
    unittest.main()
